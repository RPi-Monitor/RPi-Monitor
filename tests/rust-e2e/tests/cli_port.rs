use std::ffi::OsString;

use rpimonitor_e2e::{run, write_config, TempDir};

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
