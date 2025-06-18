use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;

/// A borrowed or an owned string.
pub struct Str<'a> {
    inner: Cow<'a, str>,
}

impl Debug for Str<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.inner.as_ref(), f)
    }
}

impl<'a> Str<'a> {
    #[must_use]
    pub fn owned(owned: String) -> Self {
        Str {
            inner: Cow::Owned(owned),
        }
    }

    #[must_use]
    pub const fn borrowed(borrowed: &'a str) -> Self {
        Str {
            inner: Cow::Borrowed(borrowed),
        }
    }
}

impl AsRef<str> for Str<'_> {
    fn as_ref(&self) -> &str {
        self.inner.as_ref()
    }
}

impl Deref for Str<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}
