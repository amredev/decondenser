//! Adapted from `prettyplease`:
//! <https://github.com/dtolnay/prettyplease/blob/0.2.34/src/algorithm.rs>
//!
//! which is itself the adaptation of the original `rustc_ast_pretty`:
//! <https://github.com/rust-lang/rust/blob/1.57.0/compiler/rustc_ast_pretty/src/pp.rs>
//!
//! which is, in turn, based on the Philip Karlton's Mesa pretty-printer, as
//!   described in the appendix to Derek C. Oppen, "Pretty Printing" (1979),
//!   Stanford Computer Science Department STAN-CS-79-770,
//!   <http://i.stanford.edu/pub/cstr/reports/cs/tr/79/770/CS-TR-79-770.pdf>.
//!
//! Also, this blog post by @mcyoung is a great resource for understanding:
//! <https://mcyoung.xyz/2025/03/11/formatters/>

mod convenience;
mod renderer;
mod ring;
mod token;

use self::renderer::Printer;
use self::ring::RingBuffer;
use self::token::{Begin, Break, Token};
use crate::Decondenser;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::fmt;
use token::{BreaksKind, Literal};

const SIZE_INFINITY: isize = 0xffff;

#[derive(Debug)]
pub(crate) struct Layout<'a> {
    /// Ring-buffer of tokens and sizes that are candidates for the next
    /// line.
    tokens: RingBuffer<Token<'a>>,

    /// Holds the ring-buffer index of the Begin that started the current block,
    /// possibly with the most recent Break after that Begin (if there is any) on
    /// top of it. Values are pushed and popped on the back of the queue using it
    /// like stack, and elsewhere old values are popped from the front of the
    /// queue as they become irrelevant due to the primary ring-buffer advancing.
    ///
    /// This basically serves as a cache. The algorithm could work without it
    /// by scanning the ring-buffer to search for tokens with negative sizes.
    unmeasured: VecDeque<usize>,

    /// Size of tokens that were already printed
    printed_size: isize,

    /// Size of tokens enqueued, including already printed and not yet printed.
    /// I.e. this is guaranteed to be >= [`Self::printed_size`].
    planned_size: isize,

    printer: Printer,
}

impl fmt::Debug for RingBuffer<Token<'_>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let index_range = self.index_range();
        writeln!(f, "index_range: {index_range:?}")?;

        let mut indent = 0usize;
        let indent_size = 4;

        for (entry, i) in self.iter().zip(index_range) {
            match entry {
                Token::Begin(_) => indent += indent_size,
                Token::End => indent = indent.saturating_sub(indent_size),
                _ => {}
            }

            writeln!(f, "[{i:>2}] {:indent$}{entry:?}", "")?;
        }

        Ok(())
    }
}

#[derive(Default)]
struct BreakParams {
    offset: isize,
    blank_space: usize,
    if_nonempty: bool,
    never_break: bool,
}

impl<'a> Layout<'a> {
    pub(crate) fn new(config: &Decondenser<'_>) -> Self {
        Layout {
            tokens: RingBuffer::new(),
            printed_size: 0,
            planned_size: 0,
            unmeasured: VecDeque::new(),
            printer: Printer::new(config),
        }
    }

    pub(crate) fn eof(mut self) -> String {
        if !self.unmeasured.is_empty() {
            self.update_measurements();
            self.print_measured_tokens();
        }
        self.printer.output
    }

    fn push_unmeasured(&mut self, token: Token<'a>) {
        let index = self.tokens.push(token);
        self.unmeasured.push_back(index);
    }

    fn begin(&mut self, offset: isize, breaks_kind: BreaksKind) {
        if self.unmeasured.is_empty() {
            self.printed_size = 1;
            self.planned_size = 1;
            self.tokens.clear();
        }
        self.push_unmeasured(Token::Begin(Begin {
            size: -self.planned_size,
            offset,
            breaks_kind,
        }));
    }

    pub(crate) fn end(&mut self) {
        if self.unmeasured.is_empty() {
            self.printer.end();
            return;
        }

        if let Some(&Token::Break(break_token)) = self.tokens.last() {
            if let Some(Token::Begin(_)) = self.tokens.second_last() {
                self.tokens.pop_last();
                self.tokens.pop_last();
                self.unmeasured.pop_back();
                self.unmeasured.pop_back();
                self.planned_size -= break_token.blank_space as isize;
                return;
            }

            if break_token.if_nonempty {
                self.tokens.pop_last();
                self.unmeasured.pop_back();
                self.planned_size -= break_token.blank_space as isize;
            }
        }

        let right = self.tokens.push(Token::End);

        self.unmeasured.push_back(right);
    }

    fn break_(&mut self, params: BreakParams) {
        if self.unmeasured.is_empty() {
            self.printed_size = 1;
            self.planned_size = 1;
            self.tokens.clear();
        } else {
            self.update_measurements();
        }
        self.push_unmeasured(Token::Break(Break {
            offset: params.offset,
            blank_space: params.blank_space,
            if_nonempty: params.if_nonempty,
            never_break: params.never_break,
            size: -self.planned_size,
        }));
        self.planned_size += params.blank_space as isize;
    }

    pub(crate) fn literal(&mut self, text: &'a str) {
        if self.unmeasured.is_empty() {
            self.printer.literal(&text);
            return;
        }

        let len = text.len() as isize;
        self.tokens
            .push(Token::Literal(Literal { size: len, text }));

        self.planned_size += len;
        self.break_if_overflow();
    }

    fn break_if_overflow(&mut self) {
        while self.planned_size - self.printed_size > self.printer.size_budget {
            // We know that the content overflows, and if there is a chance to
            // break a group or turn a break into a line break, do it by
            // assigning infinite size to the unmeasured token.
            if !self.tokens.first_mut().is_measured() {
                self.unmeasured.pop_front().unwrap();
                self.tokens.first_mut().set_infinite_size();
            }

            self.print_measured_tokens();

            if self.tokens.is_empty() {
                break;
            }
        }
    }

    fn print_measured_tokens(&mut self) {
        dbg!(&self);

        while self.tokens.first().is_measured() {
            let token = self.tokens.pop_first();

            match token {
                Token::Literal(literal) => {
                    self.printed_size += literal.size;
                    self.printer.literal(&literal.text);
                }
                Token::Break(token) => {
                    self.printed_size += token.blank_space as isize;
                    self.printer.break_(token, token.size);
                }
                Token::Begin(token) => self.printer.begin(&token, token.size),
                Token::End => self.printer.end(),
            }

            if self.tokens.is_empty() {
                break;
            }
        }

        dbg!(&self.printer.output);
    }

    fn update_measurements(&mut self) {
        let mut depth: usize = 0;
        while let Some(&index) = self.unmeasured.back() {
            let token = &mut self.tokens[index];
            match token {
                Token::Begin(token) => {
                    if depth == 0 {
                        break;
                    }
                    self.unmeasured.pop_back().unwrap();
                    token.size += self.planned_size;
                    depth -= 1;
                }
                Token::End => {
                    self.unmeasured.pop_back().unwrap();
                    depth += 1;
                }
                Token::Break(token) => {
                    self.unmeasured.pop_back().unwrap();
                    token.size += self.planned_size;
                    if depth == 0 {
                        break;
                    }
                }
                Token::Literal(_) => unreachable!(),
            }
        }
    }
}
