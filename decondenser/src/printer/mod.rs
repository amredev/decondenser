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
/// Note the logic of beaking is optional. It only takes place if the content of
/// the group can not fit on a single line. If it does fit - it won't be broken.
#[derive(Debug, Clone, Copy, PartialEq)]
enum BreakKind {
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
    break_kind: BreakKind,
}

#[derive(Debug)]
enum Token<'a> {
    String(Cow<'a, str>),
    Break(BreakToken),
    Begin(BeginToken),
    End,
}

const SIZE_INFINITY: isize = 0xffff;

/// Every line is allowed at least this much space, even if highly indented.
const MIN_SPACE: isize = 60;

#[derive(Debug)]
pub(crate) struct Printer<'a> {
    /// Ring-buffer of tokens and calculated sizes
    buf: RingBuffer<BufEntry<'a>>,

    /// Total size of tokens already printed
    left_total: isize,

    /// Total size of tokens enqueued, including printed and not yet printed
    right_total: isize,

    /// Holds the ring-buffer index of the Begin that started the current block,
    /// possibly with the most recent Break after that Begin (if there is any) on
    /// top of it. Values are pushed and popped on the back of the queue using it
    /// like stack, and elsewhere old values are popped from the front of the
    /// queue as they become irrelevant due to the primary ring-buffer advancing.
    scan_deque: VecDeque<usize>,

    renderer: Renderer,
}

#[derive(Debug)]
struct BufEntry<'a> {
    token: Token<'a>,
    size: isize,
}

impl<'a> Printer<'a> {
    pub(crate) fn new(config: &Decondenser<'_>) -> Self {
        Printer {
            buf: RingBuffer::new(),
            left_total: 0,
            right_total: 0,
            scan_deque: VecDeque::new(),
            renderer: Renderer::new(config),
        }
    }

    pub(crate) fn eof(mut self) -> String {
        if !self.scan_deque.is_empty() {
            self.check_stack(0);
            self.advance_left();
        }
        self.renderer.output
    }

    fn scan_begin(&mut self, token: BeginToken) {
        if self.scan_deque.is_empty() {
            self.left_total = 1;
            self.right_total = 1;
            self.buf.clear();
        }
        let right = self.buf.push(BufEntry {
            token: Token::Begin(token),
            size: -self.right_total,
        });
        self.scan_deque.push_back(right);
    }

    pub(crate) fn scan_end(&mut self) {
        if self.scan_deque.is_empty() {
            self.renderer.print_end();
            return;
        }

        if !self.buf.is_empty() {
            if let Token::Break(break_token) = self.buf.last().token {
                if self.buf.len() >= 2 {
                    if let Token::Begin(_) = self.buf.second_last().token {
                        self.buf.pop_last();
                        self.buf.pop_last();
                        self.scan_deque.pop_back();
                        self.scan_deque.pop_back();
                        self.right_total -= break_token.blank_space as isize;
                        return;
                    }
                }
                if break_token.if_nonempty {
                    self.buf.pop_last();
                    self.scan_deque.pop_back();
                    self.right_total -= break_token.blank_space as isize;
                }
            }
        }

        let right = self.buf.push(BufEntry {
            token: Token::End,
            size: -1,
        });

        self.scan_deque.push_back(right);
    }

    fn scan_break(&mut self, token: BreakToken) {
        if self.scan_deque.is_empty() {
            self.left_total = 1;
            self.right_total = 1;
            self.buf.clear();
        } else {
            self.check_stack(0);
        }
        let right = self.buf.push(BufEntry {
            token: Token::Break(token),
            size: -self.right_total,
        });
        self.scan_deque.push_back(right);
        self.right_total += token.blank_space as isize;
    }

    pub(crate) fn scan_string(&mut self, string: Cow<'a, str>) {
        if self.scan_deque.is_empty() {
            self.renderer.print_string(&string);
        } else {
            let len = string.len() as isize;
            self.buf.push(BufEntry {
                token: Token::String(string),
                size: len,
            });
            self.right_total += len;
            self.check_stream();
        }
    }

    #[track_caller]
    pub(crate) fn offset(&mut self, offset: isize) {
        match &mut self.buf.last_mut().token {
            Token::Break(token) => token.offset += offset,
            Token::Begin(_) => {}
            Token::String(_) | Token::End => unreachable!(),
        }
    }

    pub(crate) fn end_with_max_width(&mut self, max: isize) {
        let mut depth = 1;
        for &index in self.scan_deque.iter().rev() {
            let entry = &self.buf[index];
            match entry.token {
                Token::Begin(_) => {
                    depth -= 1;
                    if depth == 0 {
                        if entry.size < 0 {
                            let actual_width = entry.size + self.right_total;
                            if actual_width > max {
                                self.buf.push(BufEntry {
                                    token: Token::String(Cow::Borrowed("")),
                                    size: SIZE_INFINITY,
                                });
                                self.right_total += SIZE_INFINITY;
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
        for i in self.buf.index_range().rev() {
            if let Token::String(token) = &self.buf[i].token {
                return token.ends_with(ch);
            }
        }
        self.renderer.output.ends_with(ch)
    }

    fn check_stream(&mut self) {
        while self.right_total - self.left_total > self.renderer.space {
            if *self.scan_deque.front().unwrap() == self.buf.index_range().start {
                self.scan_deque.pop_front().unwrap();
                self.buf.first_mut().size = SIZE_INFINITY;
            }

            self.advance_left();

            if self.buf.is_empty() {
                break;
            }
        }
    }

    fn advance_left(&mut self) {
        while self.buf.first().size >= 0 {
            let left = self.buf.pop_first();

            match left.token {
                Token::String(string) => {
                    self.left_total += left.size;
                    self.renderer.print_string(&string);
                }
                Token::Break(token) => {
                    self.left_total += token.blank_space as isize;
                    self.renderer.print_break(token, left.size);
                }
                Token::Begin(token) => self.renderer.print_begin(&token, left.size),
                Token::End => self.renderer.print_end(),
            }

            if self.buf.is_empty() {
                break;
            }
        }
    }

    fn check_stack(&mut self, mut depth: usize) {
        while let Some(&index) = self.scan_deque.back() {
            let entry = &mut self.buf[index];
            match entry.token {
                Token::Begin(_) => {
                    if depth == 0 {
                        break;
                    }
                    self.scan_deque.pop_back().unwrap();
                    entry.size += self.right_total;
                    depth -= 1;
                }
                Token::End => {
                    self.scan_deque.pop_back().unwrap();
                    entry.size = 1;
                    depth += 1;
                }
                Token::Break(_) => {
                    self.scan_deque.pop_back().unwrap();
                    entry.size += self.right_total;
                    if depth == 0 {
                        break;
                    }
                }
                Token::String(_) => unreachable!(),
            }
        }
    }
}
