use decondenser::Decondenser;
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

    for (_test_name, test) in tests.as_table_mut().iter_mut() {
        let test = test.as_table_mut().unwrap();

        let input = test["input"].as_str().unwrap();

        let mut decondenser = Decondenser::generic();

        if let Some(line_size) = test.get("line_size") {
            let line_size = line_size.as_integer().unwrap().try_into().unwrap();
            decondenser = decondenser.line_size(line_size);
        }

        test["output"] = decondenser.decondense(input).into();
    }

    let actual = tests.to_string();

    expect_test::expect_file![tests_file].assert_eq(&actual);
}
