pub(crate) use super::l1;

#[derive(Debug)]
pub(crate) enum AstNode<'a> {
    Whitespace(&'a str),
    Raw(&'a str),
    Punct(&'a str),
    Group(Group<'a>),
    Quoted(Quoted<'a>),
}

#[derive(Debug)]
pub(crate) struct Quoted<'a> {
    pub(crate) opening: &'a str,
    pub(crate) content: Vec<QuotedContent<'a>>,
    pub(crate) closing: Option<&'a str>,
}

#[derive(Debug)]
pub(crate) enum QuotedContent<'a> {
    Raw(&'a str),
    Escape(&'a str),
}

impl<'a> QuotedContent<'a> {
    pub(crate) fn text(&self) -> &'a str {
        match self {
            QuotedContent::Raw(text) => text,
            QuotedContent::Escape(text) => text,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Group<'a> {
    pub(crate) opening: &'a str,
    pub(crate) content: Vec<AstNode<'a>>,
    pub(crate) closing: Option<&'a str>,
}

pub(crate) fn parse<'a>(params: &l1::ParseParams<'a>) -> Vec<AstNode<'a>> {
    let ast = l1::parse(params);
    ast_from_l1(params.input, &ast)
}

fn ast_from_l1<'a>(input: &'a str, ast: &[l1::AstNode]) -> Vec<AstNode<'a>> {
    ast.iter()
        .enumerate()
        .map(|(i, node)| {
            let next = ast.get(i + 1);
            let next_offset = next.map(l1::AstNode::start).unwrap_or_else(|| input.len());

            match node {
                l1::AstNode::Whitespace { start } => {
                    AstNode::Whitespace(&input[*start..next_offset])
                }
                l1::AstNode::Raw { start } => AstNode::Raw(&input[*start..next_offset]),
                l1::AstNode::Punct { start } => AstNode::Punct(&input[*start..next_offset]),
                l1::AstNode::Group(group) => {
                    let opening_end = group
                        .content
                        .first()
                        .map(l1::AstNode::start)
                        .unwrap_or_else(|| group.closing.unwrap_or_else(|| input.len()));

                    AstNode::Group(Group {
                        opening: &input[group.opening..opening_end],
                        content: ast_from_l1(input, &group.content),
                        closing: group.closing.map(|closing| &input[closing..next_offset]),
                    })
                }
                l1::AstNode::Quoted(quoted) => {
                    let closing_start = quoted.closing.unwrap_or_else(|| input.len());
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
                            let end = next.map(l1::QuotedContent::start).unwrap_or(closing_start);
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
                        closing: quoted.closing.map(|closing| &input[closing..next_offset]),
                    })
                }
            }
        })
        .collect()
}
