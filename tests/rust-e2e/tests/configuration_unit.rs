use std::ffi::OsString;

use rpimonitor_e2e::{run_perl_unit, write_config, TempDir};

fn run_unit(directory: &TempDir, expression: &str, files: &[&str]) -> String {
    let args: Vec<OsString> = files.iter().map(OsString::from).collect();
    let output = run_perl_unit(expression, &args, directory.path());
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
fn loadfile_ignores_comments_and_blank_lines() {
    let directory = TempDir::new("rpimonitor-config-comments");
    let config = write_config(
        directory.path(),
        "comments.conf",
        "# whole-line comment\n\ndynamic.1.name=load\n  # indented comment\n",
    );
    let actual = run_unit(
        &directory,
        "my $c = Configuration->new; $c->LoadFile($_) foreach @ARGV; $c->{dynamic}",
        &[config.to_str().expect("UTF-8 config path")],
    );
    assert_eq!(actual, r#"[{"name":"load"}]"#);
}

#[test]
fn include_directive_loads_existing_file_after_the_parent_file() {
    let directory = TempDir::new("rpimonitor-config-include");
    let child = write_config(directory.path(), "included.conf", "dynamic.1.name=child\n");
    let parent = write_config(
        directory.path(),
        "parent.conf",
        &format!("dynamic.1.name=parent\ninclude={}\n", child.display()),
    );
    let actual = run_unit(
        &directory,
        "my $c = Configuration->new; $c->LoadFile($ARGV[0]); $c->LoadFile($_) foreach @{$c->{daemon}->{confFiles}}; $c->{dynamic}",
        &[parent.to_str().expect("UTF-8 parent config path")],
    );
    assert_eq!(
        actual,
        r#"[{"name":"parent"},{"name":"child"}]"#,
        "included configuration should be loaded after the parent configuration",
    );
}

#[test]
fn include_directive_ignores_a_missing_target() {
    let directory = TempDir::new("rpimonitor-config-missing-include");
    let parent = write_config(
        directory.path(),
        "parent.conf",
        "dynamic.1.name=parent\ninclude=does-not-exist.conf\n",
    );
    let actual = run_unit(
        &directory,
        "my $c = Configuration->new; $c->LoadFile($_) foreach @ARGV; $c->{dynamic}",
        &[parent.to_str().expect("UTF-8 parent config path")],
    );
    assert_eq!(actual, r#"[{"name":"parent"}]"#);
}
#[test]
fn interval_combination_returns_the_least_common_multiple() {
    let directory = TempDir::new("rpimonitor-interval-lcm");
    let actual = run_unit(
        &directory,
        "my $c = Configuration->new; [$c->GreatestCommonDivisor(2, 3), $c->GreatestCommonDivisor(4, 6), $c->GreatestCommonDivisor(5, 10)]",
        &[],
    );
    assert_eq!(actual, "[6,12,10]");
}

#[test]
fn loadfile_preserves_nested_keys_and_values_containing_equals() {
    let directory = TempDir::new("rpimonitor-config-nested");
    let config = write_config(
        directory.path(),
        "nested.conf",
        "web.page.title=Monitor\ndynamic.1.source=/proc/loadavg\ndynamic.1.name=load\ndynamic.1.postprocess=$1*2\n",
    );
    let actual = run_unit(
        &directory,
        "my $c = Configuration->new; $c->LoadFile($_) foreach @ARGV; { page => $c->{web}->{page}, dynamic => $c->{dynamic} }",
        &[config.to_str().expect("UTF-8 config path")],
    );
    assert_eq!(
        actual,
        r#"{"dynamic":[{"name":"load","postprocess":"$1*2","source":"/proc/loadavg"}],"page":{"title":"Monitor"}}"#,
    );
}
