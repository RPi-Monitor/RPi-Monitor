use std::ffi::OsString;

use rpimonitor_e2e::{run_perl_unit, write_config, TempDir};

fn run_process(directory: &TempDir, source: &str, regexp: &str, postprocess: &str, default: &str, names: &str) -> String {
    let source_path = write_config(directory.path(), "source.txt", source);
    let args = [OsString::from(source_path)];
    let expression = format!(
        "my $m = Monitor->new; my $c = {{ daemon => {{ timeout => 2 }}, dynamic => [{{ source => $ARGV[0], regexp => {}, postprocess => {}, default => {}, name => {}, interval => 1 }}] }}; $m->Process($c, 'dynamic'); $m->{{dynamic}}",
        perl_quote(regexp), perl_quote(postprocess), perl_quote(default), perl_quote(names),
    );
    let output = run_perl_unit(&expression, &args, directory.path());
    assert!(
        output.status.success(),
        "Monitor::Process failed (status: {:?})\nstdout:\n{}\nstderr:\n{}\nexpression: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        expression,
    );
    String::from_utf8(output.stdout).expect("monitor output should be UTF-8").trim().to_owned()
}

fn perl_quote(value: &str) -> String {
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}

#[test]
fn regex_extraction_supports_multiple_capture_groups_and_postprocessing() {
    let directory = TempDir::new("rpimonitor-regex-postprocess");
    let actual = run_process(
        &directory,
        "12 7\n",
        r"(\d+)\s+(\d+)",
        "$1*2,$1+3",
        "0,0",
        "first,second",
    );
    assert_eq!(actual, r#"{"first":24,"second":10}"#);
}

#[test]
fn regex_extraction_skips_nonmatching_lines_and_uses_the_matching_capture() {
    let directory = TempDir::new("rpimonitor-regex-skip-lines");
    let actual = run_process(
        &directory,
        "unrelated line\nvalue=42\n",
        r"value=(\d+)",
        "",
        "0",
        "value",
    );
    assert_eq!(actual, r#"{"value":"42"}"#);
}

#[test]
fn regex_extraction_uses_defaults_when_the_source_does_not_match() {
    let directory = TempDir::new("rpimonitor-regex-defaults");
    let actual = run_process(
        &directory,
        "no numeric value here\n",
        r"value=(\d+)",
        "",
        "-1",
        "value",
    );
    assert_eq!(actual, r#"{"value":"-1"}"#);
}

#[test]
fn postprocessing_evaluates_arithmetic_expressions_for_each_capture() {
    let directory = TempDir::new("rpimonitor-postprocess-arithmetic");
    let actual = run_process(
        &directory,
        "5 9\n",
        r"(\d+)\s+(\d+)",
        "$1*100,$1/3",
        "0,0",
        "scaled,third",
    );
    assert_eq!(actual, r#"{"scaled":500,"third":3}"#);
}