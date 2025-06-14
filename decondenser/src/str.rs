use std::borrow::Cow;
use std::ops::Deref;

/// A borrowed or an owned string.
pub struct Str<'a> {
    inner: Cow<'a, str>,
}

impl<'a> Str<'a> {
    pub fn owned(owned: String) -> Self {
        Str {
            inner: Cow::Owned(owned),
        }
    }

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
