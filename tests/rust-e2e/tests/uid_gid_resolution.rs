use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;
use std::time::{Duration, Instant};

use rpimonitor_e2e::{run, write_config, TempDir};

struct BackgroundDaemon {
    pid_file: PathBuf,
}

impl BackgroundDaemon {
    fn stop_and_wait(&self, timeout: Duration) -> bool {
        let Ok(contents) = fs::read_to_string(&self.pid_file) else {
            return false;
        };
        let pids: Vec<&str> = contents.split_whitespace().collect();
        if pids.is_empty() {
            return false;
        }

        for pid in &pids {
            let _ = Command::new("kill").args(["-TERM", pid]).status();
        }

        let deadline = Instant::now() + timeout;
        loop {
            let all_gone = pids.iter().all(|pid| {
                let output = Command::new("ps").args(["-o", "stat=", "-p", pid]).output();
                match output {
                    Ok(output) => {
                        let status = String::from_utf8_lossy(&output.stdout);
                        status.trim().is_empty() || status.trim_start().starts_with('Z')
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
        if let Ok(contents) = fs::read_to_string(&self.pid_file) {
            for pid in contents.split_whitespace() {
                let _ = Command::new("kill").args(["-TERM", pid]).status();
            }
        }
    }
}

fn id_output(option: &str) -> String {
    let output = Command::new("id")
        .arg(option)
        .output()
        .expect("run id to identify the test process");
    assert!(
        output.status.success(),
        "id {option} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("id output should be UTF-8")
        .trim()
        .to_owned()
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

#[test]
fn daemon_assigns_empty_rrd_to_configured_user_and_group() {
    if id_output("-u") == "0" {
        return;
    }

    let user = id_output("-un");
    let group = id_output("-gn");
    let expected_uid = id_output("-u")
        .parse::<u32>()
        .expect("process UID should be numeric");
    let expected_gid = id_output("-g")
        .parse::<u32>()
        .expect("process GID should be numeric");

    let directory = TempDir::new("rpimonitor-e2e-uid-gid");
    let datastore = directory.path().join("data");
    let webroot = directory.path().join("web");
    fs::create_dir_all(&datastore).expect("create isolated datastore");
    fs::create_dir_all(&webroot).expect("create isolated webroot");
    let config = write_config(
        directory.path(),
        "uid-gid.conf",
        &format!(
            "daemon.user={user}\ndaemon.group={group}\ndaemon.datastore={}\ndaemon.webroot={}\ndaemon.delay=1\ndaemon.readonly=0\ndaemon.noserver=1\n",
            datastore.display(),
            webroot.display(),
        ),
    );
    let pid_file = directory.path().join("rpimonitor-uid-gid.pid");
    let daemon = BackgroundDaemon {
        pid_file: pid_file.clone(),
    };

    let output: Output = run(
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
        "daemon launch failed: status={:?}; stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let empty_rrd = datastore.join("stat/empty.rrd");
    assert!(
        wait_for_file(&empty_rrd, Duration::from_secs(8)),
        "daemon did not create empty.rrd; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata = fs::metadata(&empty_rrd).expect("read empty.rrd metadata");
    assert_eq!(
        metadata.uid(),
        expected_uid,
        "empty.rrd should be owned by configured user {user}"
    );
    assert_eq!(
        metadata.gid(),
        expected_gid,
        "empty.rrd should belong to configured group {group}"
    );

    assert!(
        daemon.stop_and_wait(Duration::from_secs(5)),
        "daemon did not stop after SIGTERM"
    );
}
