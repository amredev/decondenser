use crate::BreakStyle;
use crate::layout::{Layout, SpaceParams};
use crate::parse::l2::AstNode;

impl crate::Decondenser {
    pub(crate) fn print<'a>(&self, layout: &mut Layout<'a>, nodes: &[AstNode<'a>]) {
        let mut nodes = nodes.iter();

        // Skip leading space if it exists
        if let Some(AstNode::Space(_)) = nodes.clone().next() {
            nodes.next();
        }

        while let Some(node) = nodes.next() {
            match node {
                &AstNode::Space(content) => {
                    let has_empty_line =
                        content.chars().filter(|&c| c == '\n').take(2).count() == 2;

                    if has_empty_line {
                        // TODO: Make this configurable with something like
                        // `preserve_newlines` bool?
                    }

                    let next = nodes.clone().next();

                    match next {
                        Some(AstNode::Punct(_) | AstNode::Group(_)) | None => {
                            // Punct and Group delimiters define their own
                            // leading whitespace, and we also don't want to
                            // output trailing whitespace at the end of output,
                            // so skip this space.
                        }
                        _ => {
                            layout.raw(" ");
                        }
                    }
                }
                &AstNode::Raw(content) => {
                    layout.raw(content);
                }
                &AstNode::Punct(punct) => {
                    // TODO: DRY-it-up
                    if punct.leading_space.break_if_needed {
                        layout.space(SpaceParams {
                            size: (self.visual_size)(&punct.leading_space.content),
                            indent_diff: 0,
                        });
                    } else {
                        layout.raw(&punct.leading_space.content);
                    }

                    layout.raw(&punct.content);

                    let next = nodes.clone().next();

                    if let Some(AstNode::Space(_)) = next {
                        nodes.next();
                    }

                    if nodes.clone().next().is_none() {
                        return;
                    }

                    if punct.trailing_space.break_if_needed {
                        layout.space(SpaceParams {
                            // TODO: preserve the content of `trailing_space`
                            size: (self.visual_size)(&punct.trailing_space.content),
                            indent_diff: 0,
                        });
                    } else {
                        layout.raw(&punct.trailing_space.content);
                    }
                }
                AstNode::Group(group) => {
                    let indent = (self.visual_size)(&self.indent).try_into().unwrap();

                    layout.raw(&group.config.opening.leading_space);
                    layout.raw(&group.config.opening.content);

                    layout.begin(indent, BreakStyle::Consistent);

                    layout.space(SpaceParams {
                        size: (self.visual_size)(&group.config.opening.trailing_space),
                        indent_diff: 0,
                    });

                    self.print(layout, &group.content);

                    if !group.content.is_empty() {
                        layout.space(SpaceParams {
                            size: (self.visual_size)(&group.config.closing.leading_space),
                            indent_diff: -indent,
                        });
                    }

                    layout.end();

                    if group.closed {
                        layout.raw(&group.config.closing.content);
                        layout.raw(&group.config.closing.trailing_space);

                        if let Some(AstNode::Space(_)) = nodes.clone().next() {
                            nodes.next();
                        }
                    }
                }
                AstNode::Quoted(quoted) => {
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
