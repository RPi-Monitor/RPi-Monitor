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
            "rpimonitor-e2e-version-{}-{nonce}",
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

fn launch_version_option(option: &str, root: &Path, working_dir: &Path) -> Output {
    let binary = root.join(BINARY_RELATIVE_PATH);
    assert!(binary.is_file(), "rpimonitord executable not found at {}", binary.display());

    let mut command = if env::var("PERL_COVERAGE").as_deref() == Ok("1") {
        let coverage_db = root.join("tests/cover_db");
        let coverage_options = format!(
            "-MDevel::Cover=-db,{},-silent,1,+ignore,perl_unit\\.pl",
            coverage_db.display()
        );
        let mut cmd = Command::new("perl");
        cmd.arg(coverage_options).arg(&binary).arg(option);
        cmd
    } else {
        let mut cmd = Command::new(&binary);
        cmd.arg(option);
        cmd
    };

    // Mirror the pytest harness: isolate the working directory and force the C locale,
    // while preserving PATH and HOME for commands launched by the SUT.
    command.current_dir(working_dir);
    command.env_clear();
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

#[test]
fn version_options_print_version_to_stderr_and_exit_with_perl_die_status() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("resolve repository root")
        .to_path_buf();
    let temp_dir = TempDir::new();

    for option in ["-V", "--Version"] {
        let output = launch_version_option(option, &root, temp_dir.path());

        // Native Perl exits 255 for die; Devel::Cover may rewrite this to 2.
        assert!(
            matches!(output.status.code(), Some(2 | 255)),
            "{option}: expected die exit (2 or 255), got {:?}; stdout={:?}; stderr={:?}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let expected = format!("{} version {{DEVELOPMENT}}", root.join(BINARY_RELATIVE_PATH).display());
        let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
        assert_eq!(
            stderr.trim(),
            expected,
            "{option}: version output did not match expected output"
        );
    }
}
