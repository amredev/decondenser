use std::fmt;
use std::ops::Deref;

#[derive(Clone, Copy)]
pub(crate) struct MeasuredStr<'a> {
    /// The size of the string in characters.
    visual_size: usize,
    content: &'a str,
}

impl<'a> MeasuredStr<'a> {
    pub(crate) fn new(content: &'a str, visual_size: fn(&str) -> usize) -> Self {
        Self {
            visual_size: visual_size(content),
            content,
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        self.content
    }

    pub(crate) fn visual_size(&self) -> usize {
        self.visual_size
    }
}

impl Deref for MeasuredStr<'_> {
    type Target = str;

    fn deref(&self) -> &str {
        self.content
    }
}

impl fmt::Debug for MeasuredStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.content, f)
    }
}
