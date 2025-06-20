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
mod printer;
mod ring;
mod token;

use self::printer::Printer;
use self::ring::RingBuffer;
use self::token::{Begin, Break, Token};
use crate::Decondenser;
use std::collections::VecDeque;
use std::fmt;
use token::{BreaksKind, End, Literal};

const SIZE_INFINITY: isize = 0xffff;

#[derive(Debug)]
pub(crate) struct Layout<'a> {
    /// Ring-buffer of tokens and sizes that are candidates for the next
    /// line.
    tokens: RingBuffer<Token<'a>>,

    /// Holds indices of [`Token::Begin`] and [`Token::Break`] tokens that are
    /// not yet measured, i.e. we don't know the size of the group that was
    /// begun or the size of the next chunk of tokens after the break. Also,
    /// includes [`Token::End`] tokens so that we can track the levels of
    /// nesting when traversing this buffer.
    unmeasured: VecDeque<usize>,

    /// Size of tokens that were already printed
    printed_size: isize,

    /// Size of all [`Self::tokens`] plus the ones that are already printed and
    /// not yet printed. This is guaranteed to be >= [`Self::printed_size`].
    planned_size: isize,

    printer: Printer,
}

impl fmt::Debug for RingBuffer<Token<'_>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let index_range = self.index_range();
        writeln!(f, "index_range: {index_range:?}")?;

        let mut indent = 0usize;
        let indent_size = 2;

        for (entry, i) in self.iter().zip(index_range) {
            match entry {
                Token::Begin(_) => {
                    indent += indent_size;
                    writeln!(f, "[{i:>2}] {:indent$}{entry:?}", "")?;
                    indent += indent_size;
                }
                Token::End { .. } => {
                    indent = indent.saturating_sub(indent_size);
                    writeln!(f, "[{i:>2}] {:indent$}{entry:?}", "")?;
                    indent = indent.saturating_sub(indent_size);
                }
                _ => {
                    writeln!(f, "[{i:>2}] {:indent$}{entry:?}", "")?;
                }
            }
        }

        Ok(())
    }
}

#[derive(Default)]
pub(crate) struct BreakParams {
    pub(crate) offset: isize,
    pub(crate) blank_space: usize,
    pub(crate) if_nonempty: bool,
    pub(crate) never_break: bool,
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
            self.measure();
            self.print_measured_tokens();
        }
        self.printer.output
    }

    fn push_unmeasured(&mut self, token: Token<'a>) {
        let index = self.tokens.push(token);
        self.unmeasured.push_back(index);
    }

    fn reset_tokens(&mut self) {
        self.printed_size = 1;
        self.planned_size = 1;
        self.tokens.clear();
    }

    fn begin(&mut self, offset: isize, breaks_kind: BreaksKind) {
        if self.unmeasured.is_empty() {
            self.reset_tokens();
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

        self.push_unmeasured(Token::End(End { measured: false }));
    }

    pub(crate) fn break_(&mut self, params: BreakParams) {
        if self.unmeasured.is_empty() {
            self.reset_tokens();
        } else {
            self.measure();
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
            self.printer.literal(text);
            return;
        }

        let size = text.len() as isize;
        self.tokens.push(Token::Literal(Literal { size, text }));

        self.planned_size += size;
        self.break_if_overflow();
    }

    fn break_if_overflow(&mut self) {
        while self.planned_size - self.printed_size > self.printer.size_budget {
            // We know that the content overflows, and if there is a chance to
            // break a group or turn a break into a line break, do it by
            // assigning infinite size to the unmeasured token.
            if !self.tokens.first().is_measured() {
                let index = self.unmeasured.pop_front();
                debug_assert_eq!(index, Some(self.tokens.index_range().start));

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
                Token::Break(break_) => {
                    self.printed_size += break_.blank_space as isize;
                    self.printer.break_(break_, break_.size);
                }
                Token::Begin(begin) => self.printer.begin(&begin, begin.size),
                Token::End { .. } => self.printer.end(),
            }

            if self.tokens.is_empty() {
                break;
            }
        }
    }

    fn measure(&mut self) {
        let mut depth: usize = 0;
        while let Some(&index) = self.unmeasured.back() {
            let mut pop_back_unmeasured = || {
                let index = self.unmeasured.pop_back();
                debug_assert_ne!(index, None);
            };

            let token = &mut self.tokens[index];
            match token {
                Token::Begin(token) => {
                    if depth == 0 {
                        return;
                    }
                    pop_back_unmeasured();
                    token.size += self.planned_size;
                    depth -= 1;
                }
                Token::End(token) => {
                    pop_back_unmeasured();
                    token.measured = true;
                    depth += 1;
                }
                Token::Break(token) => {
                    pop_back_unmeasured();
                    token.size += self.planned_size;
                    if depth == 0 {
                        return;
                    }
                }
                Token::Literal(_) => debug_assert!(
                    false,
                    "Literals should never be part of unmeasured token indices"
                ),
            }
        }
    }
}
