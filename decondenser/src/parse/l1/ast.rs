use std::fmt;

pub(crate) enum AstNode {
    Space { start: usize },
    Raw { start: usize },
    Punct { start: usize },
    Group(Group),
    Quoted(Quoted),
}

impl fmt::Debug for AstNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Space { start } => write!(f, "space {start}"),
            Self::Raw { start } => write!(f, "raw {start}"),
            Self::Punct { start } => write!(f, "punct {start}"),
            Self::Group(group) => write!(f, "group{group:#?}"),
            Self::Quoted(quoted) => write!(f, "quoted{quoted:#?}"),
        }
    }
}

impl AstNode {
    pub(crate) fn start(&self) -> usize {
        match self {
            Self::Space { start } | Self::Raw { start } | Self::Punct { start } => *start,
            Self::Group(group) => group.opening,
            Self::Quoted(quoted) => quoted.opening,
        }
    }
}

pub(crate) struct Quoted {
    /// Offset of the opening quote
    pub(crate) opening: usize,

    /// Offset of the content, which is also the offset of the character that
    /// follows the opening quote. Will be equal to `closing` if the content
    /// is empty.
    pub(crate) content: Vec<QuotedContent>,

    /// Offset of the closing quote. Can be `None` if the quotes are not closed
    /// (probably a malformed input).
    pub(crate) closing: Option<usize>,
}

impl fmt::Debug for Quoted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({} -> {:?}) {:?}",
            self.opening,
            MaybeUsize(self.closing),
            self.content
        )
    }
}

pub(crate) enum QuotedContent {
    Raw { start: usize },
    Escape { start: usize },
}

impl fmt::Debug for QuotedContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Raw { start } => write!(f, "qr:{start}"),
            Self::Escape { start } => write!(f, "qe:{start}"),
        }
    }
}

impl QuotedContent {
    pub(crate) fn start(&self) -> usize {
        match self {
            Self::Raw { start } | Self::Escape { start } => *start,
        }
    }
}

pub(crate) struct Group {
    /// The start offset of the opening delimiter
    pub(crate) opening: usize,

    /// The first node contains the start offset of the content of the group,
    /// unless the group is empty.
    pub(crate) content: Vec<AstNode>,

    /// Offset of the closing delimiter. Can be `None` if the group is not closed
    /// (probably a malformed input).
    pub(crate) closing: Option<usize>,
}

impl fmt::Debug for Group {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({} -> {:?}) {:#?}",
            self.opening,
            MaybeUsize(self.closing),
            self.content
        )
    }
}

struct MaybeUsize(Option<usize>);

impl fmt::Debug for MaybeUsize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(num) => write!(f, "{num}"),
            None => f.write_str("{none]"),
        }
    }
}
