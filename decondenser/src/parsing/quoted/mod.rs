pub(crate) mod l1;
pub(crate) mod l2;

use std::borrow::Cow;

/// Unescape the string by replacing the escape sequences with their actual
/// characters.
///
/// Uses the superset of string literls escaping grammar from the languages:
/// - JSON
/// - Rust
/// - Elixir
/// - Python
///
/// By using the superset of the grammars, this function isn't 100% compliant
/// with these languages' literals. I.e. it can unescape a Python escape such as
/// `\N{name}` even if the input isn't related to Python, but it's, for example,
/// a Rust debug output. This is because this function doesn't know the origin
/// of its input string and doesn't attempt to detect it.
///
/// So, this function is only suited as a debugging/testing tool where
/// unescaping extra characters doesn't break anything.
pub fn unescape(input: &str) -> Cow<'_, str> {
    let mut tokens = l2::unescape(input);
    let Some(first) = tokens.next() else {
        return Cow::Borrowed("");
    };

    // Optimize for the case of no escapes
    let Some(second) = tokens.next() else {
        return match first {
            l2::Token::Raw(str) => Cow::Borrowed(str),
            l2::Token::Escape(_) => {
                let mut output = String::new();
                write_token(&mut output, first);
                Cow::Owned(output)
            }
        };
    };

    let mut output = String::with_capacity(input.len());

    write_token(&mut output, first);
    write_token(&mut output, second);

    for token in tokens {
        write_token(&mut output, token);
    }

    Cow::Owned(output)
}

fn write_token(buf: &mut String, token: l2::Token<'_>) {
    match token {
        l2::Token::Raw(str) => buf.push_str(str),
        l2::Token::Escape(escape) => match escape.unescaped {
            l1::Unescaped::Char(char) => buf.push(char),
            l1::Unescaped::Ignore => {}
            l1::Unescaped::Invalid => buf.push_str(escape.source),
        },
    }
}
