use super::{Config, Escape, Formatting, Group, Lang, Punct, Quote, Space};
use crate::yaml::{self, Deserialize, Node, NodeExt, Object, Result};
use decondenser::BreakStyle;

impl Deserialize for Config {
    fn deserialize(value: Node) -> Result<Self> {
        value.object(|obj| Self {
            formatting: Formatting::flattened(obj),
            langs: obj.optional("lang").unwrap_or_default(),
            debug_layout: obj.optional("debug_layout").unwrap_or_default(),
            debug_indent: obj.optional("debug_indent").unwrap_or_default(),
        })
    }
}

impl Deserialize for Lang {
    fn deserialize(value: Node) -> Result<Self> {
        value.object(|table| Self {
            formatting: Formatting::flattened(table),
            groups: table.optional("groups"),
            quotes: table.optional("quotes"),
            puncts: table.optional("puncts"),
        })
    }
}

impl Formatting {
    fn flattened(table: &mut Object) -> Self {
        Self {
            indent: table.optional("indent"),
            max_line_size: table.optional("max_line_size"),
            no_break_size: table.optional("no_break_size"),
        }
    }
}

impl Deserialize for Group {
    fn deserialize(value: Node) -> Result<Self> {
        let span = *value.span();

        value
            .any_of()
            .array(|array| {
                let [opening, closing] = array.try_into().map_err(|array: Vec<_>| {
                    yaml::Errors::unexpected_type(
                        span,
                        "an object or an array of two items ([opening, closing] delimiters)",
                        format_args!("array of size {}", array.len()),
                    )
                })?;

                Ok(Self {
                    opening: Punct::deserialize(opening)?,
                    closing: Punct::deserialize(closing)?,
                    break_style: None,
                })
            })
            .object(|obj| Self {
                opening: obj.required("opening"),
                closing: obj.required("closing"),
                break_style: obj
                    .optional::<YamlBreakStyle>("break_style")
                    .map(|style| style.0),
            })
            .finish()
    }
}

impl Deserialize for Punct {
    fn deserialize(value: Node) -> Result<Self> {
        value
            .any_of()
            .string(|symbol| {
                Ok(Self {
                    symbol,
                    leading_space: None,
                    trailing_space: None,
                })
            })
            .object(|obj| Self {
                symbol: obj.required("symbol"),
                leading_space: obj.optional("leading_space"),
                trailing_space: obj.optional("trailing_space"),
            })
            .finish()
    }
}

impl Deserialize for Space {
    fn deserialize(value: Node) -> Result<Self> {
        value
            .any_of()
            .usize(|size| {
                Ok(Self {
                    size: Some(size),
                    breakable: None,
                })
            })
            .object(|obj| Self {
                size: obj.optional("size"),
                breakable: obj.optional("breakable"),
            })
            .finish()
    }
}

struct YamlBreakStyle(BreakStyle);

impl Deserialize for YamlBreakStyle {
    fn deserialize(value: Node) -> Result<Self> {
        value
            .enumeration(&[
                ("consistent", BreakStyle::consistent),
                ("compact", BreakStyle::compact),
            ])
            .map(Self)
    }
}

impl Deserialize for Quote {
    fn deserialize(value: Node) -> Result<Self> {
        value.object(|obj| Self {
            opening: obj.required("opening"),
            closing: obj.required("closing"),
            escapes: obj.optional("escapes"),
        })
    }
}

impl Deserialize for Escape {
    fn deserialize(value: Node) -> Result<Self> {
        value.object(|obj| Self {
            escaped: obj.required("escaped"),
            unescaped: obj.required("unescaped"),
        })
    }
}
