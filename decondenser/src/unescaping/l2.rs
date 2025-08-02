use super::l1::{self, Unescaped};

pub(super) fn unescape(input: &str) -> impl Iterator<Item = Token<'_>> {
    let mut tokens = l1::Lexer::new(input).peekable();

    std::iter::from_fn(move || {
        let token = tokens.next()?;

        let end = tokens
            .peek()
            .map(|token| token.start())
            .unwrap_or(input.len());

        Some(match token {
            l1::Token::Raw(start) => Token::Raw(&input[start..end]),
            l1::Token::Escape(escape) => Token::Escape(Escape {
                escaped: &input[escape.start..end],
                unescaped: escape.unescaped,
            }),
        })
    })
}

pub(super) enum Token<'i> {
    Raw(&'i str),
    Escape(Escape<'i>),
}

pub(super) struct Escape<'i> {
    pub(super) escaped: &'i str,
    pub(super) unescaped: Unescaped,
}
