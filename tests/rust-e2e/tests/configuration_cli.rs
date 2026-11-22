use std::ffi::OsString;

use rpimonitor_e2e::{run, write_config, TempDir};

fn show_configuration(directory: &TempDir, files: &[&str]) -> String {
    let mut args = Vec::new();
    for file in files {
        args.push(OsString::from("-c"));
        args.push(OsString::from(file));
    }
    args.extend([OsString::from("-r"), OsString::from("-s")]);

    let output = run(&args, directory.path());
    assert!(
        matches!(output.status.code(), Some(0 | 2 | 255)),
        "rpimonitord --show exited unexpectedly: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8(output.stdout)
        .expect("configuration dump should be UTF-8")
}

#[test]
fn comments_and_blank_lines_are_ignored_when_loading_configuration() {
    let directory = TempDir::new("rpimonitor-config-comments");
    let config = write_config(
        directory.path(),
        "comments.conf",
        "# whole-line comment\n\ndynamic.1.name=load\n  # indented comment\n",
    );

    let output = show_configuration(
        &directory,
        &[config.to_str().expect("UTF-8 config path")],
    );

    assert!(output.contains("'name' => 'load'"), "missing configured item: {output}");
    assert!(!output.contains("whole-line comment"), "comment leaked into config: {output}");
}

#[test]
fn included_configuration_is_loaded_after_the_parent_file() {
    let directory = TempDir::new("rpimonitor-config-include");
    let child = write_config(directory.path(), "included.conf", "dynamic.1.name=child\n");
    let parent = write_config(
        directory.path(),
        "parent.conf",
        &format!("dynamic.1.name=parent\ninclude={}\n", child.display()),
    );

    let output = show_configuration(
        &directory,
        &[parent.to_str().expect("UTF-8 parent config path")],
    );

    let parent_position = output.find("'name' => 'parent'").expect("parent entry missing");
    let child_position = output.find("'name' => 'child'").expect("included entry missing");
    assert!(parent_position < child_position, "included file should be loaded after parent: {output}");
}

#[test]
fn missing_include_target_does_not_prevent_startup() {
    let directory = TempDir::new("rpimonitor-config-missing-include");
    let parent = write_config(
        directory.path(),
        "parent.conf",
        "dynamic.1.name=parent\ninclude=does-not-exist.conf\n",
    );

    let output = show_configuration(
        &directory,
        &[parent.to_str().expect("UTF-8 parent config path")],
    );

    assert!(output.contains("'name' => 'parent'"), "parent config was not loaded: {output}");
}

#[test]
fn combined_interval_is_visible_in_the_loaded_configuration() {
    let directory = TempDir::new("rpimonitor-interval-lcm");
    let config = write_config(
        directory.path(),
        "intervals.conf",
        "dynamic.1.name=first\ndynamic.1.interval=2\ndynamic.2.name=second\ndynamic.2.interval=3\n",
    );

    let output = show_configuration(
        &directory,
        &[config.to_str().expect("UTF-8 config path")],
    );

    assert!(output.contains("'maxinterval' => 6"), "expected combined interval 6 for intervals 2 and 3: {output}");
}

#[test]
fn nested_keys_and_values_containing_equals_are_preserved() {
    let directory = TempDir::new("rpimonitor-config-nested");
    let config = write_config(
        directory.path(),
        "nested.conf",
        "web.page.title=Monitor\ndynamic.1.source=/proc/loadavg\ndynamic.1.name=load\ndynamic.1.postprocess=$1*2\n",
    );

    let output = show_configuration(
        &directory,
        &[config.to_str().expect("UTF-8 config path")],
    );

    for expected in ["'title' => 'Monitor'", "'name' => 'load'", "'source' => '/proc/loadavg'", "'postprocess' => '$1*2'"] {
        assert!(output.contains(expected), "missing {expected:?} in configuration dump: {output}");
    }
}
