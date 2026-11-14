use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const BINARY_RELATIVE_PATH: &str = "src/usr/bin/rpimonitord";

/// Resolve the repository root from this standalone Cargo package.
pub fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("resolve repository root")
        .to_path_buf()
}

/// Absolute path to the rpimonitord executable.
pub fn binary_path() -> PathBuf {
    repository_root().join(BINARY_RELATIVE_PATH)
}

/// A temporary directory removed when the value is dropped.
pub struct TempDir(PathBuf);

impl TempDir {
    pub fn new(prefix: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before UNIX epoch")
            .as_nanos();
        let path = env::temp_dir().join(format!("{prefix}-{}-{nonce}", std::process::id()));
        fs::create_dir(&path).expect("create isolated working directory");
        Self(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Write a test configuration file and return its path.
pub fn write_config(directory: &Path, name: &str, content: &str) -> PathBuf {
    let path = directory.join(name);
    fs::write(&path, content).expect("write test configuration");
    path
}

/// Run rpimonitord with isolated environment settings.
/// If PERL_COVERAGE=1, launch it through Devel::Cover as the Python harness does.
/// PERL_COVERAGE_DB overrides the coverage database path.
fn command(program: &Path, args: &[OsString], working_dir: &Path) -> Command {
    let root = repository_root();
    let binary = binary_path();
    assert!(
        binary.is_file(),
        "rpimonitord executable not found at {}",
        binary.display()
    );

    let mut command = if env::var("PERL_COVERAGE").as_deref() == Ok("1") {
        let coverage_db = env::var_os("PERL_COVERAGE_DB")
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join("tests/cover_db"));
        let coverage_options = format!(
            "-MDevel::Cover=-db,{},-silent,1,+ignore,perl_unit\\.pl",
            coverage_db.display()
        );
        let mut cmd = Command::new("perl");
        let coverage_support = Path::new(env!("CARGO_MANIFEST_DIR")).join("support");
        cmd.arg(coverage_options)
            .arg(format!("-I{}", coverage_support.display()))
            .arg("-MDevel::Cover::Signal");
        if program.file_name() == Some(OsStr::new("rpimonitord-snmp")) && program.is_symlink() {
            cmd.arg("-e")
                .arg("$0 = shift; my $script = shift; do $script or die $@ || $!;")
                .arg("--")
                .arg(program)
                .arg(&binary);
        } else {
            cmd.arg(program);
        }
        cmd.args(args);
        cmd
    } else {
        let mut cmd = Command::new(program);
        cmd.args(args);
        cmd
    };

    command.current_dir(working_dir).env_clear();
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
    command
}

pub fn run(args: &[OsString], working_dir: &Path) -> Output {
    command(&binary_path(), args, working_dir)
        .output()
        .expect("launch rpimonitord")
}

/// Run rpimonitord with scripted standard input and isolated environment settings.
pub fn run_with_stdin(args: &[OsString], working_dir: &Path, input: &[u8]) -> Output {
    run_program_with_stdin(&binary_path(), args, working_dir, input)
}

/// Run a named rpimonitord entry point with scripted standard input.
pub fn run_program_with_stdin(
    program: &Path,
    args: &[OsString],
    working_dir: &Path,
    input: &[u8],
) -> Output {
    let mut child = command(program, args, working_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("launch rpimonitord");
    child
        .stdin
        .take()
        .expect("open rpimonitord stdin")
        .write_all(input)
        .expect("write scripted stdin");
    child.wait_with_output().expect("wait for rpimonitord")
}
