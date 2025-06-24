use super::Size;
use super::token::{Begin, Break};
use crate::{BreakStyle, Decondenser};
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Copy, Clone)]
enum LineFit {
    /// Group can fit on the current line
    Fits,

    /// Group can't fit on a single line, it must be broken into several lines
    Broken {
        /// The state of indent before the group was broken.
        prev_indent: usize,
    },
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
struct RendererConfig {
    line_size: usize,

    /// Output control characters for debugging the layout logic
    debug_layout: bool,

    /// Output indentation characters for debugging the indent logic
    debug_indent: bool,
}

#[derive(Debug)]
pub(super) struct Printer {
    /// Constant values intentionally separated out of the struct to group them
    /// together for readability. Everything else in this struct is mutable.
    config: RendererConfig,

    /// Output string being built
    pub(super) output: String,

    /// Spare budget of size left on the current line.
    ///
    /// Can be zero if the last printed token was >= in size than the
    /// [`RendererConfig::line_size`] limit, and all possible breaks on the left
    /// side of that could be done were already done. I.e. - there is no way to
    /// fit the token into the limit without breaking somewhere in the middle of
    /// some token, which is not allowed.
    pub(super) line_size_budget: usize,

    /// Number of spaces for indenting the current line
    indent: usize,

    /// If we were to eagerly push space in [`Self::print_break()`], we could
    /// leave unnecessary trailing spaces if the output just cuts off after the
    /// last break.
    ///
    /// So, instead of pushing the space immediately, we just increase this
    /// counter so that next time a non-space token is printed it is prefixed
    /// with the pending amount spaces.
    pending_spaces: usize,

    /// Stack of groups-in-progress nested one inside another
    groups_stack: Vec<Group>,
}

impl Printer {
    pub(super) fn new(config: &Decondenser) -> Self {
        Self {
            config: RendererConfig {
                line_size: config.line_size,
                debug_layout: config.debug_layout,
                debug_indent: config.debug_indent,
            },
            output: String::new(),
            line_size_budget: config.line_size,
            indent: 0,
            pending_spaces: 0,
            groups_stack: Vec::new(),
        }
    }

    pub(super) fn begin(&mut self, token: &Begin, size: Size) {
        if self.config.debug_layout {
            self.output.push(match token.break_style {
                BreakStyle::Consistent => '«',
                BreakStyle::Compact => '‹',
            });
        }

        if self.config.debug_indent {
            let offset = token.indent_diff.to_string();
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

        if matches!(size, Size::Fixed(size) if size <= self.line_size_budget) {
            let group = Group::new(LineFit::Fits, token.break_style);
            self.groups_stack.push(group);
            return;
        }

        let line_fit = LineFit::Broken {
            prev_indent: self.indent,
        };

        self.groups_stack
            .push(Group::new(line_fit, token.break_style));

        self.indent = self.add_indent(token.indent_diff);
    }

    pub(super) fn end(&mut self) {
        let top_group = self.groups_stack.pop().unwrap();

        if let LineFit::Broken { prev_indent } = top_group.line_fit {
            self.indent = prev_indent;
        }

        if self.config.debug_layout {
            self.output.push(match top_group.break_style {
                BreakStyle::Consistent => '»',
                BreakStyle::Compact => '›',
            });
        }
    }

    #[must_use]
    fn add_indent(&self, diff: isize) -> usize {
        self.indent.checked_add_signed(diff).unwrap_or_else(|| {
            debug_assert!(
                false,
                "Indent overflow: indent_diff: {diff}, self.indent: {}",
                self.indent
            );
            self.indent.saturating_add_signed(diff)
        })
    }

    fn break_fits(&self, size: Size) -> bool {
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

    pub(super) fn break_(&mut self, token: &Break, size: Size) {
        if self.break_fits(size) {
            self.pending_spaces += token.blank_space;
            self.line_size_budget = self.line_size_budget.saturating_sub(token.blank_space);

            if self.config.debug_layout {
                self.output.push('·');
            }

            return;
        }

        if self.config.debug_layout {
            self.output.push('·');
        }

        self.output.push('\n');

        let indent = self.add_indent(token.indent_diff);

        self.pending_spaces = indent;
        self.line_size_budget = self.config.line_size.saturating_sub(indent);
    }

    pub(super) fn raw(&mut self, text: &str) {
        self.print_pending_spaces();
        self.output.push_str(text);
        self.line_size_budget = self.line_size_budget.saturating_sub(text.width());
    }

    fn print_pending_spaces(&mut self) {
        let spaces = std::iter::repeat_n(' ', self.pending_spaces);
        self.output.extend(spaces);
        self.pending_spaces = 0;
    }
}
