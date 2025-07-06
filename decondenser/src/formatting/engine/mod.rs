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
mod norm;
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
use norm::Normalization;
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
    tokens: Tokens<'a>,

    /// Metadata about the last [`Token::Newline`], [`Token::Bsp`] or
    /// [`Token::Nbsp`] in [`Self::tokens`]. Used for delayed size calculation
    /// of blanks to delay printing blanks until all consequent blanks are
    /// accumulated and a new [`Token::Raw`] is received. This way we also avoid
    /// printing trailing blanks at the end of lines.
    norm: Normalization,

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
    fn push_unmeasured(&mut self, token: Token<'a>) {
        let index = self.deque.push_back(token);
        self.unmeasured.push_back(index);
    }

    fn starts_with_unmeasured(&self) -> bool {
        self.unmeasured.front() == Some(&self.deque.basis())
    }
}

impl<'a> Formatter<'a> {
    pub(crate) fn new(config: &'a Decondenser) -> Self {
        Formatter {
            tokens: Tokens {
                deque: SlidingDeque::new(),
                unmeasured: VecDeque::new(),
            },
            printed_single_line_size: 0,
            total_single_line_size: 0,
            norm: Normalization::default(),
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
        self.finish_normalization();
        if !self.tokens.unmeasured.is_empty() {
            self.measure_tokens();
            self.print_measured_tokens();
        }
        self.printer.eof()
    }

    pub(crate) fn begin(&mut self, style: BreakStyle) {
        self.norm.begin(style);
    }

    pub(crate) fn indent(&mut self, diff: isize) {
        self.norm.indent(diff);
    }

    pub(crate) fn end(&mut self) {
        self.norm.end();
    }

    pub(crate) fn newline(&mut self, size: usize) {
        self.norm.blank(norm::Blank::Newline(size));
    }

    pub(crate) fn bsp(&mut self, size: usize) {
        self.norm.blank(norm::Blank::Space(norm::Space {
            size,
            breakable: true,
        }));
    }

    pub(crate) fn nbsp(&mut self, size: usize) {
        self.norm.blank(norm::Blank::Space(norm::Space {
            size,
            breakable: false,
        }));
    }

    pub(crate) fn raw(&mut self, content: MeasuredStr<'a>) {
        self.finish_normalization();
        self.tokens.deque.push_back(Token::Raw(content));
        self.total_single_line_size += content.visual_size();
        self.break_while_overflows();
    }

    fn finish_normalization(&mut self) {
        for suffix in self.norm.suffixes.drain(..) {
            match suffix {
                norm::Suffix::End => self.tokens.push_unmeasured(Token::End),
                norm::Suffix::Indent(diff) => {
                    self.tokens.deque.push_back(Token::Indent(diff));
                }
            }
        }

        if let Some(blank) = self.norm.blank.take() {
            match blank {
                norm::Blank::Newline(size) => self.norm_newline(size),
                norm::Blank::Space(space) => {
                    if space.breakable {
                        self.norm_bsp(space.size);
                    } else {
                        self.norm_nbsp(space.size);
                    }
                }
            }
        }

        for group in self.norm.pending_groups.drain(..) {
            self.tokens.push_unmeasured(Token::Begin {
                next_bsp_distance: Measurement::Unmeasured {
                    preceding_tokens_size: self.total_single_line_size,
                },
                break_style: group.break_style,
            });
            if group.indent != 0 {
                self.tokens.deque.push_back(Token::Indent(group.indent));
            }
        }
    }

    fn norm_newline(&mut self, size: usize) {
        self.measure_tokens();
        self.break_while(|_| true);
        self.printer.newline(size);
    }

    fn norm_bsp(&mut self, size: usize) {
        self.measure_tokens();
        self.tokens.push_unmeasured(Token::Bsp {
            size,
            next_bsp_distance: Measurement::Unmeasured {
                preceding_tokens_size: self.total_single_line_size,
            },
        });
        self.total_single_line_size += size;
    }

    fn norm_nbsp(&mut self, size: usize) {
        self.tokens.deque.push_back(Token::Nbsp(size));
        self.total_single_line_size += size;
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
            if let Token::Bsp {
                next_bsp_distance, ..
            }
            | Token::Begin {
                next_bsp_distance, ..
            } = token
            {
                if let Measurement::Unmeasured { .. } = next_bsp_distance {
                    *next_bsp_distance = Measurement::Measured(Size::Infinite);
                }
            }

            if self.tokens.starts_with_unmeasured() {
                self.tokens.unmeasured.pop_front();
            }

            self.print_measured_tokens();
        }
    }

    fn print_measured_tokens(&mut self) {
        // dbg!(&self);
        // todo!(
        //     "figure out space squashing, line_size_budget changes for nbsp/newline; \
        //     preserve whitespace around punctuation as much as possible default in generic \
        //     config; i.e. use `Option` for space config"
        // );

        debug_assert_ne!(self.tokens.deque.len(), 0);

        while let Some(&token) = self.tokens.deque.front() {
            match token {
                Token::Raw(content) => {
                    self.printed_single_line_size += content.visual_size();
                    self.printer.raw(content);
                }
                Token::Newline(size) => {
                    self.printer.newline(size);
                }
                Token::Nbsp(size) => {
                    self.printed_single_line_size += size;
                    self.printer.nbsp(size);
                }
                Token::Bsp {
                    next_bsp_distance,
                    size,
                } => {
                    let Measurement::Measured(distance) = next_bsp_distance else {
                        return;
                    };
                    self.printed_single_line_size += size;
                    self.printer.bsp(size, distance);
                }
                Token::Begin {
                    next_bsp_distance,
                    break_style,
                } => {
                    let Measurement::Measured(distance) = next_bsp_distance else {
                        return;
                    };
                    self.printer.begin(break_style, distance);
                }
                Token::Indent(diff) => {
                    self.printer.indent(diff);
                }
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
                    next_bsp_distance, ..
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
                    next_bsp_distance.measure_from(self.total_single_line_size);
                    depth -= 1;
                }
                Token::End => {
                    remove_unmeasured();
                    depth += 1;
                }
                Token::Bsp {
                    next_bsp_distance, ..
                } => {
                    remove_unmeasured();
                    next_bsp_distance.measure_from(self.total_single_line_size);
                    if depth == 0 {
                        return;
                    }
                }
                Token::Raw(_) | Token::Nbsp(_) | Token::Newline(_) | Token::Indent(_) => {
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
