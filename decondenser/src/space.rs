/// Defines the rules for inserting space characters and line breaks.
///
/// Use the [`From`] impl of this type to create either one of the two kinds of
/// space:
/// - [`PreservingSpace`] - which preserves the number of spaces from the input
/// - [`FixedSpace`] - which inserts a fixed number of spaces.
#[derive(Debug, Clone)]
pub struct Space {
    pub(crate) kind: SpaceKind,
}

impl From<PreservingSpace> for Space {
    fn from(value: PreservingSpace) -> Self {
        Self {
            kind: SpaceKind::Preserving(value),
        }
    }
}

impl From<FixedSpace> for Space {
    fn from(value: FixedSpace) -> Self {
        Self {
            kind: SpaceKind::Fixed(value),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum SpaceKind {
    Preserving(PreservingSpace),
    Fixed(FixedSpace),
}

/// A space that preserves the number of spaces from the input
#[derive(Debug, Clone)]
pub struct PreservingSpace {
    pub(crate) soft_break: Option<SoftBreak>,
}

impl PreservingSpace {
    /// Creates a new [`PreservingSpace`] with the default settings.
    ///
    /// Make sure to explicitly enable [`Self::soft_break`] if you want the space to
    /// be considered for turning into a newline when the content does not fit on a
    /// single line, otherwise the space will always stay static and it'll never be
    /// turned into a line break.
    #[must_use]
    pub fn new() -> Self {
        Self { soft_break: None }
    }

    /// Sets whether this space can be turned into a line break if content
    /// overflows or not. The [`SoftBreak`] provides the configuration of the
    /// conditions under which the space will be considered for a soft break.
    ///
    /// By default, the space is not considered for a soft break.
    #[must_use]
    pub fn soft_break(mut self, value: SoftBreak) -> Self {
        self.soft_break = Some(value);
        self
    }
}

/// Defines the conditions under which a preserving space will be considered
/// breakable.
#[derive(Debug, Clone)]
#[cfg_attr(not(feature = "unstable"), non_exhaustive)]
pub enum SoftBreak {
    /// The [`PreservingSpace`] will unconditionally be a soft break
    Always,

    /// The [`PreservingSpace`] will have a soft break only if it's not empty
    WhenNonEmpty,
}

/// Creates a [`Space`] with the fixed number of whitespace characters.
///
/// Make sure to explicitly enable [`Space::breakable`] if you want the
/// space to be considered for turning into a newline when the content does
/// not fit on a single line, otherwise the space will always stay static
/// and it'll never be turned into a line break.
#[derive(Debug, Clone)]
pub struct FixedSpace {
    pub(crate) size: usize,
    pub(crate) soft_break: bool,
}

impl FixedSpace {
    /// Creates a new [`FixedSpace`] with the given size.
    #[must_use]
    pub fn new(size: usize) -> Self {
        Self {
            size,
            soft_break: false,
        }
    }

    /// Sets whether this space can be turned into a line break if content
    /// overflows or not.
    ///
    /// By default, this is set to `false`.
    #[must_use]
    pub fn soft_break(mut self, value: bool) -> Self {
        self.soft_break = value;
        self
    }
}
