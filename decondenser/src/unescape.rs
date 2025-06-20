use std::collections::BTreeMap;

pub struct EscapingConfig<'a> {
    /// The prefix character that indicates the beginning of an escape sequence.
    pub escape_char: char,

    /// The mapping of characters that follow the prefix character to their
    /// unescaped counterparts.
    pub mapping: &'a [(char, char)],
}

/// A sensible default config with the most common `\` escape char and mapping.
impl Default for EscapingConfig<'_> {
    fn default() -> Self {
        EscapingConfig {
            escape_char: '\\',
            mapping: &[('n', '\n'), ('r', '\r'), ('t', '\t'), ('\\', '\\')],
        }
    }
}

pub struct UnescapeParams<'a> {
    pub input: &'a str,

    /// If not specified the [`EscapingConfig::default()`] will be used
    pub config: Option<EscapingConfig<'a>>,
}

pub struct UnescapeOutput {
    pub output: String,
}

/// Unescape the string by replacing the escape sequences with their actual characters.
#[must_use = "this is a pure function; ignoring its result is definitely a bug"]
pub fn unescape(params: UnescapeParams<'_>) -> UnescapeOutput {
    let UnescapeParams { input, config } = params;
    let config = config.unwrap_or_default();

    let mut output = String::with_capacity(input.len());

    let mut cursor = input.chars();

    while let Some(char) = cursor.next() {
        if char != config.escape_char {
            output.push(char);
            continue;
        }

        let Some(next) = cursor.next() else {
            // If the escape character is at the end of the string, we can't
            // unescape it. Just leave it as is.
            output.push(config.escape_char);
            break;
        };

        let replacement = config.mapping.iter().find(|&&(src, _)| src == char);

        if let Some(&(_, dest)) = replacement {
            output.push(dest);
        } else {
            // If the character is not in the mapping, keep it as is
            output.extend([config.escape_char, char]);
        }
    }

    UnescapeOutput { output }
}
