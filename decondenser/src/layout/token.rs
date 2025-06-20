use crate::ansi::{BLACK, BLUE, BOLD, DIM, GREEN, NO_BOLD, NO_DIM, RESET, WHITE, YELLOW};
use std::fmt;

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

#[derive(Clone, Copy, Default)]
pub(super) struct Break {
    /// Negative size means we don't know yet what size to assign to the token.
    ///
    /// The size should eventually be set to the token's [`Break::blank_space`]
    ///  plus the sum of sizes of following [`Token::String`] tokens until the
    ///  next [`Token::Break`] or [`Token::End`].
    pub(super) size: isize,

    /// Summed with the indent before the break to calculate the indent for the
    /// following content.
    pub(super) offset: isize,

    /// Number of spaces to insert if the break isn't turned into a line break.
    pub(super) blank_space: usize,
    pub(super) if_nonempty: bool,
    pub(super) never_break: bool,
}

pub(super) struct Begin {
    /// Negative size means we don't know yet what size to assign to the token.
    ///
    /// The size should eventually be set to the sum of sizes of all tokens
    /// between this token and the next [`Token::End`].
    pub(super) size: isize,

    /// Summed with the indent before the group to calculate the indent for the
    /// content of the group.
    pub(super) offset: isize,

    pub(super) breaks_kind: BreaksKind,
}

pub(super) struct Literal<'a> {
    pub(super) size: isize,

    pub(super) text: &'a str,
}

pub(super) struct End {
    pub(super) measured: bool,
}

pub(super) enum Token<'a> {
    Literal(Literal<'a>),
    Break(Break),
    Begin(Begin),
    End(End),
}

const SIZE_INFINITY: isize = 0xffff;

impl Token<'_> {
    pub(super) fn is_measured(&self) -> bool {
        match self {
            Self::Break(token) => token.size >= 0,
            Self::Begin(token) => token.size >= 0,
            Self::End(token) => token.measured,
            Self::Literal(_) => true,
        }
    }

    pub(super) fn set_infinite_size(&mut self) {
        match self {
            Self::Break(token) => token.size = SIZE_INFINITY,
            Self::Begin(token) => token.size = SIZE_INFINITY,
            Self::End(token) => token.measured = true,
            Self::Literal(_) => unreachable!(),
        }
    }
}

impl fmt::Debug for Literal<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { size, text } = self;

        write!(f, "{}{WHITE}{text:?}{RESET}", DebugSize(*size))
    }
}

impl fmt::Debug for Break {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            size,
            offset,
            blank_space,
            if_nonempty,
            never_break,
        } = *self;

        write!(
            f,
            "{}{GREEN}{BOLD}Break{NO_BOLD} {blank_space}",
            DebugSize(size)
        )?;

        if offset != 0 {
            write!(f, ", offset: {offset}")?;
        }

        if if_nonempty {
            write!(f, ", if_nonempty: {if_nonempty}")?;
        }

        if never_break {
            write!(f, ", never_break: {never_break}")?;
        }

        write!(f, "{RESET}")
    }
}

impl fmt::Debug for Begin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            size,
            offset,
            breaks_kind,
        } = *self;

        write!(
            f,
            "{}{YELLOW}{BOLD}Begin{NO_BOLD} {breaks_kind:?}",
            DebugSize(size)
        )?;

        if offset != 0 {
            write!(f, ", offset: {offset}")?;
        }

        write!(f, "{RESET}")
    }
}

impl fmt::Debug for End {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { measured } = *self;

        let measured = if measured { '0' } else { '-' };

        write!(f, "{BLACK}({measured}){BLUE} {BOLD}End{NO_BOLD}")?;

        write!(f, "{RESET}")?;

        Ok(())
    }
}

impl fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Literal(token) => write!(f, "{token:?}"),
            Token::Break(token) => write!(f, "{token:?}"),
            Token::Begin(token) => write!(f, "{token:?}"),
            Token::End(token) => write!(f, "{token:?}"),
        }
    }
}

struct DebugSize(isize);

impl fmt::Display for DebugSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(size) = *self;
        write!(f, "{BLACK}")?;
        if size == SIZE_INFINITY {
            write!(f, "(∞)")?;
        } else {
            write!(f, "{:>5}", format!("({size})"))?;
        }
        write!(f, "{RESET} ")?;
        Ok(())
    }
}
