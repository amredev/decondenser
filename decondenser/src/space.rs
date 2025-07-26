use crate::sealed::Sealed;

/// Defines the rules for inserting space characters and line breaks.
///
/// Use the [`From`] impl of this type to create either one of the two kinds of
/// space:
/// - [`PreservingSpace`] - which preserves the number of spaces from the input
/// - [`FixedSpace`] - which inserts a fixed number of spaces.
#[derive(Debug, Clone)]
pub struct Space {
    pub(crate) size: Option<usize>,
    pub(crate) breakable: SpaceFilter,
}

impl Space {
    /// Creates a new [`Space`] with the default settings.
    ///
    /// Make sure to explicitly enable [`Space::breakable`] if you want the
    /// space to be considered for turning into a newline when the content does
    /// not fit on a single line, otherwise the space will always stay static
    /// and it'll never be turned into a line break.
    #[must_use]
    pub fn new() -> Self {
        Self {
            size: None,
            breakable: SpaceFilter::bool(false),
        }
    }

    /// Sets the fixed number of whitespace characters for this [`Space`].
    ///
    /// By default, the whitespace content is dynamic - preserved from input.
    #[must_use]
    pub fn size(mut self, value: usize) -> Self {
        self.size = Some(value);
        self
    }

    /// Sets whether this space can be turned into a line break if content
    /// overflows or not.
    ///
    /// This can be supplied with either a [`bool`] or a [`SpaceFilter`].
    /// Use the [`SpaceFilter`] to set a more granular condition for when the
    /// space should be considered breakable.
    ///
    /// By default, the space is not considered breakable, analogous to `false`.
    #[must_use]
    pub fn breakable(mut self, value: impl IntoSpaceFilter) -> Self {
        self.breakable = value.into_space_filter(Sealed);
        self
    }
}

/// Convenience conversion into a [`SpaceFilter`].
pub trait IntoSpaceFilter {
    #[doc(hidden)]
    fn into_space_filter(self, _: Sealed) -> SpaceFilter;
}

/// A shortcut for [`SpaceFilter::bool`]
impl IntoSpaceFilter for bool {
    fn into_space_filter(self, _: Sealed) -> SpaceFilter {
        SpaceFilter::bool(self)
    }
}

/// Identity conversion from [`SpaceFilter`].
impl IntoSpaceFilter for SpaceFilter {
    fn into_space_filter(self, _: Sealed) -> SpaceFilter {
        self
    }
}

/// Defines a for filtering a [`Space`].
#[derive(Debug, Clone)]
pub struct SpaceFilter(pub(crate) SpaceFilterEnum);

#[derive(Debug, Clone)]
#[expect(variant_size_differences, reason = "it's not too bad")]
pub(crate) enum SpaceFilterEnum {
    Bool(bool),
    MinSize(usize),
}

impl SpaceFilter {
    /// A static filter that either always accepts any [`Space`] (if `true`)
    /// or rejects any [`Space`] (if `false`).
    #[must_use]
    pub fn bool(value: bool) -> Self {
        Self(SpaceFilterEnum::Bool(value))
    }

    /// A dynamic filter that accepts a [`Space`] only if its size is greater or
    /// equal to the specified minimum size.
    ///
    /// This only makes sense for [`Space`]s of dynamic size. It doesn't make
    /// much sense in combination with a fixed [`Space::size`]. If it's used
    /// with the fixed size [`Space`] - it will always evaluate to `true` or
    /// `false` depending on whether the set [`Space::size`] is greater than or
    /// equal to the minimum size specified in this filter.
    #[must_use]
    pub fn min_size(min_size: usize) -> Self {
        Self(SpaceFilterEnum::MinSize(min_size))
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

/// A shortcut for a space with a fixed [`size`] specified as the [`usize`].
///
/// [`size`]: Space::size
impl IntoSpace for usize {
    fn into_space(self, _: Sealed) -> Space {
        Space::new().size(self)
    }
}
