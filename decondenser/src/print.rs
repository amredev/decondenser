use crate::layout::{Layout, SpaceParams};
use crate::{BreakStyle, parse};

impl crate::Decondenser {
    pub(crate) fn print<'a>(&self, layout: &mut Layout<'a>, nodes: &[parse::l2::AstNode<'a>]) {
        let mut nodes = nodes.iter();

        while let Some(node) = nodes.next() {
            match node {
                &parse::l2::AstNode::Space(content) => {
                    let has_empty_line =
                        content.chars().filter(|&c| c == '\n').take(2).count() == 2;

                    if has_empty_line {
                        layout.raw(content);
                    } else {
                        layout.raw(" ");
                    }
                }
                &parse::l2::AstNode::Raw(content) => {
                    layout.raw(content);
                }
                &parse::l2::AstNode::Punct(punct) => {
                    // if self.puncts(value) {}

                    layout.raw(&punct.content);

                    if matches!(punct.content.as_str(), "," | "?") {
                        layout.space(SpaceParams {
                            size: 1,
                            indent_diff: 0,
                        });
                    }
                }
                parse::l2::AstNode::Group(group) => {
                    let indent = (self.visual_size)(&self.indent).try_into().unwrap();

                    layout.raw(&group.config.opening);
                    layout.begin(indent, BreakStyle::Consistent);

                    layout.space(SpaceParams {
                        size: 1,
                        indent_diff: 0,
                    });

                    self.print(layout, &group.content);

                    if !group.content.is_empty() {
                        layout.space(SpaceParams {
                            size: 1,
                            indent_diff: -indent,
                        });
                    }

                    layout.end();

                    if group.closed {
                        layout.raw(&group.config.closing);
                    }
                }
                parse::l2::AstNode::Quoted(quoted) => {
                    layout.raw(&quoted.config.opening);

                    for content in &quoted.content {
                        layout.raw(content.text());
                    }

                    if quoted.closed {
                        layout.raw(&quoted.config.closing);
                    }
                }
            }
        }
    }
}
