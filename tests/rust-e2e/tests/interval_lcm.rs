/// Black-box tests for maxinterval / LCM calculation.
///
/// Ported from `test_maxinterval_lcm.py` (PERL_AND_RUST).
///
/// The SUT function `Configuration::LeastCommonMultiple` computes the LCM of
/// successive pairs of intervals.  For intervals 12, 24, 30:
///   LCM(12, 24) = 24 ; LCM(24, 30) = 120  →  maxinterval = 120.
///
/// The distinguishing test case uses three intervals where LCM ≠ GCD so that
/// the wrong algorithm would produce a clearly wrong result:
///   GCD(12, 24, 30) = 6   ≠  LCM(12, 24, 30) = 120.
use std::ffi::OsString;

use rpimonitor_e2e::{write_config, run, TempDir};

/// `test_maxinterval_is_lcm_of_dynamic_intervals` (Python — PERL_AND_RUST)
#[test]
fn maxinterval_equals_lcm_of_all_dynamic_intervals() {
    let dir = TempDir::new("rpimonitor-e2e-lcm");
    let webroot = dir.path().join("web");
    let datastore = dir.path().join("data");
    std::fs::create_dir_all(&webroot).expect("create webroot");
    std::fs::create_dir_all(datastore.join("stat")).expect("create datastore/stat");

    let config = write_config(
        dir.path(),
        "lcm.conf",
        &format!(
            "daemon.delay=1\n\
             daemon.datastore={}\n\
             daemon.webroot={}\n\
             dynamic.1.name=cpu_temp\n\
             dynamic.1.source=/bin/echo 0\n\
             dynamic.1.regexp=(\\d+)\n\
             dynamic.1.interval=12\n\
             dynamic.2.name=memory_usage\n\
             dynamic.2.source=/bin/echo 0\n\
             dynamic.2.regexp=(\\d+)\n\
             dynamic.2.interval=24\n\
             dynamic.3.name=disk_space\n\
             dynamic.3.source=/bin/echo 0\n\
             dynamic.3.regexp=(\\d+)\n\
             dynamic.3.interval=30\n",
            datastore.display(),
            webroot.display(),
        ),
    );

    let output = run(
        &[OsString::from("-c"), config.into_os_string(), OsString::from("-s")],
        dir.path(),
    );

    // -s prints config then dies; exit 0, 2, or 255 are all valid.
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "unexpected exit code: {:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Primary assertion: maxinterval must equal the LCM, not the GCD.
    // LCM(12, 24, 30) = 120 ; GCD(12, 24, 30) = 6.
    assert!(
        stdout.contains("'maxinterval' => 120"),
        "maxinterval should equal LCM(12,24,30)=120; stdout:\n{stdout}"
    );

    // All three metrics must have been loaded.
    for metric in ["cpu_temp", "memory_usage", "disk_space"] {
        assert!(
            stdout.contains(metric),
            "metric '{metric}' not found in -s dump; stdout:\n{stdout}"
        );
    }
}

/// Boundary: a single interval must set maxinterval to itself.
#[test]
fn maxinterval_with_single_dynamic_metric_equals_that_interval() {
    let dir = TempDir::new("rpimonitor-e2e-lcm-single");
    let webroot = dir.path().join("web");
    let datastore = dir.path().join("data");
    std::fs::create_dir_all(&webroot).expect("create webroot");
    std::fs::create_dir_all(datastore.join("stat")).expect("create datastore/stat");

    let config = write_config(
        dir.path(),
        "lcm-single.conf",
        &format!(
            "daemon.delay=1\n\
             daemon.datastore={}\n\
             daemon.webroot={}\n\
             dynamic.1.name=only_metric\n\
             dynamic.1.source=/bin/echo 0\n\
             dynamic.1.regexp=(\\d+)\n\
             dynamic.1.interval=7\n",
            datastore.display(),
            webroot.display(),
        ),
    );

    let output = run(
        &[OsString::from("-c"), config.into_os_string(), OsString::from("-s")],
        dir.path(),
    );
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "unexpected exit code: {:?}",
        output.status.code()
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("'maxinterval' => 7"),
        "single interval 7 should produce maxinterval=7; stdout:\n{stdout}"
    );
}

/// Two coprime intervals: LCM = product, GCD = 1.
/// intervals 2 and 3: LCM=6, GCD=1 — the original partial coverage used
/// these but they don't distinguish LCM from simple multiplication.
/// Use 7 and 11 (coprime primes): LCM = 77.
#[test]
fn maxinterval_of_two_coprime_intervals_equals_their_product() {
    let dir = TempDir::new("rpimonitor-e2e-lcm-coprime");
    let webroot = dir.path().join("web");
    let datastore = dir.path().join("data");
    std::fs::create_dir_all(&webroot).expect("create webroot");
    std::fs::create_dir_all(datastore.join("stat")).expect("create datastore/stat");

    let config = write_config(
        dir.path(),
        "lcm-coprime.conf",
        &format!(
            "daemon.delay=1\n\
             daemon.datastore={}\n\
             daemon.webroot={}\n\
             dynamic.1.name=metric_a\n\
             dynamic.1.source=/bin/echo 0\n\
             dynamic.1.regexp=(\\d+)\n\
             dynamic.1.interval=7\n\
             dynamic.2.name=metric_b\n\
             dynamic.2.source=/bin/echo 0\n\
             dynamic.2.regexp=(\\d+)\n\
             dynamic.2.interval=11\n",
            datastore.display(),
            webroot.display(),
        ),
    );

    let output = run(
        &[OsString::from("-c"), config.into_os_string(), OsString::from("-s")],
        dir.path(),
    );
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "unexpected exit code: {:?}",
        output.status.code()
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // LCM(7,11) = 77
    assert!(
        stdout.contains("'maxinterval' => 77"),
        "LCM(7,11) should be 77; stdout:\n{stdout}"
    );
}
