# Test porting plan

## Principles

- Keep the Rust test harness as the single entry point for the Perl behavior tests.
- Exercise Perl packages directly through `run_perl_unit` when a test does not need process startup, daemonization, a listener, signals, or operating-system identity changes.
- Use subprocess/integration tests only when the behavior genuinely depends on process arguments, filesystem side effects, RRDs, HTTP, shared memory, SNMP, or daemon lifecycle.
- Preserve assertions and edge cases; do not duplicate the same scenario merely to run it once per implementation.
- Keep each test isolated with a unique temporary directory and avoid shared fixed paths.

## Inventory and migration status

| Area | Preferred level | Status / next action |
| --- | --- | --- |
| Configuration file parsing | Unit | Existing load-file tests; extend for comments, nested keys, arrays, RRD declarations, and errors |
| Include directive | Unit | Add coverage for existing include files and missing include targets |
| Default configuration values | Unit where possible; otherwise `--show` subprocess | Cover defaults and explicit overrides without starting the daemon |
| Interval / maximum interval | Unit | Cover interval normalization and LCM/GCD behavior; identify the exact legacy contract before asserting |
| Post-processing expressions | Unit | Cover expression evaluation and raw result |
| Regex extraction | Unit for matching; integration only for RRD/runtime effects | Cover capture groups, no match, and invalid expressions |
| CLI options, help, version | Subprocess without daemon | Existing CLI suite; fill gaps from the legacy cases |
| JSON generation / no-server mode | Integration | Verify generated files and read-only behavior |
| RRD creation and updates | Integration | Verify metadata, data-source bounds, RRAs, and update results |
| Alerts and command execution | Unit for parsing/evaluation; integration for side effects | Separate pure logic from file/shell execution |
| HTTP endpoints, static files, menu | Integration | Start server only where endpoint behavior is under test |
| PID files, daemonization, signals, permissions | Integration | Use bounded waits and reliable cleanup |
| SNMP and interactive mode | Integration | Port after the lower-level suites are stable |

## Execution phases

1. **Unit-test foundation** — keep the Perl loader and temporary-directory helpers small; add focused tests for parsing and includes.
2. **Pure configuration behavior** — defaults, overrides, interval calculations, post-processing, regex extraction, and alert/menu transformations where callable without runtime startup.
3. **CLI subprocess behavior** — help/version, option aliases, invalid arguments, and show mode; no daemon unless the option itself is the subject.
4. **Runtime integrations** — no-server/JSON, command execution, RRD, alerts, HTTP/static assets, PID/signals, SNMP, and interactive flows.
5. **Stabilization** — run the full suite repeatedly, check parallel isolation and cleanup, and collect coverage when `Devel::Cover` is installed.

## Acceptance criteria

- Every legacy test scenario is mapped to a test or explicitly marked as requiring unavailable infrastructure.
- Scenarios are represented once rather than duplicated for implementation variants.
- Tests that can run against Perl packages directly do not start the daemon.
- Assertions remain behavior-specific and are not weakened merely to make tests pass.
- Test commands and any external requirements are documented.

## Running the suite

```sh
cargo test --manifest-path tests/rust-e2e/Cargo.toml
PERL_COVERAGE=1 cargo test --manifest-path tests/rust-e2e/Cargo.toml
```

Coverage mode requires the Perl `Devel::Cover` module.