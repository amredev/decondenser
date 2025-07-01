mod token_tree;

pub(crate) use token_tree::*;

use super::l1;
use crate::Decondenser;

pub(crate) fn parse<'a>(config: &'a Decondenser, input: &'a str) -> Vec<TokenTree<'a>> {
    let tokens = l1::parse(config, input);

    TokenTreesFromL1 { input }.convert(&tokens, input.len())
}

#[derive(Copy, Clone)]
struct TokenTreesFromL1<'a> {
    input: &'a str,
}

impl<'a> TokenTreesFromL1<'a> {
    fn convert(
        self,
        nodes: &[l1::TokenTree<'a>],

        // Offset after the last node in the list. This is equal to the node,
        // that follows the surrounding group or the input length if the is no
        // node after the surrounding group.
        end: usize,
    ) -> Vec<TokenTree<'a>> {
        let input = self.input;
        nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                let next = nodes.get(i + 1);
                let end = next.map(l1::TokenTree::start).unwrap_or(end);

                match node {
                    l1::TokenTree::Space { start } => TokenTree::Space(&input[*start..end]),
                    l1::TokenTree::NewLine { start } => TokenTree::NewLine(end - *start),
                    l1::TokenTree::Raw { start } => TokenTree::Raw(&input[*start..end]),
                    l1::TokenTree::Punct(punct) => TokenTree::Punct(punct.config),
                    l1::TokenTree::Group(group) => {
                        let content_end = group.closing.unwrap_or(end);

                        TokenTree::Group(Group {
                            content: self.convert(&group.content, content_end),
                            closed: group.closing.is_some(),
                            config: group.config,
                        })
                    }
                    l1::TokenTree::Quoted(quoted) => {
                        let content_end = quoted.closing.unwrap_or(end);

                        let content = quoted
                            .content
                            .iter()
                            .enumerate()
                            .map(|(i, content)| {
                                let next = quoted.content.get(i + 1);
                                let end = next.map(l1::QuotedContent::start).unwrap_or(content_end);
                                let text = &input[content.start()..end];

                                match content {
                                    l1::QuotedContent::Raw { .. } => QuotedContent::Raw(text),
                                    l1::QuotedContent::Escape { .. } => QuotedContent::Escape(text),
                                }
                            })
                            .collect();

                        TokenTree::Quoted(Quoted {
                            content,
                            closed: quoted.closing.is_some(),
                            config: quoted.config,
                        })
                    }
                }
            })
            .collect()
    }
}
