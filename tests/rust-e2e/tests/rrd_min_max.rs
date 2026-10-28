/// Black-box tests for RRD data-source min/max bounds.
///
/// Ported from `test_rrd_min_max_values.py`.
/// The daemon creates an RRD with the configured min/max bounds; `rrdtool info`
/// is used to verify them without touching internal Perl state.
use std::ffi::OsString;
use std::fs;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use rpimonitor_e2e::{run, write_config, TempDir};

// ---------------------------------------------------------------------------
// Helpers (same pattern as rrd_creation.rs)
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

/// Run `rrdtool info <file>` and return its stdout.
fn rrdtool_info(rrd_path: &std::path::Path) -> String {
    let out = Command::new("rrdtool")
        .args(["info", &rrd_path.to_string_lossy()])
        .output()
        .expect("rrdtool info");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Extract the float value of a key like `ds[name].min = 0,0000000000e+00` from
/// `rrdtool info` output.  Returns `None` if the key is absent or the value is
/// `NaN` (which rrdtool prints as `NaN`).
fn parse_ds_float(info: &str, ds_name: &str, field: &str) -> Option<f64> {
    let key = format!("ds[{ds_name}].{field}");
    for line in info.lines() {
        if let Some(rest) = line.strip_prefix(&key) {
            let val = rest.trim_start_matches([' ', '=']).trim();
            // rrdtool may print NaN or the value with a locale-dependent decimal sep
            if val.eq_ignore_ascii_case("nan") {
                return None;
            }
            // Normalise locale decimal separator (comma → dot)
            let normalised = val.replace(',', ".");
            return normalised.parse().ok();
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Shared launcher
// ---------------------------------------------------------------------------

fn launch_daemon_with_rrd_bounds(
    dir: &std::path::Path,
    metric_name: &str,
    source_value: &str,
    min: &str,
    max: &str,
) -> (BackgroundDaemon, std::path::PathBuf) {
    let datastore = dir.join("data");
    let webroot = dir.join("web");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(&datastore).expect("create datastore");

    let config = write_config(
        dir,
        "rrd-bounds.conf",
        &format!(
            "daemon.datastore={}\ndaemon.webroot={}\ndaemon.delay=1\ndaemon.readonly=0\ndaemon.noserver=1\n\
             dynamic.1.name={metric_name}\ndynamic.1.source=/bin/echo {source_value}\n\
             dynamic.1.regexp=([0-9U.-]+)\ndynamic.1.postprocess=$1\n\
             dynamic.1.rrd=GAUGE\ndynamic.1.min={min}\ndynamic.1.max={max}\ndynamic.1.interval=1\n",
            datastore.display(),
            webroot.display(),
        ),
    );
    let pid_file = dir.join(format!("{metric_name}.pid"));
    let daemon = BackgroundDaemon { pid_file: pid_file.clone() };

    let output = run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-b"),
            pid_file.into_os_string(),
        ],
        dir,
    );
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "{metric_name}: daemon launch failed: status={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let rrd_file = datastore.join("stat").join(format!("{metric_name}.rrd"));
    (daemon, rrd_file)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// `test_rrd_min_max_values` — normal_values case
///
/// DS min=0 max=100, source echoes 50 → both bounds must be stored in the RRD.
#[test]
fn rrd_ds_bounds_are_stored_for_in_range_values() {
    let dir = TempDir::new("rpimonitor-e2e-rrd-bounds-normal");
    let (daemon, rrd_file) = launch_daemon_with_rrd_bounds(
        dir.path(), "normal_values", "50", "0", "100",
    );

    assert!(
        wait_for_path(&rrd_file, Duration::from_secs(8)),
        "normal_values.rrd was not created"
    );

    let info = rrdtool_info(&rrd_file);

    let min = parse_ds_float(&info, "normal_values", "min")
        .expect("ds[normal_values].min should be a number");
    let max = parse_ds_float(&info, "normal_values", "max")
        .expect("ds[normal_values].max should be a number");

    assert!(
        (min - 0.0_f64).abs() < 1e-6,
        "DS min should be 0.0, got {min}; info:\n{info}"
    );
    assert!(
        (max - 100.0_f64).abs() < 1e-6,
        "DS max should be 100.0, got {max}; info:\n{info}"
    );

    assert!(daemon.stop_and_wait(Duration::from_secs(5)), "daemon did not stop");
}

/// `test_rrd_min_max_values` — below_min case
///
/// DS min=0 max=100, source echoes -10 → out-of-range values are stored as
/// Unknown (NaN) in the DS.  The `ds[name].min` bound itself must still be 0.
#[test]
fn rrd_ds_min_bound_is_zero_even_when_source_is_below_range() {
    let dir = TempDir::new("rpimonitor-e2e-rrd-bounds-below");
    let (daemon, rrd_file) = launch_daemon_with_rrd_bounds(
        dir.path(), "below_min", "-10", "0", "100",
    );

    assert!(
        wait_for_path(&rrd_file, Duration::from_secs(8)),
        "below_min.rrd was not created"
    );

    let info = rrdtool_info(&rrd_file);

    let min = parse_ds_float(&info, "below_min", "min")
        .expect("ds[below_min].min should be a number");
    let max = parse_ds_float(&info, "below_min", "max")
        .expect("ds[below_min].max should be a number");

    assert!(
        (min - 0.0_f64).abs() < 1e-6,
        "DS min should be 0.0, got {min}"
    );
    assert!(
        (max - 100.0_f64).abs() < 1e-6,
        "DS max should be 100.0, got {max}"
    );

    assert!(daemon.stop_and_wait(Duration::from_secs(5)), "daemon did not stop");
}

/// `test_rrd_min_max_values` — above_max case
///
/// DS min=0 max=100, source echoes 150 → out-of-range. Bounds must be preserved.
#[test]
fn rrd_ds_max_bound_is_100_even_when_source_exceeds_range() {
    let dir = TempDir::new("rpimonitor-e2e-rrd-bounds-above");
    let (daemon, rrd_file) = launch_daemon_with_rrd_bounds(
        dir.path(), "above_max", "150", "0", "100",
    );

    assert!(
        wait_for_path(&rrd_file, Duration::from_secs(8)),
        "above_max.rrd was not created"
    );

    let info = rrdtool_info(&rrd_file);

    let min = parse_ds_float(&info, "above_max", "min")
        .expect("ds[above_max].min should be a number");
    let max = parse_ds_float(&info, "above_max", "max")
        .expect("ds[above_max].max should be a number");

    assert!(
        (min - 0.0_f64).abs() < 1e-6,
        "DS min should be 0.0, got {min}"
    );
    assert!(
        (max - 100.0_f64).abs() < 1e-6,
        "DS max should be 100.0, got {max}"
    );

    assert!(daemon.stop_and_wait(Duration::from_secs(5)), "daemon did not stop");
}

/// `test_rrd_min_max_values` — no explicit bounds (U:U defaults)
///
/// When no min/max are configured the DS should have NaN (Unknown) bounds,
/// meaning rrdtool accepts any numeric value.
#[test]
fn rrd_ds_has_unknown_bounds_when_no_min_max_configured() {
    let dir = TempDir::new("rpimonitor-e2e-rrd-bounds-unset");
    let datastore = dir.path().join("data");
    let webroot = dir.path().join("web");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(&datastore).expect("create datastore");

    // No dynamic.1.min / dynamic.1.max → defaults to U:U
    let config = write_config(
        dir.path(),
        "rrd-bounds-unset.conf",
        &format!(
            "daemon.datastore={}\ndaemon.webroot={}\ndaemon.delay=1\ndaemon.readonly=0\ndaemon.noserver=1\n\
             dynamic.1.name=unbounded_metric\ndynamic.1.source=/bin/echo 42\n\
             dynamic.1.regexp=([0-9]+)\ndynamic.1.rrd=GAUGE\ndynamic.1.interval=1\n",
            datastore.display(),
            webroot.display(),
        ),
    );
    let pid_file = dir.path().join("rpimonitor-bounds-unset.pid");
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
        "daemon launch failed: {:?}",
        output.status.code()
    );

    let rrd_file = datastore.join("stat").join("unbounded_metric.rrd");
    assert!(
        wait_for_path(&rrd_file, Duration::from_secs(8)),
        "unbounded_metric.rrd was not created"
    );

    let info = rrdtool_info(&rrd_file);

    // With U:U bounds, rrdtool info reports NaN for min and max
    assert!(
        parse_ds_float(&info, "unbounded_metric", "min").is_none(),
        "DS min should be NaN (Unknown) when not configured; info:\n{info}"
    );
    assert!(
        parse_ds_float(&info, "unbounded_metric", "max").is_none(),
        "DS max should be NaN (Unknown) when not configured; info:\n{info}"
    );

    assert!(daemon.stop_and_wait(Duration::from_secs(5)), "daemon did not stop");
}
