use super::utils::{self, DeserResult, integer, table};
use super::{Config, Escape, Formatting, Group, Lang, Punct, Quote, Space};
use decondenser::BreakStyle;
use toml_span::de_helpers::{TableHelper, expected};
use toml_span::value::ValueInner;
use toml_span::{Deserialize, Value};

impl<'de> Deserialize<'de> for Config {
    fn deserialize(value: &mut Value<'de>) -> DeserResult<Self> {
        table(value, |table| Self {
            formatting: Formatting::flattened(table),
            langs: table
                .optional::<utils::TomlMap<_>>("lang")
                .map(|map| map.0)
                .unwrap_or_default(),
            debug_layout: table.optional("debug_layout").unwrap_or_default(),
            debug_indent: table.optional("debug_indent").unwrap_or_default(),
        })
    }
}

impl<'de> Deserialize<'de> for Lang {
    fn deserialize(value: &mut Value<'de>) -> DeserResult<Self> {
        table(value, |table| Self {
            formatting: Formatting::flattened(table),
            groups: table.optional("groups"),
            quotes: table.optional("quotes"),
            puncts: table.optional("puncts"),
        })
    }
}

impl Formatting {
    fn flattened(table: &mut TableHelper<'_>) -> Self {
        Self {
            indent: table.optional("indent"),
            max_line_size: table.optional("max_line_size"),
            no_break_size: table.optional("no_break_size"),
            preserve_newlines: table.optional("preserve_newlines"),
        }
    }
}

impl<'de> Deserialize<'de> for Group {
    fn deserialize(value: &mut Value<'de>) -> DeserResult<Self> {
        let span = value.span;
        let err = |found| {
            expected(
                "a table or an array of two items with [opening, closing] delimiters",
                found,
                span,
            )
        };

        match value.take() {
            ValueInner::Array(array) => {
                let [opening, closing] =
                    array.try_into().map_err(ValueInner::Array).map_err(err)?;

                Ok(Self {
                    opening: Punct::deserialize(&mut { opening })?,
                    closing: Punct::deserialize(&mut { closing })?,
                    break_style: None,
                })
            }
            ValueInner::Table(map) => table((map, value.span), |table| Self {
                opening: table.required("opening").unwrap_or_default(),
                closing: table.required("closing").unwrap_or_default(),
                break_style: table
                    .optional::<TomlBreakStyle>("break_style")
                    .map(|style| style.0),
            }),
            other => Err(err(other).into()),
        }
    }
}

impl<'de> Deserialize<'de> for Punct {
    fn deserialize(value: &mut Value<'de>) -> DeserResult<Self> {
        match value.take() {
            ValueInner::String(symbol) => Ok(Self {
                symbol: symbol.into_owned(),
                leading_space: None,
                trailing_space: None,
            }),
            ValueInner::Table(map) => table((map, value.span), |table| Self {
                symbol: table.required("symbol").unwrap_or_default(),
                leading_space: table.optional("leading_space"),
                trailing_space: table.optional("trailing_space"),
            }),
            other => Err(expected("a string or table", other, value.span).into()),
        }
    }
}

impl<'de> Deserialize<'de> for Space {
    fn deserialize(value: &mut Value<'de>) -> DeserResult<Self> {
        match value.take() {
            ValueInner::Integer(int) => Ok(Self {
                size: Some(integer(int, value.span)?),
                breakable: None,
            }),
            ValueInner::Table(map) => table((map, value.span), |table| Self {
                size: table.optional("size"),
                breakable: table.optional("breakable"),
            }),
            other => Err(expected(
                "an integer meaning the number of spaces (size) or table",
                other,
                value.span,
            )
            .into()),
        }
    }
}

utils::foreign_enum_deser! {
    struct TomlBreakStyle: BreakStyle {
        Consistent => "consistent",
        Compact => "compact",
    }
}

impl<'de> Deserialize<'de> for Quote {
    fn deserialize(value: &mut Value<'de>) -> DeserResult<Self> {
        table(value, |table| Self {
            opening: table.required("opening").unwrap_or_default(),
            closing: table.required("closing").unwrap_or_default(),
            escapes: table.optional("escapes"),
        })
    }
}

impl<'de> Deserialize<'de> for Escape {
    fn deserialize(value: &mut Value<'de>) -> DeserResult<Self> {
        table(value, |table| Self {
            escaped: table.required("escaped").unwrap_or_default(),
            unescaped: table.required("unescaped").unwrap_or_default(),
        })
    }
}
