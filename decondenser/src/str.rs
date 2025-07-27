use self::inline_str::InlineStr;
use crate::sealed::Sealed;
use std::fmt;
use std::ops::Deref;

/// Poor-man's small-string optimization
#[expect(unnameable_types)]
pub struct Str {
    kind: StrKind,
}

enum StrKind {
    Heap(String),
    Inline(InlineStr),
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
            // The string is already allocated. No need to move it into an
            // inline buffer, just keep it as it is.
            kind: StrKind::Heap(self),
        }
    }
}

impl IntoStr for &str {
    fn into_str(self, _: Sealed) -> Str {
        InlineStr::new(self)
            .map(|inline| Str {
                kind: StrKind::Inline(inline),
            })
            .unwrap_or_else(|| Str {
                kind: StrKind::Heap(self.to_owned()),
            })
    }
}

impl fmt::Display for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <str as fmt::Display>::fmt(self, f)
    }
}

impl fmt::Debug for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <str as fmt::Debug>::fmt(self, f)
    }
}

impl Deref for Str {
    type Target = str;

    fn deref(&self) -> &str {
        match &self.kind {
            StrKind::Heap(str) => str,
            StrKind::Inline(str) => str.as_str(),
        }
    }
}

impl Str {
    pub(crate) fn new(str: impl IntoStr) -> Self {
        str.into_str(Sealed)
    }

    /// Optimized constructor that creates an [`Str`] with the given number of
    /// spaces.
    pub(crate) fn n_spaces(count: usize) -> Self {
        const SPACES: &str = {
            match str::from_utf8(&[b' '; inline_str::CAPACITY]) {
                Ok(str) => str,
                Err(_) => unreachable!(),
            }
        };

        SPACES
            .get(0..count)
            // SAFETY: a sequence of space characters is guaranteed to be valid UTF8
            .map(Self::new)
            .unwrap_or_else(|| Self::new(" ".repeat(count)))
    }
}

/// A boundary for unsafe code that relies on the invariants of private fields
mod inline_str {
    pub(super) struct InlineStr {
        bytes: [u8; CAPACITY],
        len: u8,
    }

    /// This size of the inline buffer keeps the `Str` at 3 words size.
    pub(super) const CAPACITY: usize = size_of::<usize>() * 2 - 1;

    impl InlineStr {
        pub(super) fn new(str: &str) -> Option<Self> {
            let Ok(len) = u8::try_from(str.len()) else {
                return None;
            };

            if str.len() > CAPACITY {
                return None;
            }

            let mut bytes = [0u8; CAPACITY];
            bytes[..str.len()].copy_from_slice(str.as_bytes());

            Some(Self { bytes, len })
        }

        pub(super) fn as_str(&self) -> &str {
            let len = usize::from(self.len);

            // SAFETY: `len` is guaranteed to be less than or equal to
            // `CAPACITY`, and denote the end of the initialized valid
            // UTF8 sequence. This invariant is enforced upon construction.
            unsafe { std::str::from_utf8_unchecked(self.bytes.get_unchecked(..len)) }
        }
    }
}
