mod l1;
mod l2;

use std::borrow::Cow;

/// Unescape the string by replacing the escape sequences with their actual
/// characters.
///
/// The superset of escapes grammar from the following languages is used:
/// - JSON
/// - Rust
/// - Elixir
/// - Python
///
/// By using the superset of the grammars, this function isn't 100% compliant
/// with these languages' literals. I.e. it can unescape an escape, that is a
/// valid Python escape such as `\N{name}` in Rust debug output since this
/// function doesn't know the origin of its input string.
///
/// So, this function is only suited as a debugging/testing tool where your
/// input is free of such edge cases, and even if not - unescaping extra
/// characters won't break anything.
pub fn unescape(input: &str) -> Cow<'_, str> {
    let mut tokens = l2::unescape(input);
    let Some(first) = tokens.next() else {
        return "".into();
    };

    // Optimize for the case of no escapes
    let Some(second) = tokens.next() else {
        return match first {
            l2::Token::Raw(str) => str.into(),
            l2::Token::Escape(_) => {
                let mut output = String::new();
                write_token(&mut output, first);
                output.into()
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
            l1::Unescaped::Invalid => buf.push_str(escape.escaped),
        },
    }
}
