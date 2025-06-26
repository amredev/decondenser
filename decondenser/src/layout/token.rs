use crate::BreakStyle;
use crate::ansi::{BLACK, BLUE, BOLD, GREEN, NO_BOLD, RESET, WHITE, YELLOW};
use crate::utils::{debug_panic, scope_path};
use std::fmt;

pub(super) enum Token<'a> {
    Begin(Begin),
    Raw(Raw<'a>),
    Space(Space),
    End,
}

pub(super) struct Begin {
    /// Calculated as the distance to the next [`Token::Space`] that follows
    /// the paired [`Token::End`] on the same level of nesting or EOF.
    pub(super) next_space_distance: Measurement,

    /// Summed with the indent before the group to calculate the indent for the
    /// content of the group.
    pub(super) indent_diff: isize,

    pub(super) break_style: BreakStyle,
}

pub(super) struct Raw<'a> {
    pub(super) size: usize,

    pub(super) text: &'a str,
}

pub(super) struct Space {
    /// Calculated as the this token's [`Space::size`] plus the sum of sizes of
    /// all tokens until the next [`Token::Space`] on the same level of nesting
    /// or EOF.
    pub(super) next_space_distance: Measurement,

    /// Summed with the indent before the break to calculate the indent for the
    /// following content.
    pub(super) indent_diff: isize,

    /// Number of spaces to insert if this space isn't turned into a line break.
    pub(super) size: usize,
}

#[derive(Clone, Copy)]
pub(super) enum Size {
    Fixed(usize),

    /// The token is "logically" infinitely large. This is used to indicate
    /// that the token should always be broken into a new line.
    Infinite,
}

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

impl fmt::Debug for Raw<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { size, text } = self;

        write!(f, "{:?}{WHITE}{text:?}{RESET}", Size::Fixed(*size))
    }
}

impl fmt::Debug for Space {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            next_space_distance: size,
            indent_diff,
            size: blank_space,
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
            next_space_distance: size,
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
            Self::Space(token) => write!(f, "{token:?}"),
            Self::Begin(token) => write!(f, "{token:?}"),
            Self::End => write!(f, "{BLACK}  - {BLUE}End{RESET}"),
        }
    }
}
