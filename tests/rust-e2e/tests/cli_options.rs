use std::ffi::OsString;

use rpimonitor_e2e::{binary_path, run, write_config, TempDir};

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

fn assert_address_option(option: &str) {
    let temp_dir = TempDir::new("rpimonitor-e2e-cli-addr");
    let config = write_config(
        temp_dir.path(),
        "cli_options_1.conf",
        "\ndaemon.delay=1\ndaemon.datastore=/tmp/test_addr\ndaemon.webroot=/tmp/test_web\n",
    );
    let args = [
        OsString::from("-c"),
        config.into_os_string(),
        OsString::from(option),
        OsString::from("127.0.0.1"),
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
        stdout.contains("'addr' => '127.0.0.1'"),
        "{option}: address not set correctly; stdout={stdout:?}"
    );
}

#[test]
fn short_addr_option_sets_web_server_bind_address() {
    assert_address_option("-a");
}

#[test]
fn long_addr_option_sets_web_server_bind_address() {
    assert_address_option("--addr");
}

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

fn assert_noserver_option(option: &str) {
    let temp_dir = TempDir::new("rpimonitor-e2e-cli-noserver");
    let config = write_config(
        temp_dir.path(),
        "cli_options_6.conf",
        "\ndaemon.delay=1\ndaemon.datastore=/tmp/test_noserver\ndaemon.webroot=/tmp/test_web\n",
    );
    let args = [
        OsString::from("-c"),
        config.into_os_string(),
        OsString::from(option),
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
        stdout.contains("'noserver' => 1"),
        "{option}: noserver flag not set correctly; stdout={stdout:?}"
    );
}

#[test]
fn short_noserver_option_disables_web_server() {
    assert_noserver_option("-n");
}

#[test]
fn long_noserver_option_disables_web_server() {
    assert_noserver_option("--noserver");
}

fn assert_port_option(option: &str) {
    let temp_dir = TempDir::new("rpimonitor-e2e-cli-port");
    let config = write_config(
        temp_dir.path(),
        "cli_options_7.conf",
        "\ndaemon.delay=1\ndaemon.datastore=/tmp/test_port\ndaemon.webroot=/tmp/test_web\n",
    );
    let args = [
        OsString::from("-c"),
        config.into_os_string(),
        OsString::from(option),
        OsString::from("9999"),
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
        stdout.contains("'port' => '9999'"),
        "{option}: port not set correctly; stdout={stdout:?}"
    );
}

#[test]
fn short_port_option_sets_web_server_port() {
    assert_port_option("-p");
}

#[test]
fn long_port_option_sets_web_server_port() {
    assert_port_option("--port");
}
