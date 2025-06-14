use crate::Str;

pub struct EscapeConfig<'a> {
    pub escaped: Str<'a>,
    pub unescaped: Str<'a>,
}

pub struct GroupConfig<'a> {
    /// The sequence that opens the group.
    pub opening: Str<'a>,

    /// The sequence that closes the group.
    pub closing: Str<'a>,
}

pub struct QuoteConfig<'a> {
    /// The sequence that opens the quoted content.
    pub opening: Str<'a>,

    /// The sequence that closes the quoted content.
    pub closing: Str<'a>,

    /// The sequences that are used to escape special characters in the quoted
    /// content.
    pub escapes: &'a [EscapeConfig<'a>],
}

pub struct Decondenser<'a> {
    /// A string used to indent a single level of nesting.
    pub indent: Str<'a>,

    /// Max number of characters per line.
    ///
    /// The width of each character is measured with the `unicode_width` crate.
    pub max_width: usize,

    /// Groups of sequences used to nest content.
    pub groups: &'a [GroupConfig<'a>],

    /// Quotes notations that enclose unbreakable string-literal content.
    pub quotes: &'a [QuoteConfig<'a>],

    /// Punctuation sequences used to separate content.
    pub puncts: &'a [Str<'a>],
}

impl Decondenser<'_> {
    pub fn generic() -> Decondenser<'static> {
        const {
            Decondenser {
                max_width: 80,
                indent: Str::borrowed("  "),
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
                ],
            }
        }
    }
}
