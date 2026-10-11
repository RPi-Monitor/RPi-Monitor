#!/usr/bin/env python3
"""HTTP connection regression test for RPi-Monitor (Linux)."""

import getpass
import grp
import json
import os
from pathlib import Path
import random
import signal
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
DAEMON = ROOT / "src/usr/bin/rpimonitord"
WEBROOT = ROOT / "src/usr/share/rpimonitor/web"

CASES = [
    ("/status.html", 200),
    ("/static.json", 200),
    ("/dynamic.json", 200),
    ("/nonexistent-page", 404),
    ("/status.html", 200),
    ("/another-missing-page", 404),
    ("/dynamic.json", 200),
]

WARNINGS = (
    "closed filehandle",
    "uninitialized value in vec",
    "broken pipe",
)


def free_port():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def request(port, path):
    try:
        with urllib.request.urlopen(
            f"http://127.0.0.1:{port}{path}", timeout=3
        ) as response:
            return response.status, response.read()
    except urllib.error.HTTPError as error:
        return error.code, error.read()


def stop_process_group(proc):
    try:
        os.killpg(proc.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass

    try:
        proc.wait(timeout=2)
    except subprocess.TimeoutExpired:
        pass

    try:
        os.killpg(proc.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass

    proc.wait(timeout=3)


def main():
    host_ipc_ns = os.environ.get("RPIMONITOR_HOST_IPC_NS")
    current_ipc_ns = os.readlink("/proc/self/ns/ipc")

    if not host_ipc_ns or host_ipc_ns == current_ipc_ns:
        raise RuntimeError(
            "Run inside a separate IPC namespace and provide "
            "RPIMONITOR_HOST_IPC_NS"
        )
    if not DAEMON.is_file() or not WEBROOT.is_dir():
        raise RuntimeError("RPi-Monitor source tree not found")

    port = free_port()

    key = random.randrange(0x40000000, 0x7fffffff)

    with tempfile.TemporaryDirectory(prefix="rpimonitor-test-") as tmp:
        work = Path(tmp)
        datastore = work / "data"
        (datastore / "stat").mkdir(parents=True)
        config = work / "test.conf"
        logfile = work / "daemon.log"

        username = getpass.getuser()
        groupname = grp.getgrgid(os.getgid()).gr_name

        config.write_text(
            f"daemon.addr=127.0.0.1\n"
            f"daemon.port={port}\n"
            f"daemon.user={username}\n"
            f"daemon.group={groupname}\n"
            f"daemon.webroot={WEBROOT}\n"
            f"daemon.datastore={datastore}\n"
            f"daemon.sharedmemkey={key}\n"
            f"daemon.delay=10\n"
            f"web.status.1.name=Test\n"
            f"web.statistics.1.name=Test\n"
            f"dynamic.1.name=uptime_seconds\n"
            f"dynamic.1.source=/proc/uptime\n"
            f"dynamic.1.regexp=^([0-9]+)\n"
            f"dynamic.1.rrd=GAUGE\n"
        )

        proc = None
        try:
            with logfile.open("w") as log:
                proc = subprocess.Popen(
                    ["perl", str(DAEMON), "-c", str(config), "-vv"],
                    stdout=subprocess.DEVNULL,
                    stderr=log,
                    start_new_session=True,
                )

            # Wait for HTTP readiness, not a fixed sleep.
            deadline = time.monotonic() + 15
            while time.monotonic() < deadline:
                if proc.poll() is not None:
                    raise RuntimeError("Daemon exited during startup")
                try:
                    request(port, "/status.html")
                    break
                except (OSError, TimeoutError):
                    time.sleep(0.2)
            else:
                raise RuntimeError("HTTP server startup timeout")

            for path, expected in CASES:
                actual, body = request(port, path)
                print(f"{path}: {actual} (expected {expected})")
                if actual != expected:
                    raise AssertionError(f"Unexpected status: {path}")

                if path.endswith(".json") and actual == 200:
                    json.loads(body)

            # Allow Perl warnings to be flushed.
            time.sleep(0.3)
            logs = logfile.read_text(errors="replace")
            found = [
                line for line in logs.splitlines()
                if any(w in line.lower() for w in WARNINGS)
            ]

            if found:
                raise AssertionError(
                    "HTTP warnings detected:\n" + "\n".join(found)
                )

            print("PASS: HTTP status, JSON and connection lifecycle")

        except Exception:
            print("=== DAEMON LOG ===")
            print(logfile.read_text(errors="replace")[-4000:])
            raise

        finally:
            if proc is not None:
                stop_process_group(proc)




if __name__ == "__main__":
    main()
