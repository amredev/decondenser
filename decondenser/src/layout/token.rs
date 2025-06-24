use crate::BreakStyle;
use crate::ansi::{BLACK, BLUE, BOLD, GREEN, NO_BOLD, RESET, WHITE, YELLOW};
use crate::utils::{debug_panic, scope_path};
use std::fmt;

pub(super) enum Token<'a> {
    Begin(Begin),
    Raw(Raw<'a>),
    Break(Break),
    End,
}

pub(super) struct Begin {
    /// The size measurement for this token is initially delayed. It is
    /// eventually set to the sum of single-line sizes of all [`Token::Raw`] and
    /// [`Token::Break`].
    pub(super) size: SizeMeasurement,

    /// Summed with the indent before the group to calculate the indent for the
    /// content of the group.
    pub(super) indent_diff: isize,

    pub(super) break_style: BreakStyle,
}

pub(super) struct Raw<'a> {
    pub(super) size: usize,

    pub(super) text: &'a str,
}

pub(super) struct Break {
    /// The size should eventually be set to the token's [`Break::blank_space`]
    /// plus the sum of sizes of following [`Token::Raw`] tokens until the
    /// next [`Token::Break`] or [`Token::End`].
    pub(super) size: SizeMeasurement,

    /// Summed with the indent before the break to calculate the indent for the
    /// following content.
    pub(super) indent_diff: isize,

    /// Number of spaces to insert if the break isn't turned into a line break.
    pub(super) blank_space: usize,
}

#[derive(Clone, Copy)]
pub(super) enum Size {
    Fixed(usize),

    /// The token is "logically" infinitely large. This is used to indicate
    /// that the token should always be broken into a new line.
    Infinite,
}

pub(super) enum SizeMeasurement {
    Measured(Size),

    /// We are yet in the process of calculating the size of the token
    Unmeasured {
        preceding_tokens_size: usize,
    },
}

impl SizeMeasurement {
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

impl fmt::Debug for SizeMeasurement {
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

impl fmt::Debug for Raw<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { size, text } = self;

        write!(f, "{:?}{WHITE}{text:?}{RESET}", Size::Fixed(*size))
    }
}

impl fmt::Debug for Break {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            size,
            indent_diff,
            blank_space,
        } = self;

        write!(f, "{size:?}{GREEN}{BOLD}Break{NO_BOLD} {blank_space}")?;

        if *indent_diff != 0 {
            write!(f, ", indent_diff: {indent_diff}")?;
        }

        write!(f, "{RESET}")
    }
}

impl fmt::Debug for Begin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            size,
            indent_diff,
            break_style,
        } = self;

        write!(f, "{size:?}{YELLOW}{BOLD}Begin{NO_BOLD} {break_style:?}")?;

        if *indent_diff != 0 {
            write!(f, ", indent_diff: {indent_diff}")?;
        }

        write!(f, "{RESET}")
    }
}

impl fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Raw(token) => write!(f, "{token:?}"),
            Self::Break(token) => write!(f, "{token:?}"),
            Self::Begin(token) => write!(f, "{token:?}"),
            Self::End => write!(f, "{BLACK}  - {BLUE}End{RESET}"),
        }
    }
}
