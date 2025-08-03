use super::l1::{self, Unescaped};
use crate::cursor::Cursor;
use std::fmt;

pub(super) fn unescape(input: &str) -> impl Iterator<Item = Token<'_>> {
    let cursor = Cursor::new(input);
    let mut tokens = l1::Lexer::new(cursor).peekable();

    std::iter::from_fn(move || {
        let token = tokens.next()?;

        let end = tokens
            .peek()
            .map(|token| token.start())
            .unwrap_or(input.len());

        Some(match token {
            l1::Token::Raw(start) => Token::Raw(&input[start..end]),
            l1::Token::Escape(escape) => Token::Escape(Escape {
                source: &input[escape.start..end],
                unescaped: escape.unescaped,
            }),
        })
    })
}

pub(crate) enum Token<'i> {
    Raw(&'i str),
    Escape(Escape<'i>),
}

pub(crate) struct Escape<'i> {
    pub(crate) source: &'i str,
    pub(crate) unescaped: Unescaped,
}

impl fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Raw(text) => write!(f, "qraw:{text:?}"),
            Self::Escape(escape) => write!(f, "qesc:{:?} ({:?})", escape.source, escape.unescaped),
        }
    }
}

impl<'a> Token<'a> {
    pub(crate) fn source(&self) -> &'a str {
        match self {
            Self::Raw(text) => text,
            Self::Escape(escape) => escape.source,
        }
    }
}
