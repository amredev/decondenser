use crate::parse;
use crate::printer::Printer;
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

        let mut printer = Printer::new(self);

        printer.begin_consistent(0);
        self.print(&mut printer, &ast);
        printer.scan_end();

        Ok(printer.eof())
    }

    fn print<'a>(&self, printer: &mut Printer<'a>, nodes: &[parse::l2::AstNode<'a>]) {
        for node in nodes {
            match node {
                &parse::l2::AstNode::Space(content) => {
                    printer.nbsp();
                }
                &parse::l2::AstNode::Raw(content) => {
                    printer.scan_string(content.into());
                }
                &parse::l2::AstNode::Punct(content) => {
                    printer.scan_string(content.into());
                    if content == "," {
                        printer.space();
                    }
                }
                parse::l2::AstNode::Group(group) => {
                    printer.scan_string(group.opening.into());
                    printer.begin_consistent(self.indent.len() as isize);
                    printer.space_if_nonempty();
                    self.print(printer, &group.content);
                    printer.space();
                    printer.offset(-(self.indent.len() as isize));
                    printer.scan_end();
                    if let Some(closing) = group.closing {
                        printer.scan_string(closing.into());
                    }
                }
                parse::l2::AstNode::Quoted(quoted) => {
                    printer.scan_string(quoted.opening.into());

                    for content in &quoted.content {
                        printer.scan_string(content.text().into());
                    }

                    if let Some(closing) = quoted.closing {
                        printer.scan_string(closing.into());
                    }
                }
            }
        }
    }
}
