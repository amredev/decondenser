use crate::ansi::{BLACK, BLUE, BOLD, GREEN, NO_BOLD, RESET, WHITE, YELLOW};
use crate::utils::{debug_panic, scope_path};
use std::fmt;

pub(super) enum Token<'a> {
    Begin(Begin),
    Literal(Literal<'a>),
    Break(Break),
    End,
}

pub(super) struct Begin {
    /// The size measurement for this token is initially delayed. It is
    /// eventually set to the sum of single-line sizes of all [`Literal`] and
    /// [`Break`].
    pub(super) size: SizeMeasurement,

    /// Summed with the indent before the group to calculate the indent for the
    /// content of the group.
    pub(super) indent_diff: isize,

    pub(super) breaks_kind: BreaksKind,
}

/// Sets the algorithm used to decide whether to turn a given [`Token::Break`]
/// into a line break or not. The examples below are based on this input:
///
/// ```ignore
/// foo(aaa, bbb, ccc, ddd);
/// ```
///
/// Note that beaking is optional. It only takes place if the content of the
/// group can not fit on a single line. If it does fit - it won't be broken
/// disregarding the [`BreaksKind`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum BreaksKind {
    /// Turn **all** breaks into a line break.
    ///
    /// ```ignore
    /// foo(
    ///     aaaa,
    ///     bbb,
    ///     ccc,
    ///     ddd
    /// );
    /// ```
    Consistent,

    /// Try to fit as much content as possible on a single line and create a
    /// newline only for the last break on the line after which the content
    /// would overflow.
    ///
    /// ```ignore
    /// foo(
    ///     aaaa, bbb,
    ///     ccc, ddd
    /// );
    /// ```
    Inconsistent,
}

pub(super) struct Literal<'a> {
    pub(super) size: usize,

    pub(super) text: &'a str,
}

pub(super) struct Break {
    /// The size should eventually be set to the token's [`Break::blank_space`]
    /// plus the sum of sizes of following [`Token::Literal`] tokens until the
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

impl fmt::Debug for Literal<'_> {
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
            breaks_kind,
        } = self;

        write!(f, "{size:?}{YELLOW}{BOLD}Begin{NO_BOLD} {breaks_kind:?}")?;

        if *indent_diff != 0 {
            write!(f, ", indent_diff: {indent_diff}")?;
        }

        write!(f, "{RESET}")
    }
}

impl fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(token) => write!(f, "{token:?}"),
            Self::Break(token) => write!(f, "{token:?}"),
            Self::Begin(token) => write!(f, "{token:?}"),
            Self::End => write!(f, "{BLACK}  - {BLUE}End{RESET}"),
        }
    }
}
