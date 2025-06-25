use crate::config;
use std::fmt;

pub(crate) enum AstNode<'a> {
    Space { start: usize },
    Raw { start: usize },
    Punct(Punct<'a>),
    Group(Group<'a>),
    Quoted(Quoted<'a>),
}

impl AstNode<'_> {
    pub(crate) fn start(&self) -> usize {
        match self {
            Self::Space { start } | Self::Raw { start } => *start,
            Self::Punct(punct) => punct.start,
            Self::Group(group) => group.opening,
            Self::Quoted(quoted) => quoted.opening,
        }
    }
}

impl fmt::Debug for AstNode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Space { start } => write!(f, "space {start}"),
            Self::Raw { start } => write!(f, "raw {start}"),
            Self::Punct(punct) => write!(f, "punct{punct:#?}"),
            Self::Group(group) => write!(f, "group{group:#?}"),
            Self::Quoted(quoted) => write!(f, "quoted{quoted:#?}"),
        }
    }
}

pub(crate) struct Quoted<'a> {
    pub(crate) opening: usize,
    pub(crate) content: Vec<QuotedContent>,
    pub(crate) closing: Option<usize>,
    pub(crate) config: &'a config::Quote,
}

impl fmt::Debug for Quoted<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let closing: &dyn fmt::Display = if let Some(closing) = self.closing {
            &format!("{closing}:{:?}", self.config.closing)
        } else {
            &"{none}"
        };

        write!(
            f,
            "({}: {} -> {closing}) {:?}",
            self.opening, self.config.opening, self.content
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

pub(crate) struct Punct<'a> {
    pub(crate) start: usize,
    pub(crate) config: &'a config::Punct,
}

impl fmt::Debug for Punct<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.start, self.config.content,)
    }
}

pub(crate) struct Group<'a> {
    pub(crate) opening: usize,
    pub(crate) content: Vec<AstNode<'a>>,
    pub(crate) closing: Option<usize>,
    pub(crate) config: &'a config::Group,
}

impl fmt::Debug for Group<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let closing: &dyn fmt::Display = if let Some(closing) = self.closing {
            &format!("{closing}:{:?}", self.config.closing)
        } else {
            &"{none}"
        };

        write!(
            f,
            "({}: {} -> {closing}) {:#?}",
            self.opening, self.config.opening.content, self.content
        )
    }
}
