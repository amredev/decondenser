use crate::config;
use std::fmt;

pub(crate) enum TokenTree<'a> {
    Space(&'a str),
    NewLine(usize),
    Raw(&'a str),
    Punct(&'a config::Punct),
    Group(Group<'a>),
    Quoted(Quoted<'a>),
}

impl fmt::Debug for TokenTree<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Space(text) => write!(f, "space {text:?}"),
            Self::NewLine(count) => write!(f, "newline {count}"),
            Self::Raw(text) => write!(f, "raw {text:?}"),
            Self::Punct(punct) => write!(f, "punct {:?}", punct.symbol),
            Self::Group(group) => write!(f, "group {group:?}"),
            Self::Quoted(quoted) => write!(f, "quoted {quoted:?}"),
        }
    }
}

pub(crate) struct Quoted<'a> {
    pub(crate) content: Vec<QuotedContent<'a>>,
    pub(crate) closed: bool,
    pub(crate) config: &'a config::Quote,
}

impl fmt::Debug for Quoted<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let closing = if self.closed {
            &self.config.closing
        } else {
            "{none}"
        };

        write!(
            f,
            "{:?} -> {closing:?} {:?}",
            self.config.opening, self.content
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
            Self::Raw(text) => write!(f, "qr:{text:?}"),
            Self::Escape(text) => write!(f, "qe:{text:?}"),
        }
    }
}

impl<'a> QuotedContent<'a> {
    pub(crate) fn text(&self) -> &'a str {
        match self {
            Self::Raw(text) | Self::Escape(text) => text,
        }
    }
}

pub(crate) struct Group<'a> {
    pub(crate) content: Vec<TokenTree<'a>>,
    pub(crate) closed: bool,
    pub(crate) config: &'a config::Group,
}

impl fmt::Debug for Group<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let closing = if self.closed {
            &self.config.closing.symbol
        } else {
            "{none}"
        };

        write!(
            f,
            "{:?} -> {closing:?} {:#?}",
            self.config.opening, self.content
        )
    }
}
