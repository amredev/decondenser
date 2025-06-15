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
mod ring;

use crate::Decondenser;
use ring::RingBuffer;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::{cmp, iter};

#[cfg_attr(decondenser_debug_impls, derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Breaks {
    Consistent,
    Inconsistent,
}

#[cfg_attr(decondenser_debug_impls, derive(Debug))]
#[derive(Clone, Copy, Default)]
pub(crate) struct BreakToken<'a> {
    pub(crate) offset: isize,
    pub(crate) blank_space: usize,
    pub(crate) pre_break: Option<char>,
    pub(crate) post_break: &'a str,
    pub(crate) no_break: Option<char>,
    pub(crate) if_nonempty: bool,
    pub(crate) never_break: bool,
}

#[cfg_attr(decondenser_debug_impls, derive(Debug))]
pub(crate) struct BeginToken {
    pub(crate) offset: isize,
    pub(crate) breaks: Breaks,
}

#[cfg_attr(decondenser_debug_impls, derive(Debug))]
pub(crate) enum Token<'a> {
    String(Cow<'a, str>),
    Break(BreakToken<'a>),
    Begin(BeginToken),
    End,
}

#[cfg_attr(decondenser_debug_impls, derive(Debug))]
#[derive(Copy, Clone)]
enum PrintFrame {
    Fits(Breaks),
    Broken { offset: usize, breaks: Breaks },
}

const SIZE_INFINITY: isize = 0xffff;

/// Every line is allowed at least this much space, even if highly indented.
const MIN_SPACE: isize = 60;

#[cfg_attr(decondenser_debug_impls, derive(Debug))]
pub(crate) struct Printer<'a> {
    /// Constant
    max_line_width: usize,

    out: String,

    /// Number of spaces left on line
    space: isize,

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

    /// Stack of groups-in-progress being flushed by print
    print_stack: Vec<PrintFrame>,

    /// Level of indentation of current line
    indent: usize,

    /// Buffered indentation to avoid writing trailing whitespace
    pending_indent: usize,
}

#[cfg_attr(decondenser_debug_impls, derive(Debug))]
struct BufEntry<'a> {
    token: Token<'a>,
    size: isize,
}

impl<'a> Printer<'a> {
    pub(crate) fn new(config: &Decondenser) -> Self {
        Printer {
            max_line_width: config.max_line_width,
            out: String::new(),
            space: config.max_line_width.try_into().unwrap_or(SIZE_INFINITY),
            buf: RingBuffer::new(),
            left_total: 0,
            right_total: 0,
            scan_deque: VecDeque::new(),
            print_stack: Vec::new(),
            indent: 0,
            pending_indent: 0,
        }
    }

    pub(crate) fn eof(mut self) -> String {
        if !self.scan_deque.is_empty() {
            self.check_stack(0);
            self.advance_left();
        }
        self.out
    }

    pub(crate) fn scan_begin(&mut self, token: BeginToken) {
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
            self.print_end();
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

    pub(crate) fn scan_break(&mut self, token: BreakToken<'a>) {
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
            self.print_string(string);
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
        self.out.ends_with(ch)
    }

    fn check_stream(&mut self) {
        while self.right_total - self.left_total > self.space {
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
                    self.print_string(string);
                }
                Token::Break(token) => {
                    self.left_total += token.blank_space as isize;
                    self.print_break(token, left.size);
                }
                Token::Begin(token) => self.print_begin(token, left.size),
                Token::End => self.print_end(),
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

    fn get_top(&self) -> PrintFrame {
        self.print_stack
            .last()
            .copied()
            .unwrap_or(PrintFrame::Broken(0, Breaks::Inconsistent))
    }

    fn print_begin(&mut self, token: BeginToken, size: isize) {
        if cfg!(decondenser_debug) {
            self.out.push(match token.breaks {
                Breaks::Consistent => '«',
                Breaks::Inconsistent => '‹',
            });
            if cfg!(decondenser_debug_indent) {
                self.out
                    .extend(token.offset.to_string().chars().map(|ch| match ch {
                        '0'..='9' => ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉']
                            [(ch as u8 - b'0') as usize],
                        '-' => '₋',
                        _ => unreachable!(),
                    }));
            }
        }
        if size > self.space {
            self.print_stack
                .push(PrintFrame::Broken(self.indent, token.breaks));
            self.indent = usize::try_from(self.indent as isize + token.offset).unwrap();
        } else {
            self.print_stack.push(PrintFrame::Fits(token.breaks));
        }
    }

    fn print_end(&mut self) {
        let breaks = match self.print_stack.pop().unwrap() {
            PrintFrame::Broken(indent, breaks) => {
                self.indent = indent;
                breaks
            }
            PrintFrame::Fits(breaks) => breaks,
        };
        if cfg!(decondenser_debug) {
            self.out.push(match breaks {
                Breaks::Consistent => '»',
                Breaks::Inconsistent => '›',
            });
        }
    }

    fn print_break(&mut self, token: BreakToken, size: isize) {
        let fits = token.never_break
            || match self.get_top() {
                PrintFrame::Fits(..) => true,
                PrintFrame::Broken(.., Breaks::Consistent) => false,
                PrintFrame::Broken(.., Breaks::Inconsistent) => size <= self.space,
            };
        if fits {
            self.pending_indent += token.blank_space;
            self.space -= token.blank_space as isize;
            if let Some(no_break) = token.no_break {
                self.out.push(no_break);
                self.space -= no_break.len_utf8() as isize;
            }
            if cfg!(decondenser_debug) {
                self.out.push('·');
            }
        } else {
            if let Some(pre_break) = token.pre_break {
                self.print_indent();
                self.out.push(pre_break);
            }
            if cfg!(decondenser_debug) {
                self.out.push('·');
            }
            self.out.push('\n');
            let indent = self.indent as isize + token.offset;
            self.pending_indent = usize::try_from(indent).unwrap();
            self.space = cmp::max(self.max_line_width as isize - indent, MIN_SPACE);
            if !token.post_break.is_empty() {
                self.print_indent();
                self.out.push_str(token.post_break);
                self.space -= token.post_break.len() as isize;
            }
        }
    }

    fn print_string(&mut self, string: Cow<'a, str>) {
        self.print_indent();
        self.out.push_str(&string);
        self.space -= string.len() as isize;
    }

    fn print_indent(&mut self) {
        self.out.extend(iter::repeat_n(' ', self.pending_indent));
        self.pending_indent = 0;
    }
}
