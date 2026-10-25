use std::ffi::OsString;

use rpimonitor_e2e::{run_perl_unit, write_config, TempDir};

const LOAD_DYNAMIC: &str =
    "my $c = Configuration->new; $c->LoadFile($_) foreach @ARGV; $c->{dynamic}";

fn run_load_files(directory: &TempDir, files: &[&str]) -> String {
    let args: Vec<OsString> = files.iter().map(OsString::from).collect();
    let output = run_perl_unit(LOAD_DYNAMIC, &args, directory.path());

    assert!(
        output.status.success(),
        "Perl unit loader failed (status: {:?})\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    String::from_utf8(output.stdout)
        .expect("Perl unit loader should emit UTF-8 JSON")
        .trim()
        .to_owned()
}

#[test]
fn dotted_keys_sharing_an_index_are_grouped_into_one_entry() {
    let directory = TempDir::new("rpimonitor-loadfile-shared-index");
    let config = write_config(
        directory.path(),
        "shared.conf",
        "dynamic.1.name=load\ndynamic.1.source=/proc/loadavg\n",
    );

    let actual = run_load_files(&directory, &[config.to_str().expect("UTF-8 config path")]);

    assert_eq!(
        actual,
        r#"[{"name":"load","source":"/proc/loadavg"}]"#,
        "keys sharing dynamic.1 must populate the same array entry",
    );
}

#[test]
fn reusing_an_index_in_a_second_file_appends_instead_of_overwriting() {
    let directory = TempDir::new("rpimonitor-loadfile-multiple-files");
    let first = write_config(
        directory.path(),
        "first.conf",
        "dynamic.1.name=first\n",
    );
    let second = write_config(
        directory.path(),
        "second.conf",
        "dynamic.1.name=second\n",
    );

    let actual = run_load_files(
        &directory,
        &[
            first.to_str().expect("UTF-8 first config path"),
            second.to_str().expect("UTF-8 second config path"),
        ],
    );

    assert_eq!(
        actual,
        r#"[{"name":"first"},{"name":"second"}]"#,
        "reusing dynamic.1 in another file must append a new entry",
    );
}
