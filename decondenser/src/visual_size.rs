use crate::VisualSize;
use std::fmt;
use std::ops::Deref;

pub(crate) struct BoxedVisualSize {
    inner: Box<dyn VisualSize>,

    /// Store the additional info about the type of the function for debugging
    /// purposes.
    name: &'static str,
}

impl BoxedVisualSize {
    pub(crate) fn new<T: VisualSize>(inner: T) -> Self {
        Self {
            inner: Box::new(inner),
            name: std::any::type_name::<T>(),
        }
    }

    pub(crate) fn measure(&self, content: &str) -> usize {
        (self.inner).visual_size(content)
    }

    pub(crate) fn measured_str<'s>(&self, content: &'s str) -> MeasuredStr<'s> {
        MeasuredStr {
            visual_size: self.measure(content),
            content,
        }
    }
}

impl fmt::Debug for BoxedVisualSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)
    }
}

#[derive(Default, Clone, Copy)]
pub(crate) struct MeasuredStr<'a> {
    /// The size of the string in characters.
    visual_size: usize,
    content: &'a str,
}

impl MeasuredStr<'_> {
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
