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
        let mut nodes = nodes.iter();

        while let Some(node) = nodes.next() {
            match node {
                &parse::l2::AstNode::Space(content) => {
                    let has_empty_line =
                        content.chars().filter(|&c| c == '\n').take(2).count() == 2;

                    todo!("Handle empty lines properly");
                    if has_empty_line {
                        layout.literal(content);
                    } else {
                        layout.literal(" ");
                    }
                }
                &parse::l2::AstNode::Raw(content) => {
                    layout.literal(content);
                }
                &parse::l2::AstNode::Punct(content) => {
                    layout.literal(content);
                    if matches!(content, "," | "?") {
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
