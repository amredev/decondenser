mod engine;

use self::engine::{Formatter, MeasuredStr};
use crate::config::BreakStyleEnum as BreakStyle;
use crate::parsing;
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

impl<'i> FormattingCtx<'_, 'i> {
    pub(crate) fn format(mut self) {
        // Skip leading blanks if they exist
        while self.tokens.optional_space().is_some() {}

        while let Some(node) = self.tokens.next() {
            match node {
                TokenTree::Space(space) => self.on_space(space),
                TokenTree::Newline(_count) => self.on_newline(),
                TokenTree::Raw(content) => self.fmt.raw(self.measured_str(content)),
                TokenTree::Punct(punct) => self.on_punct(None, punct),
                TokenTree::Group(group) => self.on_group(None, group),
                TokenTree::Quoted(quoted) => self.on_quoted(quoted),
            }
        }
    }

    fn on_quoted(&mut self, quoted: &'i parsing::l2::Quoted<'i>) {
        self.fmt.raw(self.measured_str(&quoted.config.opening));

        for content in &quoted.content {
            self.fmt.raw(self.measured_str(content.text()));
        }

        if quoted.closed {
            self.fmt.raw(self.measured_str(&quoted.config.closing));
        }
    }

    fn on_newline(&mut self) {
        self.fmt.soft_break();
        self.fmt.space(1);
    }

    fn on_space(&mut self, space: &'i str) {
        let Some(peeked) = self.tokens.peek() else {
            // No need for trailing blanks
            return;
        };

        match peeked.token {
            TokenTree::Punct(punct) => {
                peeked.consume();
                self.on_punct(Some(space), punct);
            }
            TokenTree::Group(group) => {
                peeked.consume();
                self.on_group(Some(space), group);
            }
            _ => self.fmt.space(1),
        }
    }

    fn measured_str<'a>(&self, str: &'a str) -> MeasuredStr<'a> {
        self.config.visual_size.measured_str(str)
    }

    fn on_group(&mut self, leading_space: Option<&'i str>, group: &'i parsing::l2::Group<'i>) {
        let config = group.config;

        self.fmt.begin(group.config.break_style.0);

        let is_empty_group = group
            .content
            .iter()
            .all(|token| matches!(token, TokenTree::Newline(_) | TokenTree::Space(_)));

        // Trim blank-only groups to a single line always
        if is_empty_group {
            self.empty_group(leading_space, group);
            return;
        }

        let mut tokens = group.content.iter();

        let closing_punct_leading_blank = tokens
            .clone()
            .next_back()
            .and_then(token_tree_to_space)
            .inspect(|_| _ = tokens.next_back());

        let mut content = FormattingCtx {
            config: self.config,
            fmt: &mut *self.fmt,
            tokens: TokensCursor { tokens },
        };

        content.on_punct(leading_space, &config.opening);
        content.fmt.indent(1);
        content.format();
        self.fmt.indent(-1);

        if group.closed {
            self.on_punct(closing_punct_leading_blank, &config.closing);
        }

        self.fmt.end();
    }

    // Special case for an empty group where we don't want any internal space,
    // and instead have a pair of adjacent opening and closing punctuation.
    fn empty_group(&mut self, leading_space: Option<&'i str>, group: &parsing::l2::Group<'i>) {
        let config = &group.config;
        self.space_near_punct(leading_space, &config.opening.leading_space);
        self.fmt.raw(self.measured_str(&config.opening.symbol));

        if group.closed {
            self.fmt.raw(self.measured_str(&config.closing.symbol));
            let trailing_space = self.tokens.optional_space();
            self.space_near_punct(trailing_space, &config.closing.trailing_space);
        }
    }

    fn on_punct(&mut self, leading_space: Option<&'i str>, punct: &'i crate::Punct) {
        self.space_near_punct(leading_space, &punct.leading_space);
        self.fmt.raw(self.measured_str(&punct.symbol));

        let trailing_space = self.tokens.optional_space();
        self.space_near_punct(trailing_space, &punct.trailing_space);
    }

    fn space_near_punct(&mut self, input: Option<&'i str>, config: &'i crate::Space) {
        let input = input.unwrap_or("");

        let (min, max) = config.size;
        let size = if min == max {
            // We have a fixed-size space. No need to measure it.
            min
        } else {
            // Preserve the size from input, but clamp it to the range
            self.config.visual_size.measure(input).clamp(min, max)
        };

        if config.breakable {
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
    fn optional_space(&mut self) -> Option<&'i str> {
        self.peek()?.consume_space()
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

    fn consume_space(self) -> Option<&'i str> {
        token_tree_to_space(self.token).inspect(|_| self.consume())
    }
}

fn token_tree_to_space<'i>(token: &'i TokenTree<'i>) -> Option<&'i str> {
    match token {
        TokenTree::Space(space) => Some(space),
        _ => None,
    }
}
