use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use rpimonitor_e2e::{run, write_config, TempDir};

struct BackgroundDaemon {
    pid_file: PathBuf,
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
        let pids = contents.split_whitespace().collect::<Vec<_>>();
        if pids.is_empty() {
            return false;
        }

        let deadline = Instant::now() + timeout;
        loop {
            let all_stopped = pids.iter().all(|pid| {
                let output = Command::new("ps").args(["-o", "stat=", "-p", pid]).output();
                match output {
                    Ok(output) => {
                        let state = String::from_utf8_lossy(&output.stdout);
                        state.trim().is_empty() || state.trim_start().starts_with('Z')
                    }
                    Err(_) => false,
                }
            });
            if all_stopped {
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

fn wait_for_file(path: &Path, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if path.is_file() {
            return true;
        }
        thread::sleep(Duration::from_millis(100));
    }
    path.is_file()
}

fn json_scalar(json: &str, key: &str) -> Option<String> {
    let key = format!("\"{key}\":");
    let rest = json.split_once(&key)?.1.trim_start();
    let value = rest.split([',', '}']).next()?.trim();
    Some(value.trim_matches('"').to_owned())
}

fn assert_json_scalar(json: &str, key: &str, expected: &str) {
    assert_eq!(
        json_scalar(json, key).as_deref(),
        Some(expected),
        "JSON field {key:?} should equal {expected:?}; JSON: {json}"
    );
}

#[test]
fn no_server_mode_writes_runtime_extracted_and_postprocessed_json() {
    let directory = TempDir::new("rpimonitor-e2e-noserver-runtime");
    let webroot = directory.path().join("web");
    let datastore = directory.path().join("data");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(datastore.join("stat")).expect("create RRD stat directory");

    let config = write_config(
        directory.path(),
        "noserver-runtime.conf",
        &format!(
            "daemon.noserver=1\ndaemon.readonly=0\ndaemon.delay=1\ndaemon.timeout=5\ndaemon.datastore={}\ndaemon.webroot={}\ndynamic.1.source=/bin/echo 21\ndynamic.1.regexp=(\\d+)\ndynamic.1.name=processed_metric\ndynamic.1.postprocess=$1*2\ndynamic.1.default=0\ndynamic.1.interval=1\n",
            datastore.display(),
            webroot.display()
        ),
    );
    let pid_file = directory.path().join("rpimonitor.pid");
    let daemon = BackgroundDaemon {
        pid_file: pid_file.clone(),
    };

    let output = run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-b"),
            pid_file.into_os_string(),
        ],
        directory.path(),
    );
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "daemon launch failed: status={:?}; stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let dynamic_json = webroot.join("dynamic.json");
    assert!(
        wait_for_file(&dynamic_json, Duration::from_secs(8)),
        "dynamic.json was not created; launch stderr={:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    let data = fs::read_to_string(&dynamic_json).expect("read generated dynamic.json");
    assert_json_scalar(&data, "processed_metric", "42");

    assert!(
        daemon.stop_and_wait(Duration::from_secs(5)),
        "daemon processes did not stop after SIGTERM"
    );
}

#[test]
fn runtime_extraction_handles_multiple_capture_groups_and_postprocessing() {
    let directory = TempDir::new("rpimonitor-e2e-noserver-multiple-groups");
    let webroot = directory.path().join("web");
    let datastore = directory.path().join("data");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(datastore.join("stat")).expect("create RRD stat directory");

    let config = write_config(
        directory.path(),
        "noserver-multiple-groups.conf",
        &format!(
            "daemon.noserver=1\ndaemon.readonly=0\ndaemon.delay=1\ndaemon.timeout=5\ndaemon.datastore={}\ndaemon.webroot={}\ndynamic.1.source=/bin/echo 21 9\ndynamic.1.regexp=(\\d+)\\s+(\\d+)\ndynamic.1.name=first_metric,second_metric\ndynamic.1.postprocess=$1*2,$1+3\ndynamic.1.default=0,0\ndynamic.1.interval=1\n",
            datastore.display(),
            webroot.display()
        ),
    );
    let pid_file = directory.path().join("rpimonitor-multiple-groups.pid");
    let daemon = BackgroundDaemon { pid_file: pid_file.clone() };
    let output = run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-b"),
            pid_file.into_os_string(),
        ],
        directory.path(),
    );
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "daemon launch failed: status={:?}; stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let dynamic_json = webroot.join("dynamic.json");
    assert!(
        wait_for_file(&dynamic_json, Duration::from_secs(8)),
        "dynamic.json was not created; launch stderr={:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let data = fs::read_to_string(&dynamic_json).expect("read generated dynamic.json");
    assert_json_scalar(&data, "first_metric", "42");
    assert_json_scalar(&data, "second_metric", "12");
    assert!(
        daemon.stop_and_wait(Duration::from_secs(5)),
        "daemon processes did not stop after SIGTERM"
    );
}

#[test]
fn no_match_uses_configured_default_in_dynamic_json() {
    let directory = TempDir::new("rpimonitor-e2e-noserver-no-match");
    let webroot = directory.path().join("web");
    let datastore = directory.path().join("data");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(datastore.join("stat")).expect("create RRD stat directory");

    let config = write_config(
        directory.path(),
        "noserver-no-match.conf",
        &format!(
            "daemon.noserver=1\ndaemon.readonly=0\ndaemon.delay=1\ndaemon.timeout=5\ndaemon.datastore={}\ndaemon.webroot={}\ndynamic.1.source=/bin/echo no-numeric-value\ndynamic.1.regexp=(\\d+)\ndynamic.1.name=fallback_metric\ndynamic.1.postprocess=$1*2\ndynamic.1.default=7\ndynamic.1.interval=1\n",
            datastore.display(),
            webroot.display()
        ),
    );
    let pid_file = directory.path().join("rpimonitor-no-match.pid");
    let daemon = BackgroundDaemon {
        pid_file: pid_file.clone(),
    };

    let output = run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-b"),
            pid_file.into_os_string(),
        ],
        directory.path(),
    );
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "daemon launch failed: status={:?}; stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let dynamic_json = webroot.join("dynamic.json");
    assert!(
        wait_for_file(&dynamic_json, Duration::from_secs(8)),
        "dynamic.json was not created; launch stderr={:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    let data = fs::read_to_string(&dynamic_json).expect("read generated dynamic.json");
    assert_json_scalar(&data, "fallback_metric", "7");

    assert!(
        daemon.stop_and_wait(Duration::from_secs(5)),
        "daemon processes did not stop after SIGTERM"
    );
}

#[test]
fn runtime_command_expands_previously_extracted_data_values() {
    let directory = TempDir::new("rpimonitor-e2e-noserver-data-expansion");
    let webroot = directory.path().join("web");
    let datastore = directory.path().join("data");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(datastore.join("stat")).expect("create RRD stat directory");

    let config = write_config(
        directory.path(),
        "noserver-data-expansion.conf",
        &format!(
            "daemon.noserver=1\ndaemon.readonly=0\ndaemon.delay=1\ndaemon.timeout=5\ndaemon.datastore={}\ndaemon.webroot={}\ndynamic.1.source=/bin/echo 23\ndynamic.1.regexp=(\\d+)\ndynamic.1.name=base_metric\ndynamic.1.default=0\ndynamic.1.interval=1\ndynamic.2.source=/bin/echo data.base_metric\ndynamic.2.regexp=(\\d+)\ndynamic.2.name=expanded_metric\ndynamic.2.default=0\ndynamic.2.interval=1\n",
            datastore.display(),
            webroot.display()
        ),
    );
    let pid_file = directory.path().join("rpimonitor-data-expansion.pid");
    let daemon = BackgroundDaemon {
        pid_file: pid_file.clone(),
    };
    let output = run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-b"),
            pid_file.into_os_string(),
        ],
        directory.path(),
    );
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "daemon launch failed: status={:?}; stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let dynamic_json = webroot.join("dynamic.json");
    assert!(
        wait_for_file(&dynamic_json, Duration::from_secs(8)),
        "dynamic.json was not created; launch stderr={:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let data = fs::read_to_string(&dynamic_json).expect("read generated dynamic.json");
    assert_json_scalar(&data, "base_metric", "23");
    assert_json_scalar(&data, "expanded_metric", "23");
    assert!(
        daemon.stop_and_wait(Duration::from_secs(5)),
        "daemon processes did not stop after SIGTERM"
    );
}

#[test]
fn no_server_mode_writes_static_values_extracted_at_startup() {
    let directory = TempDir::new("rpimonitor-e2e-noserver-static");
    let webroot = directory.path().join("web");
    let datastore = directory.path().join("data");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(datastore.join("stat")).expect("create RRD stat directory");

    let config = write_config(
        directory.path(),
        "noserver-static.conf",
        &format!(
            "daemon.noserver=1\ndaemon.readonly=0\ndaemon.delay=1\ndaemon.timeout=5\ndaemon.datastore={}\ndaemon.webroot={}\nstatic.1.source=/bin/echo 17\nstatic.1.regexp=(\\d+)\nstatic.1.name=startup_metric\nstatic.1.postprocess=$1*3\nstatic.1.default=0\n",
            datastore.display(),
            webroot.display()
        ),
    );
    let pid_file = directory.path().join("rpimonitor-static.pid");
    let daemon = BackgroundDaemon {
        pid_file: pid_file.clone(),
    };
    let output = run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-b"),
            pid_file.into_os_string(),
        ],
        directory.path(),
    );
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "daemon launch failed: status={:?}; stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let static_json = webroot.join("static.json");
    assert!(
        wait_for_file(&static_json, Duration::from_secs(8)),
        "static.json was not created; launch stderr={:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let data = fs::read_to_string(&static_json).expect("read generated static.json");
    assert_json_scalar(&data, "startup_metric", "51");
    assert!(
        daemon.stop_and_wait(Duration::from_secs(5)),
        "daemon processes did not stop after SIGTERM"
    );
}

#[test]
fn no_server_mode_writes_configured_companion_json_files() {
    let directory = TempDir::new("rpimonitor-e2e-noserver-companion-json");
    let webroot = directory.path().join("web");
    let datastore = directory.path().join("data");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(datastore.join("stat")).expect("create RRD stat directory");

    let config = write_config(
        directory.path(),
        "noserver-companion-json.conf",
        &format!(
            "daemon.noserver=1\ndaemon.readonly=0\ndaemon.delay=1\ndaemon.timeout=5\ndaemon.datastore={}\ndaemon.webroot={}\nweb.status.1.title=System Status\nweb.status.1.content=Status body\nweb.statistics.1.title=Trend Statistics\nweb.addons.1.title=Test Addon\nweb.page.pagetitle=Test Dashboard\n",
            datastore.display(),
            webroot.display()
        ),
    );
    let pid_file = directory.path().join("rpimonitor-companion-json.pid");
    let daemon = BackgroundDaemon {
        pid_file: pid_file.clone(),
    };
    let output = run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-b"),
            pid_file.into_os_string(),
        ],
        directory.path(),
    );
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "daemon launch failed: status={:?}; stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let version_json = webroot.join("version.json");
    assert!(
        wait_for_file(&version_json, Duration::from_secs(8)),
        "version.json was not created; launch stderr={:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    for filename in [
        "status.json",
        "page.json",
        "statistics.json",
        "friends.json",
        "menu.json",
        "addons.json",
        "version.json",
    ] {
        assert!(
            webroot.join(filename).is_file(),
            "expected no-server mode to create {filename}"
        );
    }

    let status = fs::read_to_string(webroot.join("status.json")).expect("read status.json");
    assert!(
        status.contains("System Status") && status.contains("Status body"),
        "status.json should contain configured status fields: {status}"
    );
    let page = fs::read_to_string(webroot.join("page.json")).expect("read page.json");
    assert!(
        page.contains("Test Dashboard"),
        "page.json should contain configured page title: {page}"
    );
    let statistics =
        fs::read_to_string(webroot.join("statistics.json")).expect("read statistics.json");
    assert!(
        statistics.contains("Trend Statistics"),
        "statistics.json should contain configured statistics: {statistics}"
    );
    let addons = fs::read_to_string(webroot.join("addons.json")).expect("read addons.json");
    assert!(
        addons.contains("Test Addon"),
        "addons.json should contain configured addon: {addons}"
    );
    let menu = fs::read_to_string(webroot.join("menu.json")).expect("read menu.json");
    assert!(
        menu.contains("System Status")
            && menu.contains("Trend Statistics")
            && menu.contains("Test Addon"),
        "menu.json should expose configured navigation entries: {menu}"
    );
    let version = fs::read_to_string(&version_json).expect("read version.json");
    assert!(
        json_scalar(&version, "version").is_some_and(|value| !value.is_empty()),
        "version.json should contain a non-empty version field: {version}"
    );

    assert!(
        daemon.stop_and_wait(Duration::from_secs(5)),
        "daemon processes did not stop after SIGTERM"
    );
}

#[test]
fn runtime_extraction_reads_flat_files_and_uses_last_matching_line() {
    let directory = TempDir::new("rpimonitor-e2e-noserver-flat-file");
    let webroot = directory.path().join("web");
    let datastore = directory.path().join("data");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(datastore.join("stat")).expect("create RRD stat directory");

    let source_file = directory.path().join("metric.txt");
    fs::write(&source_file, "no metric here\nvalue=11\nstill no metric\nvalue=29\n")
        .expect("write flat-file source");

    let config = write_config(
        directory.path(),
        "noserver-flat-file.conf",
        &format!(
            "daemon.noserver=1\ndaemon.readonly=0\ndaemon.delay=1\ndaemon.timeout=5\ndaemon.datastore={}\ndaemon.webroot={}\ndynamic.1.source={}\ndynamic.1.regexp=value=(\\d+)\ndynamic.1.name=last_match\ndynamic.1.default=0\ndynamic.1.interval=1\n",
            datastore.display(),
            webroot.display(),
            source_file.display()
        ),
    );
    let pid_file = directory.path().join("rpimonitor-flat-file.pid");
    let daemon = BackgroundDaemon {
        pid_file: pid_file.clone(),
    };
    let output = run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-b"),
            pid_file.into_os_string(),
        ],
        directory.path(),
    );
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "daemon launch failed: status={:?}; stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let dynamic_json = webroot.join("dynamic.json");
    assert!(
        wait_for_file(&dynamic_json, Duration::from_secs(8)),
        "dynamic.json was not created; launch stderr={:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let data = fs::read_to_string(&dynamic_json).expect("read generated dynamic.json");
    assert_json_scalar(&data, "last_match", "29");
    assert!(
        daemon.stop_and_wait(Duration::from_secs(5)),
        "daemon processes did not stop after SIGTERM"
    );
}

#[test]
fn read_only_no_server_mode_does_not_write_json_files() {
    let directory = TempDir::new("rpimonitor-e2e-noserver-readonly");
    let webroot = directory.path().join("web");
    let datastore = directory.path().join("data");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(datastore.join("stat")).expect("create RRD stat directory");

    let config = write_config(
        directory.path(),
        "noserver-readonly.conf",
        &format!(
            "daemon.noserver=1\ndaemon.readonly=1\ndaemon.delay=1\ndaemon.timeout=5\ndaemon.datastore={}\ndaemon.webroot={}\ndynamic.1.source=/bin/echo 31\ndynamic.1.regexp=(\\d+)\ndynamic.1.name=readonly_metric\ndynamic.1.default=0\ndynamic.1.interval=1\n",
            datastore.display(),
            webroot.display()
        ),
    );
    let pid_file = directory.path().join("rpimonitor-readonly.pid");
    let daemon = BackgroundDaemon {
        pid_file: pid_file.clone(),
    };
    let output = run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-b"),
            pid_file.into_os_string(),
        ],
        directory.path(),
    );
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "daemon launch failed: status={:?}; stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    thread::sleep(Duration::from_millis(1500));
    for filename in [
        "static.json",
        "dynamic.json",
        "status.json",
        "page.json",
        "statistics.json",
        "friends.json",
        "menu.json",
        "addons.json",
        "version.json",
    ] {
        assert!(
            !webroot.join(filename).exists(),
            "read-only no-server mode must not write {filename}"
        );
    }

    assert!(
        daemon.stop_and_wait(Duration::from_secs(5)),
        "daemon processes did not stop after SIGTERM"
    );
}


#[test]
fn runtime_timeout_kills_slow_source_and_continues_processing() {
    let directory = TempDir::new("rpimonitor-e2e-noserver-timeout");
    let webroot = directory.path().join("web");
    let datastore = directory.path().join("data");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(datastore.join("stat")).expect("create RRD stat directory");

    let config = write_config(
        directory.path(),
        "noserver-timeout.conf",
        &format!(
            "daemon.noserver=1\ndaemon.readonly=0\ndaemon.delay=1\ndaemon.timeout=1\ndaemon.datastore={}\ndaemon.webroot={}\ndynamic.1.source=/bin/sh -c 'exec /bin/sleep 10'\ndynamic.1.regexp=(\\d+)\ndynamic.1.name=slow_metric\ndynamic.1.default=0\ndynamic.1.interval=1\ndynamic.2.source=/bin/echo 37\ndynamic.2.regexp=(\\d+)\ndynamic.2.name=healthy_metric\ndynamic.2.default=0\ndynamic.2.interval=1\n",
            datastore.display(),
            webroot.display()
        ),
    );
    let pid_file = directory.path().join("rpimonitor-timeout.pid");
    let daemon = BackgroundDaemon {
        pid_file: pid_file.clone(),
    };
    let output = run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-b"),
            pid_file.into_os_string(),
        ],
        directory.path(),
    );
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "daemon launch failed: status={:?}; stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let dynamic_json = webroot.join("dynamic.json");
    assert!(
        wait_for_file(&dynamic_json, Duration::from_secs(8)),
        "dynamic.json was not created after the source timeout; launch stderr={:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let data = fs::read_to_string(&dynamic_json).expect("read generated dynamic.json");
    assert_json_scalar(
        &data,
        "healthy_metric",
        "37",
    );

    assert!(
        daemon.stop_and_wait(Duration::from_secs(5)),
        "daemon processes did not stop after SIGTERM"
    );
}


#[test]
fn readonly_web_server_serves_static_and_dynamic_json_over_http() {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};

    let directory = TempDir::new("rpimonitor-e2e-http-json");
    let datastore = directory.path().join("data");
    fs::create_dir_all(datastore.join("stat")).expect("create RRD stat directory");

    // Pick an available loopback port, then release it before starting the daemon.
    let probe = TcpListener::bind(("127.0.0.1", 0)).expect("reserve an ephemeral port");
    let port = probe.local_addr().expect("read ephemeral port").port();
    drop(probe);

    let sharedmemkey = 300000 + (std::process::id() % 100000) as u32;
    let config = write_config(
        directory.path(),
        "http-json.conf",
        &format!(
            "daemon.addr=127.0.0.1\ndaemon.port={port}\ndaemon.readonly=1\ndaemon.delay=1\ndaemon.sharedmemkey={sharedmemkey}\ndaemon.datastore={}\ndynamic.1.source=/bin/echo 43\ndynamic.1.regexp=(\\d+)\ndynamic.1.name=http_metric\ndynamic.1.default=0\ndynamic.1.interval=1\n",
            datastore.display()
        ),
    );
    let pid_file = directory.path().join("rpimonitor-http.pid");
    let daemon = BackgroundDaemon {
        pid_file: pid_file.clone(),
    };
    let output = run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-b"),
            pid_file.into_os_string(),
        ],
        directory.path(),
    );
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "daemon launch failed: status={:?}; stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    fn get(path: &str, port: u16) -> String {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            match TcpStream::connect(("127.0.0.1", port)) {
                Ok(mut stream) => {
                    stream
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .expect("set HTTP read timeout");
                    write!(stream, "GET {path} HTTP/1.0\r\nHost: 127.0.0.1\r\n\r\n")
                        .expect("write HTTP request");
                    let mut response = String::new();
                    stream
                        .read_to_string(&mut response)
                        .expect("read HTTP response");
                    return response;
                }
                Err(error) if Instant::now() < deadline => {
                    let _ = error;
                    thread::sleep(Duration::from_millis(100));
                }
                Err(error) => panic!("HTTP server did not accept {path}: {error}"),
            }
        }
    }

    let static_response = get("/static.json", port);
    assert!(
        static_response.starts_with("HTTP/1.1 200") || static_response.starts_with("HTTP/1.0 200"),
        "static.json should return HTTP 200; response={static_response:?}"
    );
    assert!(
        static_response.contains("hostname"),
        "static.json should contain static monitor data; response={static_response:?}"
    );

    let dynamic_response = get("/dynamic.json", port);
    assert!(
        dynamic_response.starts_with("HTTP/1.1 200") || dynamic_response.starts_with("HTTP/1.0 200"),
        "dynamic.json should return HTTP 200; response={dynamic_response:?}"
    );
    assert!(
        dynamic_response.contains("http_metric") && dynamic_response.contains("43"),
        "dynamic.json should expose the configured dynamic metric; response={dynamic_response:?}"
    );

    assert!(
        daemon.stop_and_wait(Duration::from_secs(5)),
        "daemon processes did not stop after SIGTERM"
    );
}


#[test]
fn readonly_web_server_rejects_unknown_http_path() {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};

    let directory = TempDir::new("rpimonitor-e2e-http-404");
    let datastore = directory.path().join("data");
    fs::create_dir_all(datastore.join("stat")).expect("create RRD stat directory");

    let probe = TcpListener::bind(("127.0.0.1", 0)).expect("reserve an ephemeral port");
    let port = probe.local_addr().expect("read ephemeral port").port();
    drop(probe);

    let sharedmemkey = 400000 + (std::process::id() % 100000) as u32;
    let config = write_config(
        directory.path(),
        "http-404.conf",
        &format!(
            "daemon.addr=127.0.0.1\ndaemon.port={port}\ndaemon.readonly=1\ndaemon.delay=1\ndaemon.sharedmemkey={sharedmemkey}\ndaemon.datastore={}\n",
            datastore.display()
        ),
    );
    let pid_file = directory.path().join("rpimonitor-http-404.pid");
    let daemon = BackgroundDaemon { pid_file: pid_file.clone() };
    let output = run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-b"),
            pid_file.into_os_string(),
        ],
        directory.path(),
    );
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "daemon launch failed: status={:?}; stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let response = {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            match TcpStream::connect(("127.0.0.1", port)) {
                Ok(mut stream) => {
                    stream.set_read_timeout(Some(Duration::from_secs(2)))
                        .expect("set HTTP read timeout");
                    write!(stream, "GET /not-a-real-endpoint HTTP/1.0\r\nHost: 127.0.0.1\r\n\r\n")
                        .expect("write HTTP request");
                    let mut response = String::new();
                    stream.read_to_string(&mut response).expect("read HTTP response");
                    break response;
                }
                Err(error) if Instant::now() < deadline => {
                    let _ = error;
                    thread::sleep(Duration::from_millis(100));
                }
                Err(error) => panic!("HTTP server did not start accepting requests: {error}"),
            }
        }
    };

    assert!(
        response.starts_with("HTTP/1.1 404") || response.starts_with("HTTP/1.0 404"),
        "unknown endpoint should return HTTP 404; response={response:?}"
    );
    assert!(
        daemon.stop_and_wait(Duration::from_secs(5)),
        "daemon processes did not stop after SIGTERM"
    );
}


#[test]
fn web_server_serves_existing_static_file_from_webroot() {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};

    let directory = TempDir::new("rpimonitor-e2e-http-file");
    let webroot = directory.path().join("web");
    let datastore = directory.path().join("data");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(datastore.join("stat")).expect("create RRD stat directory");
    let marker = b"rpimonitor-static-file-e2e";
    fs::write(webroot.join("favicon.ico"), marker).expect("write fixture file");

    let probe = TcpListener::bind(("127.0.0.1", 0)).expect("reserve an ephemeral port");
    let port = probe.local_addr().expect("read ephemeral port").port();
    drop(probe);

    let config = write_config(
        directory.path(),
        "http-file.conf",
        &format!(
            "daemon.addr=127.0.0.1\ndaemon.port={port}\ndaemon.noserver=0\ndaemon.readonly=0\ndaemon.delay=1\ndaemon.timeout=5\ndaemon.datastore={}\ndaemon.webroot={}\n",
            datastore.display(),
            webroot.display()
        ),
    );
    let pid_file = directory.path().join("rpimonitor-http-file.pid");
    let daemon = BackgroundDaemon { pid_file: pid_file.clone() };
    let output = run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-b"),
            pid_file.into_os_string(),
        ],
        directory.path(),
    );
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "daemon launch failed: status={:?}; stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let response = {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            match TcpStream::connect(("127.0.0.1", port)) {
                Ok(mut stream) => {
                    stream.set_read_timeout(Some(Duration::from_secs(2)))
                        .expect("set HTTP read timeout");
                    write!(stream, "GET /favicon.ico HTTP/1.0\r\nHost: 127.0.0.1\r\n\r\n")
                        .expect("write HTTP request");
                    let mut response = Vec::new();
                    stream.read_to_end(&mut response).expect("read HTTP response");
                    break response;
                }
                Err(error) if Instant::now() < deadline => {
                    let _ = error;
                    thread::sleep(Duration::from_millis(100));
                }
                Err(error) => panic!("HTTP server did not start accepting requests: {error}"),
            }
        }
    };

    let response_text = String::from_utf8_lossy(&response);
    assert!(
        response_text.starts_with("HTTP/1.1 200") || response_text.starts_with("HTTP/1.0 200"),
        "existing favicon should return HTTP 200; response={response_text:?}"
    );
    assert!(
        response.windows(marker.len()).any(|window| window == marker),
        "HTTP response should contain the fixture file bytes; response={response_text:?}"
    );

    assert!(
        daemon.stop_and_wait(Duration::from_secs(5)),
        "daemon processes did not stop after SIGTERM"
    );
}
