use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
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
pub fn run(args: &[OsString], working_dir: &Path) -> Output {
    let root = repository_root();
    let binary = binary_path();
    assert!(
        binary.is_file(),
        "rpimonitord executable not found at {}",
        binary.display()
    );

    let mut command = if env::var("PERL_COVERAGE").as_deref() == Ok("1") {
        let coverage_db = root.join("tests/cover_db");
        let coverage_options = format!(
            "-MDevel::Cover=-db,{},-silent,1,+ignore,perl_unit\\.pl",
            coverage_db.display()
        );
        let mut cmd = Command::new("perl");
        cmd.arg(coverage_options).arg(&binary).args(args);
        cmd
    } else {
        let mut cmd = Command::new(&binary);
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

    command.output().expect("launch rpimonitord")
}


/// Evaluate an expression against rpimonitord's Perl packages without running
/// the daemon's `package main` section, using the same loader as the pytest harness.
pub fn run_perl_unit(expression: &str, args: &[OsString], working_dir: &Path) -> Output {
    let root = repository_root();
    let binary = binary_path();
    let loader = root.join("tests/perl_unit.pl");
    assert!(binary.is_file(), "rpimonitord executable not found at {}", binary.display());
    assert!(loader.is_file(), "Perl unit-test loader not found at {}", loader.display());

    let mut command = Command::new("perl");
    if env::var("PERL_COVERAGE").as_deref() == Ok("1") {
        let coverage_db = root.join("tests/cover_db");
        let coverage_options = format!(
            "-MDevel::Cover=-db,{},-silent,1,+ignore,perl_unit\\.pl",
            coverage_db.display()
        );
        command.arg(coverage_options);
    }
    command.arg(loader).arg(binary).arg(expression).args(args);

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

    command.output().expect("launch Perl unit-test loader")
}
