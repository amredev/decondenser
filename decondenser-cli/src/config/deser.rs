use super::{Config, Group, Indent, Preset, Punct, Quote, Space};
use crate::yaml::{self, Deserialize, Node, NodeExt, Result};
use decondenser::BreakStyle;

impl Deserialize for Config {
    fn deserialize(value: Node) -> Result<Self> {
        value.object(|obj| Self {
            extends: obj.optional("extends"),
            indent: obj.optional("indent"),
            max_line_size: obj.optional("max_line_size"),
            no_break_size: obj.optional("no_break_size"),
            groups: obj.optional("groups"),
            quotes: obj.optional("quotes"),
            puncts: obj.optional("puncts"),
            debug_layout: obj.optional("debug_layout"),
            debug_indent: obj.optional("debug_indent"),
        })
    }
}

impl Deserialize for Preset {
    fn deserialize(value: Node) -> Result<Self> {
        value.enumeration(&[("empty", || Self::Empty), ("generic", || Self::Generic)])
    }
}

impl Deserialize for Indent {
    fn deserialize(value: Node) -> Result<Self> {
        value
            .any_of()
            .usize(|n_spaces| Ok(Self::NSpaces(n_spaces)))
            .string(|string| Ok(Self::String(string)))
            .finish()
    }
}

impl Deserialize for Group {
    fn deserialize(value: Node) -> Result<Self> {
        let span = *value.span();

        value
            .any_of()
            .array(|array| {
                let [opening, closing] = array.try_into().map_err(|array: Vec<_>| {
                    yaml::Errors::unexpected_type_detailed(
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
        })
    }
}
