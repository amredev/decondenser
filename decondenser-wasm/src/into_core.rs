use crate::wit::{BreakStyle, DecondenserParams, Group, Indent, Preset, Punct, Quote, Space};

impl DecondenserParams {
    pub(crate) fn into_decondenser(self) -> decondenser::Decondenser {
        let Self {
            extends,
            indent,
            max_line_size,
            no_break_size,
            groups,
            quotes,
            puncts,
        } = self;

        let mut decondenser = match extends.unwrap_or(Preset::Generic) {
            Preset::Empty => decondenser::Decondenser::empty(),
            Preset::Generic => decondenser::Decondenser::generic(),
        };

        if let Some(indent) = indent {
            decondenser = match indent {
                Indent::NSpaces(n_spaces) => decondenser.indent(uint_to_core(n_spaces)),
                Indent::Str(string) => decondenser.indent(string),
            }
        }

        if let Some(max_line_size) = max_line_size {
            decondenser = decondenser.max_line_size(uint_to_core(max_line_size));
        }

        if let Some(no_break_size) = no_break_size {
            decondenser = decondenser.no_break_size(uint_to_core(no_break_size));
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
            group = group.break_style(break_style.into_core());
        }

        group
    }
}

impl BreakStyle {
    fn into_core(self) -> decondenser::BreakStyle {
        match self {
            Self::Consistent => decondenser::BreakStyle::consistent(),
            Self::Compact => decondenser::BreakStyle::compact(),
        }
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

        let mut space = decondenser::Space::new();

        if let Some(size) = size {
            space = space.size(uint_to_core(size));
        }

        if let Some(breakable) = breakable {
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

/// We are using [`usize`] internally, but WIT spec supports only fixed-size
/// integers. It's highly unlikely a value greater than ~100 will be used
/// anywhere in the decondenser config. However, if someone tries to test the
/// limits of the decondenser, it's okay to fallback to [`usize::MAX`].
fn uint_to_core(value: u32) -> usize {
    value.try_into().unwrap_or(usize::MAX)
}
