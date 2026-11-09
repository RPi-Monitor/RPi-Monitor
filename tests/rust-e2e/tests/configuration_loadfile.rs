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
    String::from_utf8(output.stdout).expect("configuration dump should be UTF-8")
}

#[test]
fn dotted_keys_sharing_an_index_are_grouped_into_one_entry() {
    let directory = TempDir::new("rpimonitor-loadfile-shared-index");
    let config = write_config(
        directory.path(),
        "shared.conf",
        "dynamic.1.name=load\ndynamic.1.source=/proc/loadavg\n",
    );

    let output = show_configuration(
        &directory,
        &[config.to_str().expect("UTF-8 config path")],
    );

    assert!(output.contains("'name' => 'load'"), "name missing from loaded entry: {output}");
    assert!(output.contains("'source' => '/proc/loadavg'"), "source missing from loaded entry: {output}");
    assert_eq!(output.matches("'name' => 'load'").count(), 1, "same index should produce one entry: {output}");
}

#[test]
fn reusing_an_index_in_a_second_file_appends_instead_of_overwriting() {
    let directory = TempDir::new("rpimonitor-loadfile-multiple-files");
    let first = write_config(directory.path(), "first.conf", "dynamic.1.name=first\n");
    let second = write_config(directory.path(), "second.conf", "dynamic.1.name=second\n");

    let output = show_configuration(
        &directory,
        &[
            first.to_str().expect("UTF-8 first config path"),
            second.to_str().expect("UTF-8 second config path"),
        ],
    );

    let first_position = output.find("'name' => 'first'").expect("first entry missing");
    let second_position = output.find("'name' => 'second'").expect("second entry missing");
    assert!(first_position < second_position, "second file should append another entry: {output}");
}

#[test]
fn indexed_configuration_entries_are_kept_in_order() {
    let directory = TempDir::new("rpimonitor-loadfile-arrays");
    let config = write_config(
        directory.path(),
        "arrays.conf",
        "web.status.1.title=First Status\nweb.status.2.title=Second Status\nweb.status.3.title=Third Status\ndynamic.1.name=cpu_usage\ndynamic.2.name=memory_usage\ndynamic.3.name=disk_usage\n",
    );

    let output = show_configuration(
        &directory,
        &[config.to_str().expect("UTF-8 config path")],
    );

    let expected = [
        "'title' => 'First Status'",
        "'title' => 'Second Status'",
        "'title' => 'Third Status'",
        "'name' => 'cpu_usage'",
        "'name' => 'memory_usage'",
        "'name' => 'disk_usage'",
    ];
    for entry in expected {
        assert!(output.contains(entry), "missing indexed entry {entry:?}: {output}");
    }

    let first = output.find("'title' => 'First Status'").expect("first status missing");
    let second = output.find("'title' => 'Second Status'").expect("second status missing");
    let third = output.find("'title' => 'Third Status'").expect("third status missing");
    assert!(first < second && second < third, "indexed status entries are out of order: {output}");
}

#[test]
fn dynamic_rrd_declarations_are_preserved_in_loaded_configuration() {
    let directory = TempDir::new("rpimonitor-loadfile-rrd");
    let config = write_config(
        directory.path(),
        "rrd.conf",
        "dynamic.1.name=cpu_usage\ndynamic.1.source=/proc/loadavg\ndynamic.1.rrd=GAUGE\ndynamic.2.name=memory_usage\ndynamic.2.source=/proc/meminfo\ndynamic.2.rrd=COUNTER\n",
    );

    let output = show_configuration(
        &directory,
        &[config.to_str().expect("UTF-8 config path")],
    );

    assert!(
        output.contains("'rrd' => 'GAUGE'"),
        "GAUGE declaration missing from loaded configuration: {output}"
    );
    assert!(
        output.contains("'rrd' => 'COUNTER'"),
        "COUNTER declaration missing from loaded configuration: {output}"
    );
    assert!(
        output.contains("'rrd' => ["),
        "derived RRD list missing from loaded configuration: {output}"
    );
    let references_rrd_from_dynamic =
        output.contains("$VAR1->{'rrd'}[0]") && output.contains("$VAR1->{'rrd'}[1]");
    let references_dynamic_from_rrd =
        output.contains("$VAR1->{'dynamic'}[0]") && output.contains("$VAR1->{'dynamic'}[1]");
    assert!(
        output.contains("'dynamic' => [")
            && (references_rrd_from_dynamic || references_dynamic_from_rrd),
        "dynamic and RRD entries should share the same underlying entries: {output}"
    );
}

#[test]
fn missing_configuration_file_reports_an_error() {
    let directory = TempDir::new("rpimonitor-loadfile-missing");
    let missing = directory.path().join("missing.conf");
    let output = run(
        &[
            OsString::from("-c"),
            missing.clone().into_os_string(),
        ],
        directory.path(),
    );

    assert_ne!(
        output.status.code(),
        Some(0),
        "missing configuration should fail; stdout={:?}; stderr={:?}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Error while openning configuration file")
            && stderr.contains(&missing.display().to_string()),
        "expected a missing-configuration diagnostic for {}; stderr={stderr:?}",
        missing.display()
    );
}

#[test]
fn nested_status_fields_and_dynamic_regex_are_preserved() {
    let directory = TempDir::new("rpimonitor-loadfile-nested");
    let config = write_config(
        directory.path(),
        "nested.conf",
        "web.status.1.title=System Status\nweb.status.1.content=CPU: %cpu.0.percent%%\nweb.status.2.title=Network Status\nweb.status.2.content=IP: %network.0.ip%\ndynamic.1.name=cpu_usage\ndynamic.1.source=cat /proc/loadavg\ndynamic.1.regexp=(\\S+)\\s+(\\S+)\\s+(\\S+)\n",
    );

    let output = show_configuration(
        &directory,
        &[config.to_str().expect("UTF-8 config path")],
    );

    for expected in [
        "'title' => 'System Status'",
        "'content' => 'CPU: %cpu.0.percent%%'",
        "'title' => 'Network Status'",
        "'content' => 'IP: %network.0.ip%'",
        "'name' => 'cpu_usage'",
        "'source' => 'cat /proc/loadavg'",
        r"'regexp' => '(\\S+)\\s+(\\S+)\\s+(\\S+)'",
    ] {
        assert!(
            output.contains(expected),
            "missing nested configuration value {expected:?}: {output}"
        );
    }
}

#[test]
fn equals_signs_inside_values_are_not_treated_as_key_separators() {
    let directory = TempDir::new("rpimonitor-loadfile-equals-value");
    let config = write_config(
        directory.path(),
        "equals-value.conf",
        "dynamic.1.name=conditional\ndynamic.1.source=printf 0\ndynamic.1.postprocess=$1==0?1:2\nweb.page.pagetitle=Monitor=Status\n",
    );

    let output = show_configuration(
        &directory,
        &[config.to_str().expect("UTF-8 config path")],
    );

    for expected in [
        "'name' => 'conditional'",
        "'postprocess' => '$1==0?1:2'",
        "'pagetitle' => 'Monitor=Status'",
    ] {
        assert!(
            output.contains(expected),
            "value containing '=' was not preserved ({expected:?}): {output}"
        );
    }
}

#[test]
fn dynamic_intervals_set_maxinterval_and_missing_intervals_use_default() {
    let directory = TempDir::new("rpimonitor-loadfile-intervals");
    let config = write_config(
        directory.path(),
        "intervals.conf",
        "daemon.defaultinterval=4\ndynamic.1.name=two_seconds\ndynamic.1.source=/bin/echo 2\ndynamic.1.regexp=(\\d+)\ndynamic.1.interval=2\ndynamic.2.name=three_seconds\ndynamic.2.source=/bin/echo 3\ndynamic.2.regexp=(\\d+)\ndynamic.2.interval=3\ndynamic.3.name=default_interval\ndynamic.3.source=/bin/echo 4\ndynamic.3.regexp=(\\d+)\n",
    );

    let output = show_configuration(
        &directory,
        &[config.to_str().expect("UTF-8 config path")],
    );

    assert!(
        output.contains("'maxinterval' => 6"),
        "intervals 2 and 3 should produce maxinterval 6; dump: {output}"
    );
    let default_entry = output
        .find("'name' => 'default_interval'")
        .expect("entry with implicit interval should exist");
    let entry_start = output[..default_entry]
        .rfind('{')
        .expect("default interval entry should have an opening brace");
    let entry_end = output[default_entry..]
        .find('}')
        .map(|offset| default_entry + offset)
        .expect("default interval entry should have a closing brace");
    let entry = &output[entry_start..=entry_end];
    assert!(
        entry.contains("'interval' => 4"),
        "entry without explicit interval should inherit defaultinterval=4; entry: {entry}; dump: {output}"
    );
}

#[test]
fn include_directive_loads_additional_configuration_file() {
    let directory = TempDir::new("rpimonitor-loadfile-include");
    write_config(
        directory.path(),
        "included.conf",
        "daemon.port=9001\ndynamic.1.name=included_metric\ndynamic.1.source=/bin/echo 5\ndynamic.1.regexp=(\\\\d+)\n",
    );
    let main = write_config(
        directory.path(),
        "main.conf",
        "include=included.conf\ndaemon.port=9002\ndynamic.1.name=main_metric\n",
    );

    let output = show_configuration(
        &directory,
        &[main.to_str().expect("UTF-8 main config path")],
    );

    assert!(
        output.contains("'port' => '9001'"),
        "included configuration's scalar value should be retained: {output}"
    );
    assert!(
        output.contains("'name' => 'main_metric'"),
        "main configuration's dynamic entry should be loaded: {output}"
    );
    assert!(
        output.contains("'name' => 'included_metric'"),
        "included configuration's dynamic entry should also be loaded: {output}"
    );
}
