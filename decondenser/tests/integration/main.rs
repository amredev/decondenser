//! Integration tests for the decondenser library.

use decondenser::Decondenser;
use std::borrow::Cow;
use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;
use std::str::FromStr;

/// This test updates the `output` values in the `formatting.toml` file. It
/// never fails, it only updates the file. However, on CI we make sure that the
/// `formatting.toml` file is fresh by checking if it changes after the test
/// run. This way we can observe the changes to the output values in PR diffs
/// and have them automatically updated via `scripts/update-tests.sh`.
#[test]
fn formatting() {
    Snapshot::new("formatting.toml").update(|test| {
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

        if let Some(indent) = test.get("indent") {
            if let Some(indent) = indent.as_str() {
                decondenser = decondenser.indent(indent.to_owned());
            } else if let Some(indent) = indent.as_integer() {
                decondenser = decondenser.indent(usize::try_from(indent).unwrap());
            }
        }

        test["output"] = decondenser.decondense(input).into();
    });
}

#[test]
fn unescaping() {
    Snapshot::new("unescaping.toml").update(|test| {
        let input = test["input"].as_str().unwrap();
        let output = decondenser::unescape(input);
        let output = match output {
            Cow::Borrowed(str) => format!("Borrowed({str})"),
            Cow::Owned(str) => str,
        };
        test["output"] = output.into();
    });
}

struct Snapshot {
    path: PathBuf,
    original: String,
    doc: toml_edit::DocumentMut,
}

impl Snapshot {
    fn new(file_name: &str) -> Self {
        let path = PathBuf::from_iter([
            std::env::var("CARGO_MANIFEST_DIR")
                .as_deref()
                .unwrap_or(env!("CARGO_MANIFEST_DIR")),
            "tests",
            "integration",
            file_name,
        ]);

        let file = std::fs::read_to_string(&path).unwrap();

        let doc = toml_edit::DocumentMut::from_str(&file).unwrap();

        Self {
            path,
            original: file,
            doc,
        }
    }

    /// Updates the original snapshot file. It never fails, it only updates the
    /// file. However, on CI we make sure that the snapshot file is fresh by
    /// checking if it changes after the test run. This way we can observe the
    /// changes to the output values in PR diffs and have them automatically
    /// updated via `scripts/update-tests.sh`.
    fn update(mut self, run_test: impl Fn(&mut toml_edit::Table)) {
        let tests = self.doc.as_table_mut();

        let solo = tests
            .iter_mut()
            .find(|(_, test)| test.get("solo").is_some());

        let snapshot_file_name = self.path.file_name().unwrap().display();

        {
            let tests: &mut dyn Iterator<Item = _> = if let Some(solo) = solo {
                eprintln!(
                    "[{snapshot_file_name}] Running a single (solo-ed) test: {}",
                    solo.0
                );
                &mut std::iter::once(solo)
            } else {
                &mut tests.iter_mut()
            };

            for (_test_name, test) in tests {
                let test = test.as_table_mut().unwrap();
                run_test(test);
            }
        }

        let actual = self.doc.to_string();
        let actual = format_toml(&actual);

        if actual == self.original {
            eprintln!("[{snapshot_file_name}] No changes to tests");
            return;
        }

        eprintln!("[{snapshot_file_name}] Updating tests");
        std::fs::write(self.path, actual).unwrap();
    }
}

fn format_toml(input: &str) -> String {
    let mut child = std::process::Command::new("taplo")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(["fmt", "-"])
        .spawn()
        .unwrap_or_else(|err| panic!("Failed to invoke taplo: {err:#?}"));

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
