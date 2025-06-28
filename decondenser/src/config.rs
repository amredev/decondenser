use crate::sealed::Sealed;
use crate::str::{IntoStr, Str};

/// Describes a grouping of content delimited via opening and closing sequences
/// (usually some kind of brackets).
///
/// Can be broken into multiple lines if it takes too much space to fit on a
/// single line.
#[derive(Debug, Clone)]
pub struct Group {
    /// The sequence that opens the group.
    pub(crate) opening: GroupDelim,

    /// The sequence that closes the group.
    pub(crate) closing: GroupDelim,

    pub(crate) break_style: BreakStyle,
}

impl Group {
    /// Creates a new [`Group`] with the given opening and closing delimiters.
    #[must_use]
    pub fn new(opening: GroupDelim, closing: GroupDelim) -> Self {
        Self {
            opening,
            closing,
            break_style: BreakStyle::Consistent,
        }
    }

    /// Sets the [`BreakStyle`] for this group. See its docs for more.
    ///
    /// Default is [`BreakStyle::Consistent`].
    #[must_use]
    pub fn break_style(mut self) -> Self {
        self.break_style = BreakStyle::Consistent;
        self
    }
}

/// Describes the delimiters of a group that can be used to nest content.
#[derive(Debug, Clone)]
pub struct GroupDelim {
    pub(crate) leading_space: Space,
    pub(crate) content: Str,
    pub(crate) trailing_space: Space,
}

impl GroupDelim {
    /// Creates a new [`GroupDelim`] with the given leading, content and
    /// trailing spaces.
    #[must_use]
    pub fn new(content: impl IntoStr) -> Self {
        Self {
            leading_space: 0.into(),
            content: Str::new(content),
            trailing_space: 0.into(),
        }
    }

    /// Defines both the leading and trailing space that will be added to the
    /// content of the group.
    #[must_use]
    pub fn surrounding_space(mut self, value: impl Into<Space>) -> Self {
        self.leading_space = value.into();
        self.trailing_space = self.leading_space.clone();
        self
    }

    /// Defines the leading space that will be added before the content of the
    /// group.
    #[must_use]
    pub fn leading_space(mut self, value: impl Into<Space>) -> Self {
        self.leading_space = value.into();
        self
    }

    /// Defines the trailing space that will be added after the content of the
    /// group.
    #[must_use]
    pub fn trailing_space(mut self, value: impl Into<Space>) -> Self {
        self.trailing_space = value.into();
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
/// Note that beaking is optional. It only takes place if the content of the
/// group can not fit on a single line. If it does fit - it won't be broken
/// disregarding the [`BreakStyle`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum BreakStyle {
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
    Compact,
}

/// Describes a quoted content that can not be broken into multiple lines.
///
/// The content is delimited by the opening and closing sequences, and can
/// contain special characters that are escaped using the provided escape
/// sequences logic.
#[derive(Debug, Clone)]
pub struct Quote {
    /// The sequence that opens the quoted content.
    pub(crate) opening: Str,

    /// The sequence that closes the quoted content.
    pub(crate) closing: Str,

    /// The sequences that are used to escape special characters in the quoted
    /// content.
    pub(crate) escapes: Vec<Escape>,
}

impl Quote {
    /// Creates a new [`Quote`] with the given opening and closing delimiters.
    #[must_use]
    pub fn new(opening: impl IntoStr, closing: impl IntoStr) -> Self {
        Self {
            opening: Str::new(opening),
            closing: Str::new(closing),
            escapes: vec![],
        }
    }

    /// Sets the escape sequences that are used to escape special characters in
    /// the quoted content. See the [`Escape`] struct for more details.
    ///
    /// By default, no escape sequences are defined.
    #[must_use]
    pub fn escapes(mut self, value: impl IntoIterator<Item = Escape>) -> Self {
        self.escapes = Vec::from_iter(value);
        self
    }
}

/// Describes a single escape sequence inside of a quoted content.
#[derive(Debug, Clone)]
pub struct Escape {
    pub(crate) escaped: Str,

    #[expect(dead_code, reason = "TODO: immplement unescaping API")]
    pub(crate) unescaped: Str,
}

impl Escape {
    /// Creates a new [`Escape`] with the given escaped and unescaped
    /// representations.
    #[must_use]
    pub fn new(escaped: impl IntoStr, unescaped: impl IntoStr) -> Self {
        Self {
            escaped: Str::new(escaped),
            unescaped: Str::new(unescaped),
        }
    }
}

/// The punctuation character. This would typically be a single character,
/// but it can also be a sequence of characters like `=>`.
#[derive(Debug, Clone)]
pub struct Punct {
    pub(crate) content: Str,

    pub(crate) leading_space: Space,
    pub(crate) trailing_space: Space,
}

impl Punct {
    /// Creates a new [`Punct`] with the given content.
    #[must_use]
    pub fn new(content: impl IntoStr) -> Self {
        Self {
            content: Str::new(content),
            leading_space: Space::empty(),
            trailing_space: Space::empty(),
        }
    }

    /// Defines both the leading and trailing space handling for this [`Punct`].
    ///
    /// By default no leading or trailing space is added.
    #[must_use]
    pub(crate) fn surrounding_space(mut self, value: impl Into<Space>) -> Self {
        self.leading_space = value.into();
        self.trailing_space = self.leading_space.clone();
        self
    }

    /// Defines the logic leading space handling for this [`Punct`].
    ///
    /// By default no leading space is added.
    #[must_use]
    pub fn leading_space(mut self, value: impl Into<Space>) -> Self {
        self.leading_space = value.into();
        self
    }

    /// Defines the logic trailing space handling for this [`Punct`].
    ///
    /// By default no trailing space is added.
    #[must_use]
    pub fn trailing_space(mut self, value: impl Into<Space>) -> Self {
        self.trailing_space = value.into();
        self
    }
}

/// Defines the rules for inserting spaces and line breaks.
#[derive(Debug, Clone)]
pub struct Space {
    pub(crate) content: Str,
    pub(crate) breakable: bool,
}

impl Space {
    /// Creates a zero-width [`Space`]. Equivalent to `Space::new(0)`.
    #[must_use]
    pub fn empty() -> Self {
        Self::new(0)
    }

    /// Creates a [`Space`] with the given content.
    ///
    /// Accepts [`SpaceContent`] that enables using [`usize`] or string-like
    /// types for defining the content of a [`Space`]. Passing [`usize`] means
    /// creating a space of the number of whitespace characters equal to the
    /// value of the number.
    ///
    /// Make sure to explicitly enable [`Space::breakable`] if you want the
    /// space to be considered for turning into a new line when the content does
    /// not fit on a single line, otherwise the space will always stay static
    /// and never be turned into a line break.
    #[must_use]
    pub fn new(content: impl SpaceContent) -> Self {
        Self {
            content: content.convert(Sealed),
            breakable: false,
        }
    }

    /// If `true`, the space will be considered for breaking into a new line
    /// if the content does not fit on a single line. If `false`, the space
    /// will never be turned into a line break.
    ///
    /// Default is `false`.
    #[must_use]
    pub fn breakable(mut self, value: bool) -> Self {
        self.breakable = value;
        self
    }
}

impl<T: SpaceContent> From<T> for Space {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

/// A trait to enable using [`usize`] or string-like types for defining the
/// content of a [`Space`].
///
/// Passing [`usize`] means creating a space of the
/// number of whitespace characters equal to the value of the number.
pub trait SpaceContent {
    /// Sealed method. Can't be called outside of this crate.
    fn convert(self, _: Sealed) -> Str;
}

impl SpaceContent for usize {
    fn convert(self, _: Sealed) -> Str {
        Str::n_spaces(self)
    }
}

impl<S: IntoStr> SpaceContent for S {
    fn convert(self, _: Sealed) -> Str {
        Str::new(self)
    }
}
