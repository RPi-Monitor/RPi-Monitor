/// Black-box tests for daemon lifecycle: PID file creation, graceful shutdown
/// via SIGTERM/SIGINT, and SIGUSR1/SIGUSR2 log-level adjustment.
///
/// Ported from `test_daemonize_and_writepid.py`.  Only observable, external
/// behaviour is checked; no internal Perl methods are invoked.
use std::ffi::OsString;
use std::fs;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use rpimonitor_e2e::{run, write_config, TempDir};

// ---------------------------------------------------------------------------
// Helpers shared across tests
// ---------------------------------------------------------------------------

struct BackgroundDaemon {
    pid_file: std::path::PathBuf,
}

impl BackgroundDaemon {
    fn stop(&self) {
        let Ok(contents) = fs::read_to_string(&self.pid_file) else {
            return;
        };
        for pid in contents.split_whitespace() {
            let _ = Command::new("kill").args(["-TERM", pid]).status();
        }
    }

    fn stop_and_wait(&self, timeout: Duration) -> bool {
        self.stop();
        let Ok(contents) = fs::read_to_string(&self.pid_file) else {
            return false;
        };
        let pids: Vec<&str> = contents.split_whitespace().collect();
        if pids.is_empty() {
            return false;
        }
        let deadline = Instant::now() + timeout;
        loop {
            let all_gone = pids.iter().all(|pid| {
                let out = Command::new("ps").args(["-o", "stat=", "-p", pid]).output();
                match out {
                    Ok(o) => {
                        let s = String::from_utf8_lossy(&o.stdout);
                        s.trim().is_empty() || s.trim_start().starts_with('Z')
                    }
                    Err(_) => false,
                }
            });
            if all_gone {
                let _ = fs::remove_file(&self.pid_file);
                return true;
            }
            if Instant::now() >= deadline {
                return false;
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}

impl Drop for BackgroundDaemon {
    fn drop(&mut self) {
        self.stop();
    }
}

fn wait_for_file(path: &std::path::Path, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if path.exists() {
            return true;
        }
        thread::sleep(Duration::from_millis(100));
    }
    path.exists()
}

fn wait_for_file_content(path: &std::path::Path, needle: &str, timeout: Duration) -> Option<String> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Ok(content) = fs::read_to_string(path) {
            if content.contains(needle) {
                return Some(content);
            }
        }
        if Instant::now() >= deadline {
            return None;
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn read_pids(pid_file: &std::path::Path) -> Vec<u32> {
    fs::read_to_string(pid_file)
        .unwrap_or_default()
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect()
}

fn send_signal(pid: u32, sig: &str) {
    let _ = Command::new("kill")
        .args([&format!("-{sig}"), &pid.to_string()])
        .status();
}

fn minimal_noserver_config(dir: &std::path::Path, log_file: Option<&std::path::Path>) -> String {
    let webroot = dir.join("web");
    let datastore = dir.join("data");
    let mut cfg = format!(
        "daemon.noserver=1\ndaemon.delay=1\ndaemon.timeout=5\ndaemon.datastore={}\ndaemon.webroot={}\n",
        datastore.display(),
        webroot.display(),
    );
    if let Some(lf) = log_file {
        cfg.push_str(&format!("daemon.logfile={}\n", lf.display()));
    }
    cfg
}

fn setup_dirs(dir: &std::path::Path) {
    fs::create_dir_all(dir.join("web")).expect("create webroot");
    fs::create_dir_all(dir.join("data").join("stat")).expect("create datastore/stat");
}

// ---------------------------------------------------------------------------
// PID file creation
// ---------------------------------------------------------------------------

/// `test_background_mode_creates_pid_file` (Python)
/// Background mode must write at least one valid PID to the pid file.
#[test]
fn background_mode_creates_pid_file_with_valid_content() {
    let dir = TempDir::new("rpimonitor-e2e-daemon-pid");
    setup_dirs(dir.path());

    let config = write_config(dir.path(), "daemon-pid.conf", &minimal_noserver_config(dir.path(), None));
    let pid_file = dir.path().join("rpimonitor.pid");
    let daemon = BackgroundDaemon { pid_file: pid_file.clone() };

    let output = run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-b"),
            pid_file.clone().into_os_string(),
        ],
        dir.path(),
    );
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "daemon launch failed: status={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        wait_for_file(&pid_file, Duration::from_secs(5)),
        "PID file was not created; stderr={:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    let pids = read_pids(&pid_file);
    assert!(!pids.is_empty(), "PID file should contain at least one valid PID");
    assert!(
        pids.iter().all(|&p| p > 0),
        "all entries in PID file should be positive integers; got: {:?}",
        pids
    );

    assert!(
        daemon.stop_and_wait(Duration::from_secs(5)),
        "daemon did not stop after SIGTERM"
    );
}

// ---------------------------------------------------------------------------
// Daemon detaches from terminal
// ---------------------------------------------------------------------------

/// `test_daemonize_detaches_from_terminal` (Python)
/// The PID recorded in the file must be different from the test process PID.
#[test]
fn daemonized_process_has_different_pid_than_test_process() {
    let dir = TempDir::new("rpimonitor-e2e-daemon-detach");
    setup_dirs(dir.path());

    let config = write_config(dir.path(), "daemon-detach.conf", &minimal_noserver_config(dir.path(), None));
    let pid_file = dir.path().join("rpimonitor-detach.pid");
    let daemon = BackgroundDaemon { pid_file: pid_file.clone() };

    run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-b"),
            pid_file.clone().into_os_string(),
        ],
        dir.path(),
    );

    assert!(
        wait_for_file(&pid_file, Duration::from_secs(5)),
        "PID file was not created"
    );

    let pids = read_pids(&pid_file);
    assert!(!pids.is_empty(), "PID file must contain at least one PID");
    let own_pid = std::process::id();
    assert!(
        pids.iter().all(|&p| p != own_pid),
        "daemon PID ({pids:?}) must differ from test process PID ({own_pid})"
    );

    assert!(
        daemon.stop_and_wait(Duration::from_secs(5)),
        "daemon did not stop after SIGTERM"
    );
}

// ---------------------------------------------------------------------------
// Graceful shutdown on SIGTERM
// ---------------------------------------------------------------------------

/// `test_stop_handles_graceful_shutdown` (Python) — SIGTERM variant
#[test]
fn daemon_terminates_gracefully_on_sigterm() {
    let dir = TempDir::new("rpimonitor-e2e-daemon-sigterm");
    setup_dirs(dir.path());

    let config = write_config(dir.path(), "daemon-sigterm.conf", &minimal_noserver_config(dir.path(), None));
    let pid_file = dir.path().join("rpimonitor-sigterm.pid");
    let daemon = BackgroundDaemon { pid_file: pid_file.clone() };

    run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-b"),
            pid_file.clone().into_os_string(),
        ],
        dir.path(),
    );

    assert!(
        wait_for_file(&pid_file, Duration::from_secs(5)),
        "PID file was not created before SIGTERM"
    );

    // Brief settle time avoids a race when tests run in parallel under load.
    thread::sleep(Duration::from_millis(200));

    assert!(
        daemon.stop_and_wait(Duration::from_secs(10)),
        "daemon did not terminate gracefully within 10 s after SIGTERM"
    );
}

// ---------------------------------------------------------------------------
// Graceful shutdown on SIGINT
// ---------------------------------------------------------------------------

/// `test_stop_handles_graceful_shutdown` (Python) — SIGINT variant
#[test]
fn daemon_terminates_gracefully_on_sigint() {
    let dir = TempDir::new("rpimonitor-e2e-daemon-sigint");
    setup_dirs(dir.path());

    let config = write_config(dir.path(), "daemon-sigint.conf", &minimal_noserver_config(dir.path(), None));
    let pid_file = dir.path().join("rpimonitor-sigint.pid");
    let daemon = BackgroundDaemon { pid_file: pid_file.clone() };

    run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-b"),
            pid_file.clone().into_os_string(),
        ],
        dir.path(),
    );

    assert!(
        wait_for_file(&pid_file, Duration::from_secs(5)),
        "PID file was not created before SIGINT"
    );

    // Send SIGINT instead of SIGTERM
    let Ok(contents) = fs::read_to_string(&pid_file) else {
        panic!("cannot read pid file");
    };
    for pid in contents.split_whitespace() {
        let _ = Command::new("kill").args(["-INT", pid]).status();
    }

    // Re-read pids after signal and wait for exit
    let pids: Vec<String> = contents.split_whitespace().map(|s| s.to_owned()).collect();
    let deadline = Instant::now() + Duration::from_secs(8);
    let all_gone = loop {
        let all = pids.iter().all(|pid| {
            let out = Command::new("ps").args(["-o", "stat=", "-p", pid]).output();
            match out {
                Ok(o) => {
                    let s = String::from_utf8_lossy(&o.stdout);
                    s.trim().is_empty() || s.trim_start().starts_with('Z')
                }
                Err(_) => false,
            }
        });
        if all {
            break true;
        }
        if Instant::now() >= deadline {
            break false;
        }
        thread::sleep(Duration::from_millis(100));
    };
    // Prevent drop from re-sending TERM
    drop(daemon);
    assert!(all_gone, "daemon did not terminate gracefully within 8 s after SIGINT");
}

// ---------------------------------------------------------------------------
// show mode + -b must NOT create PID file
// ---------------------------------------------------------------------------

/// `test_show_mode_with_pidfile_never_creates_pidfile` (Python)
/// `-s` (show) dies after printing config; it must not call writePID.
#[test]
fn show_mode_with_background_flag_does_not_create_pid_file() {
    let dir = TempDir::new("rpimonitor-e2e-daemon-show-pid");
    setup_dirs(dir.path());

    let config = write_config(dir.path(), "daemon-show.conf", &minimal_noserver_config(dir.path(), None));
    let pid_file = dir.path().join("rpimonitor-show.pid");

    let output = run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-b"),
            pid_file.clone().into_os_string(),
            OsString::from("-s"),
        ],
        dir.path(),
    );

    // -s dies immediately: exit code 2 (via Devel::Cover) or 255 (native Perl die)
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "-s exit code unexpected: {:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    // Allow brief settle time, then assert no PID file was written
    thread::sleep(Duration::from_millis(300));
    assert!(
        !pid_file.exists(),
        "-s + -b must not create the PID file, but found: {pid_file:?}"
    );
}

// ---------------------------------------------------------------------------
// SIGUSR1 increments log level
// ---------------------------------------------------------------------------

/// `test_sigusr1_increments_loglevel` (Python)
/// After SIGUSR1, the daemon logs "Change loglevel to 1" in its log file.
#[test]
fn sigusr1_increments_loglevel_and_logs_new_level() {
    let dir = TempDir::new("rpimonitor-e2e-sigusr1");
    setup_dirs(dir.path());
    let log_file = dir.path().join("sigusr1.log");

    let config_content = minimal_noserver_config(dir.path(), Some(&log_file));
    let config = write_config(dir.path(), "sigusr1.conf", &config_content);
    let pid_file = dir.path().join("rpimonitor-sigusr1.pid");
    let daemon = BackgroundDaemon { pid_file: pid_file.clone() };

    run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-b"),
            pid_file.clone().into_os_string(),
        ],
        dir.path(),
    );

    assert!(
        wait_for_file(&pid_file, Duration::from_secs(5)),
        "PID file was not created"
    );

    let pids = read_pids(&pid_file);
    assert!(!pids.is_empty(), "PID file empty");
    let daemon_pid = pids[0];

    send_signal(daemon_pid, "USR1");

    let content = wait_for_file_content(&log_file, "Change loglevel to 1", Duration::from_secs(5));
    assert!(
        content.is_some(),
        "SIGUSR1 did not produce 'Change loglevel to 1' in {log_file:?}; log={:?}",
        fs::read_to_string(&log_file).unwrap_or_default()
    );

    assert!(
        daemon.stop_and_wait(Duration::from_secs(5)),
        "daemon did not stop after SIGTERM"
    );
}

// ---------------------------------------------------------------------------
// SIGUSR2 decrements log level
// ---------------------------------------------------------------------------

/// `test_sigusr2_decrements_loglevel` (Python)
/// Start with `-v` (loglevel=1), send SIGUSR2 → daemon logs "Change loglevel to 0".
#[test]
fn sigusr2_decrements_loglevel_and_logs_new_level() {
    let dir = TempDir::new("rpimonitor-e2e-sigusr2");
    setup_dirs(dir.path());
    let log_file = dir.path().join("sigusr2.log");

    let config_content = minimal_noserver_config(dir.path(), Some(&log_file));
    let config = write_config(dir.path(), "sigusr2.conf", &config_content);
    let pid_file = dir.path().join("rpimonitor-sigusr2.pid");
    let daemon = BackgroundDaemon { pid_file: pid_file.clone() };

    // -v sets loglevel=1 so that SIGUSR2 has a truthy branch to decrement
    run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-b"),
            pid_file.clone().into_os_string(),
            OsString::from("-v"),
        ],
        dir.path(),
    );

    assert!(
        wait_for_file(&pid_file, Duration::from_secs(5)),
        "PID file was not created"
    );

    let pids = read_pids(&pid_file);
    assert!(!pids.is_empty(), "PID file empty");
    let daemon_pid = pids[0];

    send_signal(daemon_pid, "USR2");

    let content = wait_for_file_content(&log_file, "Change loglevel to 0", Duration::from_secs(5));
    assert!(
        content.is_some(),
        "SIGUSR2 did not produce 'Change loglevel to 0' in {log_file:?}; log={:?}",
        fs::read_to_string(&log_file).unwrap_or_default()
    );

    assert!(
        daemon.stop_and_wait(Duration::from_secs(5)),
        "daemon did not stop after SIGTERM"
    );
}
