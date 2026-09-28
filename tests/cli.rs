//! Black-box checks for the stable command-line boundary.

use std::process::Command;

#[test]
fn mount_file_override_preserves_all_records_and_decodes_paths() {
    let output = Command::new(env!("CARGO_BIN_EXE_dfrs"))
        .args([
            "--color",
            "never",
            "--all",
            "--mounts",
            "tests/fixtures/mounts.txt",
        ])
        .output()
        .expect("dfrs should start");

    assert!(output.status.success(), "dfrs failed: {output:?}");
    let stdout = String::from_utf8(output.stdout).expect("dfrs output should be UTF-8");
    assert!(stdout.contains("/dev/test-root"));
    assert!(stdout.contains("/Volumes/My Data"));
    assert!(stdout.contains("tmpfs"));
}

#[test]
fn unmatched_path_returns_a_successful_empty_report() {
    let output = Command::new(env!("CARGO_BIN_EXE_dfrs"))
        .args([
            "--color",
            "never",
            "--all",
            "--mounts",
            "tests/fixtures/mounts.txt",
            "/definitely/not/in/the-fixture",
        ])
        .output()
        .expect("dfrs should start");

    assert!(output.status.success(), "dfrs failed: {output:?}");
    let stdout = String::from_utf8(output.stdout).expect("dfrs output should be UTF-8");
    assert_eq!(
        stdout.lines().count(),
        1,
        "only the header should be printed"
    );
}
