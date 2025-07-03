use super::error::primary_label;
use super::{NodeExt, Result};
use crate::Diagnostic;
use marked_yaml::Node;

pub(crate) fn deserialize_enum<T>(
    names: &'static [&'static str],
    values: &[T],
    value: Node,
) -> Result<T>
where
    T: Clone + Copy,
{
    let span = *value.span();
    let string = value.string()?;
    let value = names.iter().zip(values).find(|&(&name, _)| name == string);

    if let Some((_, &value)) = value {
        return Ok(value);
    }

    let expected = names
        .iter()
        .map(|&str| format!("{str:?}"))
        .collect::<Vec<_>>()
        .join(", ");

    let diag = Diagnostic::error()
        .with_message(format!("expected one of the values: {expected}"))
        .with_labels(vec![primary_label(span).with_message("unexpected value")]);

    Err(diag.into())
}

macro_rules! impl_deserialize_for_foreign_enum {
    (
        $wrapper:ident($enum_name:ident {
            $($variant_ident:ident => $variant_str:literal),* $(,)?
        })
    ) => {
        impl $crate::yaml::Deserialize for $wrapper {
            fn deserialize(value: $crate::yaml::Node) -> $crate::yaml::Result<Self> {
                fn _exhaustiveness_check(enum_value: $enum_name) {
                    match enum_value {
                        $($enum_name::$variant_ident => {},)*
                    }
                }

                $crate::yaml::enums::deserialize_enum(
                    &[$($variant_str,)*],
                    &[$($enum_name::$variant_ident,)*],
                    value,
                )
                .map(Self)
            }
        }
    };
}

pub(crate) use impl_deserialize_for_foreign_enum;
