mod deser;
mod into_core;

use crate::{Files, Result};
use anyhow::Context;
use decondenser::BreakStyle;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Default)]
pub(crate) struct Config {
    formatting: Formatting,
    langs: BTreeMap<String, Lang>,

    // Only used for debugging. No stability guarantees are provided for these
    //
    // Enable outputting of the special control characters to review the layout
    // and/or indentation.
    debug_layout: bool,
    debug_indent: bool,
}

#[derive(Default)]
struct Formatting {
    indent: Option<String>,
    max_line_size: Option<usize>,
    no_break_size: Option<usize>,
    preserve_newlines: Option<bool>,
}

struct Lang {
    formatting: Formatting,
    groups: Option<Vec<Group>>,
    quotes: Option<Vec<Quote>>,
    puncts: Option<Vec<Punct>>,
}

struct Group {
    opening: Punct,
    closing: Punct,
    break_style: Option<BreakStyle>,
}

#[derive(Default)]
struct Punct {
    symbol: String,
    leading_space: Option<Space>,
    trailing_space: Option<Space>,
}

struct Space {
    size: Option<usize>,
    breakable: Option<bool>,
}

struct Quote {
    opening: String,
    closing: String,
    escapes: Option<Vec<Escape>>,
}

struct Escape {
    escaped: String,
    unescaped: String,
}

impl Config {
    pub(crate) fn discover(files: &mut Files) -> Result<Option<Self>> {
        std::env::current_dir()
            .context("Failed to get the current directory of the process")?
            .ancestors()
            .find_map(|path| Self::from_file(files, &path.join("decondenser.yml")).transpose())
            .transpose()
    }

    pub(crate) fn from_file(files: &mut Files, path: &Path) -> Result<Option<Self>> {
        let content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => {
                return Err(anyhow::Error::new(err)
                    .context(format!("Failed to read the config file {}", path.display()))
                    .into());
            }
        };

        let file_id = files.add(path.to_string_lossy().into_owned(), content);
        let file = files.get(file_id).unwrap();

        let config = crate::yaml::from_str(file_id, file.source())?;

        Ok(Some(config))
    }
}
