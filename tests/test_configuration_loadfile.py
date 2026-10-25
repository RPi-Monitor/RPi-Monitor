#!/usr/bin/env python3
"""
Unit tests for Configuration::LoadFile, run without starting rpimonitord
"""

import tempfile

import pytest
from test_base import RPiMonitorTestBase

# Unblessed copy of the state: JSON refuses to encode a blessed object.
LOAD_FILES = "my $c = Configuration->new; $c->LoadFile($_) foreach @ARGV; +{ %$c }"


class TestConfigurationLoadFile(RPiMonitorTestBase):
    """Test class for the configuration file parser"""

    def test_should_build_one_entry_when_dotted_keys_share_index(self):
        """Test that keys sharing a dotted index are gathered into a single entry"""

        with tempfile.TemporaryDirectory() as temp_dir:
            conf = self.write_config(temp_dir, "dynamic.1.name=load\ndynamic.1.source=/proc/loadavg\n")

            config = self.run_perl_unit(LOAD_FILES, conf)

        assert config["dynamic"] == [{"name": "load", "source": "/proc/loadavg"}]

    def test_should_append_entry_when_second_file_reuses_index(self):
        """Test that an index reused by another file adds an entry instead of overwriting"""

        with tempfile.TemporaryDirectory() as temp_dir:
            first = self.write_config(temp_dir, "dynamic.1.name=first\n", "first.conf")
            second = self.write_config(temp_dir, "dynamic.1.name=second\n", "second.conf")

            config = self.run_perl_unit(LOAD_FILES, first, second)

        assert config["dynamic"] == [{"name": "first"}, {"name": "second"}]


if __name__ == "__main__":
    # Run tests directly for debugging
    pytest.main([__file__, "-v"])
