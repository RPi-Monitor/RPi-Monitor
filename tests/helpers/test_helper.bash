# Common helper for Bats tests

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INSTALL_SH="${REPO_ROOT}/install.sh"
FIXTURES_DIR="${REPO_ROOT}/tests/fixtures"

# Start mock server if not running
# Args: port, version, corrupt_checksum (0|1), omit_zip_asset (0|1)
start_mock_server() {
    local port="${1:-8080}"
    local version="${2:-2.14}"
    local corrupt="${3:-0}"
    local omit_zip="${4:-0}"
    local assets_dir="${REPO_ROOT}/build"

    # Ensure archive exists (unless testing a release that omits the ZIP asset)
    if [[ "${omit_zip}" != "1" && ! -f "${assets_dir}/rpimonitor-${version}.zip" ]]; then
        "${REPO_ROOT}/tools/build-zip.sh" "${version}" >/dev/null 2>&1
    fi

    # Check if already running on this port
    if curl -s "http://127.0.0.1:${port}/health" >/dev/null 2>&1; then
        return 0
    fi

    MOCK_PORT="${port}" ASSETS_DIR="${assets_dir}" MOCK_VERSION="${version}" \
        MOCK_CORRUPT_CHECKSUM="${corrupt}" MOCK_OMIT_ZIP_ASSET="${omit_zip}" \
        python3 "${FIXTURES_DIR}/mock_github_server.py" &
    local pid=$!

    # Wait for mock server
    for i in {1..30}; do
        if curl -s "http://127.0.0.1:${port}/health" >/dev/null 2>&1; then
            echo "$pid" > "/tmp/mock_server_${port}.pid"
            return 0
        fi
        sleep 0.1
    done

    echo "Failed to start mock server on port ${port}" >&2
    return 1
}

stop_mock_server() {
    local port="${1:-8080}"
    if [[ -f "/tmp/mock_server_${port}.pid" ]]; then
        kill "$(cat "/tmp/mock_server_${port}.pid")" >/dev/null 2>&1 || true
        rm -f "/tmp/mock_server_${port}.pid"
    fi
}
