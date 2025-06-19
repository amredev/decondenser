use crate::layout::Layout;
use crate::parse;
use crate::{Result, Str};
use std::path::PathBuf;

impl crate::Decondenser<'_> {
    /// Format any text according to brackets nesting and other simple rules.
    #[must_use = "this is a pure function; ignoring its result is definitely a bug"]
    pub fn decondense(&self, input: &str) -> Result<String> {
        let ast = parse::l2::parse(&parse::l1::ParseParams {
            input,
            config: self,
        });

        let mut layout = Layout::new(self);

        layout.begin_consistent(0);
        self.print(&mut layout, &ast);
        layout.end();

        Ok(layout.eof())
    }

    fn print<'a>(&self, layout: &mut Layout<'a>, nodes: &[parse::l2::AstNode<'a>]) {
        for node in nodes {
            match node {
                &parse::l2::AstNode::Space(content) => {
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
                        layout.space();
                    }
                }
                parse::l2::AstNode::Group(group) => {
                    layout.begin_consistent(self.indent.len() as isize);
                    layout.literal(group.opening);
                    layout.space_if_nonempty();
                    self.print(layout, &group.content);
                    layout.space_with_offset(-(self.indent.len() as isize));
                    if let Some(closing) = group.closing {
                        layout.literal(closing);
                    }
                    layout.end();
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
