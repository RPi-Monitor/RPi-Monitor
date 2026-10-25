#!/usr/bin/env python3
"""
E2E tests for RPi-Monitor CLI options
Tests all CLI options for both Perl and Rust implementations
"""

import os
import signal
import tempfile
import time

import psutil
import pytest
from test_base import RPiMonitorTestBase


class TestCLIOptions(RPiMonitorTestBase):
    """Test class for RPi-Monitor CLI options functionality"""

    @pytest.mark.parametrize("addr_option", ["-a", "--addr"])
    def test_addr_option(self, addr_option):
        """Test -a/--addr option for web server bind address"""

        with tempfile.TemporaryDirectory() as temp_dir:
            # Create minimal configuration
            config_file = os.path.join(temp_dir, "cli_options_1.conf")
            with open(config_file, "w", encoding="utf-8") as f:
                f.write("""
daemon.delay=1
daemon.datastore=/tmp/test_addr
daemon.webroot=/tmp/test_web
""")

            # Test with specific address
            test_address = "127.0.0.1"
            self.launch_rpimonitord(["-c", config_file, addr_option, test_address, "-s"], temp_dir)

            # Wait for process to complete
            stdout, _ = self.proc.communicate(timeout=3)
            return_code = self.proc.returncode

            # Cleanup processes
            self.terminate_rpimonitord_processes()

            # Convert to string
            output_str = stdout.decode() if isinstance(stdout, bytes) else stdout

            # Validate that address is set correctly in configuration
            assert return_code in [0, 2], f"Process failed with return code {return_code}"
            assert f"'addr' => '{test_address}'" in output_str, f"Address not set correctly"

    @pytest.mark.parametrize("background_option", ["-b", "--background"])
    def test_background_option(self, background_option):
        """Test -b/--background option with PID file"""

        with tempfile.TemporaryDirectory() as temp_dir:
            # Create minimal configuration
            config_file = os.path.join(temp_dir, "cli_options_2.conf")
            with open(config_file, "w", encoding="utf-8") as f:
                f.write("""
daemon.delay=1
daemon.datastore=/tmp/test_background
daemon.webroot=/tmp/test_web
""")

            # Create PID file path
            pid_file = os.path.join(temp_dir, "test.pid")

            # Launch in background mode
            self.launch_rpimonitord(["-c", config_file, background_option, pid_file], temp_dir)

            self.wait_for_port("127.0.0.1", "8888")

            # Check if PID file was created
            assert os.path.exists(pid_file), "PID file not created"

            # Terminate process
            # First pass: send SIGINT to allow graceful shutdown
        for proc in psutil.process_iter(["pid", "name", "cmdline"]):
            if proc.info["name"] == "rpimonitord" or (
                proc.info["cmdline"] and "rpimonitord" in " ".join(proc.info["cmdline"])
            ):
                try:
                    proc.send_signal(signal.SIGTERM)
                except (
                    psutil.NoSuchProcess,
                    psutil.AccessDenied,
                    psutil.ZombieProcess,
                ):
                    pass

        # Wait a short time for processes to shutdown gracefully
        time.sleep(0.1)

        # Second pass: force kill any remaining processes
        for proc in psutil.process_iter(["pid", "name", "cmdline"]):
            if proc.info["name"] == "rpimonitord" or (
                proc.info["cmdline"] and "rpimonitord" in " ".join(proc.info["cmdline"])
            ):
                try:
                    proc.kill()
                except (
                    psutil.NoSuchProcess,
                    psutil.AccessDenied,
                    psutil.ZombieProcess,
                ):
                    pass

    @pytest.mark.parametrize("delay_option", ["-d", "--delay"])
    def test_delay_option(self, delay_option):
        """Test -d/--delay option for check interval"""

        with tempfile.TemporaryDirectory() as temp_dir:
            # Create minimal configuration
            config_file = os.path.join(temp_dir, "cli_options_3.conf")
            with open(config_file, "w", encoding="utf-8") as f:
                f.write("""
daemon.datastore=/tmp/test_delay
daemon.webroot=/tmp/test_web
""")

            # Test with specific delay
            test_delay = "3"
            self.launch_rpimonitord(["-c", config_file, delay_option, test_delay, "-s"], temp_dir)

            # Wait for process to complete
            stdout, _ = self.proc.communicate(timeout=3)
            return_code = self.proc.returncode

            # Cleanup processes
            self.terminate_rpimonitord_processes()

            # Convert to string
            output_str = stdout.decode() if isinstance(stdout, bytes) else stdout

            # Validate that delay is set correctly
            assert return_code in [0, 2], f"Process failed with return code {return_code}"
            assert f"'delay' => '{test_delay}'" in output_str, f"Delay not set correctly"

    @pytest.mark.parametrize("keep_option", ["-k", "--keep"])
    def test_keep_option(self, keep_option):
        """Test -k/--keep option for log file preservation"""

        with tempfile.TemporaryDirectory() as temp_dir:
            # Create a log file first
            log_file = os.path.join(temp_dir, "test.log")
            with open(log_file, "w", encoding="utf-8") as f:
                f.write("Existing log content\n")

            # Launch with keep option
            self.launch_rpimonitord([keep_option, "-l", log_file], temp_dir)

            # Verify log file still exists (was not deleted)
            assert os.path.exists(log_file), f"Log file was deleted despite keep option"

            # Terminate process
            self.terminate_rpimonitord_processes()

    @pytest.mark.parametrize("logfile_option", ["-l", "--logfile"])
    def test_logfile_option(self, logfile_option):
        """Test -l/--logfile option for log file specification"""

        with tempfile.TemporaryDirectory() as temp_dir:
            # Create minimal configuration
            config_file = os.path.join(temp_dir, "cli_options_5.conf")
            with open(config_file, "w", encoding="utf-8") as f:
                f.write("""
daemon.delay=1
daemon.datastore=/tmp/test_logfile
daemon.webroot=/tmp/test_web
""")

            # Create custom log file path
            custom_log = os.path.join(temp_dir, "custom.log")

            # Launch with custom log file
            self.launch_rpimonitord(["-c", config_file, logfile_option, custom_log], temp_dir)

            # Wait a moment
            self.check_rpimonitord_started()

            # Terminate process
            self.wait_for_port("127.0.0.1", "8888")

            # Verify custom log file was created
            assert os.path.exists(custom_log), "Custom log file not created"

            # Cleanup
            if os.path.exists(custom_log):
                os.unlink(custom_log)

    @pytest.mark.parametrize("noserver_option", ["-n", "--noserver"])
    def test_noserver_option(self, noserver_option):
        """Test -n/--noserver option (no web server)"""

        with tempfile.TemporaryDirectory() as temp_dir:
            # Create minimal configuration
            config_file = os.path.join(temp_dir, "cli_options_6.conf")
            with open(config_file, "w", encoding="utf-8") as f:
                f.write("""
daemon.delay=1
daemon.datastore=/tmp/test_noserver
daemon.webroot=/tmp/test_web
""")

            # Launch without server
            self.launch_rpimonitord(["-c", config_file, noserver_option, "-s"], temp_dir)

            # Wait for process to complete
            stdout, _ = self.proc.communicate(timeout=3)
            return_code = self.proc.returncode

            # Cleanup processes
            self.terminate_rpimonitord_processes()

            # Convert to string
            output_str = stdout.decode() if isinstance(stdout, bytes) else stdout

            # Validate that noserver flag is set
            assert return_code in [0, 2], f"Process failed with return code {return_code}"
            assert "'noserver' => 1" in output_str, f"Noserver flag not set correctly"

    @pytest.mark.parametrize("port_option", ["-p", "--port"])
    def test_port_option(self, port_option):
        """Test -p/--port option for web server port"""

        with tempfile.TemporaryDirectory() as temp_dir:
            # Create minimal configuration
            config_file = os.path.join(temp_dir, "cli_options_7.conf")
            with open(config_file, "w", encoding="utf-8") as f:
                f.write("""
daemon.delay=1
daemon.datastore=/tmp/test_port
daemon.webroot=/tmp/test_web
""")

            # Test with specific port
            test_port = "9999"
            self.launch_rpimonitord(["-c", config_file, port_option, test_port, "-s"], temp_dir)

            # Wait for process to complete
            stdout, _ = self.proc.communicate(timeout=3)
            return_code = self.proc.returncode

            # Cleanup processes
            self.terminate_rpimonitord_processes()

            # Convert to string
            output_str = stdout.decode() if isinstance(stdout, bytes) else stdout

            # Validate that port is set correctly
            assert return_code in [0, 2], f"Process failed with return code {return_code}"
            assert f"'port' => '{test_port}'" in output_str, f"Port not set correctly"

    @pytest.mark.parametrize("readonly_option", ["-r", "--readonly"])
    def test_readonly_option(self, readonly_option):
        """Test -r/--readonly option (read only mode)"""

        with tempfile.TemporaryDirectory() as temp_dir:
            # Create minimal configuration
            config_file = os.path.join(temp_dir, "cli_options_8.conf")
            with open(config_file, "w", encoding="utf-8") as f:
                f.write("""
daemon.delay=1
daemon.datastore=/tmp/test_readonly
daemon.webroot=/tmp/test_web
""")

            # Launch in readonly mode
            self.launch_rpimonitord(["-c", config_file, readonly_option, "-s"], temp_dir)

            # Wait for process to complete
            stdout, _ = self.proc.communicate(timeout=3)
            return_code = self.proc.returncode

            # Cleanup processes
            self.terminate_rpimonitord_processes()

            # Convert to string
            output_str = stdout.decode() if isinstance(stdout, bytes) else stdout

            # Validate that readonly flag is set
            assert return_code in [0, 2], f"Process failed with return code {return_code}"
            assert "'readonly' => 1" in output_str, f"Readonly flag not set correctly"

    @pytest.mark.parametrize("timeout_option", ["-t", "--timeout"])
    def test_timeout_option(self, timeout_option):
        """Test -t/--timeout option for command timeout"""

        with tempfile.TemporaryDirectory() as temp_dir:
            # Create minimal configuration
            config_file = os.path.join(temp_dir, "cli_options_9.conf")
            with open(config_file, "w", encoding="utf-8") as f:
                f.write("""
daemon.delay=1
daemon.datastore=/tmp/test_timeout
daemon.webroot=/tmp/test_web
""")

            # Test with specific timeout
            test_timeout = "15"
            self.launch_rpimonitord(["-c", config_file, timeout_option, test_timeout, "-s"], temp_dir)

            # Wait for process to complete
            stdout, _ = self.proc.communicate(timeout=3)
            return_code = self.proc.returncode

            # Cleanup processes
            self.terminate_rpimonitord_processes()

            # Convert to string
            output_str = stdout.decode() if isinstance(stdout, bytes) else stdout

            # Validate that timeout is set correctly
            assert return_code in [0, 2], f"Process failed with return code {return_code}"
            assert f"'timeout' => '{test_timeout}'" in output_str, f"Timeout not set correctly"

    def test_verbose_option(self):
        """Test -v option for verbosity levels"""

        with tempfile.TemporaryDirectory() as temp_dir:
            # Create minimal configuration
            config_file = os.path.join(temp_dir, "cli_options_0.conf")
            with open(config_file, "w", encoding="utf-8") as f:
                f.write("""
daemon.delay=1
daemon.datastore=/tmp/test_verbose
daemon.webroot=/tmp/test_web
""")

            # Test with different verbosity levels
            for v_level in ["-v", "-vv", "-vvv"]:
                self.launch_rpimonitord(["-c", config_file, v_level, "-s"], temp_dir)

                # For -s (show) mode, wait for process to complete (it should exit immediately)
                try:
                    self.proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    self.terminate_rpimonitord_processes()
                    self.proc = None
                
                return_code = self.proc.returncode

                # Cleanup processes if still running
                self.terminate_rpimonitord_processes()

                # Just verify it doesn't crash with different verbosity levels
                assert return_code in [0, 2], (
                    f"Verbose mode {v_level} failed with return code {return_code}"
                )


if __name__ == "__main__":
    # Run tests directly for debugging
    pytest.main([__file__, "-v", "-k", "Perl"])
