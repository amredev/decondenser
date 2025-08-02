use crate::sealed::Sealed;
use crate::str::{IntoStr, Str};
use crate::{IntoSpace, Space};

/// Describes a grouping of content delimited via opening and closing sequences
/// (usually some kind of brackets).
///
/// Can be broken into multiple lines if it takes too much space to fit on a
/// single line.
#[derive(Debug)]
pub struct Group {
    pub(crate) opening: Punct,
    pub(crate) closing: Punct,
    pub(crate) break_style: BreakStyle,
}

impl Group {
    /// Creates a new [`Group`] with the given opening and closing delimiters.
    #[must_use]
    pub fn new(opening: Punct, closing: Punct) -> Self {
        Self {
            opening,
            closing,
            break_style: BreakStyle::consistent(),
        }
    }

    /// Sets the [`BreakStyle`] for this group. See its docs for more.
    ///
    /// Default is [`BreakStyle::consistent()`].
    #[must_use]
    pub fn break_style(mut self, value: BreakStyle) -> Self {
        self.break_style = value;
        self
    }
}

/// Defines the algorithm used to decide whether to turn a space into a line
/// break or not. The examples below are based on this input:
///
/// ```ignore
/// foo(aaa, bbb, ccc, ddd);
/// ```
///
/// Note that breaking is optional. It only takes place if the content of the
/// group can not fit on a single line. If it does fit - it won't be broken
/// disregarding the [`BreakStyle`].
#[derive(Debug, Clone)]
pub struct BreakStyle(pub(crate) BreakStyleEnum);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BreakStyleEnum {
    Consistent,
    Compact,
}

impl BreakStyle {
    /// Turn **all** breaks into a line break so that every item appears on its
    /// own line.
    ///
    /// ```ignore
    /// foo(
    ///     aaaa,
    ///     bbb,
    ///     ccc,
    ///     ddd
    /// );
    /// ```
    #[must_use]
    pub fn consistent() -> Self {
        Self(BreakStyleEnum::Consistent)
    }

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
    #[must_use]
    pub fn compact() -> Self {
        Self(BreakStyleEnum::Compact)
    }
}

/// Describes a quoted content that can not be broken into multiple lines.
///
/// The content is delimited by the opening and closing sequences, and can
/// contain special characters that are escaped using the provided escape
/// sequences logic.
#[derive(Debug)]
pub struct Quote {
    pub(crate) opening: Str,
    pub(crate) closing: Str,
}

impl Quote {
    /// Creates a new [`Quote`] with the given opening and closing delimiters.
    #[must_use]
    pub fn new(opening: impl IntoStr, closing: impl IntoStr) -> Self {
        Self {
            opening: Str::new(opening),
            closing: Str::new(closing),
        }
    }
}

/// The punctuation character. This would typically be a single character,
/// but it can also be a sequence of characters like `=>`.
#[derive(Debug)]
pub struct Punct {
    pub(crate) symbol: Str,
    pub(crate) leading_space: Space,
    pub(crate) trailing_space: Space,
}

impl Punct {
    /// Creates a new [`Punct`] with the given symbol string.
    #[must_use]
    pub fn new(symbol: impl IntoStr) -> Self {
        Self {
            symbol: Str::new(symbol),
            leading_space: Space::new(),
            trailing_space: Space::new(),
        }
    }

    /// Defines the logic leading space handling for this [`Punct`].
    ///
    /// By default no leading space is added.
    #[must_use]
    pub fn leading_space(mut self, value: impl IntoSpace) -> Self {
        self.leading_space = value.into_space(Sealed);
        self
    }

    /// Defines the logic trailing space handling for this [`Punct`].
    ///
    /// By default no trailing space is added.
    #[must_use]
    pub fn trailing_space(mut self, value: impl IntoSpace) -> Self {
        self.trailing_space = value.into_space(Sealed);
        self
    }
}
