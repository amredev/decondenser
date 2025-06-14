mod ast;
mod cursor;

pub(crate) use ast::*;

use crate::{Decondenser, EscapeConfig, GroupConfig, QuoteConfig, Str};

use crate::error::Result;
use cursor::Cursor;
use std::borrow::Cow;
use std::marker::PhantomData;
use std::mem;

pub(crate) struct ParseParams<'a> {
    pub(crate) input: &'a str,
    pub(crate) config: &'a Decondenser<'a>,
}

pub(crate) fn parse(params: &ParseParams<'_>) -> Vec<AstNode> {
    let mut lexer = Parser {
        config: params.config,
        cursor: Cursor::new(params.input),
        output: Vec::new(),
    };
    lexer.parse(None);
    lexer.output
}

struct Parser<'a> {
    config: &'a Decondenser<'a>,
    cursor: Cursor<'a>,
    output: Vec<AstNode>,
}

impl Parser<'_> {
    fn parse(&mut self, terminator: Option<&Str<'_>>) -> Option<usize> {
        while self.cursor.peek().is_some() {
            self.whitespace();

            if let Some(start) = terminator.and_then(|term| self.cursor.strip_prefix(term)) {
                return Some(start);
            }

            let group_cfg = self.config.groups.iter().find_map(|group_cfg| {
                Some((self.cursor.strip_prefix(&group_cfg.opening)?, group_cfg))
            });

            if let Some((opening, group_cfg)) = group_cfg {
                self.parse_group(opening, group_cfg);
                continue;
            }

            let quote_cfg = self.config.quotes.iter().find_map(|quote_cfg| {
                Some((self.cursor.strip_prefix(&quote_cfg.opening)?, quote_cfg))
            });

            if let Some((opening, quote_cfg)) = quote_cfg {
                self.parse_quoted(opening, quote_cfg);
                continue;
            }

            let punct = self
                .config
                .puncts
                .iter()
                .find_map(|punct| self.cursor.strip_prefix(punct));

            if let Some(start) = punct {
                self.output.push(AstNode::Punct { start });
                continue;
            }

            if !matches!(self.output.last(), Some(AstNode::Raw { .. })) {
                let start = self.cursor.byte_offset();
                self.output.push(AstNode::Raw { start });
            }

            self.cursor.next();
        }

        None
    }

    fn parse_group(&mut self, opening: usize, group_cfg: &GroupConfig<'_>) {
        let prev = mem::take(&mut self.output);

        let closing = self.parse(Some(&group_cfg.closing));

        let group = Group {
            opening,
            content: mem::replace(&mut self.output, prev).into(),
            closing,
        };

        self.output.push(AstNode::Group(group));
    }

    fn parse_quoted(&mut self, opening: usize, quote_cfg: &QuoteConfig<'_>) {
        let mut content = vec![];

        let closing = loop {
            let escape = quote_cfg
                .escapes
                .iter()
                .find_map(|escape| self.cursor.strip_prefix(&escape.escaped));

            if let Some(start) = escape {
                content.push(QuotedContent::Escape { start });
                continue;
            }

            if let Some(closing) = self.cursor.strip_prefix(&quote_cfg.closing) {
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
        };

        self.output.push(AstNode::Quoted(quoted));
    }

    fn whitespace(&mut self) {
        let Some(char) = self.cursor.peek() else {
            return;
        };

        if !char.is_whitespace() {
            return;
        }

        let start = self.cursor.byte_offset();
        self.output.push(AstNode::Whitespace { start });

        while let Some(char) = self.cursor.peek() {
            if !char.is_whitespace() {
                return;
            }
            self.cursor.next();
        }
    }
}
