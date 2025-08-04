use super::{Config, Group, Indent, Preset, Punct, Quote, Space};
use decondenser::Decondenser;

impl Config {
    pub(crate) fn into_decondenser(self) -> Decondenser {
        let Self {
            extends,
            indent,
            max_line_size,
            no_break_size,
            groups,
            quotes,
            puncts,
            debug_layout,
            debug_indent,
        } = self;

        let mut decondenser = match extends.unwrap_or(Preset::Generic) {
            Preset::Empty => Decondenser::empty(),
            Preset::Generic => Decondenser::generic(),
        };

        if let Some(indent) = indent {
            decondenser = match indent {
                Indent::NSpaces(n_spaces) => decondenser.indent(n_spaces),
                Indent::String(string) => decondenser.indent(string),
            }
        }

        if let Some(max_line_size) = max_line_size {
            decondenser = decondenser.max_line_size(max_line_size);
        }

        if let Some(no_break_size) = no_break_size {
            decondenser = decondenser.no_break_size(no_break_size);
        }

        if let Some(groups) = groups {
            decondenser = decondenser.groups(groups.into_iter().map(Group::into_core));
        }

        if let Some(quotes) = quotes {
            decondenser = decondenser.quotes(quotes.into_iter().map(Quote::into_core));
        }

        if let Some(puncts) = puncts {
            decondenser = decondenser.puncts(puncts.into_iter().map(Punct::into_core));
        }

        if let Some(debug_layout) = debug_layout {
            decondenser = decondenser.debug_layout(debug_layout);
        }

        if let Some(debug_indent) = debug_indent {
            decondenser = decondenser.debug_indent(debug_indent);
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
