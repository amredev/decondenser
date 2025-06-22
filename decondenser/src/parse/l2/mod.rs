mod ast;

pub(crate) use ast::*;

use super::l1;

pub(crate) fn parse<'a>(params: &l1::ParseParams<'a>) -> Vec<AstNode<'a>> {
    let ast = l1::parse(params);

    AstFromL1 {
        input: params.input,
    }
    .convert(&ast, params.input.len())
}

#[derive(Copy, Clone)]
struct AstFromL1<'a> {
    input: &'a str,
}

impl<'a> AstFromL1<'a> {
    fn convert(
        self,
        nodes: &[l1::AstNode],

        // Offset after the last node in the list. This is equal to the node,
        // that follows the surrounding group or the input length if the is no
        // node after the surrounding group.
        end: usize,
    ) -> Vec<AstNode<'a>> {
        let input = self.input;
        nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                let next = nodes.get(i + 1);
                let end = next.map(l1::AstNode::start).unwrap_or(end);

                match node {
                    l1::AstNode::Space { start } => AstNode::Space(&input[*start..end]),
                    l1::AstNode::Raw { start } => AstNode::Raw(&input[*start..end]),
                    l1::AstNode::Punct { start } => AstNode::Punct(&input[*start..end]),
                    l1::AstNode::Group(group) => {
                        let opening_end = group
                            .content
                            .first()
                            .map(l1::AstNode::start)
                            .unwrap_or_else(|| group.closing.unwrap_or(end));

                        let content_end = group.closing.unwrap_or(end);

                        AstNode::Group(Group {
                            opening: &input[group.opening..opening_end],
                            content: self.convert(&group.content, content_end),
                            closing: group.closing.map(|closing| &input[closing..end]),
                        })
                    }
                    l1::AstNode::Quoted(quoted) => {
                        let closing_start = quoted.closing.unwrap_or(end);
                        let opening_end = quoted
                            .content
                            .first()
                            .map(l1::QuotedContent::start)
                            .unwrap_or(closing_start);

                        let content = quoted
                            .content
                            .iter()
                            .enumerate()
                            .map(|(i, content)| {
                                let next = quoted.content.get(i + 1);
                                let end =
                                    next.map(l1::QuotedContent::start).unwrap_or(closing_start);
                                let text = &input[content.start()..end];

                                match content {
                                    l1::QuotedContent::Raw { .. } => QuotedContent::Raw(text),
                                    l1::QuotedContent::Escape { .. } => QuotedContent::Escape(text),
                                }
                            })
                            .collect();

                        AstNode::Quoted(Quoted {
                            opening: &input[quoted.opening..opening_end],
                            content,
                            closing: quoted.closing.map(|closing| &input[closing..end]),
                        })
                    }
                }
            })
            .collect()
    }
}
