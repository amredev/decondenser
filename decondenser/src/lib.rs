//! The API of this crate is not stable yet! It's not yet intended for public use.
#![forbid(unsafe_code)]

mod ansi;
mod config;
mod formatting;
mod parsing;
mod sealed;
mod space;
mod str;
mod unescape;
mod utils;
mod visual_size;

#[cfg(feature = "unstable")]
mod unstable;

pub use self::config::*;
pub use self::space::*;
pub use self::str::IntoStr;

use self::sealed::Sealed;
use self::str::Str;
use self::visual_size::BoxedVisualSize;

/// Provide configuration and run [`Decondenser::decondense()`] to format the
/// input.
#[derive(Debug)]
#[must_use = "Decondenser doesn't produce side effects. Make sure to call `decondense()` to use it"]
pub struct Decondenser {
    indent: Str,
    max_line_size: usize,
    no_break_size: Option<usize>,
    preserve_newlines: bool,
    groups: Vec<Group>,
    quotes: Vec<Quote>,
    puncts: Vec<Punct>,
    visual_size: BoxedVisualSize,
    debug_layout: bool,
    debug_indent: bool,
}

fn default_visual_size(str: &str) -> usize {
    str.chars().filter(|&char| char != '\r').count()
}

impl Decondenser {
    /// Creates an empty [`Decondenser`] instance without any groups, quotes, or
    /// punctuation sequences configured. It is only useful as a base for custom
    /// configurations. Use [`Decondenser::generic()`] to get a general-purpose
    /// [`Decondenser`] configured for free-form text formatting.
    pub fn empty() -> Self {
        Self {
            indent: Str::n_spaces(4),
            max_line_size: 80,
            no_break_size: None,
            preserve_newlines: false,
            groups: vec![],
            quotes: vec![],
            puncts: vec![],
            // Not using closure syntax here for the `default_visual_size` to
            // make its type name (that is used in `VisualSizeAlgorithm` Debug
            // impl) much nicer.
            visual_size: BoxedVisualSize::new(default_visual_size),
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
        let group = |start, end| {
            let space = Space::new().breakable(true);
            Group::new(
                Punct::new(start).trailing_space(space.clone()),
                Punct::new(end).leading_space(space),
            )
        };

        let punct = |symbol| {
            Punct::new(symbol).trailing_space(Space::new().breakable(SpaceFilter::min_size(1)))
        };

        Self::empty()
            .groups([
                group("(", ")"),
                group("[", "]"),
                group("{", "}"),
                // Elixir bitstrings
                group("<<", ">>"),
                // Many languages use these for generic types/functions
                group("<", ">"),
            ])
            .puncts([punct(","), punct(";")])
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
    pub fn indent(mut self, value: impl IntoIndent) -> Self {
        self.indent = value.into_indent(Sealed);
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
    /// [`visual_size`]: Self::visual_size()
    /// [`no_break_size`]: Self::no_break_size()
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

    /// Keep line breaks from the input in the output.
    ///
    /// By default, newlines will be treated as regular spaces.
    pub fn preserve_newlines(mut self, value: bool) -> Self {
        self.preserve_newlines = value;
        self
    }

    /// Algorithm used to calculate the effective "visual" size of a string.
    ///
    /// The default algorithm uses [`str::chars()`] to count the number of
    /// [`char`]s in the string with the exception of `\r` characters. It
    /// doesn't take into account printable/non-printable characters other than
    /// that.
    ///
    /// For more robust size calculation, the crate [`unicode_width`] can be
    /// used like this ([`VisualSize`] is implemented for `Fn(&str) -> usize`):
    ///
    /// ```
    /// # use decondenser::Decondenser;
    /// # let decondenser = Decondenser::empty();
    /// #
    /// decondenser.visual_size(unicode_width::UnicodeWidthStr::width);
    /// ```
    ///
    /// Importantly, a single white space character (' ') is always considered
    /// to have the size of 1 regardless of the configured algorithm.
    ///
    /// # Semver Guarantees
    ///
    /// The default algorithm MAY NOT change across patch versions, but it MAY
    /// change between minor/major versions.
    ///
    /// [`unicode_width`]: https://docs.rs/unicode-width
    pub fn visual_size(mut self, value: impl VisualSize) -> Self {
        self.visual_size = BoxedVisualSize::new(value);
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

/// Defines the algorithm for calculating the "visual" size of a string. See
/// [`Decondenser::visual_size`] for more details.
///
/// You probably don't want to implement this trait by hand, and instead use a
/// closure since this trait is implemented for `Fn(&str) -> usize`.
pub trait VisualSize: Send + Sync + 'static {
    /// The main implementation. It is assumed to be cheap, and it'll be called
    /// many times during the formatting.
    fn visual_size(&self, str: &str) -> usize;
}

impl<F: Fn(&str) -> usize + Send + Sync + 'static> VisualSize for F {
    fn visual_size(&self, str: &str) -> usize {
        self(str)
    }
}

/// A trait used to specify "string-like" values (`&str`, `String`, etc.) and
/// also the special case of a [`usize`] that represents a number of whitespace
/// characters to use.
pub trait IntoIndent {
    /// Sealed method. Can't be called outside of this crate.
    fn into_indent(self, _: Sealed) -> Str;
}

impl<T: IntoStr> IntoIndent for T {
    fn into_indent(self, _: Sealed) -> Str {
        Str::new(self)
    }
}

impl IntoIndent for usize {
    fn into_indent(self, _: Sealed) -> Str {
        Str::n_spaces(self)
    }
}
