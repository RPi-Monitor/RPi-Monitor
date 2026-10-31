/// Black-box tests for RRD file creation.
///
/// Ported from `test_rrd_creation.py`.  The tests launch the real
/// `rpimonitord` daemon, let it initialise, then inspect the file-system
/// side-effects that `rrdtool` leaves behind — no internal Perl state is
/// inspected.
///
/// `rrdtool info` is invoked as an external process to avoid adding a
/// third-party Rust crate.
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

fn wait_for_path(path: &std::path::Path, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if path.exists() {
            return true;
        }
        thread::sleep(Duration::from_millis(100));
    }
    path.exists()
}

/// Run `rrdtool info <file>` and return its stdout as a String.
fn rrdtool_info(rrd_path: &std::path::Path) -> String {
    let out = Command::new("rrdtool")
        .args(["info", &rrd_path.to_string_lossy()])
        .output()
        .expect("rrdtool info");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Assert that `rrdtool info` output contains a DS entry of the given type.
fn assert_ds_type(info: &str, ds_name: &str, expected_type: &str) {
    let key = format!("ds[{ds_name}].type");
    let found = info
        .lines()
        .find(|l| l.contains(&key))
        .unwrap_or_else(|| panic!("key '{key}' not found in rrdtool info output:\n{info}"));
    assert!(
        found.contains(expected_type),
        "DS type for '{ds_name}' should be '{expected_type}'; got: {found}"
    );
}

/// Assert that `rrdtool info` output contains exactly the expected number of RRA archives.
fn assert_rra_count(info: &str, expected: usize) {
    let count = info
        .lines()
        .filter(|l| l.contains(".cf = "))
        .count();
    assert_eq!(
        count, expected,
        "expected {expected} RRAs, found {count}; rrdtool output:\n{info}"
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// `test_rrd_directory_and_empty_rrd_creation` (Python)
///
/// After a short run the daemon must:
///   1. create `<datastore>/stat/` directory, and
///   2. create `<datastore>/stat/empty.rrd` with the standard schema.
#[test]
fn daemon_creates_rrd_stat_directory_and_empty_rrd() {
    let dir = TempDir::new("rpimonitor-e2e-rrd-empty");
    let datastore = dir.path().join("data");
    let webroot = dir.path().join("web");
    fs::create_dir_all(&webroot).expect("create webroot");
    // Pre-create the datastore parent (the SUT's mkdir is non-recursive)
    fs::create_dir_all(&datastore).expect("create datastore");

    let config = write_config(
        dir.path(),
        "rrd-empty.conf",
        &format!(
            "daemon.datastore={}\ndaemon.webroot={}\ndaemon.delay=1\ndaemon.readonly=0\ndaemon.noserver=1\n",
            datastore.display(),
            webroot.display(),
        ),
    );
    let pid_file = dir.path().join("rpimonitor-rrd-empty.pid");
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

    let stat_dir = datastore.join("stat");
    assert!(
        wait_for_path(&stat_dir, Duration::from_secs(8)),
        "stat directory was not created at {stat_dir:?}; stderr={:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stat_dir.is_dir(), "stat path exists but is not a directory");

    let empty_rrd = stat_dir.join("empty.rrd");
    assert!(
        wait_for_path(&empty_rrd, Duration::from_secs(5)),
        "empty.rrd was not created at {empty_rrd:?}"
    );
    assert!(empty_rrd.is_file(), "empty.rrd path exists but is not a regular file");
    assert!(empty_rrd.metadata().unwrap().len() > 0, "empty.rrd file should not be zero bytes");

    // Validate the RRD schema via rrdtool info
    let info = rrdtool_info(&empty_rrd);
    assert!(info.contains("ds[empty].type"), "empty.rrd missing 'empty' data source");
    assert_rra_count(&info, 5);
    assert!(
        info.contains("rra[0].cf = \"AVERAGE\"") || info.contains("rra[0].cf = 'AVERAGE'"),
        "first RRA should be AVERAGE; info:\n{info}"
    );

    assert!(
        daemon.stop_and_wait(Duration::from_secs(5)),
        "daemon did not stop after SIGTERM"
    );
}

/// `test_dynamic_rrd_creation` (Python) — GAUGE variant
#[test]
fn daemon_creates_gauge_rrd_for_dynamic_metric() {
    assert_dynamic_rrd_type("GAUGE");
}

/// `test_dynamic_rrd_creation` (Python) — COUNTER variant
#[test]
fn daemon_creates_counter_rrd_for_dynamic_metric() {
    assert_dynamic_rrd_type("COUNTER");
}

/// `test_dynamic_rrd_creation` (Python) — DERIVE variant
#[test]
fn daemon_creates_derive_rrd_for_dynamic_metric() {
    assert_dynamic_rrd_type("DERIVE");
}

fn assert_dynamic_rrd_type(rrd_type: &str) {
    let metric_name = format!("temperature_{}", rrd_type.to_lowercase());
    let prefix = format!("rpimonitor-e2e-rrd-{}", rrd_type.to_lowercase());

    let dir = TempDir::new(&prefix);
    let datastore = dir.path().join("data");
    let webroot = dir.path().join("web");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(&datastore).expect("create datastore");

    let config = write_config(
        dir.path(),
        "rrd-dynamic.conf",
        &format!(
            "daemon.datastore={}\ndaemon.webroot={}\ndaemon.delay=1\ndaemon.readonly=0\ndaemon.noserver=1\n\
             dynamic.1.name={metric_name}\ndynamic.1.source=/bin/echo 42\ndynamic.1.regexp=([0-9]+)\ndynamic.1.rrd={rrd_type}\ndynamic.1.interval=1\n",
            datastore.display(),
            webroot.display(),
        ),
    );
    let pid_file = dir.path().join(format!("rpimonitor-rrd-{}.pid", rrd_type.to_lowercase()));
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
        "{rrd_type}: daemon launch failed: status={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let stat_dir = datastore.join("stat");
    assert!(
        wait_for_path(&stat_dir, Duration::from_secs(8)),
        "{rrd_type}: stat directory not created"
    );

    let rrd_file = stat_dir.join(format!("{metric_name}.rrd"));
    assert!(
        wait_for_path(&rrd_file, Duration::from_secs(8)),
        "{rrd_type}: {metric_name}.rrd not created; stat dir contents: {:?}",
        fs::read_dir(&stat_dir)
            .ok()
            .map(|e| e.filter_map(|e| e.ok()).map(|e| e.file_name()).collect::<Vec<_>>())
    );
    assert!(rrd_file.is_file());

    let info = rrdtool_info(&rrd_file);
    assert_ds_type(&info, &metric_name, rrd_type);
    assert_rra_count(&info, 5);

    assert!(
        daemon.stop_and_wait(Duration::from_secs(5)),
        "{rrd_type}: daemon did not stop after SIGTERM"
    );
}
