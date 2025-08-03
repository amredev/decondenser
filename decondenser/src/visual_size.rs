use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

/// Defines the algorithm for calculating the "visual" size of a string. See
/// [`Decondenser::visual_size`] for more details.
///
/// You probably don't want to implement this trait by hand, and instead use a
/// closure since this trait is implemented for `Fn(&str) -> usize`.
///
/// [`Decondenser::visual_size`]: crate::Decondenser::visual_size
pub trait VisualSize: Send + Sync + 'static {
    /// The main implementation. It is assumed to be cheap, and it'll be called
    /// many times during the formatting.
    fn visual_size(&self, str: &str) -> usize;
}

impl<F: Fn(&str) -> usize + Send + Sync + 'static> VisualSize for F {
    fn visual_size(&self, str: &str) -> usize {
        self(str)
    }
}

#[derive(Clone)]
pub(crate) struct ArcVisualSize {
    inner: Arc<dyn VisualSize>,

    /// Store the additional info about the type of the function for debugging
    /// purposes.
    name: &'static str,
}

impl ArcVisualSize {
    pub(crate) fn new<T: VisualSize>(inner: T) -> Self {
        Self {
            inner: Arc::new(inner),
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

impl fmt::Debug for ArcVisualSize {
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
