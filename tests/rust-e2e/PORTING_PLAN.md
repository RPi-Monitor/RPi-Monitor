# Rust black-box test porting plan

## Goal

Port the legacy Python end-to-end scenarios to Rust integration tests on GitHub, preserving observable behavior and edge cases while avoiding internal implementation hooks.

## Non-negotiable rules

- **Black-box only:** launch the real `rpimonitord` process and assert its public behavior: CLI exit status/output, generated files, HTTP responses, signals, PID lifecycle, and other externally observable effects.
- Do not call internal Perl methods, import Perl packages as test subjects, or depend on the removed Perl unit-test helpers.
- Do not change `src/usr/bin/rpimonitord` or other production code without explicit user approval.
- If a test reveals a likely defect or an unclear contract, report the observed behavior and options first. Decide together whether to preserve current behavior in a test, document the ambiguity, or authorize a separate production fix.
- Keep tests isolated with per-test temporary directories, bounded waits, deterministic fixtures, and reliable cleanup. Avoid shared fixed paths and random ports when a free-port strategy is available.
- Do not weaken assertions merely to make a test pass. Distinguish a test defect, environment limitation, and product behavior.
- Keep one canonical test for each scenario unless distinct public interfaces genuinely need separate coverage.

## Legacy inventory and migration status

Source inventory: the legacy test suite under `tests/` in the original repository `XavierBerger/rpi-monitor-rs` (branch `main`), compared with the Rust tests currently present on GitHub branch `test/rust-black-box`.

Status meanings:
- **Ported:** the externally observable scenario is covered by a Rust black-box test.
- **Partial:** only a subset of the original scenario is covered, or the Rust test checks configuration serialization rather than runtime behavior.
- **Missing:** no equivalent Rust black-box scenario has been identified yet.

| Legacy file | Scenarios in the legacy suite | Current Rust coverage | Status / next action |
| --- | --- | --- | --- |
| `test_alert_comprehensive.py` | Alert configuration loading; trigger/cancel commands and file side effects | `alert_config.rs` covers config loading via `-s` dump, single trigger raisecommand file side effect, and compound `&&` trigger | **Partial** — alert cancellation and resend-period runtime scenarios are not yet ported |
| `test_background_pid.py` | Background option and PID-file creation | `cli_options.rs` checks background mode and a PID file; `daemonize.rs` verifies PID file contents and daemon detachment | **Ported** — PID file contents, valid PID values, and daemon PID differs from test process are all covered |
| `test_base.py` | Shared process launcher, temporary config/files, polling, HTTP and cleanup helpers; this is support code, not a behavior suite | `src/lib.rs` provides process/config/temp-dir helpers | **Partial** — compare helper guarantees, especially daemon process-group cleanup and bounded polling |
| `test_cli_options.py` | Short/long CLI flags; show-mode overrides; background, keep, log file, address, delay, port, timeout, no-server, read-only; combined options and zero-valued arguments | `cli_options.rs` covers address, delay, port, timeout, no-server, read-only, verbosity, keep, log file, background, combined options, and zero-valued arguments | **Partial** — add remaining show-mode aliases and strengthen exit-code/output assertions; keep help/version mapped to their dedicated cases |
| `test_command_execution.py` | External command execution, extraction from command output, timeout handling, RRD side effects | `noserver_mode.rs` now launches the real daemon with `/bin/echo`, applies post-processing, and checks `dynamic.json` | **Partial** — add command-timeout and RRD side-effect scenarios |
| `test_config_loadfile.py` | Comments, nested structures, arrays, RRD sections, missing config file errors | `configuration_cli.rs` covers comments/blank lines and nested keys; `configuration_loadfile.rs` covers indexed grouping, nested status fields, regex preservation, RRD declarations, values containing equals signs, and missing-file errors | **Partial** — compare all legacy assertions and add any remaining parser edge cases |
| `test_configuration.py` | Include loading, user/group resolution, RRD directory creation | Include behavior is covered; defaults expose user/group strings | **Partial** — test actual user/group resolution and directory side effects through the process |
| `test_daemonize_and_writepid.py` | Background PID file, terminal detachment, graceful SIGTERM/SIGINT, show-mode PID behavior, SIGUSR1/SIGUSR2 log-level changes | `daemonize.rs` covers PID file creation, daemon detachment, SIGTERM, SIGINT, show-mode no-pidfile, SIGUSR1, SIGUSR2 | **Ported** — all six scenarios from the Python suite are covered with equivalent observable assertions |
| `test_default_values.py` | Default values and explicit overrides in show mode | `configuration_defaults.rs` covers documented defaults and overrides | **Ported** — review against legacy assertions for any omitted keys |
| `test_include_directive.py` | Existing include and missing include target | `configuration_cli.rs` covers include ordering and missing target; overlapping coverage exists | **Ported** — consolidate duplicate scenarios after checking assertion equivalence |
| `test_interactive_mode.py` | Interactive mode launch/branching, validation of executable, flat file, regex, formula, and dynamic type choices | No Rust interactive-mode tests | **Missing** — port with scripted stdin/PTY interaction and bounded prompts |
| `test_interval_handling.py` | Applying the default interval to dynamic metrics | Default interval is shown in `configuration_defaults.rs`; no assertion that it is applied to a metric | **Partial** — test the observable configured/effective metric interval |
| `test_json_endpoints.py` | Static, dynamic and all-data JSON endpoints; JSON file generation | `noserver_mode.rs` covers HTTP JSON endpoints (static, dynamic, 404, file serving) | **Ported** — HTTP endpoint behavior and JSON file generation are covered |
| `test_maxinterval_lcm.py` | Maximum interval is the LCM of dynamic metric intervals | `interval_lcm.rs` covers LCM(12,24,30)=120 distinguishing LCM from GCD, single-interval boundary, and coprime pairs | **Ported** — three cases distinguish the LCM algorithm from GCD and other alternatives |
| `test_menu_generation.py` | Menu endpoint output across multiple scenarios | `noserver_mode.rs` verifies menu.json is created with configured navigation entries | **Partial** — HTTP menu endpoint content via live server not yet asserted |
| `test_noserver_mode.py` | No-server JSON writes, writable vs read-only behavior, RRD creation | `noserver_mode.rs` verifies runtime extraction, post-processing, static values, companion JSON files, read-only no-write | **Ported** — all key no-server scenarios are covered |
| `test_postprocess_eval.py` | Evaluation and result of post-processing expressions | `noserver_mode.rs` checks runtime arithmetic post-processing and the default fallback on a regex non-match; `monitor_extraction.rs` also checks config preservation | **Partial** — add more arithmetic cases and post-processing error behavior |
| `test_regex_extraction.py` | Single/multiple capture groups, no match, invalid regex | `noserver_mode.rs` covers single/multiple capture groups, no-match default, flat-file last-match, data expansion | **Ported** — runtime extraction and edge cases are covered |
| `test_rpimonitord_help.py` | `-h` and `--help` output | `cli_help.rs` asserts exit code 2/255 and checks all expected help sections | **Ported** — exit code and all public help text sections are verified |
| `test_rpimonitord_version.py` | `-V` and `--Version` output | `cli_options.rs` covers both flags and stderr output | **Ported** — confirm output/exit assertions match the legacy contract |
| `test_rrd_creation.py` | RRD directory/empty RRD creation; dynamic RRD types GAUGE/COUNTER/DERIVE | `rrd_creation.rs` covers stat directory, empty.rrd schema (5 RRAs, AVERAGE CF), and GAUGE/COUNTER/DERIVE dynamic RRDs | **Ported** — all four test cases (directory, empty.rrd, three RRD types) are covered |
| `test_rrd_min_max_values.py` | RRD data-source minimum/maximum bounds across cases | No Rust coverage | **Missing** — port all legacy bounds cases |
| `test_rrd_rra_configuration.py` | RRD archive/RRA configuration | No Rust coverage | **Missing** — verify archive metadata through an external RRD inspection command/library interface |
| `test_server_readonly_status.py` | Read-only status through server JSON | CLI read-only flag is covered, but server response is not | **Partial** — assert the endpoint response in read-only mode |
| `test_shared_memory.py` | Shared memory through web endpoint, no-server mode, and error conditions | No Rust coverage | **Missing** — verify observable endpoints/files and error behavior without inspecting internal memory APIs |
| `test_snmp_daemon_runtime.py` | Symlink dispatch to SNMP daemon | No Rust coverage | **Missing** — test the executable/symlink interface in an isolated environment |
| `test_snmp_functionality.py` | MIB generation, dynamic metrics, SNMP GET | No Rust coverage | **Missing** — use an isolated SNMP client/agent fixture and assert protocol responses |
| `test_static_file_serving.py` | Static assets and files served from the data store | `noserver_mode.rs` (`web_server_serves_existing_static_file_from_webroot`) verifies HTTP 200 and file body for a placed asset | **Ported** — HTTP static file serving behavior is covered |
| `test_uid_gid_resolution.py` | Existing/nonexistent user and group resolution; file permissions | Default `user`/`group` values are asserted, but actual resolution and permissions are not | **Partial** — test process-visible behavior using safe, available identities and temporary files |

## Existing Rust black-box suites

- `tests/rust-e2e/tests/cli_options.rs`: CLI flags, version output, log file, background/PID behavior, combined options, and zero-valued option arguments.
- `tests/rust-e2e/tests/cli_help.rs`: `-h` and `--help` exit code and help text content.
- `tests/rust-e2e/tests/configuration_defaults.rs`: defaults and explicit daemon overrides.
- `tests/rust-e2e/tests/configuration_loadfile.rs`: indexed keys and multiple configuration files.
- `tests/rust-e2e/tests/configuration_cli.rs`: comments, includes, interval summary, nested values.
- `tests/rust-e2e/tests/monitor_extraction.rs`: extraction-related configuration survives loading; **not runtime extraction coverage**.
- `tests/rust-e2e/tests/noserver_mode.rs`: starts the real daemon in background no-server mode and checks command output extraction, arithmetic post-processing, static values, companion JSON files, read-only no-write, HTTP endpoints (static.json, dynamic.json, 404, file serving), runtime timeout, multiple capture groups, flat-file last-match, and data expansion.
- `tests/rust-e2e/tests/daemonize.rs`: PID file creation and content, daemon terminal detachment, graceful SIGTERM/SIGINT, show-mode no-pidfile, SIGUSR1/SIGUSR2 log-level adjustment.
- `tests/rust-e2e/tests/rrd_creation.rs`: stat directory creation, empty.rrd schema validation (5 RRAs, AVERAGE CF), and GAUGE/COUNTER/DERIVE dynamic RRD creation via `rrdtool info`.
- `tests/rust-e2e/tests/alert_config.rs`: alert key/value config loading via `-s` dump, raisecommand file side effect, compound `&&` trigger condition.
- `tests/rust-e2e/tests/interval_lcm.rs`: maxinterval = LCM of all dynamic intervals (three-interval case distinguishing LCM from GCD, single-interval boundary, coprime-prime pairs).
- `tests/suite/*.bats`: existing install/update/security shell scenarios are separate from the Python behavior-test inventory and must be reviewed independently before claiming full repository test coverage.

## Execution plan

1. **Inventory baseline:** retain this file as the source-to-target map. Check legacy assertions and fixtures for every case before porting it.
2. **Correct the test harness:** keep helpers limited to launching the real executable, creating temporary inputs, observing outputs/files/HTTP, waiting with timeouts, and cleaning up child processes.
3. **Finish low-cost CLI/configuration coverage:** help, config-file errors, arrays/RRD declarations, effective intervals, user/group resolution.
4. **Add extraction behavior tests:** controlled input files/commands; assert values in generated JSON or another public output, including multiple groups, no match, invalid regex, and post-processing.
5. **Add runtime integration suites:** JSON/no-server, RRD, alerts, HTTP/menu/static assets, daemon/signals, shared memory, SNMP, and interactive mode.
6. **Validate fidelity:** map every legacy case to a Rust test, compare assertions, document any infrastructure-dependent exclusions, and avoid duplicate cases.
7. **Run and document:** execute focused tests and the full Rust suite in the project environment; record commands and results. A source review alone is not evidence that tests passed.

## Acceptance criteria

- Every legacy behavior test is marked ported, partial, or missing with a concrete next action.
- Ported means equivalent observable assertions, not merely similar test names or configuration serialization.
- No test calls internal Perl methods or removed unit-test tools.
- Test data and runtime side effects are isolated and cleaned up, including daemon descendants.
- Potential product defects are reported for user decision; production code remains unchanged without explicit permission.
- The Rust suite and this mapping document are kept in sync.

## Running the Rust suite

```sh
cargo test --manifest-path tests/rust-e2e/Cargo.toml
```

Run the suite in the repository's supported environment. Do not report a successful run unless the command was actually executed and its result observed.
