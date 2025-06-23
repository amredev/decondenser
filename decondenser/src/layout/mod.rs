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

mod printer;
mod sliding_deque;
mod token;

pub(crate) use token::BreaksKind;

use self::printer::Printer;
use self::sliding_deque::SlidingDeque;
use self::token::{Begin, Break, Token};
use crate::Decondenser;
use crate::utils::debug_panic;
use std::collections::VecDeque;
use std::fmt;
use token::{Literal, Size, SizeMeasurement};

#[derive(Debug)]
pub(crate) struct Layout<'a> {
    /// A "sliding" deque of tokens and sizes that are candidates for the next
    /// line. See the [`SlidingDeque`] docs for more details on how this differs
    /// from a regular [`VecDeque`].
    tokens: SlidingDeque<Token<'a>>,

    /// Holds indices of [`Token::Begin`] and [`Token::Break`] tokens that are
    /// not yet measured. Also, includes [`Token::End`] tokens so that we can
    /// track the levels of nesting when traversing this buffer for measurement.
    unmeasured_indices: VecDeque<usize>,

    /// Size of tokens that were already printed.
    ///
    /// The size is calculated as if the tokens were printed on a single line.
    printed_single_line_size: usize,

    /// Size of all [`Self::tokens`] plus the ones that are already printed.
    /// This is guaranteed to be >= [`Self::printed_single_line_size`].
    ///
    /// The size is calculated as if the tokens were printed on a single line.
    total_single_line_size: usize,

    printer: Printer,
}

#[derive(Default, Clone, Copy)]
pub(crate) struct BreakParams {
    pub(crate) indent_diff: isize,
    pub(crate) blank_space: usize,
}

impl<'a> Layout<'a> {
    pub(crate) fn new(config: &Decondenser<'_>) -> Self {
        Layout {
            tokens: SlidingDeque::new(),
            printed_single_line_size: 0,
            total_single_line_size: 0,
            unmeasured_indices: VecDeque::new(),
            printer: Printer::new(config),
        }
    }

    pub(crate) fn eof(mut self) -> String {
        if !self.unmeasured_indices.is_empty() {
            self.measure_tokens();
            self.print_measured_tokens();
        }
        self.printer.output
    }

    fn push_unmeasured(&mut self, token: Token<'a>) {
        let index = self.tokens.push_back(token);
        self.unmeasured_indices.push_back(index);
    }

    pub(crate) fn begin(&mut self, offset: isize, breaks_kind: BreaksKind) {
        self.push_unmeasured(Token::Begin(Begin {
            size: SizeMeasurement::Unmeasured {
                preceding_tokens_size: self.total_single_line_size,
            },
            indent_diff: offset,
            breaks_kind,
        }));
    }

    pub(crate) fn end(&mut self) {
        if self.unmeasured_indices.is_empty() {
            self.printer.end();
            return;
        }

        let mut tokens = self.tokens.iter();

        // Special case for a `Begin Break End` sequence. In this case, we just
        // can just remove it entirely, since the group is empty.
        if let Some(&Token::Break(Break { blank_space, .. })) = tokens.next_back() {
            if let Some(Token::Begin(_)) = tokens.next_back() {
                self.tokens.pop_back();
                self.tokens.pop_back();
                self.unmeasured_indices.pop_back();
                self.unmeasured_indices.pop_back();
                self.total_single_line_size -= blank_space;
                return;
            }
        }

        self.push_unmeasured(Token::End);
    }

    pub(crate) fn break_(&mut self, params: BreakParams) {
        self.measure_tokens();
        self.push_unmeasured(Token::Break(Break {
            indent_diff: params.indent_diff,
            blank_space: params.blank_space,
            size: SizeMeasurement::Unmeasured {
                preceding_tokens_size: self.total_single_line_size,
            },
        }));
        self.total_single_line_size = self
            .total_single_line_size
            .saturating_add(params.blank_space);
    }

    pub(crate) fn literal(&mut self, text: &'a str) {
        if self.unmeasured_indices.is_empty() {
            self.printer.literal(text);
            return;
        }

        let size = text.len();
        self.tokens
            .push_back(Token::Literal(Literal { size, text }));

        self.total_single_line_size = self.total_single_line_size.saturating_add(size);
        self.break_if_overflow();
    }

    fn break_if_overflow(&mut self) {
        while let Some(token) = self.tokens.front_mut() {
            let staged_size = self.total_single_line_size - self.printed_single_line_size;

            if staged_size <= self.printer.line_size_budget {
                return;
            }

            // We know that the content overflows, and if there is a chance to
            // break a group or turn a break into a line break, do it by
            // assigning infinite size to the unmeasured token.
            if let Token::Break(Break { size, .. }) | Token::Begin(Begin { size, .. }) = token {
                *size = SizeMeasurement::Measured(Size::Infinite);
            }

            if self.unmeasured_indices.front() == Some(&self.tokens.basis()) {
                self.unmeasured_indices.pop_front();
            }

            self.print_measured_tokens();
        }
    }

    fn print_measured_tokens(&mut self) {
        debug_assert_ne!(self.tokens.len(), 0);

        while let Some(token) = self.tokens.front() {
            match token {
                Token::Literal(literal) => {
                    self.printed_single_line_size =
                        self.printed_single_line_size.saturating_add(literal.size);

                    self.printer.literal(literal.text);
                }
                Token::Break(break_) => {
                    let SizeMeasurement::Measured(size) = break_.size else {
                        return;
                    };

                    self.printed_single_line_size = self
                        .printed_single_line_size
                        .saturating_add(break_.blank_space);

                    self.printer.break_(break_, size);
                }
                Token::Begin(begin) => {
                    let SizeMeasurement::Measured(size) = begin.size else {
                        return;
                    };
                    self.printer.begin(begin, size);
                }
                Token::End { .. } => {
                    if self.unmeasured_indices.front() == Some(&self.tokens.basis()) {
                        // This `End` is still staged for measurement, we can't
                        // print it yet.
                        return;
                    }

                    self.printer.end();
                }
            }

            self.tokens.pop_front();
        }
    }

    fn measure_tokens(&mut self) {
        let mut depth: usize = 0;
        while let Some(&index) = self.unmeasured_indices.back() {
            let mut pop_back_unmeasured = || {
                let index = self.unmeasured_indices.pop_back();
                debug_assert_ne!(index, None);
            };

            let Some(token) = self.tokens.get_mut(index) else {
                debug_panic!("Unmeasured token index {index} is out of bounds");
                pop_back_unmeasured();
                continue;
            };

            match token {
                Token::Begin(token) => {
                    if depth == 0 {
                        return;
                    }
                    pop_back_unmeasured();
                    token.size.measure_from(self.total_single_line_size);
                    depth -= 1;
                }
                Token::End => {
                    pop_back_unmeasured();
                    depth += 1;
                }
                Token::Break(token) => {
                    pop_back_unmeasured();
                    token.size.measure_from(self.total_single_line_size);
                    if depth == 0 {
                        return;
                    }
                }
                Token::Literal(_) => {
                    debug_panic!("Literals should never be part of unmeasured token indices");
                }
            }
        }
    }
}

impl fmt::Debug for SlidingDeque<Token<'_>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let basis = self.basis();
        writeln!(f, "basis: {basis}")?;

        let mut indent = 0_usize;
        let indent_size = 2_usize;

        for (entry, i) in self.iter().zip(basis..) {
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
