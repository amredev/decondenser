mod any_of;
mod error;

pub(crate) use any_of::{NodeExt, Object};
pub(crate) use error::{Errors, Result};
pub(crate) use marked_yaml::Node;

use std::collections::BTreeMap;

/// This crate's equivalent to [`serde::Deserialize`](https://docs.rs/serde/latest/serde/de/trait.Deserialize.html)
pub(crate) trait Deserialize: Sized {
    /// Given a mutable [`Value`], allows you to deserialize the type from it,
    /// or accumulate 1 or more errors
    fn deserialize(value: Node) -> Result<Self>;
}

pub(crate) fn from_str<T: Deserialize>(file_id: usize, input: &str) -> Result<T> {
    let options = marked_yaml::LoaderOptions::default().error_on_duplicate_keys(true);

    let node = marked_yaml::parse_yaml_with_options(file_id, input, options)?;

    T::deserialize(node)
}

impl Deserialize for String {
    fn deserialize(value: Node) -> Result<Self> {
        value.string()
    }
}

impl Deserialize for bool {
    fn deserialize(value: Node) -> Result<Self> {
        value.scalar()
    }
}

impl Deserialize for usize {
    fn deserialize(value: Node) -> Result<Self> {
        value.scalar()
    }
}

impl Deserialize for char {
    fn deserialize(value: Node) -> Result<Self> {
        value.scalar()
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn deserialize(value: Node) -> Result<Self> {
        value
            .any_of()
            .array(|array| {
                let mut errors = Errors::default();
                let mut output = Self::with_capacity(array.len());

                for value in array {
                    match T::deserialize(value) {
                        Ok(value) => output.push(value),
                        Err(err) => errors.extend([err]),
                    }
                }

                errors.into_result().map(|()| output)
            })
            .finish()
    }
}

impl<T: Deserialize> Deserialize for BTreeMap<String, T> {
    fn deserialize(value: Node) -> Result<Self> {
        value
            .any_of()
            .dict(|dict| {
                let mut errors = Errors::default();
                let mut output = Self::new();

                for (key, value) in dict {
                    match T::deserialize(value) {
                        Ok(value) => drop(output.insert(key.as_str().to_owned(), value)),
                        Err(err) => errors.extend([err]),
                    }
                }

                errors.into_result().map(|()| output)
            })
            .finish()
    }
}
