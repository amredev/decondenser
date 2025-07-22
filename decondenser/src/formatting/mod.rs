mod engine;

use crate::{BreakStyleEnum as BreakStyle, SpaceFilterEnum as SpaceFilter, parsing};

use self::engine::{Formatter, MeasuredStr};
use crate::parsing::l2::TokenTree;

impl crate::Decondenser {
    /// This function lives here to keep the `lib.rs` file lean and focused on
    /// the public API of the `Decondenser` struct.
    pub(crate) fn decondense_impl(&self, input: &str) -> String {
        let tokens = parsing::l2::parse(self, input);

        let mut fmt = Formatter::new(self);

        fmt.begin(BreakStyle::Consistent);

        FormattingCtx {
            config: self,
            fmt: &mut fmt,
            tokens: TokensCursor {
                tokens: tokens.iter(),
            },
        }
        .format();

        fmt.end();
        fmt.eoi()
    }
}

struct FormattingCtx<'f, 'i> {
    config: &'i crate::Decondenser,
    fmt: &'f mut Formatter<'i>,
    tokens: TokensCursor<'i>,
}

#[derive(Clone, Copy)]
enum Blank<'i> {
    Space(&'i str),
    Newline(usize),
}

impl<'i> FormattingCtx<'_, 'i> {
    pub(crate) fn format(mut self) {
        // Skip leading blanks if they exist
        while self.tokens.optional_blank().is_some() {}

        while let Some(node) = self.tokens.next() {
            match node {
                TokenTree::Space(space) => self.on_blank(Blank::Space(space)),
                TokenTree::Newline(count) => self.on_blank(Blank::Newline(*count)),
                TokenTree::Raw(content) => self.fmt.raw(self.measured_str(content)),
                TokenTree::Punct(punct) => self.on_punct(None, punct),
                TokenTree::Group(group) => self.on_group(None, group),
                TokenTree::Quoted(quoted) => {
                    self.fmt.raw(self.measured_str(&quoted.config.opening));

                    for content in &quoted.content {
                        self.fmt.raw(self.measured_str(content.text()));
                    }

                    if quoted.closed {
                        self.fmt.raw(self.measured_str(&quoted.config.closing));
                    }
                }
            }
        }
    }

    fn on_blank(&mut self, leading_blank: Blank<'i>) {
        let Some(peeked) = self.tokens.peek() else {
            // No need for trailing blanks
            return;
        };

        match peeked.token {
            TokenTree::Punct(punct) => {
                peeked.consume();
                self.on_punct(Some(leading_blank), punct);
                return;
            }
            TokenTree::Group(group) => {
                peeked.consume();
                self.on_group(Some(leading_blank), group);
                return;
            }
            _ => {}
        }

        match leading_blank {
            Blank::Space(_) => self.fmt.space(1),
            Blank::Newline(count) => {
                if self.config.preserve_newlines {
                    self.fmt.hard_break(std::cmp::min(count, 2));
                } else {
                    self.fmt.soft_break();
                    self.fmt.space(1);
                }
            }
        }
    }

    fn measured_str<'a>(&self, str: &'a str) -> MeasuredStr<'a> {
        MeasuredStr::new(str, self.config.visual_size)
    }

    fn on_group(&mut self, leading_blank: Option<Blank<'i>>, group: &'i parsing::l2::Group<'i>) {
        let config = &group.config;

        self.fmt.begin(group.config.break_style.0);

        let is_empty_group = group
            .content
            .iter()
            .all(|token| matches!(token, TokenTree::Newline(_) | TokenTree::Space(_)));

        // Trim blank-only groups to a single line always
        let tokens = if is_empty_group {
            [].iter()
        } else {
            group.content.iter()
        };

        todo!("Fix the handling of the closing token's leading blank");
        let closing_punct_leading_blank = tokens.last().clone().iter().next_back();

        let mut content = FormattingCtx {
            config: self.config,
            fmt: &mut *self.fmt,
            tokens: TokensCursor { tokens },
        };

        content.on_punct(leading_blank, &config.opening);
        content.fmt.indent(1);
        content.format();
        self.fmt.indent(-1);

        if group.closed {
            let leading_blank = self.tokens.optional_blank().filter(|_| !is_empty_group);
            self.on_punct(leading_blank, &config.closing);
        }

        self.fmt.end();
    }

    fn on_punct(&mut self, leading_blank: Option<Blank<'i>>, punct: &'i crate::Punct) {
        self.blank_near_punct(leading_blank, &punct.leading_space);
        self.fmt.raw(self.measured_str(&punct.symbol));

        let trailing_blank = self.tokens.optional_blank();
        self.blank_near_punct(trailing_blank, &punct.trailing_space);
    }

    fn blank_near_punct(&mut self, input: Option<Blank<'i>>, config: &'i crate::Space) {
        match input {
            None => self.space_near_punct("", config),
            Some(Blank::Space(space)) => self.space_near_punct(space, config),
            Some(Blank::Newline(count)) => self.newline_near_punct(count, config),
        }
    }

    fn newline_near_punct(&mut self, count: usize, config: &'i crate::Space) {
        if self.config.preserve_newlines {
            self.fmt.hard_break(std::cmp::min(count, 2));
        } else {
            self.fmt.soft_break();
            self.space_near_punct(" ", config);
        }
    }

    fn space_near_punct(&mut self, input: &str, config: &'i crate::Space) {
        let size = config
            .size
            .unwrap_or_else(|| (self.config.visual_size)(input));

        let soft_break = match config.breakable.0 {
            SpaceFilter::Bool(bool) => bool,
            SpaceFilter::MinSize(min_size) => size >= min_size,
        };

        if soft_break {
            self.fmt.soft_break();
        }

        self.fmt.space(size);
    }
}

struct TokensCursor<'i> {
    tokens: std::slice::Iter<'i, TokenTree<'i>>,
}

impl<'i> TokensCursor<'i> {
    fn next(&mut self) -> Option<&'i TokenTree<'i>> {
        self.tokens.next()
    }
    fn peek(&mut self) -> Option<Peeked<'_, 'i>> {
        Peeked::new(&mut self.tokens)
    }
    fn optional_blank(&mut self) -> Option<Blank<'i>> {
        self.peek()?.consume_blank()
    }
}

struct Peeked<'t, 'i> {
    token: &'i TokenTree<'i>,
    tokens: &'t mut std::slice::Iter<'i, TokenTree<'i>>,
}

impl<'t, 'i> Peeked<'t, 'i> {
    fn new(tokens: &'t mut std::slice::Iter<'i, TokenTree<'i>>) -> Option<Self> {
        Some(Self {
            token: tokens.clone().next()?,
            tokens,
        })
    }

    fn consume(self) {
        self.tokens.next();
    }

    fn consume_blank(self) -> Option<Blank<'i>> {
        match self.token {
            TokenTree::Space(space) => {
                self.consume();
                Some(Blank::Space(space))
            }
            TokenTree::Newline(count) => {
                self.consume();
                Some(Blank::Newline(*count))
            }
            _ => None,
        }
    }
}
