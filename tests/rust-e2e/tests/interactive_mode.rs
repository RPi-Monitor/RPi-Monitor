use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use rpimonitor_e2e::{run_with_stdin, write_config, TempDir};

static INTERACTIVE_LOCK: Mutex<()> = Mutex::new(());

struct DefaultConfigFixture {
    directory_created: bool,
    data_path: PathBuf,
    data_backup: Option<Vec<u8>>,
    daemon_path: PathBuf,
    daemon_backup: Option<Vec<u8>>,
    sudo_restore: bool,
    sudo_directory: Option<PathBuf>,
}

fn read_with_sudo(path: &Path) -> Option<Vec<u8>> {
    let output = Command::new("sudo")
        .args(["-n", "cat"])
        .arg(path)
        .output()
        .ok()?;
    output.status.success().then(|| output.stdout)
}

fn run_sudo_copy(source: &Path, destination: &Path) -> bool {
    Command::new("sudo")
        .arg("-n")
        .arg("cp")
        .arg("-a")
        .arg(source)
        .arg(destination)
        .status()
        .is_ok_and(|status| status.success())
}

impl DefaultConfigFixture {
    fn install() -> Option<Self> {
        let output = Command::new("id")
            .arg("-u")
            .output()
            .expect("run id to determine fixture privileges");
        let directory = Path::new("/etc/rpimonitor");
        if output.status.success() && String::from_utf8_lossy(&output.stdout).trim() == "0" {
            let directory_created = !directory.is_dir();
            if fs::create_dir_all(directory).is_err() {
                return None;
            }

            let data_path = directory.join("data.conf");
            let data_backup = fs::read(&data_path).ok();
            let daemon_path = directory.join("daemon.conf");
            let daemon_backup = fs::read(&daemon_path).ok();

            if fs::write(
                &data_path,
                "dynamic.1.name=existing_metric\ndynamic.1.source=/bin/echo 1\ndynamic.1.regexp=(\\d+)\\n",
            )
            .is_err()
            {
                return None;
            }
            if fs::write(&daemon_path, "daemon.readonly=1\n").is_err() {
                return None;
            }

            return Some(Self {
                directory_created,
                data_path,
                data_backup,
                daemon_path,
                daemon_backup,
                sudo_restore: false,
                sudo_directory: None,
            });
        }

        let sudo = Command::new("sudo")
            .args(["-n", "true"])
            .status()
            .ok()?;
        if !sudo.success() {
            return None;
        }

        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_nanos();
        let sudo_directory =
            std::env::temp_dir().join(format!("rpimonitor-config-fixture-{nonce}"));
        fs::create_dir_all(&sudo_directory).ok()?;

        let directory_created = !directory.is_dir();
        let mkdir = Command::new("sudo")
            .args(["-n", "mkdir", "-p", "/etc/rpimonitor"])
            .status()
            .ok()?;
        if !mkdir.success() {
            let _ = fs::remove_dir_all(&sudo_directory);
            return None;
        }

        let data_path = directory.join("data.conf");
        let daemon_path = directory.join("daemon.conf");
        let data_backup = read_with_sudo(&data_path);
        let daemon_backup = read_with_sudo(&daemon_path);

        let data_fixture = sudo_directory.join("data.conf");
        let daemon_fixture = sudo_directory.join("daemon.conf");
        fs::write(
            &data_fixture,
            "dynamic.1.name=existing_metric\ndynamic.1.source=/bin/echo 1\ndynamic.1.regexp=(\\d+)\\n",
        )
        .ok()?;
        fs::write(&daemon_fixture, "daemon.readonly=1\n").ok()?;

        if !run_sudo_copy(&data_fixture, &data_path)
            || !run_sudo_copy(&daemon_fixture, &daemon_path)
        {
            let _ = fs::remove_dir_all(&sudo_directory);
            return None;
        }

        Some(Self {
            directory_created,
            data_path,
            data_backup,
            daemon_path,
            daemon_backup,
            sudo_restore: true,
            sudo_directory: Some(sudo_directory),
        })
    }
}

impl Drop for DefaultConfigFixture {
    fn drop(&mut self) {
        if self.sudo_restore {
            if let Some(directory) = &self.sudo_directory {
                let data_backup = directory.join("data.conf.backup");
                let daemon_backup = directory.join("daemon.conf.backup");
                match &self.data_backup {
                    Some(content) => {
                        let _ = fs::write(&data_backup, content);
                        let _ = run_sudo_copy(&data_backup, &self.data_path);
                    }
                    None => {
                        let _ = Command::new("sudo")
                            .args(["-n", "rm", "-f"])
                            .arg(&self.data_path)
                            .status();
                    }
                }
                match &self.daemon_backup {
                    Some(content) => {
                        let _ = fs::write(&daemon_backup, content);
                        let _ = run_sudo_copy(&daemon_backup, &self.daemon_path);
                    }
                    None => {
                        let _ = Command::new("sudo")
                            .args(["-n", "rm", "-f"])
                            .arg(&self.daemon_path)
                            .status();
                    }
                }
                if self.directory_created {
                    let _ = Command::new("sudo")
                        .args(["-n", "rmdir", "/etc/rpimonitor"])
                        .status();
                }
                let _ = fs::remove_dir_all(directory);
            }
            return;
        }

        match &self.data_backup {
            Some(content) => {
                let _ = fs::write(&self.data_path, content);
            }
            None => {
                let _ = fs::remove_file(&self.data_path);
            }
        }
        match &self.daemon_backup {
            Some(content) => {
                let _ = fs::write(&self.daemon_path, content);
            }
            None => {
                let _ = fs::remove_file(&self.daemon_path);
            }
        }
        if self.directory_created {
            let _ = fs::remove_dir("/etc/rpimonitor");
        }
    }
}

#[test]
fn interactive_helper_processes_scripted_input_before_default_config_error() {
    let _interactive_lock = INTERACTIVE_LOCK.lock().expect("lock interactive tests");
    let directory = TempDir::new("rpimonitor-e2e-interactive");
    let webroot = directory.path().join("web");
    let datastore = directory.path().join("data");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(datastore.join("stat")).expect("create datastore stat directory");

    let source = directory.path().join("metrics.txt");
    fs::write(&source, "load=42\n").expect("write source fixture");
    let config = write_config(
        directory.path(),
        "interactive.conf",
        &format!(
            "daemon.readonly=1\ndaemon.datastore={}\ndaemon.webroot={}\ndynamic.1.name=existing_metric\ndynamic.1.source=/bin/echo 1\ndynamic.1.regexp=(\\d+)\n",
            datastore.display(),
            webroot.display()
        ),
    );
    let input = format!("\n{}\ny\n(\\d+)\ny\n$1 * 2\ny\nn\n", source.display());
    let output = run_with_stdin(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-i"),
        ],
        directory.path(),
        input.as_bytes(),
    );

    assert!(
        matches!(output.status.code(), Some(2 | 255)),
        "interactive helper should expose the missing default-config error; status={:?}; \
         stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in [
        "Applying the Regular Expression '(\\d+)'",
        "$1 = 42",
        "After applying the post precessing formule, the result is:",
        "\n84\n",
        "Is is a static data? [Yes]",
    ] {
        assert!(
            stdout.contains(expected),
            "interactive output missing {expected:?}; stdout={stdout:?}"
        );
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Error while openning configuration file \"/etc/rpimonitor/data.conf\""),
        "interactive helper should report the missing default configuration; stderr={stderr:?}"
    );
}

#[test]
fn interactive_helper_validates_executable_source_and_output() {
    let _interactive_lock = INTERACTIVE_LOCK.lock().expect("lock interactive tests");
    let directory = TempDir::new("rpimonitor-e2e-interactive-command");
    let webroot = directory.path().join("web");
    let datastore = directory.path().join("data");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(datastore.join("stat")).expect("create datastore stat directory");

    let source = directory.path().join("metric-source");
    fs::write(&source, "#!/bin/sh\nprintf 'load=42\\n'\n").expect("write source command");
    let mut permissions = fs::metadata(&source)
        .expect("read source permissions")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&source, permissions).expect("make source executable");
    let config = write_config(
        directory.path(),
        "interactive-command.conf",
        &format!(
            "daemon.readonly=1\ndaemon.datastore={}\ndaemon.webroot={}\n",
            datastore.display(),
            webroot.display()
        ),
    );
    let input = format!(
        "\n{}\ny\ny\nload=(\\d+)\ny\n$1 * 2\ny\ny\n",
        source.display()
    );
    let output = run_with_stdin(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("--interactive"),
        ],
        directory.path(),
        input.as_bytes(),
    );

    assert!(
        matches!(output.status.code(), Some(2 | 255)),
        "interactive helper should reach default-config load; status={:?}; stdout={:?}; \
         stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in [
        "Find executable file:",
        "Is that the expected output? Yes/No [Yes]:",
        "load=42",
        "$1 = 42",
        "After applying the post precessing formule, the result is:",
        "\n84\n",
        "Is is a static data? [Yes]",
    ] {
        assert!(
            stdout.contains(expected),
            "interactive output missing {expected:?}; stdout={stdout:?}"
        );
    }
}

#[test]
fn interactive_helper_reprompts_when_executable_source_is_rejected() {
    let _interactive_lock = INTERACTIVE_LOCK.lock().expect("lock interactive tests");
    let directory = TempDir::new("rpimonitor-e2e-interactive-reject");
    let webroot = directory.path().join("web");
    let datastore = directory.path().join("data");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(datastore.join("stat")).expect("create datastore stat directory");

    let source = directory.path().join("metrics.txt");
    fs::write(&source, "load=42\n").expect("write flat-file source");
    let config = write_config(
        directory.path(),
        "interactive-reject.conf",
        &format!(
            "daemon.readonly=1\ndaemon.datastore={}\ndaemon.webroot={}\n",
            datastore.display(),
            webroot.display()
        ),
    );
    let input = format!(
        "\n/bin/echo rejected\nn\n{}\ny\n(\\d+)\ny\n$1 * 2\ny\ny\n",
        source.display()
    );
    let output = run_with_stdin(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-i"),
        ],
        directory.path(),
        input.as_bytes(),
    );

    assert!(
        matches!(output.status.code(), Some(2 | 255)),
        "interactive helper should continue to default-config load; status={:?}; \
         stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Find executable file: '/bin/echo'"),
        "helper should prompt to confirm executable source; stdout={stdout:?}"
    );
    assert!(
        stdout.contains("Enter here the command or the file to use as source [/bin/echo rejected]"),
        "helper should retain the rejected source as the next prompt default; stdout={stdout:?}"
    );
    assert!(
        stdout.contains(&format!("'{}' is a flat file", source.display())),
        "helper should re-prompt and accept the supplied flat-file source; stdout={stdout:?}"
    );
    assert!(
        stdout.contains("$1 = 42") && stdout.contains("\n84\n"),
        "helper should continue with regex and formula validation; stdout={stdout:?}"
    );
}

#[test]
fn interactive_helper_reprompts_after_invalid_regular_expression() {
    let _interactive_lock = INTERACTIVE_LOCK.lock().expect("lock interactive tests");
    let directory = TempDir::new("rpimonitor-e2e-interactive-invalid-regex");
    let webroot = directory.path().join("web");
    let datastore = directory.path().join("data");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(datastore.join("stat")).expect("create datastore stat directory");

    let source = directory.path().join("metrics.txt");
    fs::write(&source, "load=42\n").expect("write flat-file source");
    let config = write_config(
        directory.path(),
        "interactive-invalid-regex.conf",
        &format!(
            "daemon.readonly=1\ndaemon.datastore={}\ndaemon.webroot={}\n",
            datastore.display(),
            webroot.display()
        ),
    );
    let input = format!("\n{}\ny\n(\n\n(\\d+)\ny\n$1 * 2\ny\ny\n", source.display());
    let output = run_with_stdin(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-i"),
        ],
        directory.path(),
        input.as_bytes(),
    );

    assert!(
        matches!(output.status.code(), Some(2 | 255)),
        "interactive helper should continue to default-config load; status={:?}; \
         stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in [
        "The regular expression is not correct:",
        "Press [Enter] to continue:",
        "Applying the Regular Expression '(\\d+)'",
        "$1 = 42",
        "\n84\n",
    ] {
        assert!(
            stdout.contains(expected),
            "interactive output missing {expected:?}; stdout={stdout:?}"
        );
    }
}

#[test]
fn interactive_helper_reprompts_when_postprocessing_formula_is_rejected() {
    let _interactive_lock = INTERACTIVE_LOCK.lock().expect("lock interactive tests");
    let directory = TempDir::new("rpimonitor-e2e-interactive-formula-reject");
    let webroot = directory.path().join("web");
    let datastore = directory.path().join("data");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(datastore.join("stat")).expect("create datastore stat directory");

    let source = directory.path().join("metrics.txt");
    fs::write(&source, "load=42\n").expect("write flat-file source");
    let config = write_config(
        directory.path(),
        "interactive-formula-reject.conf",
        &format!(
            "daemon.readonly=1\ndaemon.datastore={}\ndaemon.webroot={}\n",
            datastore.display(),
            webroot.display()
        ),
    );
    let input = format!(
        "\n{}\ny\n(\\d+)\ny\n$1 * 2\nn\n$1 + 1\ny\ny\n",
        source.display()
    );
    let output = run_with_stdin(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-i"),
        ],
        directory.path(),
        input.as_bytes(),
    );

    assert!(
        matches!(output.status.code(), Some(2 | 255)),
        "interactive helper should continue to default-config load; status={:?}; \
         stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\n84\n") && stdout.contains("\n43\n"),
        "helper should show both rejected and accepted formula results; stdout={stdout:?}"
    );
    assert!(
        stdout.contains("Enter your formula [$1 * 2]:"),
        "helper should re-prompt with the rejected formula as its default; stdout={stdout:?}"
    );
}

#[test]
fn interactive_helper_prints_dynamic_configuration_template() {
    let _interactive_lock = INTERACTIVE_LOCK.lock().expect("lock interactive tests");
    let Some(_fixture) = DefaultConfigFixture::install() else {
        return;
    };

    let directory = TempDir::new("rpimonitor-e2e-interactive-template");
    let webroot = directory.path().join("web");
    let datastore = directory.path().join("data");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(datastore.join("stat")).expect("create datastore stat directory");

    let source = directory.path().join("metrics.txt");
    fs::write(&source, "load=42\n").expect("write source fixture");
    let config = write_config(
        directory.path(),
        "interactive-template.conf",
        &format!(
            "daemon.readonly=1\ndaemon.datastore={}\ndaemon.webroot={}\n",
            datastore.display(),
            webroot.display()
        ),
    );
    let input = format!(
        "\n{}\ny\n(\\d+)\ny\n$1 * 2\ny\nn\n",
        source.display()
    );
    let output = run_with_stdin(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-i"),
        ],
        directory.path(),
        input.as_bytes(),
    );

    assert!(
        output.status.success(),
        "interactive helper should print and exit successfully; status={:?}; stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    for expected in [
        "We have now defined all the parameters allowing RPi-Monitor to",
        "dynamic.2.name=<Add here the name>",
        &format!("dynamic.2.source={}", source.display()),
        "dynamic.2.regexp=(\\d+)",
        "dynamic.2.postprocess=$1 * 2",
        "dynamic.2.rrd=<Define the rrd type>",
    ] {
        assert!(
            stdout.contains(expected),
            "configuration template missing {expected:?}; stdout={stdout:?}"
        );
    }
}
