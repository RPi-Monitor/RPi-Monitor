use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

const BINARY_RELATIVE_PATH: &str = "src/usr/bin/rpimonitord";

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before UNIX epoch")
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "rpimonitor-e2e-cli-addr-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create isolated working directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn run_address_option(option: &str) -> Output {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("resolve repository root");
    let binary = root.join(BINARY_RELATIVE_PATH);
    assert!(binary.is_file(), "rpimonitord executable not found at {}", binary.display());

    let temp_dir = TempDir::new();
    let config = temp_dir.path().join("cli_options_1.conf");
    fs::write(
        &config,
        "\ndaemon.delay=1\ndaemon.datastore=/tmp/test_addr\ndaemon.webroot=/tmp/test_web\n",
    )
    .expect("write minimal daemon configuration");

    let mut command = if env::var("PERL_COVERAGE").as_deref() == Ok("1") {
        let coverage_db = root.join("tests/cover_db");
        let coverage_options = format!(
            "-MDevel::Cover=-db,{},-silent,1,+ignore,perl\\.pl",
            coverage_db.display()
        );
        let mut cmd = Command::new("perl");
        cmd.arg(coverage_options)
            .arg(&binary)
            .arg("-c")
            .arg(&config)
            .arg(option)
            .arg("127.0.0.1")
            .arg("-s");
        cmd
    } else {
        let mut cmd = Command::new(&binary);
        cmd.arg("-c")
            .arg(&config)
            .arg(option)
            .arg("127.0.0.1")
            .arg("-s");
        cmd
    };

    command.current_dir(temp_dir.path()).env_clear();
    for key in ["PATH", "HOME"] {
        if let Some(value) = env::var_os(key) {
            command.env(key, value);
        }
    }
    command
        .env("LC_ALL", "C")
        .env("LANGUAGE", "C")
        .env("LANG", "C")
        .env("LC_CTYPE", "C");

    command.output().expect("launch rpimonitord")
}

fn assert_address_option(option: &str) {
    let output = run_address_option(option);
    assert!(
        matches!(output.status.code(), Some(0 | 2)),
        "{option}: expected exit code 0 or 2, got {:?}; stdout={:?}; stderr={:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");
    assert!(
        stdout.contains("'addr' => '127.0.0.1'"),
        "{option}: address not set correctly; stdout={stdout:?}"
    );
}

#[test]
fn short_addr_option_sets_web_server_bind_address() {
    assert_address_option("-a");
}

#[test]
fn long_addr_option_sets_web_server_bind_address() {
    assert_address_option("--addr");
}
