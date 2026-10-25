use std::ffi::OsString;

use rpimonitor_e2e::{run, write_config, TempDir};

fn run_address_option(option: &str) -> std::process::Output {
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

    run(&args, temp_dir.path())
}

fn assert_address_option(option: &str) {
    let output = run_address_option(option);
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
