use crate::sealed::Sealed;
use std::borrow::Cow;
use std::fmt;
use std::ops::Deref;

#[derive(Clone)]
#[expect(unnameable_types)]
pub struct Str {
    inner: Cow<'static, str>,
}

/// Sealed trait used to specify "string-like" values that can be converted into
/// a [`String`].
pub trait IntoStr {
    /// Sealed method. Can't be called outside of this crate.
    fn into_str(self, _: Sealed) -> Str;
}

impl IntoStr for String {
    fn into_str(self, _: Sealed) -> Str {
        Str {
            inner: Cow::Owned(self),
        }
    }
}

impl IntoStr for &'static str {
    fn into_str(self, _: Sealed) -> Str {
        Str {
            inner: Cow::Borrowed(self),
        }
    }
}

impl IntoStr for Cow<'static, str> {
    fn into_str(self, _: Sealed) -> Str {
        Str { inner: self }
    }
}

impl fmt::Display for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl fmt::Debug for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <str as fmt::Debug>::fmt(&self.inner, f)
    }
}

impl Deref for Str {
    type Target = str;

    fn deref(&self) -> &str {
        &self.inner
    }
}

impl Str {
    pub(crate) fn new(str: impl IntoStr) -> Self {
        str.into_str(Sealed)
    }

    /// Construct an [`Str`] from the given number of spaces. It's optimized for
    /// up to 10 spaces to avoid allocations for common small sizes.
    pub(crate) fn n_spaces(count: usize) -> Self {
        [
            "",
            " ",
            "  ",
            "   ",
            "    ",
            "     ",
            "      ",
            "       ",
            "        ",
            "         ",
            "          ",
        ]
        .get(count)
        .map(|&static_str| Self::new(static_str))
        .unwrap_or_else(|| Self::new(" ".repeat(count)))
    }
}
