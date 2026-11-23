use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Stdio};
use std::sync::Mutex;

use rpimonitor_e2e::{binary_path, repository_root, run_with_stdin, write_config, TempDir};

static INTERACTIVE_LOCK: Mutex<()> = Mutex::new(());

fn run_interactive_with_temp_etc(
    args: &[OsString],
    working_dir: &std::path::Path,
    input: &[u8],
    etc_directory: &std::path::Path,
) -> std::process::Output {
    let root = repository_root();
    let mut command = Command::new("bwrap");
    command
        .arg("--ro-bind")
        .arg("/")
        .arg("/")
        .arg("--proc")
        .arg("/proc")
        .arg("--dev")
        .arg("/dev")
        .arg("--bind")
        .arg(etc_directory)
        .arg("/etc")
        .arg("--bind")
        .arg(&root)
        .arg(&root);

    if std::env::var("PERL_COVERAGE").as_deref() == Ok("1") {
        let coverage_db = std::env::var_os("PERL_COVERAGE_DB")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| root.join("tests/cover_db"));
        let coverage_options = format!(
            "-MDevel::Cover=-db,{},-silent,1,+ignore,perl_unit\\.pl",
            coverage_db.display()
        );
        let support = root.join("tests/rust-e2e/support");
        command
            .arg("perl")
            .arg(coverage_options)
            .arg(format!("-I{}", support.display()))
            .arg("-MDevel::Cover::Signal")
            .arg(binary_path());
    } else {
        command.arg(binary_path());
    }
    command.args(args);
    command
        .current_dir(working_dir)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", working_dir)
        .env("LC_ALL", "C")
        .env("LANGUAGE", "C")
        .env("LANG", "C")
        .env("LC_CTYPE", "C")
        .env("PERL_COVERAGE", std::env::var("PERL_COVERAGE").unwrap_or_default())
        .env("PERL_COVERAGE_DB", std::env::var("PERL_COVERAGE_DB").unwrap_or_default())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().expect("launch rpimonitord in Bubblewrap");
    child
        .stdin
        .take()
        .expect("open rpimonitord stdin")
        .write_all(input)
        .expect("write scripted stdin");
    child.wait_with_output().expect("wait for rpimonitord")
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
    if Command::new("bwrap").arg("--version").output().is_err() {
        return;
    }

    let directory = TempDir::new("rpimonitor-e2e-interactive-template");
    let webroot = directory.path().join("web");
    let datastore = directory.path().join("data");
    fs::create_dir_all(&webroot).expect("create webroot");
    fs::create_dir_all(datastore.join("stat")).expect("create datastore stat directory");

    let source = directory.path().join("metrics.txt");
    fs::write(&source, "load=42\n").expect("write source fixture");

    let isolated_etc = directory.path().join("etc");
    fs::create_dir_all(isolated_etc.join("rpimonitor")).expect("create isolated /etc/rpimonitor");
    for filename in ["passwd", "group", "hosts", "resolv.conf", "nsswitch.conf"] {
        let source = std::path::Path::new("/etc").join(filename);
        if let Ok(content) = fs::read(&source) {
            fs::write(isolated_etc.join(filename), content)
                .expect("copy host system identity fixture");
        }
    }
    fs::write(
        isolated_etc.join("rpimonitor/data.conf"),
        "dynamic.1.name=existing_metric\ndynamic.1.source=/bin/echo 1\ndynamic.1.regexp=(\\d+)\\n",
    )
    .expect("write isolated default data config");
    fs::write(
        isolated_etc.join("rpimonitor/daemon.conf"),
        "daemon.readonly=1\n",
    )
    .expect("write isolated default daemon config");

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
    let output = run_interactive_with_temp_etc(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-i"),
        ],
        directory.path(),
        input.as_bytes(),
        &isolated_etc,
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
