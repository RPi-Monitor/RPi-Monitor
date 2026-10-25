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

fn minimal_config(temp_dir: &TempDir, datastore: &str) -> std::path::PathBuf {
    write_config(
        temp_dir.path(),
        "cli_options_extra.conf",
        &format!(
            "\ndaemon.delay=1\ndaemon.datastore=/tmp/{datastore}\ndaemon.webroot=/tmp/test_web\n"
        ),
    )
}

fn assert_cli_success_or_expected_exit(option: &str, output: &std::process::Output) {
    assert!(
        matches!(output.status.code(), Some(0 | 2)),
        "{option}: expected exit code 0 or 2, got {:?}; stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn short_readonly_option_enables_readonly_mode() {
    assert_readonly_option("-r");
}

#[test]
fn long_readonly_option_enables_readonly_mode() {
    assert_readonly_option("--readonly");
}

fn assert_readonly_option(option: &str) {
    let temp_dir = TempDir::new("rpimonitor-e2e-cli-readonly");
    let config = minimal_config(&temp_dir, "test_readonly");
    let args = [
        OsString::from("-c"),
        config.into_os_string(),
        OsString::from(option),
        OsString::from("-s"),
    ];
    let output = run(&args, temp_dir.path());
    assert_cli_success_or_expected_exit(option, &output);
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(
        stdout.contains("'readonly' => 1"),
        "{option}: readonly flag not set correctly; stdout={stdout:?}"
    );
}

#[test]
fn short_timeout_option_sets_command_timeout() {
    assert_timeout_option("-t");
}

#[test]
fn long_timeout_option_sets_command_timeout() {
    assert_timeout_option("--timeout");
}

fn assert_timeout_option(option: &str) {
    let temp_dir = TempDir::new("rpimonitor-e2e-cli-timeout");
    let config = minimal_config(&temp_dir, "test_timeout");
    let args = [
        OsString::from("-c"),
        config.into_os_string(),
        OsString::from(option),
        OsString::from("15"),
        OsString::from("-s"),
    ];
    let output = run(&args, temp_dir.path());
    assert_cli_success_or_expected_exit(option, &output);
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(
        stdout.contains("'timeout' => '15'"),
        "{option}: timeout not set correctly; stdout={stdout:?}"
    );
}

#[test]
fn short_verbose_option_is_accepted() {
    assert_verbose_option("-v");
}

#[test]
fn double_verbose_option_is_accepted() {
    assert_verbose_option("-vv");
}

#[test]
fn triple_verbose_option_is_accepted() {
    assert_verbose_option("-vvv");
}

fn assert_verbose_option(option: &str) {
    let temp_dir = TempDir::new("rpimonitor-e2e-cli-verbose");
    let config = minimal_config(&temp_dir, "test_verbose");
    let args = [
        OsString::from("-c"),
        config.into_os_string(),
        OsString::from(option),
        OsString::from("-s"),
    ];
    let output = run(&args, temp_dir.path());
    assert_cli_success_or_expected_exit(option, &output);
}

#[test]
fn keep_option_preserves_existing_log_file() {
    let temp_dir = TempDir::new("rpimonitor-e2e-cli-keep");
    let log_file = write_config(temp_dir.path(), "existing.log", "Existing log content\n");
    let config = minimal_config(&temp_dir, "test_keep");
    let args = [
        OsString::from("-c"),
        config.into_os_string(),
        OsString::from("-k"),
        OsString::from("-l"),
        log_file.clone().into_os_string(),
        OsString::from("-s"),
    ];
    let output = run(&args, temp_dir.path());
    assert_cli_success_or_expected_exit("-k", &output);
    let contents = std::fs::read_to_string(&log_file).expect("existing log should remain readable");
    assert!(
        contents.contains("Existing log content"),
        "-k: existing log content was not preserved; contents={contents:?}"
    );
}

#[test]
fn logfile_option_creates_custom_log_file() {
    let temp_dir = TempDir::new("rpimonitor-e2e-cli-logfile");
    let config = minimal_config(&temp_dir, "test_logfile");
    let log_file = temp_dir.path().join("custom.log");
    let args = [
        OsString::from("-c"),
        config.into_os_string(),
        OsString::from("-l"),
        log_file.clone().into_os_string(),
        OsString::from("-s"),
    ];
    let output = run(&args, temp_dir.path());
    assert_cli_success_or_expected_exit("-l/--logfile", &output);
    assert!(
        log_file.exists(),
        "-l/--logfile: custom log file was not created at {}",
        log_file.display()
    );
}

#[test]
fn long_logfile_option_creates_custom_log_file() {
    assert_logfile_option("--logfile");
}

#[test]
fn short_logfile_option_creates_custom_log_file() {
    assert_logfile_option("-l");
}

fn assert_logfile_option(option: &str) {
    let temp_dir = TempDir::new("rpimonitor-e2e-cli-logfile");
    let config = minimal_config(&temp_dir, "test_logfile");
    let log_file = temp_dir.path().join("custom.log");
    let args = [
        OsString::from("-c"),
        config.into_os_string(),
        OsString::from(option),
        log_file.clone().into_os_string(),
        OsString::from("-s"),
    ];
    let output = run(&args, temp_dir.path());
    assert_cli_success_or_expected_exit(option, &output);
    assert!(
        log_file.exists(),
        "{option}: custom log file was not created at {}",
        log_file.display()
    );
}

#[test]
fn short_background_option_creates_pid_file() {
    assert_background_option("-b");
}

#[test]
fn long_background_option_creates_pid_file() {
    assert_background_option("--background");
}

fn assert_background_option(option: &str) {
    let temp_dir = TempDir::new("rpimonitor-e2e-cli-background");
    let config = minimal_config(&temp_dir, "test_background");
    let pid_file = temp_dir.path().join("test.pid");
    let args = [
        OsString::from("-c"),
        config.into_os_string(),
        OsString::from(option),
        pid_file.clone().into_os_string(),
    ];

    let output = run(&args, temp_dir.path());
    assert_cli_success_or_expected_exit(option, &output);
    // The daemon forks a main process and worker processes; wait for their PIDs.
    for _ in 0..30 {
        let pid_count = std::fs::read_to_string(&pid_file)
            .map(|contents| contents.split_whitespace().count())
            .unwrap_or(0);
        if pid_count >= 3 {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    assert!(
        pid_file.is_file(),
        "{option}: PID file was not created at {}",
        pid_file.display()
    );

    let pids = std::fs::read_to_string(&pid_file)
        .expect("PID file should be readable")
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    assert!(!pids.is_empty(), "{option}: PID file is empty");

    // The daemon writes the main process and worker PIDs to this file.
    for pid in pids {
        let _ = std::process::Command::new("kill")
            .args(["-TERM", &pid])
            .status()
            .expect("invoke kill to stop test daemon");
    }
}
