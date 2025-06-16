use super::{BeginToken, BreakKind, BreakToken, MIN_SPACE, SIZE_INFINITY};
use crate::Decondenser;

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
    break_kind: BreakKind,
}

impl Group {
    fn new(line_fit: LineFit, break_kind: BreakKind) -> Self {
        Self {
            line_fit,
            break_kind,
        }
    }
}

#[derive(Debug)]
struct RendererConfig {
    max_line_width: usize,

    /// Output control characters for debugging the layout logic
    debug_layout: bool,

    /// Output indentation characters for debugging the indent logic
    debug_indent: bool,
}

#[derive(Debug)]
pub(super) struct Renderer {
    /// Constant values intentionally separated out of the struct to group them
    /// together for readability. Everything else in this struct is mutable.
    config: RendererConfig,

    /// Output string being built
    pub(super) output: String,

    /// Number of spaces left on line
    pub(super) space: isize,

    /// Level of indentation of current line
    indent: usize,

    /// Buffered indentation to avoid writing trailing whitespace
    pending_indent: usize,

    /// Stack of groups-in-progress being flushed by print
    groups: Vec<Group>,
}

impl Renderer {
    pub(super) fn new(config: &Decondenser<'_>) -> Self {
        Self {
            config: RendererConfig {
                max_line_width: config.max_line_width,
                debug_layout: config.debug_layout,
                debug_indent: config.debug_indent,
            },
            output: String::new(),
            space: config.max_line_width.try_into().unwrap_or(SIZE_INFINITY),
            indent: 0,
            pending_indent: 0,
            groups: Vec::new(),
        }
    }

    pub(super) fn print_begin(&mut self, token: &BeginToken, size: isize) {
        if self.config.debug_layout {
            self.output.push(match token.break_kind {
                BreakKind::Consistent => '«',
                BreakKind::Inconsistent => '‹',
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

        if size <= self.space {
            let group = Group::new(LineFit::Fits, token.break_kind);
            self.groups.push(group);
            return;
        }

        let line_fit = LineFit::Broken {
            prev_indent: self.indent,
        };

        self.groups.push(Group::new(line_fit, token.break_kind));

        self.indent = usize::try_from(self.indent as isize + token.offset).unwrap();
    }

    pub(super) fn print_end(&mut self) {
        let top_group = self.groups.pop().unwrap();

        if let LineFit::Broken { prev_indent } = top_group.line_fit {
            self.indent = prev_indent;
        }

        if self.config.debug_layout {
            self.output.push(match top_group.break_kind {
                BreakKind::Consistent => '»',
                BreakKind::Inconsistent => '›',
            });
        }
    }

    fn fits_on_top(&self, size: isize) -> bool {
        let top_group = self.groups.last().copied().unwrap_or_else(|| {
            Group::new(LineFit::Broken { prev_indent: 0 }, BreakKind::Inconsistent)
        });

        match top_group.line_fit {
            LineFit::Fits => true,
            LineFit::Broken { .. } => {
                // Even if the group is broken, we still try to fit the tokens
                // on the same line if the break is inconsistent, which is the
                // whole purpose if "consistent/inconsistent" distinction.
                top_group.break_kind == BreakKind::Inconsistent && size <= self.space
            }
        }
    }

    pub(super) fn print_break(&mut self, token: BreakToken, size: isize) {
        if token.never_break || self.fits_on_top(size) {
            self.pending_indent += token.blank_space;
            self.space -= token.blank_space as isize;

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
        self.pending_indent = usize::try_from(indent).unwrap();

        todo!(
            "min space allows overflowing the max line length by establishing
            a minimum number of characters a line can occupy no matter how indented
            it is"
        );
        self.space = self.config.max_line_width as isize - indent;
        // self.space = std::cmp::max(self.config.max_line_width as isize - indent, MIN_SPACE);
    }

    pub(super) fn print_string(&mut self, string: &str) {
        self.print_indent();
        self.output.push_str(&string);
        self.space -= string.len() as isize;
    }

    fn print_indent(&mut self) {
        self.output
            .extend(std::iter::repeat_n(' ', self.pending_indent));
        self.pending_indent = 0;
    }
}
