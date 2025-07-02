use std::collections::BTreeMap;
use toml_span::de_helpers::{TableHelper, expected};
use toml_span::value::ValueInner;
use toml_span::{Deserialize, Value};

pub(super) type DeserResult<T> = Result<T, toml_span::DeserError>;

pub(super) struct TomlMap<V>(pub(super) BTreeMap<String, V>);

impl<'de, V: Deserialize<'de>> Deserialize<'de> for TomlMap<V> {
    fn deserialize(value: &mut Value<'de>) -> DeserResult<Self> {
        let table = match value.take() {
            ValueInner::Table(table) => table,
            other => return Err(expected("a table", other, value.span).into()),
        };

        table
            .into_iter()
            .map(|(key, value)| Ok((key.name.into_owned(), V::deserialize(&mut { value })?)))
            .collect::<DeserResult<_>>()
            .map(Self)
    }
}

pub(super) trait Table<'de> {
    fn into_helper(self) -> DeserResult<TableHelper<'de>>;
}

impl<'de> Table<'de> for &mut Value<'de> {
    fn into_helper(self) -> DeserResult<TableHelper<'de>> {
        TableHelper::new(self)
    }
}

impl<'de> Table<'de> for (toml_span::value::Table<'de>, toml_span::Span) {
    fn into_helper(self) -> DeserResult<TableHelper<'de>> {
        Ok(TableHelper::from(self))
    }
}

/// A helper function that automatically calls [`TableHelper::finalize()`] to
/// make sure an error is returned if unknown keys are present in the table.
///
/// Note that the closure `f` should be infallible. Even if it deserializes a
/// struct with some required fields, you should fallback to default values if
/// those keys are missing. The [`TableHelper`] accumulates all errors along
/// the way, so they won't be lost, and they will be propagated via the internal
/// [`TableHelper::finalize()`] call.
pub(super) fn table<'de, T>(
    table: impl Table<'de>,
    f: impl FnOnce(&mut TableHelper<'de>) -> T,
) -> DeserResult<T> {
    let mut table = table.into_helper()?;
    let result = f(&mut table);
    table.finalize(None)?;
    Ok(result)
}

pub(super) fn integer<T: TryFrom<i64>>(int: i64, span: toml_span::Span) -> DeserResult<T> {
    int.try_into().map_err(|_| {
        toml_span::Error::from((
            toml_span::ErrorKind::OutOfRange(std::any::type_name::<T>()),
            span,
        ))
        .into()
    })
}

macro_rules! foreign_enum_deser {
    (
        struct $wrapper:ident: $enum_name:ident {
            $($variant_ident:ident => $variant_str:literal),* $(,)?
        }
    ) => {
        struct $wrapper($enum_name);

        impl<'de> Deserialize<'de> for $wrapper {
            fn deserialize(value: &mut Value<'de>) -> DeserResult<Self> {
                fn _exhaustiveness_check(enum_value: $enum_name) {
                    match enum_value {
                        $($enum_name::$variant_ident => {},)*
                    }
                }

                $crate::config::utils::deser_enum(
                    &[$($variant_str,)*],
                    &[$($enum_name::$variant_ident,)*],
                    value,
                )
                .map(Self)
            }
        }
    };
}

pub(crate) fn deser_enum<T>(
    names: &'static [&'static str],
    values: &[T],
    value: &mut Value<'_>,
) -> DeserResult<T>
where
    T: Clone + Copy,
{
    let string = value.take_string(None)?;
    (*names)
        .iter()
        .zip(values)
        .find(|&(&name, _)| name == string)
        .map(|(_, &value)| value)
        .ok_or_else(|| {
            toml_span::Error::from((
                toml_span::ErrorKind::UnexpectedValue {
                    expected: names,
                    value: None,
                },
                value.span,
            ))
            .into()
        })
}

pub(super) use foreign_enum_deser;
