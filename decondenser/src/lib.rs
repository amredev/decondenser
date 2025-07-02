//! The API of this crate is not stable yet! It's not yet intended for public use.
#![forbid(unsafe_code)]

mod ansi;
mod config;
mod formatting;
mod parsing;
mod str;
mod unescape;
mod unstable;
mod utils;

pub use self::config::*;
pub use str::IntoStr;

use self::str::Str;
use unstable::Sealed;

/// Provide configuration and run [`Decondenser::decondense()`] to format the
/// input.
#[derive(Debug, Clone)]
#[must_use = "Decondenser doesn't produce side effects. Make sure to call `decondense()` to use it"]
pub struct Decondenser {
    indent: Str,
    max_line_size: usize,
    no_break_size: Option<usize>,
    preserve_newlines: bool,
    groups: Vec<Group>,
    quotes: Vec<Quote>,
    puncts: Vec<Punct>,
    visual_size: fn(&str) -> usize,
    debug_layout: bool,
    debug_indent: bool,
}

impl Decondenser {
    /// Creates an empty [`Decondenser`] instance without any groups, quotes, or
    /// punctuation sequences configured. It is only useful as a base for custom
    /// configurations. Use [`Decondenser::generic()`] to get a general-purpose
    /// [`Decondenser`] configured for free-form text formatting.
    pub fn empty() -> Self {
        Self {
            indent: Str::new("    "),
            max_line_size: 80,
            no_break_size: None,
            preserve_newlines: false,
            groups: vec![],
            quotes: vec![],
            puncts: vec![],
            visual_size: |str| str.chars().filter(|&char| char != '\r').count(),
            debug_layout: false,
            debug_indent: false,
        }
    }

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
    pub fn generic() -> Self {
        let breakable = |size| Space::fixed(size).breakable(true);

        Self::empty()
            .groups([
                Group::new(
                    Punct::new("(").trailing_space(breakable(0)),
                    Punct::new(")").leading_space(breakable(0)),
                ),
                Group::new(
                    Punct::new("[").trailing_space(breakable(0)),
                    Punct::new("]").leading_space(breakable(0)),
                ),
                Group::new(
                    Punct::new("{")
                        .leading_space(1)
                        .trailing_space(breakable(1)),
                    Punct::new("}").leading_space(breakable(1)),
                ),
                // Elixir bitstrings
                Group::new(
                    Punct::new("<<").trailing_space(breakable(0)),
                    Punct::new(">>").leading_space(breakable(0)),
                ),
                // Many languages use these for generic types/functions
                Group::new(
                    Punct::new("<").trailing_space(breakable(0)),
                    Punct::new(">").leading_space(breakable(0)),
                ),
            ])
            .quotes([
                Quote::new("\"", "\"").escapes([
                    Escape::new("\\n", "\n"),
                    Escape::new("\\r", "\r"),
                    Escape::new("\\r", "\r"),
                    Escape::new("\\t", "\t"),
                    Escape::new("\\\\", "\\"),
                    Escape::new("\\\"", "\""),
                ]),
                Quote::new("'", "'").escapes([
                    Escape::new("\\n", "\n"),
                    Escape::new("\\r", "\r"),
                    Escape::new("\\r", "\r"),
                    Escape::new("\\t", "\t"),
                    Escape::new("\\\\", "\\"),
                    Escape::new("\\'", "'"),
                ]),
            ])
            .puncts([
                Punct::new(",").trailing_space(breakable(1)),
                Punct::new(";").trailing_space(breakable(1)),
                Punct::new(":").trailing_space(1),
                Punct::new("=>").surrounding_space(1),
                Punct::new("!==").surrounding_space(1),
                Punct::new("===").surrounding_space(1),
                Punct::new("!=").surrounding_space(1),
                Punct::new("==").surrounding_space(1),
                Punct::new("=").surrounding_space(1),
            ])
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
    #[must_use = "this is a pure function, calling it without using the result will do nothing"]
    pub fn decondense(&self, input: &str) -> String {
        self.decondense_impl(input)
    }

    /// String to used to make a single level of indentation.
    ///
    /// Defaults to 4 spaces.
    pub fn indent(mut self, value: impl Spacing) -> Self {
        self.indent = value.spacing(Sealed);
        self
    }

    /// Best-effort max size of a line to fit into.
    ///
    /// The resulting output will try to fit into this many characters per line,
    /// but it is not guaranteed. For example, the [`no_break_size`] can cause
    /// some lines to be longer than this value, or if the input has overly long
    /// sequences of non-punctuation and non-group characters that can't be
    /// broken into several lines.
    ///
    /// Line size is calculated with the [`visual_size`] algorithm, that can be
    /// overridden.
    ///
    /// [`visual_size`]: Decondenser::visual_size()
    /// [`no_break_size`]: Decondenser::no_break_size()
    pub fn max_line_size(mut self, value: usize) -> Self {
        self.max_line_size = value;
        self
    }

    /// Lines shorter than this will never be broken up at any indentation
    /// level, even if the line will be longer than the [`max_line_size`] at
    /// that indentation level.
    ///
    /// By default, this is set to `max_line_size / 2`, which is `40` if the
    /// default [`max_line_size`] of `80` is used, but is adjusted if
    /// [`max_line_size`] is overridden accordingly.
    ///
    /// [`max_line_size`]: Decondenser::max_line_size()
    pub fn no_break_size(mut self, value: usize) -> Self {
        self.no_break_size = Some(value);
        self
    }

    /// Keep line breaks from the input in the output
    pub fn preserve_newlines(mut self, value: bool) -> Self {
        self.preserve_newlines = value;
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
    /// ```
    /// # use decondenser::Decondenser;
    /// # let decondenser = Decondenser::new();
    /// #
    /// decondenser.visual_size(unicode_width::UnicodeWidthStr::width);
    /// ```
    ///
    /// Importantly, a single white space character (' ') is always considered
    /// to have the size of 1 regardless of the configured algorithm.
    ///
    /// [`unicode_width`]: https://docs.rs/unicode-width
    pub fn visual_size(mut self, value: fn(&str) -> usize) -> Self {
        self.visual_size = value;
        self
    }

    /// Set group characters that are used to nest content.
    pub fn groups(mut self, value: impl IntoIterator<Item = Group>) -> Self {
        self.groups = Vec::from_iter(value);
        self
    }

    /// Quotes notations that enclose unbreakable string-literal-like content.
    pub fn quotes(mut self, value: impl IntoIterator<Item = Quote>) -> Self {
        self.quotes = Vec::from_iter(value);
        self
    }

    /// Punctuation sequences used to separate content and potentially break it
    /// into multiple lines. This can be controlled via the [`Punct`] config.
    pub fn puncts(mut self, value: impl IntoIterator<Item = Punct>) -> Self {
        self.puncts = Vec::from_iter(value);
        self
    }
}
