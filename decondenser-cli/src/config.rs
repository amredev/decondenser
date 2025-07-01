use crate::Result;
use anyhow::{Context, bail};
use decondenser::Decondenser;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

const BUILTIN_LANGS: &[(&str, fn() -> Decondenser)] = &[("generic", Decondenser::generic)];

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    #[serde(default)]
    lang: BTreeMap<String, Lang>,

    // These fields are copied in several places because we can't use
    // #[serde(flatten)] together with #[serde(deny_unknown_fields)]
    indent: Option<String>,
    max_line_size: Option<usize>,
    no_break_size: Option<usize>,
    preserve_newlines: Option<bool>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Lang {
    #[serde(default)]
    groups: Option<Vec<Group>>,

    #[serde(default)]
    quotes: Option<Vec<Quote>>,

    #[serde(default)]
    puncts: Option<Vec<Punct>>,

    // These fields are copied in several places because we can't use
    // #[serde(flatten)] together with #[serde(deny_unknown_fields)]
    indent: Option<String>,
    max_line_size: Option<usize>,
    no_break_size: Option<usize>,
    preserve_newlines: Option<bool>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Group {
    Delims(Punct, Punct),
    Extended {
        opening: Punct,
        closing: Punct,
        break_style: Option<BreakStyle>,
    },
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Punct {
    Content(String),
    Extended {
        content: String,
        leading_space: Option<Space>,
        trailing_space: Option<Space>,
    },
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Space {
    Size(usize),
    Extended {
        size: Option<usize>,
        breakable: Option<bool>,
    },
}

#[derive(Deserialize)]
struct Quote {
    opening: String,
    closing: String,

    #[serde(default)]
    escapes: Vec<Escape>,
}

#[derive(Deserialize)]
struct Escape {
    escaped: String,
    unescaped: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum BreakStyle {
    Consistent,
    Compact,
}

impl Config {
    pub(crate) fn discover() -> Result<Option<Self>> {
        std::env::current_dir()
            .context("Failed to get the current directory of the process")?
            .ancestors()
            .find_map(|path| Self::from_file(&path.join("decondenser.toml")).transpose())
            .transpose()
    }

    pub(crate) fn from_file(path: &Path) -> Result<Option<Self>> {
        let content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => {
                return Err(anyhow::Error::new(err)
                    .context(format!("Failed to read the config file {}", path.display())));
            }
        };

        let config = toml_edit::de::from_str::<Self>(&content)
            .with_context(|| format!("Failed to parse the TOML config file {}", path.display()))?;

        Ok(Some(config))
    }

    pub(crate) fn into_decondenser(self, lang: &str) -> Result<Decondenser> {
        let Self {
            indent,
            max_line_size,
            no_break_size,
            preserve_newlines,
            lang: mut langs,
        } = self;

        let fmt = Fmt {
            indent,
            max_line_size,
            no_break_size,
            preserve_newlines,
        };

        #[expect(clippy::let_and_return, reason = "pipeline syntax consistency")]
        let decondenser = if let Some(lang) = langs.remove(lang) {
            let decondenser = Decondenser::empty();
            let decondenser = fmt.merge_into(decondenser);
            let decondenser = lang.merge_into(decondenser);
            decondenser
        } else {
            let decondenser = Self::fallback_to_builtin_lang(&langs, lang)?;
            let decondenser = fmt.merge_into(decondenser);
            decondenser
        };

        Ok(decondenser)
    }

    fn fallback_to_builtin_lang(
        custom_langs: &BTreeMap<String, Lang>,
        lang: &str,
    ) -> Result<Decondenser> {
        let config = BUILTIN_LANGS
            .iter()
            .find(|(name, _)| *name == lang)
            .map(|(_, factory)| factory());

        if let Some(config) = config {
            return Ok(config);
        }

        let available_langs = custom_langs
            .keys()
            .map(String::as_str)
            .chain(BUILTIN_LANGS.iter().map(|(name, _)| *name))
            .collect::<BTreeSet<_>>();

        bail!(
            "Unrecognized language: '{lang}'. \
            The language must be one of the following (builtin or custom): \
            {available_langs:?}",
        );
    }
}

struct Fmt {
    indent: Option<String>,
    max_line_size: Option<usize>,
    no_break_size: Option<usize>,
    preserve_newlines: Option<bool>,
}

impl Fmt {
    fn merge_into(self, mut decondenser: Decondenser) -> Decondenser {
        let Self {
            indent,
            max_line_size,
            no_break_size,
            preserve_newlines,
        } = self;

        if let Some(indent) = indent {
            decondenser = decondenser.indent(indent);
        }

        if let Some(max_line_size) = max_line_size {
            decondenser = decondenser.max_line_size(max_line_size);
        }

        if let Some(no_break_size) = no_break_size {
            decondenser = decondenser.no_break_size(no_break_size);
        }

        if let Some(preserve_newlines) = preserve_newlines {
            decondenser = decondenser.preserve_newlines(preserve_newlines);
        }

        decondenser
    }
}

impl Lang {
    fn merge_into(self, mut decondenser: Decondenser) -> Decondenser {
        let Self {
            indent,
            max_line_size,
            no_break_size,
            preserve_newlines,
            groups,
            quotes,
            puncts,
        } = self;

        let fmt = Fmt {
            indent,
            max_line_size,
            no_break_size,
            preserve_newlines,
        };

        if let Some(groups) = groups {
            decondenser = decondenser.groups(groups.into_iter().map(Group::into_core));
        }

        if let Some(quotes) = quotes {
            decondenser = decondenser.quotes(quotes.into_iter().map(Quote::into_core));
        }

        if let Some(puncts) = puncts {
            decondenser = decondenser.puncts(puncts.into_iter().map(Punct::into_core));
        }

        fmt.merge_into(decondenser)
    }
}

impl Group {
    fn into_core(self) -> decondenser::Group {
        match self {
            Self::Delims(opening, closing) => {
                decondenser::Group::new(opening.into_core(), closing.into_core())
            }
            Self::Extended {
                opening,
                closing,
                break_style,
            } => {
                let mut group = decondenser::Group::new(opening.into_core(), closing.into_core());

                if let Some(value) = break_style {
                    group = group.break_style(value.into_core());
                }

                group
            }
        }
    }
}

impl Punct {
    fn into_core(self) -> decondenser::Punct {
        match self {
            Self::Content(content) => decondenser::Punct::new(content),
            Self::Extended {
                content,
                leading_space,
                trailing_space,
            } => {
                let mut punct = decondenser::Punct::new(content);

                if let Some(value) = leading_space {
                    punct = punct.leading_space(value.into_core());
                }

                if let Some(value) = trailing_space {
                    punct = punct.trailing_space(value.into_core());
                }

                punct
            }
        }
    }
}

impl Space {
    fn into_core(self) -> decondenser::Space {
        match self {
            Self::Size(size) => decondenser::Space::fixed(size),
            Self::Extended { size, breakable } => {
                let mut space = size
                    .map(decondenser::Space::fixed)
                    .unwrap_or_else(decondenser::Space::preserving);

                if let Some(value) = breakable {
                    space = space.breakable(value);
                }

                space
            }
        }
    }
}

impl BreakStyle {
    fn into_core(self) -> decondenser::BreakStyle {
        match self {
            Self::Consistent => decondenser::BreakStyle::Consistent,
            Self::Compact => decondenser::BreakStyle::Compact,
        }
    }
}

impl Quote {
    fn into_core(self) -> decondenser::Quote {
        let Self {
            opening,
            closing,
            escapes,
        } = self;

        decondenser::Quote::new(opening, closing)
            .escapes(escapes.into_iter().map(Escape::into_core))
    }
}

impl Escape {
    fn into_core(self) -> decondenser::Escape {
        decondenser::Escape::new(self.escaped, self.unescaped)
    }
}
