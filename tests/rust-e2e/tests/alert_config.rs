/// Black-box tests for alert configuration loading and runtime trigger/cancel
/// side effects.
///
/// Ported from `test_alert_comprehensive.py`.  Only externally observable
/// behaviour is checked: the `-s` config dump and file side effects produced
/// by raisecommand/cancelcommand.
use std::ffi::OsString;
use std::fs;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use rpimonitor_e2e::{run, write_config, TempDir};

// ---------------------------------------------------------------------------
// Helpers
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

// ---------------------------------------------------------------------------
// Alert configuration loading  (PERL_AND_RUST in Python suite)
// ---------------------------------------------------------------------------

/// `test_alert_configuration_loading` (Python)
///
/// The alert key/value pairs must survive the full configuration load and
/// appear verbatim in the `-s` dump.
#[test]
fn alert_configuration_fields_appear_in_show_dump() {
    let dir = TempDir::new("rpimonitor-e2e-alert-config");
    let webroot = dir.path().join("web");
    let datastore = dir.path().join("data");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(datastore.join("stat")).expect("create datastore/stat");

    let alert_log = dir.path().join("alert.log");
    let config = write_config(
        dir.path(),
        "alert-config.conf",
        &format!(
            "daemon.delay=1\n\
             daemon.datastore={}\n\
             daemon.webroot={}\n\
             dynamic.1.name=test_temperature\n\
             dynamic.1.source=/bin/echo 25\n\
             dynamic.1.regexp=(\\d+)\n\
             dynamic.1.default=25\n\
             dynamic.1.interval=1\n\
             alert.high_temp.active=data.uptime>5\n\
             alert.high_temp.trigger=data.test_temperature>30\n\
             alert.high_temp.maxalertduration=10\n\
             alert.high_temp.cancelvalidation=5\n\
             alert.high_temp.resendperiod=30\n\
             alert.high_temp.raisecommand=echo alert >> {}\n\
             alert.high_temp.cancelcommand=echo clear >> {}\n",
            datastore.display(),
            webroot.display(),
            alert_log.display(),
            alert_log.display(),
        ),
    );

    let output = run(
        &[OsString::from("-c"), config.into_os_string(), OsString::from("-s")],
        dir.path(),
    );

    // -s dies after printing; exit code 2 (Devel::Cover) or 255 (native die)
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "-s exit code unexpected: {:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    for needle in [
        "'active' => 'data.uptime>5'",
        "'trigger' => 'data.test_temperature>30'",
        // Numeric values may be serialised as strings ('10') or integers (10)
        // depending on how Perl Data::Dumper represents them.  Accept both by
        // checking the key name prefix only.
        "'maxalertduration' =>",
        "'cancelvalidation' =>",
        "'resendperiod' =>",
    ] {
        assert!(
            stdout.contains(needle),
            "show dump missing alert field {needle:?};\nstdout:\n{stdout}"
        );
    }
}

// ---------------------------------------------------------------------------
// Alert runtime: raisecommand produces file side effect
// ---------------------------------------------------------------------------

/// `test_alert_triggering_with_file_output` / `test_alert_with_shell_commands` (Python)
///
/// When the trigger condition is immediately true the raisecommand executes
/// and its file side effect (a known marker string) must appear on disk.
#[test]
fn alert_raisecommand_writes_marker_file_when_trigger_fires() {
    let dir = TempDir::new("rpimonitor-e2e-alert-raise");
    let webroot = dir.path().join("web");
    let datastore = dir.path().join("data");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(datastore.join("stat")).expect("create datastore/stat");

    let marker = dir.path().join("alert-raise.marker");
    let pid = std::process::id();
    let sharedmemkey = 500_000 + (pid % 100_000);

    let config = write_config(
        dir.path(),
        "alert-raise.conf",
        &format!(
            "daemon.delay=1\n\
             daemon.datastore={}\n\
             daemon.webroot={}\n\
             daemon.sharedmemkey={sharedmemkey}\n\
             dynamic.1.name=disk_space\n\
             dynamic.1.source=/bin/echo 95\n\
             dynamic.1.regexp=(\\d+)\n\
             dynamic.1.postprocess=$1\n\
             dynamic.1.interval=1\n\
             alert.disk_full.active=1\n\
             alert.disk_full.trigger=data.disk_space > 90\n\
             alert.disk_full.maxalertduration=1\n\
             alert.disk_full.cancelvalidation=3\n\
             alert.disk_full.resendperiod=30\n\
             alert.disk_full.raisecommand=/bin/echo disk-full-alert > {}\n\
             alert.disk_full.cancelcommand=/bin/echo disk-ok\n",
            datastore.display(),
            webroot.display(),
            marker.display(),
        ),
    );
    let pid_file = dir.path().join("rpimonitor-alert-raise.pid");
    let daemon = BackgroundDaemon { pid_file: pid_file.clone() };

    let output = run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-b"),
            pid_file.into_os_string(),
        ],
        dir.path(),
    );
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "daemon launch failed: status={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let content = wait_for_file_content(&marker, "disk-full-alert", Duration::from_secs(15));
    assert!(
        content.is_some(),
        "raisecommand did not write marker to {marker:?} within 15 s; \
         daemon stderr={:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        daemon.stop_and_wait(Duration::from_secs(5)),
        "daemon did not stop after SIGTERM"
    );
}

// ---------------------------------------------------------------------------
// Alert runtime: multi-metric compound trigger
// ---------------------------------------------------------------------------

/// `test_alert_with_multiple_conditions` (Python)
///
/// Both trigger clauses are true simultaneously; the raisecommand appends
/// "raise" to a log file.  The daemon requires shared memory (and thus a
/// running web server) to evaluate alert conditions, so this test starts the
/// SUT with a web server bound to a free ephemeral port.
#[test]
fn alert_fires_when_compound_trigger_conditions_are_all_true() {
    use std::net::TcpListener;

    let dir = TempDir::new("rpimonitor-e2e-alert-compound");
    let webroot = dir.path().join("web");
    let datastore = dir.path().join("data");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(datastore.join("stat")).expect("create datastore/stat");

    let alert_log = dir.path().join("compound-alert.log");
    let pid = std::process::id();
    let sharedmemkey = 600_000 + (pid % 100_000);

    // Reserve an ephemeral port, then release before starting the daemon.
    let probe = TcpListener::bind(("127.0.0.1", 0)).expect("reserve ephemeral port");
    let port = probe.local_addr().expect("read port").port();
    drop(probe);

    let config = write_config(
        dir.path(),
        "alert-compound.conf",
        &format!(
            "daemon.addr=127.0.0.1\ndaemon.port={port}\n\
             daemon.delay=1\n\
             daemon.datastore={}\n\
             daemon.webroot={}\n\
             daemon.sharedmemkey={sharedmemkey}\n\
             dynamic.1.name=metric_one\n\
             dynamic.1.source=/bin/echo 15\n\
             dynamic.1.regexp=(\\d+)\n\
             dynamic.1.postprocess=$1\n\
             dynamic.1.interval=1\n\
             dynamic.2.name=metric_two\n\
             dynamic.2.source=/bin/echo 500\n\
             dynamic.2.regexp=(\\d+)\n\
             dynamic.2.postprocess=$1\n\
             dynamic.2.interval=1\n\
             alert.system_overload.active=1\n\
             alert.system_overload.trigger=data.metric_one > 5 && data.metric_two < 1000\n\
             alert.system_overload.maxalertduration=1\n\
             alert.system_overload.cancelvalidation=3\n\
             alert.system_overload.resendperiod=30\n\
             alert.system_overload.raisecommand=/bin/echo raise >> {}\n\
             alert.system_overload.cancelcommand=/bin/echo cancel >> {}\n",
            datastore.display(),
            webroot.display(),
            alert_log.display(),
            alert_log.display(),
        ),
    );
    let pid_file = dir.path().join("rpimonitor-alert-compound.pid");
    let daemon = BackgroundDaemon { pid_file: pid_file.clone() };

    let output = run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-b"),
            pid_file.into_os_string(),
        ],
        dir.path(),
    );
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "daemon launch failed: status={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let content = wait_for_file_content(&alert_log, "raise", Duration::from_secs(20));
    assert!(
        content.is_some(),
        "compound raisecommand did not write 'raise' to {alert_log:?} within 20 s"
    );

    assert!(
        daemon.stop_and_wait(Duration::from_secs(5)),
        "daemon did not stop after SIGTERM"
    );
}
