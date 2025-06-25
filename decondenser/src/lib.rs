//! The API of this crate is not stable yet! It's not yet intended for public use.

mod ansi;
mod config;
mod error;
mod layout;
mod parse;
mod print;
mod str;
mod unescape;
mod utils;

pub use self::config::*;
pub use self::error::*;

use str::IntoString;

/// Provide configuration and run [`Decondenser::decondense()`] to format the
/// input.
///
/// Use [`Decondenser::generic()`] as a preset of reasonable defaults for
/// general-purpose formatting of arbitrary text based on brackets nesting.
#[derive(Debug, Clone)]
pub struct Decondenser {
    pub(crate) indent: String,
    pub(crate) max_line_size: usize,
    pub(crate) no_break_size: usize,
    pub(crate) groups: Vec<Group>,
    pub(crate) quotes: Vec<Quote>,
    pub(crate) puncts: Vec<Punct>,
    pub(crate) visual_size: fn(&str) -> usize,
    pub(crate) debug_layout: bool,
    pub(crate) debug_indent: bool,
}

impl Decondenser {
    /// Create a new [`Decondenser`] instance with the default configuration for
    /// general-purpose formatting of arbitrary text based on brackets nesting.
    ///
    /// It strives to pvoride a reasonable set of defaults for most use cases,
    /// and it's suitable to format the following types of text:
    ///
    /// - Classic Rust [`Debug`] output
    /// - Classic Elixir [`Inspect`](https://hexdocs.pm/elixir/Inspect.html)
    ///   output
    ///
    /// The default formatting is guaranteed to be stable across patch versions,
    /// but it can change between minor and major versions.
    #[must_use]
    pub fn generic() -> Self {
        Self {
            debug_indent: false,
            debug_layout: false,
            max_line_size: 80,
            no_break_size: 40,
            indent: "    ".into_string(),
            visual_size: |str| str.chars().filter(|&char| char != '\r').count(),

            groups: vec![
                Group::new(GroupDelim::new("("), GroupDelim::new(")")),
                Group::new(GroupDelim::new("["), GroupDelim::new("]")),
                Group::new(
                    GroupDelim::new("{").leading_space(" ").trailing_space(" "),
                    GroupDelim::new("}").leading_space(" "),
                ),
                Group::new(GroupDelim::new("<"), GroupDelim::new(">")),
            ],

            quotes: vec![
                Quote::new("\"", "\"").escapes(vec![
                    Escape::new("\\n", "\n"),
                    Escape::new("\\r", "\r"),
                    Escape::new("\\r", "\r"),
                    Escape::new("\\t", "\t"),
                    Escape::new("\\\\", "\\"),
                    Escape::new("\\\"", "\""),
                ]),
                Quote::new("'", "'").escapes(vec![
                    Escape::new("\\n", "\n"),
                    Escape::new("\\r", "\r"),
                    Escape::new("\\r", "\r"),
                    Escape::new("\\t", "\t"),
                    Escape::new("\\\\", "\\"),
                    Escape::new("\\'", "'"),
                ]),
            ],

            puncts: vec![
                Punct::new(",").trailing_space(Space::new(" ").break_if_needed(true)),
                Punct::new(";").trailing_space(Space::new(" ").break_if_needed(true)),
                Punct::new(":").trailing_space(Space::new(" ")),
                Punct::new("=>")
                    .leading_space(Space::new(" "))
                    .trailing_space(Space::new(" ")),
                Punct::new("!==")
                    .leading_space(Space::new(" "))
                    .trailing_space(Space::new(" ")),
                Punct::new("===")
                    .leading_space(Space::new(" "))
                    .trailing_space(Space::new(" ")),
                Punct::new("!=")
                    .leading_space(Space::new(" "))
                    .trailing_space(Space::new(" ")),
                Punct::new("==")
                    .leading_space(Space::new(" "))
                    .trailing_space(Space::new(" ")),
                Punct::new("=")
                    .leading_space(Space::new(" "))
                    .trailing_space(Space::new(" ")),
            ],
        }
    }

    /// Pretty-print any text based on brackets nesting.
    ///
    /// If the content is too big to fit into a single line of this size, it'll
    /// be broken into several lines. If the content is too small so that it
    /// doesn't fill the entire line, then several lines can be condensed into a
    /// single line.
    ///
    /// There is no guarantee that the output will not contain lines longer than
    /// this size. For example, a single long string literal or a long sequence
    /// of non-whitespace characters may span more than this many characters,
    /// and decondenser does not currently attempt to break these up.
    #[must_use]
    pub fn decondense(&self, input: &str) -> String {
        let ast = parse::l2::parse(&parse::l1::ParseParams {
            input,
            config: self,
        });

        let mut layout = layout::Layout::new(self);

        layout.begin(0, BreakStyle::Consistent);
        self.print(&mut layout, &ast);
        layout.end();

        layout.eof()
    }

    /// String to use as a single level of indentation nesting.
    #[must_use]
    pub fn indent(mut self, value: impl IntoString) -> Self {
        self.indent = value.into_string();
        self
    }

    /// Best-effort max size of a line to fit into.
    ///
    /// See how size is calculated in the docs for [`Self::visual_size()`].
    #[must_use]
    pub fn max_line_size(mut self, value: usize) -> Self {
        self.max_line_size = value;
        self
    }

    /// Lines shorter than this will never be broken up at any indentation level.
    #[must_use]
    pub fn no_break_size(mut self, value: usize) -> Self {
        self.no_break_size = value;
        self
    }

    /// Set group characters that are used to nest content.
    #[must_use]
    pub fn groups(mut self, value: impl IntoIterator<Item = Group>) -> Self {
        self.groups = Vec::from_iter(value);
        self
    }

    /// Quotes notations that enclose unbreakable string-literal-like content.
    #[must_use]
    pub fn quotes(mut self, value: impl IntoIterator<Item = Quote>) -> Self {
        self.quotes = Vec::from_iter(value);
        self
    }

    /// Punctuation sequences used to separate content and potentially break it
    /// into multiple lines. This can be controlled via the [`Punct`] config.
    #[must_use]
    pub fn puncts(mut self, value: impl IntoIterator<Item = Punct>) -> Self {
        self.puncts = Vec::from_iter(value);
        self
    }

    /// Function used to calculate the effective "visual" size of a string.
    ///
    /// The default algorithm uses [`str::chars()`] to count the number of
    /// [`char`]s in the string with the exception of `\r` characters.
    ///
    /// For more robust size calculation, the crate [`unicode_width`] can be
    /// used like this:
    ///
    /// ```ignore
    /// use decondenser::Decondenser;
    ///
    /// Decondenser::generic().visual_size(unicode_width::UnicodeWidthStr::width);
    /// ```
    ///
    /// [`unicode_width`]: https://docs.rs/unicode-width
    #[must_use]
    pub fn visual_size(mut self, value: fn(&str) -> usize) -> Self {
        self.visual_size = value;
        self
    }
}

#[cfg(feature = "unstable")]
impl Decondenser {
    /// Display the layout using special characters in the output:
    /// - `«»` - groups with [`BreakStyle::Consistent`]
    /// - `‹›` - groups with [`BreakStyle::Compact`]
    #[must_use]
    pub fn debug_layout(mut self, value: bool) -> Self {
        self.debug_layout = value;
        self
    }

    /// Show indentation levels in the output using subscript number characters
    #[must_use]
    pub fn debug_indent(mut self, value: bool) -> Self {
        self.debug_indent = value;
        self
    }
}
