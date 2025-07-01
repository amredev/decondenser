use super::Size;
use super::measured_str::MeasuredStr;
use crate::BreakStyle;
use crate::utils::debug_panic;
use std::{cmp, iter};

#[derive(Debug, Clone, Copy)]
enum Group {
    /// The group fits on the current line. The [`BreakStyle`] is stored in
    /// this variant solely for debugging purposes - to render the matching
    /// character for the end of the group when `debug_layout` is enabled.
    Inline(BreakStyle),

    /// Group can't fit on a single line, it's broken into several lines
    Broken(BreakStyle),
}

impl Group {
    fn break_style(self) -> BreakStyle {
        match self {
            Self::Inline(style) | Self::Broken(style) => style,
        }
    }
}

#[derive(Debug)]
pub(super) struct PrinterConfig<'a> {
    pub(super) max_line_size: usize,
    pub(super) no_break_size: usize,
    pub(super) debug_layout: bool,
    pub(super) debug_indent: bool,

    /// String used to make a single level of indentation.
    pub(super) indent_str: MeasuredStr<'a>,
}

#[derive(Debug)]
enum Blanks {
    Spaces(usize),
    Breaks(usize),
}

#[derive(Debug)]
pub(super) struct Printer<'a> {
    /// Output string being built
    output: String,

    /// Spare budget of size left on the current line.
    ///
    /// Can be zero if the last printed token was >= in size than the
    /// [`PrinterConfig::line_size`] limit, and all possible breaks on the left
    /// side of that could be done were already done. I.e. - there is no way to
    /// fit the token into the limit without breaking somewhere in the middle of
    /// some token, which is not allowed.
    line_size_budget: usize,

    /// Level of indentation for the current line
    indent_level: usize,

    /// Number of space or newline characters accumulated until later.
    ///
    /// By not printing the these eagerly we avoid extra trailing space in the
    /// output, and we also squash consecutive spaces together. For example, if
    /// there is a sequence of 3, 2, 4, 1 spaces, we will print only 4 spaces
    /// (i.e. the longest sequence of spaces requested).
    pending_blanks: Blanks,

    /// Stack of groups-in-progress nested one inside another
    groups_stack: Vec<Group>,

    /// Constant values intentionally separated out of the struct to group them
    /// together for readability. Everything else in this struct is mutable.
    config: PrinterConfig<'a>,
}

impl<'a> Printer<'a> {
    pub(super) fn new(config: PrinterConfig<'a>) -> Self {
        Self {
            output: String::new(),
            line_size_budget: cmp::max(config.max_line_size, config.no_break_size),
            indent_level: 0,
            pending_blanks: Blanks::Spaces(0),
            groups_stack: Vec::new(),
            config,
        }
    }

    fn decrease_line_size_budget(&mut self, size: usize) {
        self.line_size_budget = self.line_size_budget.saturating_sub(size);
    }

    pub(super) fn begin(&mut self, break_style: BreakStyle, next_space_distance: Size) {
        if self.config.debug_layout {
            self.output.push(match break_style {
                BreakStyle::Consistent => '«',
                BreakStyle::Compact => '‹',
            });
        }

        let fits = matches!(next_space_distance, Size::Fixed(distance) if distance <= self.line_size_budget);

        let group = if fits { Group::Inline } else { Group::Broken };

        self.groups_stack.push(group(break_style));
    }

    pub(super) fn end(&mut self) {
        let top_group = self.groups_stack.pop().unwrap();

        if self.config.debug_layout {
            self.output.push(match top_group.break_style() {
                BreakStyle::Consistent => '»',
                BreakStyle::Compact => '›',
            });
        }
    }

    pub(super) fn indent(&mut self, diff: isize) {
        if self.config.debug_indent {
            self.output.extend(subscript_number(&diff.to_string()));
        }

        self.indent_level = self
            .indent_level
            .checked_add_signed(diff)
            .unwrap_or_else(|| {
                debug_assert!(
                    false,
                    "Indent overflow: indent_diff: {diff}, self.indent_level: {}",
                    self.indent_level
                );
                self.indent_level.saturating_add_signed(diff)
            });
    }

    fn next_token_sequence_fits(&self, size: Size) -> bool {
        let Size::Fixed(size) = size else {
            return false;
        };

        let Some(top_group) = self.groups_stack.last() else {
            return size <= self.line_size_budget;
        };

        match top_group {
            Group::Inline(_) => true,

            // Even if the group is broken, we still try to fit the tokens on
            // the same line if the break is compact, which is the whole purpose
            // of "consistent/compact" distinction.
            Group::Broken(BreakStyle::Compact) => size <= self.line_size_budget,
            Group::Broken(BreakStyle::Consistent) => false,
        }
    }

    /// Print the given number of line breaks
    pub(super) fn newline(&mut self, size: usize) {
        if self.config.debug_layout {
            self.output.push_str("ₙₗ");
        }

        let size = match self.pending_blanks {
            // Discard the pending spaces - we don't need them anymore, because
            // we are going to print a newline.
            Blanks::Spaces(_) => size,
            Blanks::Breaks(breaks) => cmp::max(breaks, size),
        };

        self.pending_blanks = Blanks::Breaks(size);
        // self.line_size_budget = 0;
    }

    pub(super) fn nbsp(&mut self, size: usize) {
        if self.config.debug_layout {
            self.output.push('·');
        }

        let Blanks::Spaces(pending_spaces) = &mut self.pending_blanks else {
            // If we have pending breaks, we don't need to print any spaces
            // to avoid extra leading spaces in the output.
            return;
        };

        let Some(diff) = size.checked_sub(*pending_spaces) else {
            return;
        };

        *pending_spaces = size;
        self.decrease_line_size_budget(diff);
    }

    pub(super) fn bsp(&mut self, size: usize, next_space_distance: Size) {
        if self.next_token_sequence_fits(next_space_distance) {
            self.nbsp(size);
        } else {
            self.newline(1);
        }
    }

    fn print_pending_blanks(&mut self) {
        let blanks = std::mem::replace(&mut self.pending_blanks, Blanks::Spaces(0));

        let size = match blanks {
            Blanks::Spaces(0) | Blanks::Breaks(0) => return,
            Blanks::Spaces(size) => {
                self.output.extend(iter::repeat_n(' ', size));
                return;
            }
            Blanks::Breaks(size) => size,
        };

        self.output.extend(iter::repeat_n('\n', size));
        self.output.extend(iter::repeat_n(
            self.config.indent_str.as_str(),
            self.indent_level,
        ));

        let indent_size = self.indent_level * self.config.indent_str.visual_size();

        self.line_size_budget = cmp::max(
            self.config.max_line_size.saturating_sub(indent_size),
            self.config.no_break_size,
        );
    }

    pub(super) fn raw(&mut self, str: MeasuredStr<'_>) {
        self.print_pending_blanks();
        self.decrease_line_size_budget(str.visual_size());
        self.output.push_str(&str);
    }

    pub(super) fn line_size_budget(&self) -> usize {
        self.line_size_budget
    }

    pub(super) fn eof(self) -> String {
        self.output
    }
}

fn subscript_number(number: &str) -> impl Iterator<Item = char> + use<'_> {
    number.chars().filter_map(|ch| {
        Some(match ch {
            '0' => '₀',
            '1' => '₁',
            '2' => '₂',
            '3' => '₃',
            '4' => '₄',
            '5' => '₅',
            '6' => '₆',
            '7' => '₇',
            '8' => '₈',
            '9' => '₉',
            '-' => '₋',
            _ => {
                debug_panic!("Unexpected character in a number: {ch}");
                return None;
            }
        })
    })
}
