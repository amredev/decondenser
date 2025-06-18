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

use crate::Decondenser;
use renderer::Renderer;
use ring::RingBuffer;
use std::borrow::Cow;
use std::collections::VecDeque;

/// Sets the algorithm used to decide whether to turn a given [`Token::Break`]
/// into a line break or not. The examples below are based on this input:
///
/// ```ignore
/// foo(aaa, bbb, ccc, ddd);
/// ```
///
/// Note that beaking is optional. It only takes place if the content of the
/// group can not fit on a single line. If it does fit - it won't be broken
/// disregarding the [`BreaksKind`].
#[derive(Debug, Clone, Copy, PartialEq)]
enum BreaksKind {
    /// Turn **all** breaks into a line break.
    ///
    /// ```ignore
    /// foo(
    ///     aaaa,
    ///     bbb,
    ///     ccc,
    ///     ddd
    /// );
    /// ```
    Consistent,

    /// Try to fit as much content as possible on a single line and create a
    /// newline only for the last break on the line after which the content
    /// would overflow.
    ///
    /// ```ignore
    /// foo(
    ///     aaaa, bbb,
    ///     ccc, ddd
    /// );
    /// ```
    Inconsistent,
}

#[derive(Debug, Clone, Copy, Default)]
struct BreakToken {
    offset: isize,
    blank_space: usize,
    if_nonempty: bool,
    never_break: bool,
}

#[derive(Debug)]
struct BeginToken {
    offset: isize,
    breaks_kind: BreaksKind,
}

#[derive(Debug)]
enum Token<'a> {
    String(Cow<'a, str>),
    Break(BreakToken),
    Begin(BeginToken),
    End,
}

const SIZE_INFINITY: isize = 0xffff;

#[derive(Debug)]
pub(crate) struct Printer<'a> {
    /// Ring-buffer of tokens and sizes that are candidates for the next
    /// line.
    tokens: RingBuffer<BufEntry<'a>>,

    /// Holds the ring-buffer index of the Begin that started the current block,
    /// possibly with the most recent Break after that Begin (if there is any) on
    /// top of it. Values are pushed and popped on the back of the queue using it
    /// like stack, and elsewhere old values are popped from the front of the
    /// queue as they become irrelevant due to the primary ring-buffer advancing.
    unmeasured: VecDeque<usize>,

    /// Size only of tokens that were already printed
    left_size: isize,

    /// Size of tokens enqueued, including already printed and not yet printed
    right_size: isize,

    renderer: Renderer,
}

#[derive(Debug)]
struct BufEntry<'a> {
    token: Token<'a>,

    /// Negative size means we don't know yet what size to assign to the token.
    ///
    /// The size should eventually be set according to these rules:
    ///
    /// - [`Token::Begin`] - the sum of sizes of all tokens between it and the
    ///   next [`Token::End`]
    ///
    /// - [`Token::Break`] - the size of the next [`Token::String`] or
    ///
    ///
    size: isize,
}

impl<'a> Printer<'a> {
    pub(crate) fn new(config: &Decondenser<'_>) -> Self {
        Printer {
            tokens: RingBuffer::new(),
            left_size: 0,
            right_size: 0,
            unmeasured: VecDeque::new(),
            renderer: Renderer::new(config),
        }
    }

    pub(crate) fn eof(mut self) -> String {
        if !self.unmeasured.is_empty() {
            self.update_measurements();
            self.flush_buffer();
        }
        self.renderer.output
    }

    fn scan_begin(&mut self, token: BeginToken) {
        if self.unmeasured.is_empty() {
            self.left_size = 1;
            self.right_size = 1;
            self.tokens.clear();
        }
        let right = self.tokens.push(BufEntry {
            token: Token::Begin(token),
            size: -self.right_size,
        });
        self.unmeasured.push_back(right);
    }

    pub(crate) fn scan_end(&mut self) {
        if self.unmeasured.is_empty() {
            self.renderer.print_end();
            return;
        }

        if let Some(&Token::Break(break_token)) = self.tokens.last().map(|entry| &entry.token) {
            if let Some(Token::Begin(_)) = self.tokens.second_last().map(|entry| &entry.token) {
                self.tokens.pop_last();
                self.tokens.pop_last();
                self.unmeasured.pop_back();
                self.unmeasured.pop_back();
                self.right_size -= break_token.blank_space as isize;
                return;
            }

            if break_token.if_nonempty {
                self.tokens.pop_last();
                self.unmeasured.pop_back();
                self.right_size -= break_token.blank_space as isize;
            }
        }

        let right = self.tokens.push(BufEntry {
            token: Token::End,
            size: -1,
        });

        self.unmeasured.push_back(right);
    }

    fn scan_break(&mut self, token: BreakToken) {
        if self.unmeasured.is_empty() {
            self.left_size = 1;
            self.right_size = 1;
            self.tokens.clear();
        } else {
            self.update_measurements();
        }
        let right = self.tokens.push(BufEntry {
            token: Token::Break(token),

            // TODO(assumption?):
            // When break turns into a line break, then it shifts the "caret"
            // back to the beginning of the block
            size: -self.right_size,
        });
        self.unmeasured.push_back(right);
        self.right_size += token.blank_space as isize;
    }

    pub(crate) fn scan_string(&mut self, string: Cow<'a, str>) {
        if self.unmeasured.is_empty() {
            self.renderer.print_string(&string);
            return;
        }

        let len = string.len() as isize;
        self.tokens.push(BufEntry {
            token: Token::String(string),
            size: len,
        });

        self.right_size += len;
        self.break_if_overflow();
    }

    #[track_caller]
    pub(crate) fn offset(&mut self, offset: isize) {
        match &mut self.tokens.last_mut().token {
            Token::Break(token) => token.offset += offset,
            Token::Begin(_) => {}
            Token::String(_) | Token::End => unreachable!(),
        }
    }

    pub(crate) fn end_with_max_width(&mut self, max: isize) {
        let mut depth = 1;
        for &index in self.unmeasured.iter().rev() {
            let entry = &self.tokens[index];
            match entry.token {
                Token::Begin(_) => {
                    depth -= 1;
                    if depth == 0 {
                        if entry.size < 0 {
                            let actual_width = entry.size + self.right_size;
                            if actual_width > max {
                                self.tokens.push(BufEntry {
                                    token: Token::String(Cow::Borrowed("")),
                                    size: SIZE_INFINITY,
                                });
                                self.right_size += SIZE_INFINITY;
                            }
                        }
                        break;
                    }
                }
                Token::End => depth += 1,
                Token::Break(_) => {}
                Token::String(_) => unreachable!(),
            }
        }
        self.scan_end();
    }

    pub(crate) fn ends_with(&self, ch: char) -> bool {
        for i in self.tokens.index_range().rev() {
            if let Token::String(token) = &self.tokens[i].token {
                return token.ends_with(ch);
            }
        }
        self.renderer.output.ends_with(ch)
    }

    fn break_if_overflow(&mut self) {
        // Enter the loop only if the current line is too long
        while self.right_size - self.left_size > self.renderer.line_size_budget {
            // If there is a new group pretend that it's infinite size to trick
            // the renderer into breaking it.
            if *self.unmeasured.front().unwrap() == self.tokens.index_range().start {
                self.unmeasured.pop_front().unwrap();
                self.tokens.first_mut().size = SIZE_INFINITY;
            }

            self.flush_buffer();

            if self.tokens.is_empty() {
                break;
            }
        }
    }

    fn flush_buffer(&mut self) {
        while self.tokens.first().size >= 0 {
            let left = self.tokens.pop_first();

            match left.token {
                Token::String(string) => {
                    self.left_size += left.size;
                    self.renderer.print_string(&string);
                }
                Token::Break(token) => {
                    self.left_size += token.blank_space as isize;
                    self.renderer.print_break(token, left.size);
                }
                Token::Begin(token) => self.renderer.print_begin(&token, left.size),
                Token::End => self.renderer.print_end(),
            }

            if self.tokens.is_empty() {
                break;
            }
        }
    }

    fn update_measurements(&mut self) {
        let mut depth: usize = 0;
        while let Some(&index) = self.unmeasured.back() {
            let entry = &mut self.tokens[index];
            match entry.token {
                Token::Begin(_) => {
                    if depth == 0 {
                        break;
                    }
                    self.unmeasured.pop_back().unwrap();
                    entry.size += self.right_size;
                    depth -= 1;
                }
                Token::End => {
                    self.unmeasured.pop_back().unwrap();
                    entry.size = 1;
                    depth += 1;
                }
                Token::Break(_) => {
                    self.unmeasured.pop_back().unwrap();
                    entry.size += self.right_size;
                    if depth == 0 {
                        break;
                    }
                }
                Token::String(_) => unreachable!(),
            }
        }
    }
}
