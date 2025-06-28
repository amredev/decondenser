use super::measured_str::MeasuredStr;
use crate::BreakStyle;
use crate::ansi::{BLACK, BLUE, BOLD, GREEN, NO_BOLD, RESET, WHITE, YELLOW};
use crate::utils::{debug_panic, scope_path};
use std::fmt;

pub(super) enum Token<'a> {
    /// Starts a nested group
    Begin(Begin),

    /// Raw text that should be printed as-is.
    Raw(Raw<'a>),

    /// A space can be turned into a line break if the line gets too long.
    Space(Space<'a>),

    /// Change the indent of the following content by the given number of
    /// levels. Applied only if the group is broken into multiple lines.
    Indent(isize),

    /// Closes a nested group
    End,
}

pub(super) struct Begin {
    /// Calculated as the distance to the next [`Token::Space`] that follows
    /// the paired [`Token::End`] on the same level of nesting or EOF.
    pub(super) next_space_distance: Measurement,
    pub(super) break_style: BreakStyle,
}

pub(super) struct Raw<'a> {
    pub(super) content: MeasuredStr<'a>,
}

pub(super) struct Space<'a> {
    /// Calculated as the this token's [`Space::size`] plus the sum of sizes of
    /// all tokens until the next [`Token::Space`] on the same level of nesting
    /// or EOF.
    pub(super) next_space_distance: Measurement,

    /// The content of the space if it isn't turned into a line break.
    pub(super) content: MeasuredStr<'a>,
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
        let Self { content } = self;

        write!(
            f,
            "{:?}{WHITE}{content:?}{RESET}",
            Size::Fixed(content.visual_size())
        )
    }
}

impl fmt::Debug for Space<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            next_space_distance,
            content,
        } = self;

        write!(
            f,
            "{next_space_distance:?}{GREEN}{BOLD}Break{NO_BOLD} {content:?}{RESET}"
        )
    }
}

impl fmt::Debug for Begin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            next_space_distance: size,
            break_style,
        } = self;

        write!(
            f,
            "{size:?}{YELLOW}{BOLD}Begin{NO_BOLD} {break_style:?}{RESET}"
        )
    }
}

impl fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Raw(token) => write!(f, "{token:?}"),
            Self::Space(token) => write!(f, "{token:?}"),
            Self::Begin(token) => write!(f, "{token:?}"),
            Self::Indent(diff) => write!(f, "{BLACK}  - Indent({diff}){RESET}"),
            Self::End => write!(f, "{BLACK}  - {BLUE}End{RESET}"),
        }
    }
}
