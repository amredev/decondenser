mod engine;

use crate::BreakStyle;
use crate::parsing;
use crate::parsing::l2::TokenTree;
use engine::{Formatter, MeasuredStr};

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
            tokens: tokens.iter(),
        }
        .format();

        fmt.end();
        fmt.eof()
    }
}

struct FormattingCtx<'fmt, 'input> {
    config: &'input crate::Decondenser,
    fmt: &'fmt mut Formatter<'input>,
    tokens: std::slice::Iter<'input, TokenTree<'input>>,
}

impl<'input> FormattingCtx<'_, 'input> {
    pub(crate) fn format(mut self) {
        // Skip leading space if it exists
        self.skip_space();

        while let Some(node) = self.tokens.next() {
            match node {
                TokenTree::Space(_content) => {
                    let next = self.tokens.clone().next();

                    match next {
                        Some(TokenTree::Punct(_) | TokenTree::Group(_) | TokenTree::NewLine(_))
                        | None => {
                            // Punct and Group delimiters define their own
                            // leading whitespace, and we also don't want to
                            // output trailing whitespace at the end of
                            // line/output, so skip this space.
                        }
                        _ => {
                            self.fmt.nbsp(1);
                        }
                    }
                }
                &TokenTree::NewLine(count) => {
                    if self.config.preserve_newlines {
                        self.fmt.newline(count.clamp(1, 2));
                    } else {
                        self.fmt.nbsp(1);
                    }
                    self.skip_space();
                }
                TokenTree::Raw(content) => {
                    self.fmt.raw(self.measured_str(content));
                }
                TokenTree::Punct(punct) => {
                    self.space(&punct.leading_space);
                    self.fmt.raw(self.measured_str(&punct.symbol));

                    if self.tokens.clone().next().is_none() {
                        return;
                    }

                    self.trailing_space(&punct.trailing_space);
                }
                TokenTree::Group(group) => {
                    let config = &group.config;

                    self.fmt.begin(BreakStyle::Consistent);

                    self.space(&config.opening.leading_space);
                    self.fmt.raw(self.measured_str(&config.opening.symbol));
                    self.space(&config.opening.trailing_space);

                    let indent = config.opening.trailing_space.breakable
                        || config.closing.leading_space.breakable;

                    if indent {
                        self.fmt.indent(1);
                    }

                    FormattingCtx {
                        config: self.config,
                        fmt: &mut *self.fmt,
                        tokens: group.content.iter(),
                    }
                    .format();

                    if indent {
                        self.fmt.indent(-1);
                    }

                    if group.closed {
                        if !group.content.is_empty() {
                            self.space(&config.closing.leading_space);
                        }
                        self.fmt.raw(self.measured_str(&config.closing.symbol));
                        self.space(&config.closing.trailing_space);

                        self.skip_space();
                    }

                    self.fmt.end();
                }
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

    fn skip_space(&mut self) {
        while let Some(next) = self.tokens.clone().next() {
            if !matches!(next, TokenTree::Space(_) | TokenTree::NewLine(_)) {
                return;
            }
            self.tokens.next();
        }
    }

    fn measured_str<'a>(&self, str: &'a str) -> MeasuredStr<'a> {
        MeasuredStr::new(str, self.config.visual_size)
    }

    fn trailing_space(&mut self, space: &'input crate::Space) {
        self.skip_space();

        let Some(next) = self.tokens.clone().next() else {
            // Prevent trailing space at the end of the output
            return;
        };

        if let TokenTree::NewLine(_) = next {
            return;
        }

        self.space(space);
    }

    fn space(&mut self, space: &'input crate::Space) {
        let size = space.size.expect("TODO: handle preserving spaces");

        if space.breakable {
            self.fmt.bsp(size);
        } else {
            self.fmt.nbsp(size);
        }
    }
}
