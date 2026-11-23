use std::ffi::OsString;

use rpimonitor_e2e::{run, write_config, TempDir};

fn show_dynamic_configuration(source: &str, regexp: &str, postprocess: &str, default: &str, name: &str) -> String {
    let directory = TempDir::new("rpimonitor-extraction-config");
    let config = write_config(
        directory.path(),
        "extraction.conf",
        &format!(
            "dynamic.1.name={name}\ndynamic.1.source={source}\ndynamic.1.regexp={regexp}\ndynamic.1.postprocess={postprocess}\ndynamic.1.default={default}\n"
        ),
    );
    let output = run(
        &[
            OsString::from("-c"),
            config.into_os_string(),
            OsString::from("-r"),
            OsString::from("-s"),
        ],
        directory.path(),
    );
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "rpimonitord --show failed: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8(output.stdout)
        .expect("configuration dump should be UTF-8")
}

#[test]
fn multiple_capture_groups_and_postprocessing_rules_are_loaded() {
    let output = show_dynamic_configuration(
        "/tmp/extraction-source.txt",
        r"(\d+)\s+(\d+)",
        "$1*2,$2+3",
        "0,0",
        "first,second",
    );

    for expected in [
        "'source' => '/tmp/extraction-source.txt'",
        r"'regexp' => '(\\d+)\\s+(\\d+)'",
        "'postprocess' => '$1*2,$2+3'",
        "'default' => '0,0'",
        "'name' => 'first,second'",
    ] {
        assert!(output.contains(expected), "missing {expected:?} in loaded config: {output}");
    }
}

#[test]
fn matching_expression_and_default_are_kept_in_the_loaded_configuration() {
    let output = show_dynamic_configuration(
        "/tmp/extraction-source.txt",
        r"value=(\d+)",
        "",
        "-1",
        "value",
    );

    assert!(
        output.contains(r"'regexp' => 'value=(\\d+)'"),
        "regexp was not loaded: {output}"
    );
    assert!(output.contains("'default' => '-1'"), "default was not loaded: {output}");
}

#[test]
fn arithmetic_postprocessing_expression_is_loaded_without_modification() {
    let output = show_dynamic_configuration(
        "/tmp/extraction-source.txt",
        r"(\d+)\s+(\d+)",
        "$1*100,$2/3",
        "0,0",
        "scaled,third",
    );

    assert!(output.contains("'postprocess' => '$1*100,$2/3'"), "postprocessing expression was changed: {output}");
}
