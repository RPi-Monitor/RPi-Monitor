#!/usr/bin/env python3
"""
Base test class for RPi-Monitor E2E tests

This class contains common utility functions used across all test files.
"""

import json
import os
import signal
import socket
import subprocess
import time
from pathlib import Path
from typing import Any, List, Optional, Union

import pexpect
import requests


class RPiMonitorTestBase:
    """
    Base class for RPi-Monitor E2E tests.

    This class contains common utility functions used across all test files
    for launching, monitoring, and interacting with RPi-Monitor processes.
    """

    # Configuration - update these paths as needed
    ROOT_DIR = Path(__file__).parent.parent.absolute()
    BINARY = f"{ROOT_DIR}/src/usr/bin/rpimonitord"
    UNIT_LOADER = f"{ROOT_DIR}/tests/perl_unit.pl"

    # Set by start_repl / launch_rpimonitord. Declared here because pytest
    # refuses to collect a test class that defines __init__.
    repl: Optional[pexpect.spawn] = None
    proc: Optional[subprocess.Popen] = None

    @classmethod
    def _build_perl_command(cls, binary: str, args: List[str]) -> List[str]:
        """Prefix Devel::Cover instrumentation when PERL_COVERAGE=1; else run binary directly."""
        if os.environ.get("PERL_COVERAGE", "0") == "1":
            cover_db_path = f"{cls.ROOT_DIR}/tests/cover_db"
            # -silent: the banner goes to STDOUT and would corrupt the SUT output.
            # +ignore: the loader is test tooling; recorded under a cwd-relative path,
            # its MD5 digest fails in later runs and warns on STDERR.
            options = f"-db,{cover_db_path},-silent,1,+ignore,perl_unit\\.pl"
            return ["perl", f"-MDevel::Cover={options}", binary] + list(args)
        return [binary] + list(args)

    def start_repl(self, timeout: int = 10) -> pexpect.spawn:
        """Launch rpimonitord -i and return the pexpect process."""
        argv = self._build_perl_command(self.BINARY, ["-i"])
        self.repl = pexpect.spawn(argv[0], args=argv[1:], encoding="utf-8", timeout=timeout)
        return self.repl

    @classmethod
    def run_perl_unit(cls, code: str, *args: str) -> Any:
        """Evaluate Perl ``code`` against rpimonitord's packages, main section excluded.

        ``code`` sees ``args`` in ``@ARGV``; its result comes back decoded from JSON.
        """
        command = cls._build_perl_command(cls.UNIT_LOADER, [cls.BINARY, code, *args])
        result = subprocess.run(command, capture_output=True, text=True, check=False)
        # Raised by hand: CalledProcessError would hide the Perl error held in stderr.
        if result.returncode != 0:
            raise RuntimeError(f"perl_unit.pl exited {result.returncode}: {result.stderr.strip()}")
        return json.loads(result.stdout)

    def teardown_method(self):
        """Close the processes after each test."""
        if self.repl is not None and self.repl.isalive():
            self.repl.close(force=True)
        if self.proc is not None:
            self.terminate_rpimonitord_processes()

    def launch_rpimonitord(self, options: Union[str, List[str]], test_dir: str) -> subprocess.Popen:
        """
        Launch rpimonitord with specified configuration
        Returns Popen process object
        """
        # Ensure options is a list
        if isinstance(options, str):
            options = [options]
        elif not isinstance(options, list):
            options = list(options)

        command = self._build_perl_command(self.BINARY, options)
        # Force C locale to silence Perl locale warnings, but preserve PATH and
        # HOME so the SUT can spawn `echo`/`date`/etc. from unqualified config
        # sources without relying on libc's _PATH_DEFPATH fallback.
        env = {"LC_ALL": "C", "LANGUAGE": "C", "LANG": "C", "LC_CTYPE": "C"}
        for var in ("PATH", "HOME"):
            if var in os.environ:
                env[var] = os.environ[var]

        # No `with` block (R1732): the daemon must outlive this call; teardown_method terminates it.
        self.proc = subprocess.Popen(  # pylint: disable=R1732
            command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, cwd=test_dir, env=env
        )
        return self.proc

    def check_rpimonitord_started(self, timeout: int = 0.1) -> bool:
        """Check that the process started correctly"""
        try:
            # Wait briefly to see whether the process fails immediately
            self.proc.wait(timeout=timeout)
            # Reaching here means the process has exited (startup failure)
            return False
        except subprocess.TimeoutExpired:
            # TimeoutExpired means the process is still running (success)
            return True

    def terminate_rpimonitord_processes(self, pid_file: str = None):
        """SIGTERM (then SIGKILL) self.proc; also terminate any daemon PID in ``pid_file``.

        ``-b`` mode double-forks, so the actual daemon is reparented away from self.proc.
        Pass its PID file to reach it via the same teardown path.
        """
        # First pass: send SIGTERM to allow graceful shutdown.
        if self.proc is not None:
            try:
                self.proc.send_signal(signal.SIGTERM)
            except (ProcessLookupError, PermissionError):
                pass
            try:
                self.proc.wait(timeout=3)
            except subprocess.TimeoutExpired:
                try:
                    self.proc.kill()
                    self.proc.wait()
                except (ProcessLookupError, PermissionError):
                    pass

        if pid_file:
            self._terminate_pid_file_daemon(pid_file)

    @staticmethod
    def _terminate_pid_file_daemon(pid_file: str) -> None:
        """SIGTERM then SIGKILL the daemon and its process group.

        ``-b`` daemonize calls ``setsid()`` so the daemon leads its own group; a
        signal to ``-pgid`` reaches the leader plus any respawner children.
        """
        if not os.path.exists(pid_file):
            return
        # SUT writes multiple PIDs (master + monitor respawner) space-separated;
        # first one is the daemon session leader, its pgid covers the rest.
        try:
            with open(pid_file, "r", encoding="utf-8") as f:
                pid = int(f.read().split()[0])
        except (OSError, ValueError, IndexError):
            return
        try:
            pgid = os.getpgid(pid)
        except ProcessLookupError:
            return
        for sig in (signal.SIGTERM, signal.SIGKILL):
            try:
                os.killpg(pgid, sig)
            except ProcessLookupError:
                return
            deadline = time.time() + 2
            while time.time() < deadline:
                try:
                    os.kill(pid, 0)
                except ProcessLookupError:
                    return
                time.sleep(0.05)

    def wait_for_port(self, host: str = "127.0.0.1", *, port: int, timeout: float = 1.0, interval: float = 0.1) -> bool:
        """Wait for a TCP ``port`` on ``host`` to accept connections.

        ``port`` is keyword-only and required -- each test picks a randomized
        port under xdist, so a default is an easy footgun (would silently
        target whatever daemon happens to be listening on 8888).
        """
        start_time = time.perf_counter()

        while time.perf_counter() - start_time < timeout:
            try:
                with socket.create_connection((host, int(port)), timeout=interval):
                    return True
            except (OSError, ConnectionRefusedError):
                time.sleep(interval)
        return False

    # Ceiling for _scaled_timeout -- several tests do 2-3 sequential scaled
    # waits inside a single case, so any single wait must stay well under
    # the 60 s pytest-timeout. 20 s gives ~3x the serial cycle time on this
    # box, which measurably covers observed xdist contention without letting
    # a single poll eat the whole test budget.
    _SCALED_TIMEOUT_CAP: float = 20.0

    @classmethod
    def _scaled_timeout(cls, base: float) -> float:
        """Scale ``base`` seconds by the xdist worker count, capped.

        Fork contention under ``pytest -n auto`` stretches nominal SUT
        cycle times, but sub-linearly -- 20 workers do not make a cycle
        20x slower. Serial runs (env var unset or "1") pass through
        unchanged; parallel runs get up to ``_SCALED_TIMEOUT_CAP`` seconds.
        """
        try:
            workers = max(1, int(os.environ.get("PYTEST_XDIST_WORKER_COUNT", "1")))
        except ValueError:
            workers = 1
        return min(base * workers, cls._SCALED_TIMEOUT_CAP)

    @classmethod
    def poll_file_contains(
        cls, path: str, needle: str, timeout: float, min_count: int = 1, interval: float = 0.1, scale: bool = True
    ):
        """Wait for ``path`` to exist AND contain ``needle`` at least ``min_count`` times.

        Returns file content on success, None on timeout. ``timeout`` is
        scaled through ``_scaled_timeout`` by default so xdist runs get a
        proportionally larger budget; pass ``scale=False`` for the raw
        deadline.
        """
        deadline = time.time() + (cls._scaled_timeout(timeout) if scale else timeout)
        while time.time() < deadline:
            if os.path.exists(path):
                with open(path, "r", encoding="utf-8") as f:
                    content = f.read()
                if content.count(needle) >= min_count:
                    return content
            time.sleep(interval)
        return None

    @classmethod
    def wait_for_json_key(
        cls, url: str, key: str, timeout: float, interval: float = 0.5, scale: bool = True, request_timeout: float = 2.0
    ) -> Optional[dict]:
        """Poll ``url`` until the parsed JSON body contains ``key``.

        Returns the parsed dict on success, ``None`` on timeout. ``timeout``
        is scaled via ``_scaled_timeout`` by default so xdist runs get a
        proportionally larger budget; pass ``scale=False`` for the raw
        deadline. Transient connection errors before the daemon binds are
        swallowed and retried.
        """
        deadline = time.time() + (cls._scaled_timeout(timeout) if scale else timeout)
        while time.time() < deadline:
            try:
                response = requests.get(url, timeout=request_timeout)
                if response.status_code == 200:
                    body = response.text.strip()
                    if body:
                        data = response.json()
                        if key in data:
                            return data
            except (requests.exceptions.RequestException, ValueError):
                pass
            time.sleep(interval)
        return None

    @staticmethod
    def wait_for_file_creation(file_path: str, timeout: int = 2, interval: float = 0.1) -> bool:
        """
        Wait for a file to be created within a specified timeout.

        Args:
            file_path: Path to the file to wait for
            timeout: Maximum time to wait in seconds
            interval: Polling interval in seconds

        Returns:
            True if file was created within timeout, False otherwise
        """
        start_time = time.time()

        while time.time() - start_time < timeout:
            if os.path.exists(file_path):
                return True
            time.sleep(interval)

        return False

    @staticmethod
    def wait_for_file_modification(file_path: str, timeout: float, interval: float = 0.1) -> bool:
        """Wait for ``file_path`` to be modified (mtime bump or size change).

        ``timeout`` is required -- every caller has its own SUT-cycle budget,
        so a default just papers over slow-poll bugs.
        """
        if not os.path.exists(file_path):
            return False

        start_time = time.time()
        last_modified = os.path.getmtime(file_path)
        last_size = os.path.getsize(file_path)

        while time.time() - start_time < timeout:
            current_modified = os.path.getmtime(file_path)
            current_size = os.path.getsize(file_path)
            if current_modified > last_modified or current_size != last_size:
                return True
            time.sleep(interval)

        return False

    @staticmethod
    def read_json_file(file_path: str) -> dict:
        """Read and parse any JSON file"""
        if os.path.exists(file_path):
            with open(file_path, "r", encoding="utf-8") as f:
                return json.load(f)
        return {}

    @staticmethod
    def write_config(temp_dir: str, content: str, name: str = "rpimonitor.conf") -> str:
        """Write config content to ``{temp_dir}/{name}`` and return the path."""
        path = os.path.join(temp_dir, name)
        with open(path, "w", encoding="utf-8") as f:
            f.write(content)
        return path

    @staticmethod
    def process_perl_output(output: str) -> str:
        """
        Process Perl output to remove locale warnings and normalize line endings
        """
        # Remove locale warnings
        lines = output.split("\n")
        filtered_lines = [line for line in lines if "perl: warning: Setting locale failed" not in line.lower()]
        return "\n".join(filtered_lines).strip()
