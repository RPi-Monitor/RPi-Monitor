use std::ffi::OsString;

use rpimonitor_e2e::{repository_root, run, TempDir};

const BINARY_RELATIVE_PATH: &str = "src/usr/bin/rpimonitord";

fn assert_version_option(option: &str) {
    let root = repository_root();
    // Each test gets its own temporary working directory, matching pytest's parametrization.
    let temp_dir = TempDir::new("rpimonitor-e2e-version");
    let output = run(
        BINARY_RELATIVE_PATH,
        &[OsString::from(option)],
        temp_dir.path(),
    );

    // Native Perl exits 255 for die; Devel::Cover may rewrite this to 2.
    assert!(
        matches!(output.status.code(), Some(2 | 255)),
        "{option}: expected die exit (2 or 255), got {:?}; stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let expected = format!(
        "{} version {{DEVELOPMENT}}",
        root.join(BINARY_RELATIVE_PATH).display()
    );
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
