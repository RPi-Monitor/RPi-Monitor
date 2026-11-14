use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::symlink;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use rpimonitor_e2e::{binary_path, run, run_program_with_stdin, write_config, TempDir};

struct Daemon {
    pid_file: std::path::PathBuf,
}

impl Drop for Daemon {
    fn drop(&mut self) {
        if let Ok(pid) = fs::read_to_string(&self.pid_file) {
            for pid in pid.split_whitespace() {
                let _ = Command::new("kill").args(["-TERM", pid]).status();
            }
        }
    }
}

#[test]
fn snmp_symlink_dispatches_pass_persist_ping_and_get() {
    let directory = TempDir::new("rpimonitor-e2e-snmp");
    let webroot = directory.path().join("web");
    let datastore = directory.path().join("data");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(datastore.join("stat")).expect("create datastore stat directory");

    let symlink_path = directory.path().join("rpimonitord-snmp");
    symlink(binary_path(), &symlink_path).expect("create SNMP entry-point symlink");
    let sharedmemkey = 1_000_000 + (std::process::id() % 100_000);
    let config = write_config(
        directory.path(),
        "snmp.conf",
        &format!(
            "daemon.noserver=1\ndaemon.readonly=0\ndaemon.delay=1\ndaemon.sharedmemkey={sharedmemkey}\ndaemon.datastore={}\ndaemon.webroot={}\ndynamic.1.name=fixture_metric\ndynamic.1.source=/bin/echo 17\ndynamic.1.regexp=(\\d+)\ndynamic.1.interval=1\nsnmpagent.enterpriseoid=42424\nsnmpagent.rpimonitoroid=77\nsnmp.hostname.type=string\nsnmp.hostname.id=1\n",
            datastore.display(),
            webroot.display()
        ),
    );
    let pid_file = directory.path().join("rpimonitor.pid");
    let daemon = Daemon {
        pid_file: pid_file.clone(),
    };
    let launch = run(
        &[
            OsString::from("-c"),
            config.clone().into_os_string(),
            OsString::from("-b"),
            pid_file.into_os_string(),
        ],
        directory.path(),
    );
    assert!(
        matches!(launch.status.code(), Some(0 | 2 | 255)),
        "could not launch shared-memory producer; status={:?}; stderr={:?}",
        launch.status.code(),
        String::from_utf8_lossy(&launch.stderr)
    );
    let deadline = Instant::now() + Duration::from_secs(8);
    while !webroot.join("dynamic.json").is_file() {
        assert!(
            Instant::now() < deadline,
            "daemon did not write dynamic.json to initialize shared status"
        );
        thread::sleep(Duration::from_millis(100));
    }

    let oid = ".1.3.6.1.4.1.42424.77.1";
    let input = format!("PING\nget\n{oid}\n");
    let output = run_program_with_stdin(
        &symlink_path,
        &[OsString::from("-c"), config.into_os_string()],
        directory.path(),
        input.as_bytes(),
    );

    assert_eq!(
        output.status.code(),
        Some(0),
        "SNMP pass_persist entry point should exit successfully; stdout={:?}; stderr={:?}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<_> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        4,
        "unexpected SNMP response framing: {stdout:?}"
    );
    assert_eq!(
        lines.first(),
        Some(&"PONG"),
        "unexpected SNMP ping response: {stdout:?}"
    );
    assert_eq!(
        lines[1], oid,
        "SNMP response should identify the requested OID"
    );
    assert_eq!(
        lines[2], "string",
        "hostname OID should use the string type"
    );
    assert!(
        !lines[3].is_empty(),
        "hostname OID should return a value; stdout={stdout:?}"
    );
    drop(daemon);
}
