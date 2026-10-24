#!/usr/bin/env python3
"""
E2E tests for rpimonitord CLI version options (-V/--Version)
"""

import tempfile

import pytest
from test_base import RPiMonitorTestBase


class TestRPiMonitorVersion(RPiMonitorTestBase):
    """Test class for rpimonitord version functionality"""

    @pytest.mark.parametrize("option", ["-V", "--Version"])
    def test_should_display_version_when_version_option_provided(self, option):
        """Test that rpimonitord displays version message when -V or --Version option is provided"""

        with tempfile.TemporaryDirectory() as temp_dir:
            self.launch_rpimonitord([option], temp_dir)

            # SUT emits version via `die` -> output on STDERR. Native Perl exits
            # 255; under Devel::Cover the END handler rewrites $? to 2. Both are
            # die-shaped exits -- the regression guarded against is `print + exit(0)`.
            _, stderr = self.proc.communicate()
            assert self.proc.returncode in (2, 255), f"Perl: expected die exit (2 or 255), got {self.proc.returncode}"

            expected_output = f"{self.BINARY} version {{DEVELOPMENT}}"
            assert stderr.decode("utf-8").strip() == expected_output, "Version output does not match expected output."


if __name__ == "__main__":
    # Run tests directly for debugging
    pytest.main([__file__, "-v"])
