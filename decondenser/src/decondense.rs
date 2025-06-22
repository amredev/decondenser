use crate::layout::{BreakParams, BreaksKind, Layout};
use crate::parse;
use unicode_width::UnicodeWidthStr;

impl crate::Decondenser<'_> {
    /// Format any text according to brackets nesting and other simple rules.
    #[must_use = "this is a pure function; ignoring its result is definitely a bug"]
    pub fn decondense(&self, input: &str) -> String {
        let ast = parse::l2::parse(&parse::l1::ParseParams {
            input,
            config: self,
        });

        let mut layout = Layout::new(self);

        layout.begin(0, BreaksKind::Consistent);
        self.print(&mut layout, &ast);
        layout.end();

        layout.eof()
    }

    fn print<'a>(&self, layout: &mut Layout<'a>, nodes: &[parse::l2::AstNode<'a>]) {
        for node in nodes {
            match node {
                &parse::l2::AstNode::Space(_content) => {
                    layout.literal(" ");
                    // if (content.contains("\n")) {
                    // printer.hardbreak();
                    // }
                }
                &parse::l2::AstNode::Raw(content) => {
                    layout.literal(content);
                }
                &parse::l2::AstNode::Punct(content) => {
                    layout.literal(content);
                    if content == "," {
                        layout.break_(BreakParams {
                            blank_space: 1,
                            indent_diff: 0,
                        });
                    }
                }
                parse::l2::AstNode::Group(group) => {
                    let indent = self.indent.width().try_into().unwrap();

                    layout.literal(group.opening);
                    layout.begin(indent, BreaksKind::Consistent);

                    layout.break_(BreakParams {
                        blank_space: 1,
                        indent_diff: 0,
                    });

                    self.print(layout, &group.content);

                    if !group.content.is_empty() {
                        layout.break_(BreakParams {
                            blank_space: 1,
                            indent_diff: -indent,
                        });
                    }

                    layout.end();

                    if let Some(closing) = group.closing {
                        layout.literal(closing);
                    }
                }
                parse::l2::AstNode::Quoted(quoted) => {
                    layout.literal(quoted.opening);

                    for content in &quoted.content {
                        layout.literal(content.text());
                    }

                    if let Some(closing) = quoted.closing {
                        layout.literal(closing);
                    }
                }
            }
        }
    }
}
