use super::Size;
use super::measured_str::MeasuredStr;
use super::token::{Begin, Space};
use crate::{BreakStyle, Decondenser};

#[derive(Debug, Copy, Clone)]
enum LineFit {
    /// Group can fit on the current line
    Fits,

    /// Group can't fit on a single line, it must be broken into several lines
    Broken,
}

#[derive(Debug, Clone, Copy)]
struct Group {
    line_fit: LineFit,
    break_style: BreakStyle,
}

impl Group {
    fn new(line_fit: LineFit, break_style: BreakStyle) -> Self {
        Self {
            line_fit,
            break_style,
        }
    }
}

#[derive(Debug)]
struct PrinterConfig<'a> {
    max_line_size: usize,
    no_break_size: usize,
    debug_layout: bool,
    debug_indent: bool,

    /// String used to make a single level of indentation.
    indent_str: MeasuredStr<'a>,
}

#[derive(Debug)]
pub(super) struct Printer<'a> {
    /// Constant values intentionally separated out of the struct to group them
    /// together for readability. Everything else in this struct is mutable.
    config: PrinterConfig<'a>,

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

    /// True if the next token should be indented, false otherwise.
    indent_pending: bool,

    /// Stack of groups-in-progress nested one inside another
    groups_stack: Vec<Group>,
}

impl<'a> Printer<'a> {
    pub(super) fn new(config: &'a Decondenser) -> Self {
        Self {
            config: PrinterConfig {
                max_line_size: config.max_line_size,
                no_break_size: config.no_break_size,
                debug_layout: config.debug_layout,
                debug_indent: config.debug_indent,
                indent_str: MeasuredStr::new(&config.indent, config.visual_size),
            },
            output: String::new(),
            line_size_budget: std::cmp::max(config.max_line_size, config.no_break_size),
            indent_level: 0,
            indent_pending: false,
            groups_stack: Vec::new(),
        }
    }

    pub(super) fn begin(&mut self, token: &Begin, next_space_distance: Size) {
        if self.config.debug_layout {
            self.output.push(match token.break_style {
                BreakStyle::Consistent => '«',
                BreakStyle::Compact => '‹',
            });
        }

        if matches!(next_space_distance, Size::Fixed(size) if size <= self.line_size_budget) {
            let group = Group::new(LineFit::Fits, token.break_style);
            self.groups_stack.push(group);
            return;
        }

        let line_fit = LineFit::Broken;

        self.groups_stack
            .push(Group::new(line_fit, token.break_style));
    }

    pub(super) fn end(&mut self) {
        let top_group = self.groups_stack.pop().unwrap();

        if self.config.debug_layout {
            self.output.push(match top_group.break_style {
                BreakStyle::Consistent => '»',
                BreakStyle::Compact => '›',
            });
        }
    }

    pub(super) fn indent(&mut self, diff: isize) {
        if self.config.debug_indent {
            let offset = diff.to_string();
            let chars = offset.chars().map(|ch| match ch {
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
                _ => unreachable!(),
            });

            self.output.extend(chars);
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

        if let LineFit::Fits = top_group.line_fit {
            return true;
        }

        // Even if the group is broken, we still try to fit the tokens on the
        // same line if the break is compact, which is the whole purpose of
        // "consistent/compact" distinction.
        top_group.break_style == BreakStyle::Compact && size <= self.line_size_budget
    }

    pub(super) fn space(&mut self, token: &Space<'_>, next_space_distance: Size) {
        if self.next_token_sequence_fits(next_space_distance) {
            self.output.push_str(&token.content);
            self.line_size_budget = self
                .line_size_budget
                .saturating_sub(token.content.visual_size());

            if self.config.debug_layout {
                self.output.push('·');
            }

            return;
        }

        if self.config.debug_layout {
            self.output.push('·');
        }

        self.indent_pending = true;
    }

    fn print_pending_indent(&mut self) {
        if !self.indent_pending {
            return;
        }

        self.indent_pending = false;

        self.output.push('\n');
        self.output.extend(std::iter::repeat_n(
            self.config.indent_str.as_str(),
            self.indent_level,
        ));

        let indent_size = self.indent_level * self.config.indent_str.visual_size();

        self.line_size_budget = std::cmp::max(
            self.config.max_line_size.saturating_sub(indent_size),
            self.config.no_break_size,
        );
    }

    pub(super) fn raw(&mut self, str: MeasuredStr<'_>) {
        self.print_pending_indent();
        self.line_size_budget = self.line_size_budget.saturating_sub(str.visual_size());
        self.output.push_str(&str);
    }

    pub(super) fn line_size_budget(&self) -> usize {
        self.line_size_budget
    }

    pub(super) fn eof(self) -> String {
        self.output
    }
}
