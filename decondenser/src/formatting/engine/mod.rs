//! The algorithm and code from `prettyplease` served as a base here:
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
//! The original code from `prettyplease` was heavily refactored and modified to
//! better suit the needs of `decondenser`. The following modifications are most
//! notable:
//!
//! - A normalization layer was added with the split into a [`Formatter`] and a
//!   [`NormalizedFormatter`]. This is used to squash adjacent spaces/newlines
//!   and other trivial token combinations.
//! - The printer logic was moved into a separate isolated struct/module
//!   [`printer`], which makes it easier to read and maintain.
//! - The `RingBuffer` type was renamed to a more descriptive [`sliding_deque`],
//! - The space and breaks in the [`token`] structure are now separate
//!
//! Also, this blog post by @mcyoung is a great resource for understanding:
//! <https://mcyoung.xyz/2025/03/11/formatters/>

mod normalized;
mod printer;
mod sliding_deque;
mod token;

pub(crate) use crate::visual_size::MeasuredStr;

use self::normalized::NormalizedFormatter;
use crate::formatting::BreakStyle;

/// A generic formatter that works in terms of groups, raw strings, spaces,
/// breaks, and indent. This struct is the top-level normalization layer around
/// the core [`NormalizedFormatter`]. This layer normalizes the tokens input to
/// prevent stuff like leading/trailing spaces/newlines and adds squashing of
/// adjacent spaces and newlines.
///
/// See the [`NormalizedFormatter`] for more details on the core algorithm.
pub(crate) struct Formatter<'a> {
    /// [`NormalizedFormatter`] expects a normalized sequence of tokens.
    fmt: NormalizedFormatter<'a>,

    /// The normalized indentation of the current pending block of tokens.
    indent: isize,

    /// The normalized [`Blank`] of the current pending block of tokens.
    blank: Blank,

    /// The normalized [`Control`] of the current pending block of tokens.
    control: Vec<Control>,
}

enum Blank {
    Space(usize),
    HardBreak(usize),
}

impl Default for Blank {
    fn default() -> Self {
        Self::Space(0)
    }
}

enum Control {
    SoftBreak,
    Begin(BreakStyle),
    End,
}

impl<'a> Formatter<'a> {
    pub(crate) fn new(config: &'a crate::Decondenser) -> Self {
        Self {
            fmt: NormalizedFormatter::new(config),
            indent: 0,
            blank: Blank::default(),
            control: vec![],
        }
    }

    pub(crate) fn begin(&mut self, break_style: BreakStyle) {
        self.control.push(Control::Begin(break_style));
    }

    pub(crate) fn end(&mut self) {
        if let Some(Control::Begin(_)) = self.control.last() {
            self.control.pop();
        } else {
            self.control.push(Control::End);
        }
    }

    pub(crate) fn indent(&mut self, diff: isize) {
        self.indent += diff;
    }

    pub(crate) fn soft_break(&mut self) {
        if let Some(Control::SoftBreak) = self.control.last() {
            // Avoid consecutive soft breaks
            return;
        }

        if let Blank::HardBreak(_) = self.blank {
            // If there's a hard break already, we don't need a soft break
            return;
        }

        self.control.push(Control::SoftBreak);
    }

    pub(crate) fn space(&mut self, size: usize) {
        if size == 0 {
            return;
        }

        match &mut self.blank {
            // Squash adjacent spaces
            Blank::Space(current_size) => *current_size = std::cmp::max(*current_size, size),

            // Hard break takes precedence over space
            Blank::HardBreak(_) => {}
        }
    }

    #[expect(
        dead_code,
        reason = "
            may be useful to support the feature of preserving newlines;
            that feature had even been implemented during the first iterations
            of this code, but was later removed as it wasn't found to be useful;
            maybe this ability to do hard breaks at this level will come in handy
            again, so keeping it in code for now
        "
    )]
    pub(crate) fn hard_break(&mut self, size: usize) {
        if size == 0 {
            return;
        }

        match &mut self.blank {
            // Squash adjacent hard breaks
            Blank::HardBreak(current_size) => *current_size = std::cmp::max(*current_size, size),

            // Hard break takes precedence over space
            Blank::Space(_) => {
                // No soft breaks are needed adjacently to a hard break
                self.control
                    .retain(|control| !matches!(control, Control::SoftBreak));

                self.blank = Blank::HardBreak(size);
            }
        }
    }

    /// End of input
    pub(crate) fn eoi(mut self) -> String {
        // Strip trailing whitespace/newlines from the output
        self.blank = Blank::default();
        self.flush_normalized_tokens();
        self.fmt.eoi()
    }

    pub(crate) fn raw(&mut self, content: MeasuredStr<'a>) {
        if content.is_empty() {
            return;
        }

        self.flush_normalized_tokens();
        self.fmt.raw(content);
    }

    fn flush_normalized_tokens(&mut self) {
        if self.indent != 0 {
            self.fmt.indent(self.indent);
            self.indent = 0;
        }

        for control in self.control.drain(..) {
            match control {
                Control::SoftBreak => self.fmt.soft_break(),
                Control::Begin(break_style) => self.fmt.begin(break_style),
                Control::End => self.fmt.end(),
            }
        }

        match std::mem::take(&mut self.blank) {
            Blank::Space(size) if size > 0 => self.fmt.space(size),
            Blank::HardBreak(size) if size > 0 => self.fmt.hard_break(size),
            _ => {}
        }
    }
}
