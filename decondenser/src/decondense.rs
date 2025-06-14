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

        let mut formatter = allman::Doc::new();

        Self::configure_formatter(&mut formatter, &ast);

        dbg!(&formatter);

        let mut output = vec![];

        formatter.render(
            &mut output,
            &allman::Options {
                max_columns: self.max_width,
            },
        );

        let output = String::from_utf8_lossy(&output).into_owned();

        Ok(output)
    }

    fn configure_formatter<'a>(doc: &mut allman::Doc<'a>, nodes: &[parse::l2::AstNode<'a>]) {
        use allman::{Doc, If, Tag};

        for node in nodes {
            match node {
                &parse::l2::AstNode::Whitespace(content) => {
                    doc.tag(Tag::Space);
                }
                &parse::l2::AstNode::Raw(content) => {
                    doc.tag(Tag::Text(content.into()));
                }
                &parse::l2::AstNode::Punct(content) => {
                    doc.tag(Tag::Text(content.into()));
                }
                parse::l2::AstNode::Group(group) => {
                    doc.tag_with(Tag::Group(usize::MAX), |doc| {
                        doc.tag(Tag::Text(group.opening.into()));
                        doc.tag_if(Tag::Break(1), If::Broken);
                        doc.tag_with(Tag::Indent(1), |formatter| {
                            Self::configure_formatter(formatter, &group.content);
                        });
                        if let Some(closing) = group.closing {
                            doc.tag_if(Tag::Break(1), If::Broken);
                            doc.tag(Tag::Text(closing.into()));
                        }
                    });
                }
                parse::l2::AstNode::Quoted(quoted) => {
                    doc.tag(Tag::Text(quoted.opening.into()));

                    for content in &quoted.content {
                        doc.tag(Tag::Text(content.text().into()));
                    }

                    if let Some(closing) = quoted.closing {
                        doc.tag(Tag::Text(closing.into()));
                    }
                }
            }
        }
    }
}
