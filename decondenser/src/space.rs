use crate::sealed::Sealed;

/// Defines the rules for inserting space characters and line breaks.
#[derive(Debug, Clone)]
pub struct Space {
    /// Min/max bounds for the size.
    pub(crate) size: (usize, usize),
    pub(crate) breakable: bool,
}

impl Space {
    /// Creates a new [`Space`] with the default settings.
    ///
    /// Make sure to explicitly enable [`Space::breakable`] if you want the
    /// space to be considered for turning into a newline when the content does
    /// not fit on a single line. Otherwise, the space will always stay static
    /// and it'll never be turned into a line break.
    pub fn new() -> Self {
        Self {
            size: (0, 1),
            breakable: false,
        }
    }

    /// Sets the fixed number of whitespace characters for this [`Space`].
    ///
    /// By default, the whitespace content is dynamic - preserved from input.
    #[must_use]
    pub fn size(mut self, value: impl SpaceSize) -> Self {
        self.size = value.space_size(Sealed);
        self
    }

    /// Sets whether this space can be turned into a line break if content
    /// overflows or not.
    ///
    /// Note that `true` is not equivalent to an unconditional line break. It
    /// means the space *might* turn into a line break only if the surrounding
    /// content does not fit on a single line.
    ///
    /// Defaults to `false`
    #[must_use]
    pub fn breakable(mut self, value: bool) -> Self {
        self.breakable = value;
        self
    }
}

/// Represents the size constraint for a [`Space`]. If the space is shorter or
/// longer than the given range (or fixed size), it will be clamped.
pub trait SpaceSize {
    #[doc(hidden)]
    fn space_size(self, _: Sealed) -> (usize, usize);
}

/// Creates a fixed [`SpaceSize`].
impl SpaceSize for usize {
    fn space_size(self, _: Sealed) -> (usize, usize) {
        (self, self)
    }
}

/// Creates a range-based [`SpaceSize`], where the space size is preserved from
/// input, but clamped to the given range.
impl SpaceSize for std::ops::RangeInclusive<usize> {
    fn space_size(self, _: Sealed) -> (usize, usize) {
        self.into_inner()
    }
}

/// Convenience conversion into a [`Space`]. See trait impls for possible
/// options of how a [`Space`] can be created and what each option means.
pub trait IntoSpace {
    #[doc(hidden)]
    fn into_space(self, _: Sealed) -> Space;
}

/// Identity conversion from [`Space`].
impl IntoSpace for Space {
    fn into_space(self, _: Sealed) -> Space {
        self
    }
}

/// A shortcut for an unbreakable space with the specified [`size`].
///
/// [`size`]: Space::size
impl<T: SpaceSize> IntoSpace for T {
    fn into_space(self, _: Sealed) -> Space {
        Space::new().size(self)
    }
}
