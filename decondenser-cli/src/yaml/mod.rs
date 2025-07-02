mod error;

pub(crate) use error::Error;
pub(crate) use marked_yaml::Node;

/// This crate's equivalent to [`serde::Deserialize`](https://docs.rs/serde/latest/serde/de/trait.Deserialize.html)
pub trait Deserialize: Sized {
    /// Given a mutable [`Value`], allows you to deserialize the type from it,
    /// or accumulate 1 or more errors
    fn deserialize(value: &mut Node) -> Result<Self, Error>;
}

pub(crate) fn parse<T: Deserialize>(file_id: usize, input: &str) -> Result<T, Error> {
    let options = marked_yaml::LoaderOptions::default().error_on_duplicate_keys(true);

    let yaml = marked_yaml::parse_yaml_with_options(file_id, input, options)?;
}

pub(crate) trait NodeExt {}

impl NodeExt for marked_yaml::types::Node {}
