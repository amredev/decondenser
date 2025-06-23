use crate::Str;

#[derive(Debug)]
pub struct Decondenser<'a> {
    /// A string used to indent a single level of nesting.
    pub indent: Str<'a>,

    /// Max number of characters per line.
    ///
    /// The width of each character is measured with the `unicode_width` crate.
    pub line_size: usize,

    /// Groups of sequences used to nest content.
    pub groups: &'a [GroupConfig<'a>],

    /// Quotes notations that enclose unbreakable string-literal content.
    pub quotes: &'a [QuoteConfig<'a>],

    /// Punctuation sequences used to separate content.
    pub puncts: &'a [PunctConfig<'a>],

    /// Output control characters for debugging the layout logic
    pub debug_layout: bool,

    /// Output indentation characters for debugging the indent logic
    pub debug_indent: bool,
}

#[derive(Debug)]
pub struct GroupConfig<'a> {
    /// The sequence that opens the group.
    pub opening: Str<'a>,

    /// The sequence that closes the group.
    pub closing: Str<'a>,
}

#[derive(Debug)]
pub struct QuoteConfig<'a> {
    /// The sequence that opens the quoted content.
    pub opening: Str<'a>,

    /// The sequence that closes the quoted content.
    pub closing: Str<'a>,

    /// The sequences that are used to escape special characters in the quoted
    /// content.
    pub escapes: &'a [EscapeConfig<'a>],
}

#[derive(Debug)]
pub struct EscapeConfig<'a> {
    pub escaped: Str<'a>,
    pub unescaped: Str<'a>,
}

pub struct PunctConfig<'a> {
    /// The punctuation sequence.
    pub text: Str<'a>,
}

impl Decondenser<'_> {
    pub fn generic() -> Decondenser<'static> {
        const {
            Decondenser {
                debug_indent: false,
                debug_layout: false,
                line_size: 80,
                indent: Str::borrowed("    "),
                groups: &[
                    GroupConfig {
                        opening: Str::borrowed("("),
                        closing: Str::borrowed(")"),
                    },
                    GroupConfig {
                        opening: Str::borrowed("["),
                        closing: Str::borrowed("]"),
                    },
                    GroupConfig {
                        opening: Str::borrowed("{"),
                        closing: Str::borrowed("}"),
                    },
                    GroupConfig {
                        opening: Str::borrowed("<"),
                        closing: Str::borrowed(">"),
                    },
                ],
                quotes: &[
                    QuoteConfig {
                        opening: Str::borrowed("\""),
                        closing: Str::borrowed("\""),
                        escapes: &[
                            EscapeConfig {
                                escaped: Str::borrowed("\\n"),
                                unescaped: Str::borrowed("\n"),
                            },
                            EscapeConfig {
                                escaped: Str::borrowed("\\r"),
                                unescaped: Str::borrowed("\r"),
                            },
                            EscapeConfig {
                                escaped: Str::borrowed("\\t"),
                                unescaped: Str::borrowed("\t"),
                            },
                            EscapeConfig {
                                escaped: Str::borrowed("\\\\"),
                                unescaped: Str::borrowed("\\"),
                            },
                            EscapeConfig {
                                escaped: Str::borrowed("\\\""),
                                unescaped: Str::borrowed("\""),
                            },
                        ],
                    },
                    QuoteConfig {
                        opening: Str::borrowed("'"),
                        closing: Str::borrowed("'"),
                        escapes: &[
                            EscapeConfig {
                                escaped: Str::borrowed("\\n"),
                                unescaped: Str::borrowed("\n"),
                            },
                            EscapeConfig {
                                escaped: Str::borrowed("\\r"),
                                unescaped: Str::borrowed("\r"),
                            },
                            EscapeConfig {
                                escaped: Str::borrowed("\\t"),
                                unescaped: Str::borrowed("\t"),
                            },
                            EscapeConfig {
                                escaped: Str::borrowed("\\\\"),
                                unescaped: Str::borrowed("\\"),
                            },
                            EscapeConfig {
                                escaped: Str::borrowed("\\'"),
                                unescaped: Str::borrowed("'"),
                            },
                        ],
                    },
                ],
                puncts: &[
                    Str::borrowed(","),
                    Str::borrowed(";"),
                    Str::borrowed(":"),
                    Str::borrowed("="),
                    Str::borrowed("?"),
                ],
            }
        }
    }
}
