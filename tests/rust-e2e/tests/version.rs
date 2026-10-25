use std::ffi::OsString;

use rpimonitor_e2e::{binary_path, run, TempDir};

fn assert_version_option(option: &str) {
    let temp_dir = TempDir::new("rpimonitor-e2e-version");
    let output = run(&[OsString::from(option)], temp_dir.path());

    // Native Perl exits 255 for die; Devel::Cover may rewrite this to 2.
    assert!(
        matches!(output.status.code(), Some(2 | 255)),
        "{option}: expected die exit (2 or 255), got {:?}; stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let expected = format!("{} version {{DEVELOPMENT}}", binary_path().display());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert_eq!(
        stderr.trim(),
        expected,
        "{option}: version output did not match expected output"
    );
}

#[test]
fn short_version_option_prints_version_to_stderr() {
    assert_version_option("-V");
}

#[test]
fn long_version_option_prints_version_to_stderr() {
    assert_version_option("--Version");
}
