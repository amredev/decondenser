use super::SIZE_INFINITY;
use super::token::{Begin, Break, BreaksKind};
use crate::Decondenser;

/// Every line is allowed at least this much space, even if highly indented.
const MIN_SPACE: isize = 60;

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
    breaks_kind: BreaksKind,
}

impl Group {
    fn new(line_fit: LineFit, breaks_kind: BreaksKind) -> Self {
        Self {
            line_fit,
            breaks_kind,
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
    /// Can be negative if the last printed token was larger than a the
    /// [`RendererConfig::line_size`] limit, and all possible breaks on the left
    /// side of that could be done were already done. I.e. - there is no way to
    /// fit the token into the limit without breaking somewhere in the middle of
    /// some token, which is not allowed.
    pub(super) size_budget: isize,

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

    /// Stack of groups-in-progress being flushed by print
    groups: Vec<Group>,
}

impl Printer {
    pub(super) fn new(config: &Decondenser<'_>) -> Self {
        Self {
            config: RendererConfig {
                line_size: config.line_size,
                debug_layout: config.debug_layout,
                debug_indent: config.debug_indent,
            },
            output: String::new(),
            size_budget: config.line_size.try_into().unwrap_or(SIZE_INFINITY),
            indent: 0,
            pending_spaces: 0,
            groups: Vec::new(),
        }
    }

    pub(super) fn begin(&mut self, token: &Begin, size: isize) {
        if self.config.debug_layout {
            self.output.push(match token.breaks_kind {
                BreaksKind::Consistent => '«',
                BreaksKind::Inconsistent => '‹',
            });
        }

        if self.config.debug_indent {
            let offset = token.offset.to_string();
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

        if size <= self.size_budget {
            let group = Group::new(LineFit::Fits, token.breaks_kind);
            self.groups.push(group);
            return;
        }

        let line_fit = LineFit::Broken {
            prev_indent: self.indent,
        };

        self.groups.push(Group::new(line_fit, token.breaks_kind));

        self.indent = usize::try_from(self.indent as isize + token.offset).unwrap();
    }

    pub(super) fn end(&mut self) {
        let top_group = self.groups.pop().unwrap();

        if let LineFit::Broken { prev_indent } = top_group.line_fit {
            self.indent = prev_indent;
        }

        if self.config.debug_layout {
            self.output.push(match top_group.breaks_kind {
                BreaksKind::Consistent => '»',
                BreaksKind::Inconsistent => '›',
            });
        }
    }

    fn fits_on_top(&self, size: isize) -> bool {
        let top_group = self.groups.last().copied().unwrap_or_else(|| {
            Group::new(LineFit::Broken { prev_indent: 0 }, BreaksKind::Inconsistent)
        });

        match top_group.line_fit {
            LineFit::Fits => true,
            LineFit::Broken { .. } => {
                // Even if the group is broken, we still try to fit the tokens
                // on the same line if the break is inconsistent, which is the
                // whole purpose if "consistent/inconsistent" distinction.
                top_group.breaks_kind == BreaksKind::Inconsistent && size <= self.size_budget
            }
        }
    }

    pub(super) fn break_(&mut self, token: Break, size: isize) {
        if token.never_break || self.fits_on_top(size) {
            self.pending_spaces += token.blank_space;
            self.size_budget -= token.blank_space as isize;

            if self.config.debug_layout {
                self.output.push('·');
            }

            return;
        }

        if self.config.debug_layout {
            self.output.push('·');
        }

        self.output.push('\n');

        let indent = self.indent as isize + token.offset;
        self.pending_spaces = usize::try_from(indent).unwrap();

        // todo!(
        //     "min space allows overflowing the max line length by establishing
        //     a minimum number of characters a line can occupy no matter how indented
        //     it is"
        // );
        self.size_budget = self.config.line_size as isize - indent;
        // self.space = std::cmp::max(self.config.max_line_width as isize - indent, MIN_SPACE);
    }

    pub(super) fn literal(&mut self, text: &str) {
        self.print_pending_spaces();
        self.output.push_str(text);

        // TODO: use unicode-width here
        self.size_budget -= text.len() as isize;
    }

    fn print_pending_spaces(&mut self) {
        let spaces = std::iter::repeat_n(' ', self.pending_spaces);
        self.output.extend(spaces);
        self.pending_spaces = 0;
    }
}
