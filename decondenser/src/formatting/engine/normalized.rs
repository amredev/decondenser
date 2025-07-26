use super::printer::{Printer, PrinterConfig};
use super::sliding_deque::SlidingDeque;
use super::token::{Measurement, Size, Token};
use crate::Decondenser;
use crate::formatting::BreakStyle;
use crate::utils::debug_panic;
use crate::visual_size::MeasuredStr;
use std::collections::VecDeque;
use std::fmt;

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
pub(super) struct NormalizedFormatter<'a> {
    tokens: Tokens<'a>,

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

#[derive(Debug)]
struct Tokens<'a> {
    /// A "sliding" deque of tokens and sizes that are candidates for the next
    /// line. See the [`SlidingDeque`] docs for more details on how this differs
    /// from a regular [`VecDeque`].
    deque: SlidingDeque<Token<'a>>,

    /// Holds indices of [`Token::Begin`] and [`Token::Space`] tokens that are
    /// not yet measured. Also, includes [`Token::End`] tokens so that we can
    /// track the levels of nesting when traversing this buffer for measurement.
    unmeasured: VecDeque<usize>,
}

impl<'a> Tokens<'a> {
    fn push(&mut self, token: Token<'a>) {
        self.deque.push_back(token);
    }

    fn push_unmeasured(&mut self, token: Token<'a>) {
        let index = self.deque.push_back(token);
        self.unmeasured.push_back(index);
    }

    fn starts_with_unmeasured(&self) -> bool {
        self.unmeasured.front() == Some(&self.deque.basis())
    }
}

impl<'a> NormalizedFormatter<'a> {
    pub(super) fn new(config: &'a Decondenser) -> Self {
        NormalizedFormatter {
            tokens: Tokens {
                deque: SlidingDeque::new(),
                unmeasured: VecDeque::new(),
            },
            printed_single_line_size: 0,
            total_single_line_size: 0,
            printer: Printer::new(PrinterConfig {
                max_line_size: config.max_line_size,
                no_break_size: config.no_break_size.unwrap_or(config.max_line_size / 2),
                debug_layout: config.debug_layout,
                debug_indent: config.debug_indent,
                indent_str: config.visual_size.measured_str(&config.indent),
            }),
        }
    }

    /// End of input
    pub(super) fn eoi(mut self) -> String {
        if !self.tokens.unmeasured.is_empty() {
            self.measure_tokens();
            self.print_measured_tokens();
        }
        self.printer.finish()
    }

    pub(super) fn begin(&mut self, break_style: BreakStyle) {
        self.tokens.push_unmeasured(Token::Begin {
            break_style,
            next_break_distance: Measurement::Unmeasured {
                preceding_tokens_size: self.total_single_line_size,
            },
        });
    }

    pub(super) fn indent(&mut self, diff: isize) {
        self.tokens.push(Token::Indent(diff));
    }

    pub(super) fn end(&mut self) {
        self.tokens.push_unmeasured(Token::End);
    }

    pub(super) fn hard_break(&mut self, size: usize) {
        self.measure_tokens();
        self.break_while(|_| true);
        self.printer.hard_break(size);
    }

    pub(super) fn soft_break(&mut self) {
        self.measure_tokens();
        self.tokens.push_unmeasured(Token::SoftBreak {
            next_break_distance: Measurement::Unmeasured {
                preceding_tokens_size: self.total_single_line_size,
            },
        });
    }

    pub(super) fn space(&mut self, size: usize) {
        self.tokens.push(Token::Space(size));
        self.total_single_line_size += size;
        self.break_while_overflows();
    }

    pub(super) fn raw(&mut self, content: MeasuredStr<'a>) {
        self.tokens.push(Token::Raw(content));
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

            let Some(token) = self.tokens.deque.front_mut() else {
                return;
            };

            // We know that the content overflows, and if there is a chance to
            // break a group or turn a space into a line break, do it by
            // assigning infinite size to the unmeasured token.
            if let Token::SoftBreak {
                next_break_distance,
                ..
            }
            | Token::Begin {
                next_break_distance,
                ..
            } = token
            {
                if let Measurement::Unmeasured { .. } = next_break_distance {
                    *next_break_distance = Measurement::Measured(Size::Infinite);
                }
            }

            if self.tokens.starts_with_unmeasured() {
                self.tokens.unmeasured.pop_front();
            }

            self.print_measured_tokens();
        }
    }

    fn print_measured_tokens(&mut self) {
        debug_assert_ne!(self.tokens.deque.len(), 0);

        while let Some(&token) = self.tokens.deque.front() {
            match token {
                Token::Raw(content) => {
                    self.printed_single_line_size += content.visual_size();
                    self.printer.raw(content);
                }
                Token::Space(size) => {
                    self.printed_single_line_size += size;
                    self.printer.space(size);
                }
                Token::SoftBreak {
                    next_break_distance,
                } => {
                    let Measurement::Measured(distance) = next_break_distance else {
                        return;
                    };
                    self.printer.soft_break(distance);
                }
                Token::Begin {
                    next_break_distance,
                    break_style,
                } => {
                    let Measurement::Measured(distance) = next_break_distance else {
                        return;
                    };
                    self.printer.begin(break_style, distance);
                }
                Token::Indent(diff) => self.printer.indent(diff),
                Token::End => {
                    if self.tokens.starts_with_unmeasured() {
                        // This `End` is still staged for its group measurement,
                        // we can't print it yet.
                        return;
                    }

                    self.printer.end();
                }
            }

            self.tokens.deque.pop_front();
        }
    }

    fn measure_tokens(&mut self) {
        let mut depth: usize = 0;
        let mut cursor = self.tokens.unmeasured.len();

        while let Some(new_cursor) = cursor.checked_sub(1) {
            cursor = new_cursor;

            let Some(&index) = self.tokens.unmeasured.get(cursor) else {
                debug_panic!("Unmeasured token index {cursor} is out of bounds");
                return;
            };

            let mut remove_unmeasured = || {
                let index = self.tokens.unmeasured.remove(cursor);
                debug_assert_ne!(index, None);
            };

            let Some(token) = self.tokens.deque.get_mut(index) else {
                debug_panic!("Unmeasured token index {index} is out of bounds");
                remove_unmeasured();
                continue;
            };

            match token {
                Token::Begin {
                    next_break_distance,
                    ..
                } => {
                    if depth == 0 {
                        // If we are on the first iteration, we shouldn't stop
                        // measuring tokens - this is the first break/eof of
                        // the block, that marks the end of measurement of the
                        // previous break of the parent block if there is one.
                        if cursor + 1 == self.tokens.unmeasured.len() {
                            continue;
                        }
                        return;
                    }
                    remove_unmeasured();
                    next_break_distance.measure_from(self.total_single_line_size);
                    depth -= 1;
                }
                Token::End => {
                    remove_unmeasured();
                    depth += 1;
                }
                Token::SoftBreak {
                    next_break_distance,
                } => {
                    remove_unmeasured();
                    next_break_distance.measure_from(self.total_single_line_size);
                    if depth == 0 {
                        return;
                    }
                }
                Token::Raw(_) | Token::Space(_) | Token::Indent(_) => {
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
                _ => writeln!(f, "[{i:>2}] {:indent$}{entry:?}", "")?,
            }
        }

        Ok(())
    }
}
