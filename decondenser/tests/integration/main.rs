//! Integration tests for the decondenser library.

use decondenser::Decondenser;
use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;
use std::str::FromStr;

#[test]
fn snapshot_tests() {
    let tests_file = PathBuf::from_iter([
        &std::env::var("CARGO_MANIFEST_DIR").unwrap(),
        "tests",
        "integration",
        "decondenser-tests.toml",
    ]);
    let tests = std::fs::read_to_string(&tests_file).unwrap();

    let mut tests = toml_edit::DocumentMut::from_str(&tests).unwrap();

    for (_test_name, test) in tests.as_table_mut().iter_mut() {
        let test = test.as_table_mut().unwrap();

        let input = test["input"].as_str().unwrap();

        let mut decondenser = Decondenser::generic();

        let usize = |key: &str| {
            let value = test.get(key)?;
            Some(value.as_integer().unwrap().try_into().unwrap())
        };

        let bool = |key: &str| {
            let value = test.get(key)?;
            Some(value.as_bool().unwrap())
        };

        if let Some(max_line_size) = usize("max_line_size") {
            decondenser = decondenser.max_line_size(max_line_size);
        }

        if let Some(no_break_size) = usize("no_break_size") {
            decondenser = decondenser.no_break_size(no_break_size);
        }

        if let Some(debug_layout) = bool("debug_layout") {
            decondenser = decondenser.debug_layout(debug_layout);
        }

        test["output"] = decondenser.decondense(input).into();
    }

    let actual = tests.to_string();
    let actual = format_toml(&actual);

    expect_test::expect_file![tests_file].assert_eq(&actual);
}

fn format_toml(input: &str) -> String {
    let vars: std::collections::BTreeMap<_, _> = std::env::vars().collect();
    eprintln!("{vars:#?}");

    let mut child = std::process::Command::new("taplo")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(["fmt", "-"])
        .spawn()
        .unwrap_or_else(|err| panic!("Failed to invoke `taplo fmt`: {err:#?}"));

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();

    let output = child.wait_with_output().unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(
        output.status.success(),
        "taplo fmt failed with status {}\n\
        [stdout]{stdout}\n\n[stderr]{stderr}",
        output.status,
    );

    stdout
}
