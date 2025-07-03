#![expect(missing_docs)]

// Verify that the CLI is runnable at all. It also validates some clap
// invariants, that are checked only at runtime (e.g. if arg names are reused).
#[test]
fn smoke() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_decondenser"))
        .arg("--version")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    eprintln!("[stdout]\n{stdout}\n\n[stderr]\n{stderr}");

    assert!(
        output.status.success(),
        "`decondenser --version` failed with status: {}",
        output.status
    );

    assert_eq!(
        stdout.trim(),
        concat!("decondenser ", env!("CARGO_PKG_VERSION"))
    );
}
