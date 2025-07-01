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

mod measured_str;
mod printer;
mod sliding_deque;
mod token;

pub(crate) use crate::BreakStyle;
pub(super) use measured_str::MeasuredStr;

use self::printer::Printer;
use self::sliding_deque::SlidingDeque;
use self::token::Token;
use crate::Decondenser;
use crate::utils::debug_panic;
use printer::PrinterConfig;
use std::collections::VecDeque;
use std::fmt;
use token::{Measurement, Size};

/// A primitive generic formatter that works in terms of a [`Token`] that has
/// groups, breaks, indent and raw text. It ingests the [`Token`]s in time that
/// is linear to their number and using the space that is linear to the maximum
/// size of the line. Technically it doesn't need to buffer the entire output
/// string, but it does so just in the sake of simplicity.
///
/// There are two component parts to the formatter:
/// - Calculating the single-line size of the tokens
/// - Printing the measured tokens to the output using their size to decide
///   where to place line breaks.
///
/// The measurement logic lives in this file and it drives the printing logic
/// that is implemented in the [`Printer`] type.
#[derive(Debug)]
pub(crate) struct Formatter<'a> {
    /// A "sliding" deque of tokens and sizes that are candidates for the next
    /// line. See the [`SlidingDeque`] docs for more details on how this differs
    /// from a regular [`VecDeque`].
    tokens: SlidingDeque<Token<'a>>,

    /// Holds indices of [`Token::Begin`] and [`Token::Space`] tokens that are
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

    printer: Printer<'a>,
}

impl<'a> Formatter<'a> {
    pub(crate) fn new(config: &'a Decondenser) -> Self {
        Formatter {
            tokens: SlidingDeque::new(),
            printed_single_line_size: 0,
            total_single_line_size: 0,
            unmeasured_indices: VecDeque::new(),
            printer: Printer::new(PrinterConfig {
                max_line_size: config.max_line_size,
                no_break_size: config.no_break_size.unwrap_or(config.max_line_size / 2),
                debug_layout: config.debug_layout,
                debug_indent: config.debug_indent,
                indent_str: MeasuredStr::new(&config.indent, config.visual_size),
            }),
        }
    }

    pub(crate) fn eof(mut self) -> String {
        if !self.unmeasured_indices.is_empty() {
            self.measure_tokens();
            self.print_measured_tokens();
        }
        self.printer.eof()
    }

    fn push_unmeasured(&mut self, token: Token<'a>) {
        let index = self.tokens.push_back(token);
        self.unmeasured_indices.push_back(index);
    }

    pub(crate) fn begin(&mut self, break_style: BreakStyle) {
        self.push_unmeasured(Token::Begin {
            next_space_distance: Measurement::Unmeasured {
                preceding_tokens_size: self.total_single_line_size,
            },
            break_style,
        });
    }

    pub(crate) fn indent(&mut self, diff: isize) {
        self.tokens.push_back(Token::Indent(diff));
    }

    pub(crate) fn end(&mut self) {
        if self.unmeasured_indices.is_empty() {
            self.printer.end();
            return;
        }

        let mut tokens = self.tokens.iter();

        // Special case for a `Begin Space End` sequence. In this case, we just
        // can just remove it entirely, since the group is empty.
        if let Some(&Token::Bsp { size, .. }) = tokens.next_back() {
            if let Some(Token::Begin { .. }) = tokens.next_back() {
                self.tokens.pop_back();
                self.tokens.pop_back();
                self.unmeasured_indices.pop_back();
                self.unmeasured_indices.pop_back();
                self.total_single_line_size -= size;
                return;
            }
        }

        self.push_unmeasured(Token::End);
    }

    pub(crate) fn newline(&mut self, count: usize) {
        self.measure_tokens();
        self.break_while(|_| true);
        self.printer.newline(count);
    }

    pub(crate) fn bsp(&mut self, size: usize) {
        self.measure_tokens();
        self.push_unmeasured(Token::Bsp {
            size,
            next_space_distance: Measurement::Unmeasured {
                preceding_tokens_size: self.total_single_line_size,
            },
        });
        self.total_single_line_size += size;
    }

    pub(crate) fn nbsp(&mut self, count: usize) {
        if self.unmeasured_indices.is_empty() {
            self.printer.nbsp(count);
            return;
        }

        self.tokens.push_back(Token::Nbsp(count));
        self.total_single_line_size += count;
        self.break_while_overflows();
    }

    pub(crate) fn raw(&mut self, content: MeasuredStr<'a>) {
        if self.unmeasured_indices.is_empty() {
            self.printer.raw(content);
            return;
        }

        self.tokens.push_back(Token::Raw(content));
        self.total_single_line_size += content.visual_size();
        self.break_while_overflows();
    }

    fn break_while_overflows(&mut self) {
        self.break_while(|fmt| {
            let pending_size = fmt.total_single_line_size - fmt.printed_single_line_size;
            pending_size > fmt.printer.line_size_budget()
        });
    }

    /// Flush tokens assigning "infinite size" to the unmeasured tokens to force
    /// the line breaks for those.
    fn break_while(&mut self, condition: fn(&Self) -> bool) {
        loop {
            if !condition(self) {
                return;
            }

            let Some(token) = self.tokens.front_mut() else {
                return;
            };

            // We know that the content overflows, and if there is a chance to
            // break a group or turn a space into a line break, do it by
            // assigning infinite size to the unmeasured token.
            if let Token::Bsp {
                next_space_distance,
                ..
            }
            | Token::Begin {
                next_space_distance,
                ..
            } = token
            {
                if let Measurement::Unmeasured { .. } = next_space_distance {
                    *next_space_distance = Measurement::Measured(Size::Infinite);
                }
            }

            if self.unmeasured_indices.front() == Some(&self.tokens.basis()) {
                self.unmeasured_indices.pop_front();
            }

            self.print_measured_tokens();
        }
    }

    fn print_measured_tokens(&mut self) {
        dbg!(&self);
        // todo!(
        //     "figure out space squashing, line_size_budget changes for nbsp/newline; \
        //     preserve whitespace around punctuation as much as possible default in generic \
        //     config; i.e. use `Option` for space config"
        // );

        debug_assert_ne!(self.tokens.len(), 0);

        while let Some(&token) = self.tokens.front() {
            match token {
                Token::Raw(content) => {
                    self.printed_single_line_size += content.visual_size();
                    self.printer.raw(content);
                }
                Token::Nbsp(count) => {
                    self.printed_single_line_size += count;
                    self.printer.nbsp(count);
                }
                Token::Bsp {
                    next_space_distance,
                    size,
                } => {
                    let Measurement::Measured(distance) = next_space_distance else {
                        return;
                    };
                    self.printed_single_line_size += size;
                    self.printer.bsp(size, distance);
                }
                Token::Begin {
                    next_space_distance,
                    break_style,
                } => {
                    let Measurement::Measured(distance) = next_space_distance else {
                        return;
                    };
                    self.printer.begin(break_style, distance);
                }
                Token::Indent(diff) => {
                    self.printer.indent(diff);
                }
                Token::End => {
                    if self.unmeasured_indices.front() == Some(&self.tokens.basis()) {
                        // This `End` is still staged for its group measurement,
                        // we can't print it yet.
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
        let mut cursor = self.unmeasured_indices.len();

        while let Some(new_cursor) = cursor.checked_sub(1) {
            cursor = new_cursor;

            let Some(&index) = self.unmeasured_indices.get(cursor) else {
                debug_panic!("Unmeasured token index {cursor} is out of bounds");
                return;
            };

            let mut remove_unmeasured = || {
                let index = self.unmeasured_indices.remove(cursor);
                debug_assert_ne!(index, None);
            };

            let Some(token) = self.tokens.get_mut(index) else {
                debug_panic!("Unmeasured token index {index} is out of bounds");
                remove_unmeasured();
                continue;
            };

            match token {
                Token::Begin {
                    next_space_distance,
                    ..
                } => {
                    if depth == 0 {
                        // If we are on the first iteration, we shouldn't stop
                        // measuring tokens - this is the first break/eof of
                        // the block, that marks the end of measurement of the
                        // previous break of the parent block if there is one.
                        if cursor + 1 == self.unmeasured_indices.len() {
                            continue;
                        }
                        return;
                    }
                    remove_unmeasured();
                    next_space_distance.measure_from(self.total_single_line_size);
                    depth -= 1;
                }
                Token::End => {
                    remove_unmeasured();
                    depth += 1;
                }
                Token::Bsp {
                    next_space_distance,
                    ..
                } => {
                    remove_unmeasured();
                    next_space_distance.measure_from(self.total_single_line_size);
                    if depth == 0 {
                        return;
                    }
                }
                Token::Raw(_) | Token::Nbsp(_) | Token::Indent(_) => {
                    debug_panic!(
                        "This token should never have been part of unmeasured \
                        token indices: {token:?}"
                    );
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
                Token::Begin { .. } => {
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
