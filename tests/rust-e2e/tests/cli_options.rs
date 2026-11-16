use std::ffi::OsString;
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};

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
    let datastore_path = temp_dir.path().join(datastore);
    let webroot_path = temp_dir.path().join("webroot");
    std::fs::create_dir_all(datastore_path.join("stat"))
        .expect("create isolated datastore directory");
    std::fs::create_dir_all(&webroot_path).expect("create isolated webroot directory");
    write_config(
        temp_dir.path(),
        "cli_options_extra.conf",
        &format!(
            "\ndaemon.delay=1\ndaemon.datastore={}\ndaemon.webroot={}\n",
            datastore_path.display(),
            webroot_path.display()
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
    // Each test invocation needs its own datastore because Cargo runs tests concurrently.
    let datastore = temp_dir.path().join("test_readonly");
    let config = write_config(
        temp_dir.path(),
        "cli_options_readonly.conf",
        &format!(
            "\ndaemon.delay=1\ndaemon.datastore={}\ndaemon.webroot=/tmp/test_web\n",
            datastore.display()
        ),
    );
    let args = [
        OsString::from("-c"),
        config.into_os_string(),
        OsString::from(option),
        OsString::from("-s"),
    ];
    let output = run(&args, temp_dir.path());
    // With -s/--show, rpimonitord deliberately dies after dumping the configuration.
    // A direct Perl invocation reports that die as exit code 255; Devel::Cover may use 2.
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "{option}: expected exit code 0, 2, or 255 for --show, got {:?}; stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
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

    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    let level = option.trim_start_matches('-').len();
    assert!(
        stderr.contains(&format!("loglevel = {level}")),
        "{option}: show mode should report the selected log level; stderr={stderr:?}"
    );
    if level >= 2 {
        assert!(
            stderr.contains("Configuration::Load"),
            "{option}: level 2 should emit configuration debug output; stderr={stderr:?}"
        );
    }
    if level >= 3 {
        assert!(
            stderr.contains("Loading file:"),
            "{option}: level 3 should emit configuration-file debug output; stderr={stderr:?}"
        );
    }
}

#[test]
fn keep_option_preserves_existing_log_file() {
    let temp_dir = TempDir::new("rpimonitor-e2e-cli-keep");
    let log_file = write_config(temp_dir.path(), "existing.log", "Existing log content\n");
    let config = minimal_config(&temp_dir, "test_keep");
    let args = [
        OsString::from("-c"),
        config.clone().into_os_string(),
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

    // Control case: without -k, startup removes the existing log before opening it.
    std::fs::write(&log_file, "Control log content\n").expect("restore control log");
    let control = run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-l"),
            log_file.clone().into_os_string(),
            OsString::from("-s"),
        ],
        temp_dir.path(),
    );
    assert_cli_success_or_expected_exit("without -k", &control);
    let control_contents =
        std::fs::read_to_string(&log_file).expect("control log should be recreated");
    assert!(
        !control_contents.contains("Control log content"),
        "without -k, the existing log should be removed/recreated; contents={control_contents:?}"
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
    // The default port 8888 may be held by an installed rpimonitord: the test
    // server would fail to bind and the daemon respawn with new PIDs.
    let probe = TcpListener::bind(("127.0.0.1", 0)).expect("reserve an ephemeral port");
    let port = probe.local_addr().expect("read ephemeral port").port();
    drop(probe);
    let args = [
        OsString::from("-c"),
        config.into_os_string(),
        OsString::from("-a"),
        OsString::from("127.0.0.1"),
        OsString::from("-p"),
        OsString::from(port.to_string()),
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
    assert!(
        pids.len() >= 3,
        "{option}: expected daemon, server and monitor PIDs, got {pids:?}"
    );

    // The daemon writes the main process and worker PIDs to this file.
    for pid in &pids {
        let status = std::process::Command::new("kill")
            .args(["-TERM", pid])
            .status()
            .expect("invoke kill to stop test daemon");
        assert!(status.success(), "{option}: SIGTERM failed for PID {pid}");
    }
    assert!(
        wait_for_processes_to_stop(&pids, Duration::from_secs(5)),
        "{option}: processes remained alive after SIGTERM: {pids:?}"
    );
}

fn wait_for_processes_to_stop(pids: &[String], timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        let all_stopped = pids.iter().all(|pid| {
            match std::process::Command::new("ps")
                .args(["-o", "stat=", "-p", pid])
                .output()
            {
                Ok(output) => {
                    let state = String::from_utf8_lossy(&output.stdout);
                    state.trim().is_empty() || state.trim_start().starts_with('Z')
                }
                Err(_) => false,
            }
        });
        if all_stopped {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn combined_cli_options_are_reflected_in_configuration_dump() {
    let temp_dir = TempDir::new("rpimonitor-e2e-cli-combined");
    let log_file = temp_dir.path().join("combined.log");
    let datastore = temp_dir.path().join("datastore");
    let webroot = temp_dir.path().join("web");
    let config = write_config(
        temp_dir.path(),
        "combined.conf",
        &format!(
            "daemon.datastore={}\ndaemon.webroot={}\n",
            datastore.display(),
            webroot.display()
        ),
    );
    let args = [
        OsString::from("-c"),
        config.into_os_string(),
        OsString::from("-a"),
        OsString::from("127.0.0.1"),
        OsString::from("-p"),
        OsString::from("8899"),
        OsString::from("-d"),
        OsString::from("2"),
        OsString::from("-t"),
        OsString::from("3"),
        OsString::from("-l"),
        log_file.clone().into_os_string(),
        OsString::from("-n"),
        OsString::from("-r"),
        OsString::from("-k"),
        OsString::from("-v"),
        OsString::from("-s"),
    ];

    let output = run(&args, temp_dir.path());
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "combined options exited unexpectedly: {:?}; stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("configuration dump should be UTF-8");
    for expected in [
        "'addr' => '127.0.0.1'",
        "'port' => '8899'",
        "'delay' => '2'",
        "'timeout' => '3'",
        "'noserver' => 1",
        "'readonly' => 1",
        "'keeplogfile' => 1",
    ] {
        assert!(
            stdout.contains(expected),
            "combined options: missing {expected:?}; stdout={stdout}"
        );
    }
    assert!(
        stdout.contains(&format!("'logfile' => '{}'", log_file.display())),
        "combined options: custom log file missing; stdout={stdout}"
    );
}

#[test]
fn zero_values_are_resolved_by_configuration_defaults() {
    let temp_dir = TempDir::new("rpimonitor-e2e-cli-falsy");
    let config = minimal_config(&temp_dir, "test_falsy");
    let args = [
        OsString::from("-c"),
        config.into_os_string(),
        OsString::from("-a"),
        OsString::from("0"),
        OsString::from("-d"),
        OsString::from("0"),
        OsString::from("-t"),
        OsString::from("0"),
        OsString::from("-p"),
        OsString::from("0"),
        OsString::from("-r"),
        OsString::from("-s"),
    ];

    let output = run(&args, temp_dir.path());
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "zero-valued options exited unexpectedly: {:?}; stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("configuration dump should be UTF-8");
    for expected in [
        "'addr' => '0.0.0.0'",
        "'port' => 8888",
        "'delay' => '1'",
        "'timeout' => 5",
        "'readonly' => 1",
    ] {
        assert!(
            stdout.contains(expected),
            "zero-valued CLI options should resolve as expected; missing {expected:?}: {stdout}"
        );
    }
}

#[test]
fn mib_option_emits_configured_snmp_module() {
    let temp_dir = TempDir::new("rpimonitor-e2e-cli-mib");
    let config = write_config(
        temp_dir.path(),
        "snmp-mib.conf",
        "snmpagent.mibname=TEST-RPIMONITOR-MIB\nsnmpagent.enterpriseoid=42424\nsnmpagent.rpimonitoroid=77\nsnmpagent.organisation=Test Organisation\nsnmp.hostname.type=OCTET STRING\nsnmp.hostname.description=System hostname\nsnmp.hostname.id=1\n",
    );

    let output = run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-m"),
        ],
        temp_dir.path(),
    );

    assert!(
        output.status.success(),
        "-m should exit successfully; status={:?}; stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let mib = String::from_utf8(output.stdout).expect("generated MIB should be UTF-8");
    for expected in [
        "TEST-RPIMONITOR-MIB DEFINITIONS ::= BEGIN",
        "ORGANIZATION \"Test Organisation\"",
        "::={ enterprises 42424 }",
        "rpimonitor OBJECT IDENTIFIER ::= { rpiexperiences 77}",
        "hostname OBJECT-TYPE",
        "SYNTAX OCTET STRING",
        "DESCRIPTION",
        "\"System hostname\"",
        "::= { rpimonitor 1 }",
        "END",
    ] {
        assert!(
            mib.contains(expected),
            "generated MIB should contain {expected:?}; output: {mib}"
        );
    }
}

#[test]
fn long_mib_option_emits_mib() {
    let temp_dir = TempDir::new("rpimonitor-e2e-cli-long-mib");
    let config = write_config(temp_dir.path(), "snmp-mib.conf", "");
    let output = run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("--mib"),
        ],
        temp_dir.path(),
    );
    assert!(
        output.status.success(),
        "--mib should exit successfully; status={:?}; stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    let mib = String::from_utf8(output.stdout).expect("generated MIB should be UTF-8");
    assert!(mib.contains("RPIMONITOR-MIB DEFINITIONS ::= BEGIN"));
    assert!(mib.trim_end().ends_with("END"));
}
