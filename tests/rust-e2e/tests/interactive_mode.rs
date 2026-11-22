use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;

use rpimonitor_e2e::{run_with_stdin, write_config, TempDir};

#[test]
fn interactive_helper_processes_scripted_input_before_default_config_error() {
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
