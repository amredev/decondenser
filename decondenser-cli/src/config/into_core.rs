use super::{Common, Config, Group, Lang, Punct, Quote, Space};
use crate::{Error, Result};
use decondenser::Decondenser;
use std::collections::{BTreeMap, BTreeSet};

const BUILTIN_LANGS: &[(&str, fn() -> Decondenser)] = &[("generic", Decondenser::generic)];

impl Config {
    pub(crate) fn into_decondenser(self, lang: &str) -> Result<Decondenser> {
        let Self {
            common: formatting,
            mut langs,
            debug_layout,
            debug_indent,
        } = self;

        let decondenser = BUILTIN_LANGS
            .iter()
            .find(|(name, _)| *name == lang)
            .map(|(_, factory)| factory());

        let decondenser = if let Some(lang) = langs.remove(lang) {
            let decondenser = decondenser.unwrap_or_else(Decondenser::new);
            let decondenser = formatting.apply(decondenser);
            lang.apply(decondenser)
        } else {
            let decondenser = decondenser.ok_or_else(|| Self::unknown_lang_error(&langs, lang))?;
            formatting.apply(decondenser)
        };

        Ok(decondenser
            .debug_layout(debug_layout)
            .debug_indent(debug_indent))
    }

    fn unknown_lang_error(custom_langs: &BTreeMap<String, Lang>, lang: &str) -> Error {
        let available_langs = custom_langs
            .keys()
            .map(String::as_str)
            .chain(BUILTIN_LANGS.iter().map(|(name, _)| *name))
            .collect::<BTreeSet<_>>();

        anyhow::anyhow!(
            "Unrecognized language: '{lang}'. \
            The language must be one of the following: \
            {available_langs:?}",
        )
        .into()
    }
}

impl Common {
    fn apply(self, mut decondenser: Decondenser) -> Decondenser {
        let Self {
            indent,
            max_line_size,
            no_break_size,
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

        decondenser
    }
}

impl Lang {
    fn apply(self, mut decondenser: Decondenser) -> Decondenser {
        let Self {
            common: formatting,
            groups,
            quotes,
            puncts,
        } = self;

        decondenser = formatting.apply(decondenser);

        if let Some(groups) = groups {
            decondenser = decondenser.groups(groups.into_iter().map(Group::into_core));
        }

        if let Some(quotes) = quotes {
            decondenser = decondenser.quotes(quotes.into_iter().map(Quote::into_core));
        }

        if let Some(puncts) = puncts {
            decondenser = decondenser.puncts(puncts.into_iter().map(Punct::into_core));
        }

        decondenser
    }
}

impl Group {
    fn into_core(self) -> decondenser::Group {
        let Self {
            opening,
            closing,
            break_style,
        } = self;

        let mut group = decondenser::Group::new(opening.into_core(), closing.into_core());

        if let Some(break_style) = break_style {
            group = group.break_style(break_style);
        }

        group
    }
}

impl Punct {
    fn into_core(self) -> decondenser::Punct {
        let Self {
            symbol,
            leading_space,
            trailing_space,
        } = self;

        let mut punct = decondenser::Punct::new(symbol);

        if let Some(leading_space) = leading_space {
            punct = punct.leading_space(leading_space.into_core());
        }

        if let Some(trailing_space) = trailing_space {
            punct = punct.trailing_space(trailing_space.into_core());
        }

        punct
    }
}

impl Space {
    fn into_core(self) -> decondenser::Space {
        let mut space = decondenser::Space::new();

        if let Some(size) = self.size {
            space = space.size(size);
        }

        if let Some(breakable) = self.breakable {
            space = space.breakable(breakable);
        }

        space
    }
}

impl Quote {
    fn into_core(self) -> decondenser::Quote {
        let Self { opening, closing } = self;
        decondenser::Quote::new(opening, closing)
    }
}
