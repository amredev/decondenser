use crate::Config;
use crate::parse::{QuotedContent, Token, TokenizeParams};
use std::path::PathBuf;
use std::str::FromStr;

#[test]
fn snapshot_tests() {
    let tests_file = PathBuf::from_iter([
        &std::env::var("CARGO_MANIFEST_DIR").unwrap(),
        "src",
        "tests",
        "decondenser-tests.toml",
    ]);
    let tests = std::fs::read_to_string(&tests_file).unwrap();

    let mut tests = toml_edit::DocumentMut::from_str(&tests).unwrap();

    for (test_name, test) in tests.as_table_mut().iter_mut() {
        let test = test.as_table_mut().unwrap();
        let input = test["input"].as_str().unwrap();
        let lang = &Config::generic();
        let actual_tokens = crate::parse::tokenize(TokenizeParams { input, lang }).unwrap();

        test["tokens"] = format!("{actual_tokens:#?}").into();
    }

    let actual = tests.to_string();

    expect_test::expect_file![tests_file].assert_eq(&actual);
}

fn token_tree_to_snapshot(token: Token) -> toml_edit::Value {
    match token {
        Token::Whitespace { start } => format!("Whitespace({start})").into(),
        Token::Group(group) => <_>::from_iter([
            (
                "group",
                [group.opening]
                    .into_iter()
                    .chain(group.closing)
                    .map(i64::from)
                    .collect(),
            ),
            (
                "content",
                group
                    .content
                    .into_iter()
                    .map(token_tree_to_snapshot)
                    .collect::<toml_edit::Value>(),
            ),
        ]),
        Token::Quoted(quoted) => <_>::from_iter([
            (
                "quoted",
                [quoted.opening]
                    .into_iter()
                    .chain(quoted.closing)
                    .map(i64::from)
                    .collect(),
            ),
            (
                "content",
                quoted
                    .content
                    .into_iter()
                    .map(quoted_content_to_snapshot)
                    .collect::<toml_edit::Value>(),
            ),
        ]),
        Token::Raw { start } => format!("Raw({start})").into(),
        Token::Punct { start } => format!("Punct({start})").into(),
    }
}

fn quoted_content_to_snapshot(quoted: QuotedContent) -> toml_edit::Value {
    match quoted {
        QuotedContent::Raw { start } => format!("Raw({start})").into(),
        QuotedContent::Escape { start } => format!("Escape({start})").into(),
    }
}
