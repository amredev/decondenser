mod cursor;
mod token_tree;

pub(crate) use token_tree::*;

use crate::{Decondenser, config};
use cursor::Cursor;
use std::mem;

pub(crate) fn parse<'a>(config: &'a Decondenser, input: &'a str) -> Vec<TokenTree<'a>> {
    let mut lexer = Parser {
        config,
        cursor: Cursor::new(input),
        output: Vec::new(),
    };
    lexer.parse(None);
    lexer.output
}

struct Parser<'a> {
    config: &'a Decondenser,
    cursor: Cursor<'a>,
    output: Vec<TokenTree<'a>>,
}

impl<'a> Parser<'a> {
    fn parse(&mut self, terminator: Option<&str>) -> Option<usize> {
        while let Some(char) = self.cursor.peek() {
            if char == '\n' {
                if !matches!(self.output.last(), Some(TokenTree::Newline { .. })) {
                    let start = self.cursor.byte_offset();
                    self.output.push(TokenTree::Newline { start });
                }

                self.cursor.next();
                continue;
            }

            if char.is_whitespace() {
                if !matches!(self.output.last(), Some(TokenTree::Space { .. })) {
                    let start = self.cursor.byte_offset();
                    self.output.push(TokenTree::Space { start });
                }

                self.cursor.next();
                continue;
            }

            if let Some(start) = terminator.and_then(|term| self.cursor.strip_prefix(term)) {
                return Some(start);
            }

            let group = self.config.groups.iter().find_map(|group_cfg| {
                Some((
                    self.cursor.strip_prefix(&group_cfg.opening.symbol)?,
                    group_cfg,
                ))
            });

            if let Some((opening, group_cfg)) = group {
                self.parse_group(opening, group_cfg);
                continue;
            }

            let quote = self.config.quotes.iter().find_map(|quote_cfg| {
                Some((self.cursor.strip_prefix(&quote_cfg.opening)?, quote_cfg))
            });

            if let Some((opening, quote_cfg)) = quote {
                self.parse_quoted(opening, quote_cfg);
                continue;
            }

            let punct = self
                .config
                .puncts
                .iter()
                .find_map(|punct| Some((punct, self.cursor.strip_prefix(&punct.symbol)?)));

            if let Some((config, start)) = punct {
                self.output.push(TokenTree::Punct(Punct { start, config }));
                continue;
            }

            if !matches!(self.output.last(), Some(TokenTree::Raw { .. })) {
                let start = self.cursor.byte_offset();
                self.output.push(TokenTree::Raw { start });
            }

            self.cursor.next();
        }

        None
    }

    fn parse_group(&mut self, opening: usize, config: &'a config::Group) {
        let prev = mem::take(&mut self.output);

        let closing = self.parse(Some(&config.closing.symbol));

        let group = Group {
            opening,
            content: mem::replace(&mut self.output, prev),
            closing,
            config,
        };

        self.output.push(TokenTree::Group(group));
    }

    fn parse_quoted(&mut self, opening: usize, config: &'a config::Quote) {
        let mut content = vec![];

        let closing = loop {
            let escape = config
                .escapes
                .iter()
                .find_map(|escape| self.cursor.strip_prefix(&escape.escaped));

            if let Some(start) = escape {
                content.push(QuotedContent::Escape { start });
                continue;
            }

            if let Some(closing) = self.cursor.strip_prefix(&config.closing) {
                break Some(closing);
            }

            if !matches!(content.last(), Some(QuotedContent::Raw { .. })) {
                let start = self.cursor.byte_offset();
                content.push(QuotedContent::Raw { start });
            }

            if self.cursor.next().is_none() {
                break None;
            }
        };

        let quoted = Quoted {
            opening,
            content,
            closing,
            config,
        };

        self.output.push(TokenTree::Quoted(quoted));
    }
}
