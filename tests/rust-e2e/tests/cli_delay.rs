use std::ffi::OsString;

use rpimonitor_e2e::{run, write_config, TempDir};

fn assert_delay_option(option: &str) {
    let temp_dir = TempDir::new("rpimonitor-e2e-cli-delay");
    let config = write_config(
        temp_dir.path(),
        "cli_options_3.conf",
        "\ndaemon.datastore=/tmp/test_delay\ndaemon.webroot=/tmp/test_web\n",
    );
    let args = [
        OsString::from("-c"),
        config.into_os_string(),
        OsString::from(option),
        OsString::from("3"),
        OsString::from("-s"),
    ];

    let output = run(&args, temp_dir.path());
    assert!(
        matches!(output.status.code(), Some(0 | 2)),
        "{option}: expected exit code 0 or 2, got {:?}; stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(
        stdout.contains("'delay' => '3'"),
        "{option}: delay not set correctly; stdout={stdout:?}"
    );
}

#[test]
fn short_delay_option_sets_check_interval() {
    assert_delay_option("-d");
}

#[test]
fn long_delay_option_sets_check_interval() {
    assert_delay_option("--delay");
}
