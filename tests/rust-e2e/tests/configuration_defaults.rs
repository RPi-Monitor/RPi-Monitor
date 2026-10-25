use std::ffi::OsString;

use rpimonitor_e2e::{run, write_config, TempDir};

fn show_configuration(config_contents: &str) -> String {
    let directory = TempDir::new("rpimonitor-defaults");
    let config = write_config(directory.path(), "daemon.conf", config_contents);
    let output = run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-r"),
            OsString::from("-s"),
        ],
        directory.path(),
    );
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "show mode exited unexpectedly: {:?}; stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8(output.stdout).expect("configuration dump should be UTF-8")
}

#[test]
fn readonly_show_mode_exposes_documented_daemon_defaults() {
    let output = show_configuration("");
    for expected in [
        "'user' => 'pi'",
        "'group' => 'pi'",
        "'noserver' => 0",
        "'port' => 8888",
        "'addr' => '0.0.0.0'",
        "'delay' => 10",
        "'defaultinterval' => 1",
        "'readonly' => 1",
        "'timeout' => 5",
        "'sharedmemkey' => 20130906",
    ] {
        assert!(output.contains(expected), "missing default {expected:?} in {output}");
    }
}

#[test]
fn explicit_daemon_settings_override_defaults() {
    let output = show_configuration(
        "daemon.port=9999\ndaemon.addr=127.0.0.1\ndaemon.delay=3\ndaemon.timeout=15\n",
    );
    for expected in [
        "'port' => '9999'",
        "'addr' => '127.0.0.1'",
        "'delay' => '3'",
        "'timeout' => '15'",
    ] {
        assert!(output.contains(expected), "missing override {expected:?} in {output}");
    }
}