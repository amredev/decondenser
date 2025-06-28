mod engine;

use crate::BreakStyle;
use crate::parsing;
use crate::parsing::l2::AstNode;
use engine::{Formatter, MeasuredStr};

impl crate::Decondenser {
    /// This function lives here to keep the `lib.rs` file lean and focused on
    /// the public API of the `Decondenser` struct.
    pub(crate) fn decondense_impl(&self, input: &str) -> String {
        let ast = parsing::l2::parse(self, input);

        let mut fmt = Formatter::new(self);

        fmt.begin(BreakStyle::Consistent);
        self.format_ast(&mut fmt, &ast);
        fmt.end();

        fmt.eof()
    }

    pub(crate) fn format_ast<'a>(&self, fmt: &mut Formatter<'a>, nodes: &[AstNode<'a>]) {
        let mut nodes = nodes.iter();

        // Skip leading space if it exists
        Self::skip_space(&mut nodes);

        while let Some(node) = nodes.next() {
            match node {
                &AstNode::Space(content) => {
                    let next = nodes.clone().next();

                    match next {
                        Some(AstNode::Punct(_) | AstNode::Group(_)) | None => {
                            // Punct and Group delimiters define their own
                            // leading whitespace, and we also don't want to
                            // output trailing whitespace at the end of output,
                            // so skip this space.
                        }
                        _ => {
                            fmt.raw(MeasuredStr::new(content, self.visual_size));
                        }
                    }
                }
                &AstNode::Raw(content) => {
                    fmt.raw(self.measured_str(content));
                }
                &AstNode::Punct(punct) => {
                    self.space(fmt, &punct.leading_space);
                    fmt.raw(self.measured_str(&punct.content));

                    Self::skip_space(&mut nodes);

                    if nodes.clone().next().is_none() {
                        return;
                    }

                    self.space(fmt, &punct.trailing_space);
                }
                AstNode::Group(group) => {
                    let config = &group.config;

                    fmt.begin(BreakStyle::Consistent);

                    self.space(fmt, &config.opening.leading_space);
                    fmt.raw(self.measured_str(&config.opening.content));
                    self.space(fmt, &config.opening.trailing_space);

                    fmt.indent(1);
                    self.format_ast(fmt, &group.content);
                    fmt.indent(-1);

                    if group.closed {
                        if !group.content.is_empty() {
                            self.space(fmt, &config.closing.leading_space);
                        }
                        fmt.raw(self.measured_str(&config.closing.content));
                        self.space(fmt, &config.closing.trailing_space);

                        Self::skip_space(&mut nodes);
                    }

                    fmt.end();
                }
                AstNode::Quoted(quoted) => {
                    fmt.raw(self.measured_str(&quoted.config.opening));

                    for content in &quoted.content {
                        fmt.raw(self.measured_str(content.text()));
                    }

                    if quoted.closed {
                        fmt.raw(self.measured_str(&quoted.config.closing));
                    }
                }
            }
        }
    }

    fn skip_space<'item, 'input>(iter: &mut (impl Iterator<Item = &'item AstNode<'input>> + Clone))
    where
        'input: 'item,
    {
        if let Some(AstNode::Space(_)) = iter.clone().next() {
            iter.next();
        }
    }

    fn measured_str<'a>(&self, str: &'a str) -> MeasuredStr<'a> {
        MeasuredStr::new(str, self.visual_size)
    }

    fn space<'a>(&self, fmt: &mut Formatter<'a>, space: &'a crate::Space) {
        let content = self.measured_str(&space.content);

        if space.breakable {
            fmt.space(content);
        } else {
            fmt.raw(content);
        }
    }
}
