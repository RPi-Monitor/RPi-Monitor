use std::ffi::OsString;

use rpimonitor_e2e::{run, TempDir};

fn assert_help_option(option: &str) {
    let directory = TempDir::new("rpimonitor-e2e-help");
    let output = run(&[OsString::from(option)], directory.path());

    assert!(
        matches!(output.status.code(), Some(2 | 255)),
        "{option}: expected help's die-shaped exit code 2 or 255, got {:?}; stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8(output.stderr).expect("help output should be UTF-8");
    for expected in [
        "NAME",
        "rpimonitord",
        "USAGE",
        "DESCRIPTION",
        "-h, --help",
        "-c, --conf",
        "-s, --show",
        "RAW DATA ACCESS",
        "static.json",
        "dynamic.json",
    ] {
        assert!(
            stderr.contains(expected),
            "{option}: help output missing {expected:?}; stderr={stderr:?}"
        );
    }
}

#[test]
fn short_help_option_displays_usage_on_stderr() {
    assert_help_option("-h");
}

#[test]
fn long_help_option_displays_usage_on_stderr() {
    assert_help_option("--help");
}
