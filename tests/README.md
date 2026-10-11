# HTTP regression tests

Run from the repository root on Linux:

    RPIMONITOR_HOST_IPC_NS="$(readlink /proc/self/ns/ipc)" \
      unshare --user --map-current-user --ipc --fork -- \
      python3 tests/test_http_connection.py

Requirements:
- Linux with System V IPC and user namespace support
- Python 3 (standard library)
- Perl and RPi-Monitor runtime dependencies
- util-linux (unshare)

The test uses:
- A private IPC namespace
- A temporary configuration and RRD datastore
- A dynamically selected localhost TCP port
- A separate System V shared memory key

It verifies HTTP 200/404 responses, valid JSON and
the absence of HTTP connection warnings.

The test terminates its daemon processes. IPC resources
are released when the private IPC namespace is destroyed.

Do not run the test directly in the host IPC namespace.
The installed RPi-Monitor service is not modified.
