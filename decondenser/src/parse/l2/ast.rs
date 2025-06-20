use std::fmt;

pub(crate) enum AstNode<'a> {
    Space(&'a str),
    Raw(&'a str),
    Punct(&'a str),
    Group(Group<'a>),
    Quoted(Quoted<'a>),
}

impl fmt::Debug for AstNode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AstNode::Space(text) => write!(f, "space {text:?}"),
            AstNode::Raw(text) => write!(f, "raw {text:?}"),
            AstNode::Punct(text) => write!(f, "punct {text:?}"),
            AstNode::Group(group) => write!(f, "group {group:?}"),
            AstNode::Quoted(quoted) => write!(f, "quoted {quoted:?}"),
        }
    }
}

pub(crate) struct Quoted<'a> {
    pub(crate) opening: &'a str,
    pub(crate) content: Vec<QuotedContent<'a>>,
    pub(crate) closing: Option<&'a str>,
}

impl fmt::Debug for Quoted<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} -> {:?} {:?}",
            self.opening,
            self.closing.unwrap_or("{none}"),
            self.content
        )
    }
}

pub(crate) enum QuotedContent<'a> {
    Raw(&'a str),
    Escape(&'a str),
}

impl fmt::Debug for QuotedContent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuotedContent::Raw(text) => write!(f, "qr:{text:?}"),
            QuotedContent::Escape(text) => write!(f, "qe:{text:?}"),
        }
    }
}

impl<'a> QuotedContent<'a> {
    pub(crate) fn text(&self) -> &'a str {
        match self {
            QuotedContent::Raw(text) => text,
            QuotedContent::Escape(text) => text,
        }
    }
}

pub(crate) struct Group<'a> {
    pub(crate) opening: &'a str,
    pub(crate) content: Vec<AstNode<'a>>,
    pub(crate) closing: Option<&'a str>,
}

impl fmt::Debug for Group<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} -> {:?} {:#?}",
            self.opening,
            self.closing.unwrap_or("{none}"),
            self.content
        )
    }
}
