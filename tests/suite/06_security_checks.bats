#!/usr/bin/env bats

load "../helpers/test_helper.bash"

setup() {
    export TARGET_DIR="/tmp/test_security"
    rm -rf "${TARGET_DIR}"
    mkdir -p "${TARGET_DIR}"
}

teardown() {
    rm -rf "${TARGET_DIR}"
    stop_mock_server 8081
}

@test "Security: rejects package when SHA256 checksum mismatch occurs" {
    start_mock_server 8081 "2.14" 1
    export GITHUB_API_BASE="http://127.0.0.1:8081"

    run "${INSTALL_SH}" -y --no-service
    [ "$status" -eq 1 ]
    [[ "$output" =~ "SHA256 checksum mismatch" ]]
}

@test "Security: rejects release when distribution ZIP asset is missing" {
    # corrupt=0, omit_zip=1 — release JSON has no rpimonitor-*.zip asset
    start_mock_server 8081 "2.14" 0 1
    export GITHUB_API_BASE="http://127.0.0.1:8081"

    run "${INSTALL_SH}" -y --no-service
    [ "$status" -eq 1 ]
    [[ "$output" =~ "Asset 'rpimonitor-2.14.zip' not found in release" ]]
    [[ "$output" =~ "Available assets:" ]]
}

@test "Security: rejects release when SHA256 asset is missing" {
    # corrupt=0, omit_zip=0, omit_sha256=1
    start_mock_server 8081 "2.14" 0 0 1
    export GITHUB_API_BASE="http://127.0.0.1:8081"

    run "${INSTALL_SH}" -y --no-service
    [ "$status" -eq 1 ]
    [[ "$output" =~ "Asset 'rpimonitor-2.14.zip.sha256' not found in release" ]]
}

@test "Security: refuses installation if Debian package rpimonitor is already installed" {
    local fake_bin_dir="/tmp/fake_bin_$$"
    mkdir -p "${fake_bin_dir}"
    cat << 'EOF' > "${fake_bin_dir}/dpkg-query"
#!/bin/sh
if [ "$*" = "-W -f=\${Status} rpimonitor" ]; then
    echo "install ok installed"
    exit 0
fi
exit 1
EOF
    chmod +x "${fake_bin_dir}/dpkg-query"

    PATH="${fake_bin_dir}:${PATH}" run "${INSTALL_SH}" -y --dry-run
    rm -rf "${fake_bin_dir}"

    [ "$status" -eq 1 ]
    [[ "$output" =~ "RPi-Monitor is installed from its Debian package" ]]
    [[ "$output" =~ "sudo apt remove rpimonitor" ]]
}
