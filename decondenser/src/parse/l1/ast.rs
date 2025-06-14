#[derive(Debug)]
pub(crate) enum AstNode {
    Whitespace { start: usize },
    Raw { start: usize },
    Punct { start: usize },
    Group(Group),
    Quoted(Quoted),
}

impl AstNode {
    pub(crate) fn start(&self) -> usize {
        match self {
            AstNode::Whitespace { start } => *start,
            AstNode::Raw { start } => *start,
            AstNode::Punct { start } => *start,
            AstNode::Group(group) => group.opening,
            AstNode::Quoted(quoted) => quoted.opening,
        }
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub(crate) enum QuotedContent {
    Raw { start: usize },
    Escape { start: usize },
}

impl QuotedContent {
    pub(crate) fn start(&self) -> usize {
        match self {
            QuotedContent::Raw { start } => *start,
            QuotedContent::Escape { start } => *start,
        }
    }
}

#[derive(Debug)]
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
