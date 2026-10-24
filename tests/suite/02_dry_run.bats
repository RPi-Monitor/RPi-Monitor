#!/usr/bin/env bats

load "../helpers/test_helper.bash"

setup() {
    start_mock_server 8080 "2.14" 0
    export GITHUB_API_BASE="http://127.0.0.1:8080"
    export TARGET_DIR="/tmp/test_dry_run_target"
    rm -rf "${TARGET_DIR}"
    mkdir -p "${TARGET_DIR}"
}

teardown() {
    rm -rf "${TARGET_DIR}"
    stop_mock_server 8080
}

@test "Dry Run: simulates actions without modifying filesystem" {
    run "${INSTALL_SH}" -y --dry-run --no-service
    [ "$status" -eq 0 ]
    [[ "$output" =~ "Dry run completed: the system was not modified." ]]
    [[ "$output" =~ "FRESH INSTALLATION" ]]

    # Ensure no binary or configuration was actually created in TARGET_DIR or system
    [ ! -f "${TARGET_DIR}/usr/bin/rpimonitord" ]
    [ ! -d "${TARGET_DIR}/etc/rpimonitor" ]
}

@test "Dry Run: --version downloads and targets specified release version" {
    run "${INSTALL_SH}" -y --dry-run --no-service --version 2.14
    [ "$status" -eq 0 ]
    [[ "$output" =~ "Fetching release version 2.14 from GitHub" ]]
    [[ "$output" =~ "Selected release: v2.14" ]]
    [[ "$output" =~ "Dry run completed: the system was not modified." ]]
}

@test "Dry Run: --pre-release selects newest release including pre-releases" {
    # Mock /releases lists v2.14-rc1 as newest; ensure its asset exists.
    if [[ ! -f "${REPO_ROOT}/build/rpimonitor-2.14-rc1.zip" ]]; then
        "${REPO_ROOT}/tools/build-zip.sh" "2.14-rc1" >/dev/null 2>&1
    fi

    run "${INSTALL_SH}" -y --dry-run --no-service --pre-release
    [ "$status" -eq 0 ]
    [[ "$output" =~ "Fetching newest release (pre-releases included)" ]]
    [[ "$output" =~ "Selected release: v2.14-rc1" ]]
    [[ "$output" =~ "Dry run completed: the system was not modified." ]]
}
