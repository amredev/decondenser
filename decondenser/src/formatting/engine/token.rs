use super::measured_str::MeasuredStr;
use crate::BreakStyle;
use crate::ansi::{BLACK, BLUE, BOLD, GREEN, NO_BOLD, RESET, WHITE, YELLOW};
use crate::utils::{debug_panic, scope_path};
use std::fmt;

#[derive(Clone, Copy)]
pub(super) enum Token<'a> {
    /// Starts a nested group
    Begin {
        /// Calculated as the distance to the next [`Token::Space`] that follows
        /// the paired [`Token::End`] on the same level of nesting or EOF.
        next_break_distance: Measurement,
        break_style: BreakStyle,
    },

    /// Raw text that should be printed as-is.
    Raw(MeasuredStr<'a>),

    /// A point in text where a line break can be inserted if the content does
    /// not fit on a single line.
    SoftBreak {
        /// Calculated as the this token's [`Space::size`] plus the sum of sizes of
        /// all tokens until the next [`Token::Space`] on the same level of nesting
        /// or EOF.
        next_break_distance: Measurement,
    },

    /// A sequence of the given number of whitespace characters. Will be ignored
    /// if it would be printed adjacently to a line break.
    Space(usize),

    /// Change the indent of the following content by the given number of
    /// levels. Applied only if the group is broken into multiple lines.
    Indent(isize),

    /// Closes a nested group
    End,
}

#[derive(Clone, Copy)]
pub(super) enum Size {
    Fixed(usize),

    /// The token is "logically" infinitely large. This is used to indicate
    /// that the token should always be broken into a new line.
    Infinite,
}

#[derive(Clone, Copy)]
pub(super) enum Measurement {
    Measured(Size),

    /// We are yet in the process of calculating the size of the token
    Unmeasured {
        preceding_tokens_size: usize,
    },
}

impl Measurement {
    /// Set the size to [`MaybeSize::Calculated`] as a diff between
    /// `planned_size` and the stored `preceding_tokens_size`.
    pub(super) fn measure_from(&mut self, planned_size: usize) {
        let Self::Unmeasured {
            preceding_tokens_size,
        } = *self
        else {
            debug_panic!("{} called on {self:?}", scope_path!());
            return;
        };

        *self = Self::Measured(Size::Fixed(planned_size - preceding_tokens_size));
    }
}

impl fmt::Debug for Measurement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Measured(size) => fmt::Debug::fmt(size, f),
            Self::Unmeasured {
                preceding_tokens_size,
            } => {
                write!(
                    f,
                    "{BLACK}{:>3}{RESET} ",
                    format!("?{preceding_tokens_size}")
                )?;
                Ok(())
            }
        }
    }
}

impl fmt::Debug for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{BLACK}")?;

        let size: &dyn fmt::Display = match self {
            Self::Fixed(size) => size,
            Self::Infinite => &"∞",
        };

        write!(f, "{size:>3}{RESET} ")?;
        Ok(())
    }
}

impl fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Raw(content) => write!(
                f,
                "{:?}{WHITE}{content:?}",
                Size::Fixed(content.visual_size())
            ),
            Self::SoftBreak {
                next_break_distance,
            } => {
                write!(f, "{next_break_distance:?}{GREEN}{BOLD}SoftBreak{NO_BOLD}")
            }
            Self::Space(size) => write!(f, "{:?}{WHITE}Space", Size::Fixed(*size)),
            Self::Begin {
                break_style,
                next_break_distance,
            } => {
                write!(
                    f,
                    "{next_break_distance:?}{YELLOW}{BOLD}Begin{NO_BOLD} {break_style:?}"
                )
            }
            Self::Indent(diff) => write!(f, "{BLACK}  - Indent({diff})"),
            Self::End => write!(f, "{BLACK}  - {BLUE}End"),
        }?;

        write!(f, "{RESET}")
    }
}
