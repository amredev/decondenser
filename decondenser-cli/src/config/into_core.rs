use super::{Config, Escape, Formatting, Group, Lang, Punct, Quote, Space};
use crate::Result;
use decondenser::Decondenser;
use std::collections::{BTreeMap, BTreeSet};

const BUILTIN_LANGS: &[(&str, fn() -> Decondenser)] = &[("generic", Decondenser::generic)];

impl Config {
    pub(crate) fn into_decondenser(self, lang: &str) -> Result<Decondenser> {
        let Self {
            formatting,
            mut langs,
            debug_layout,
            debug_indent,
        } = self;

        let decondenser = if let Some(lang) = langs.remove(lang) {
            let decondenser = Decondenser::empty();
            let decondenser = formatting.apply(decondenser);
            lang.apply(decondenser)
        } else {
            let decondenser = Self::builtin_lang(&langs, lang)?;
            formatting.apply(decondenser)
        };

        Ok(decondenser
            .debug_layout(debug_layout)
            .debug_indent(debug_indent))
    }

    fn builtin_lang(custom_langs: &BTreeMap<String, Lang>, lang: &str) -> Result<Decondenser> {
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

        return Err(anyhow::anyhow!(
            "Unrecognized language: '{lang}'. \
            The language must be one of the following (builtin or custom): \
            {available_langs:?}",
        )
        .into());
    }
}

impl Formatting {
    fn apply(self, mut decondenser: Decondenser) -> Decondenser {
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
    fn apply(self, mut decondenser: Decondenser) -> Decondenser {
        let Self {
            formatting,
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
        let Self { size, breakable } = self;

        let mut space = size
            .map(decondenser::Space::fixed)
            .unwrap_or_else(decondenser::Space::preserving);

        if let Some(breakable) = breakable {
            space = space.breakable(breakable);
        }

        space
    }
}

impl Quote {
    fn into_core(self) -> decondenser::Quote {
        let Self {
            opening,
            closing,
            escapes,
        } = self;

        let mut quote = decondenser::Quote::new(opening, closing);

        if let Some(escapes) = escapes {
            quote = quote.escapes(escapes.into_iter().map(Escape::into_core));
        }

        quote
    }
}

impl Escape {
    fn into_core(self) -> decondenser::Escape {
        let Self { escaped, unescaped } = self;
        decondenser::Escape::new(escaped, unescaped)
    }
}
