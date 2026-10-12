#!/usr/bin/env python3
"""Regression tests for the CPU frequency template."""

import subprocess
import tempfile
import unittest
from pathlib import Path

TEMPLATE = (
    Path(__file__).resolve().parents[1]
    / "src/etc/rpimonitor/template/cpu.conf"
)

SYSFS = "/sys/devices/system/cpu/cpu0/cpufreq"

# Use Perl's own regexp and postprocess syntax, as rpimonitord does.
PERL_EXTRACT = r"""
my ($regexp, $postprocess) = @ARGV;
my @values;

while (<STDIN>) {
    /$regexp/ and (@values = /$regexp/) or next;
}

my $val = $values[0];

if (defined($val) && $postprocess) {
    $val =~ /(.*)/;
    $val = eval($postprocess);
    die $@ if $@;
}

print $val if defined($val);
"""


def read_template():
    config = {}
    for line in TEMPLATE.read_text().splitlines():
        if line and not line.startswith("#") and "=" in line:
            key, value = line.split("=", 1)
            config[key] = value
    return config


def collect_frequency(config, root):
    source = config["dynamic.1.source"]
    assert SYSFS in source, "Expected CPU frequency sysfs source"

    # Redirect only sysfs paths to temporary test fixtures.
    source = source.replace(SYSFS, str(root))

    # RPi-Monitor reads regular files directly and executes commands.
    if source.startswith("/") and " " not in source:
        path = Path(source)
        output = path.read_text() if path.is_file() else ""
    else:
        result = subprocess.run(
            source,
            shell=True,
            capture_output=True,
            text=True,
            timeout=5,
        )
        if result.returncode != 0:
            raise AssertionError(result.stderr)
        output = result.stdout

    result = subprocess.run(
        [
            "perl",
            "-e",
            PERL_EXTRACT,
            config["dynamic.1.regexp"],
            config["dynamic.1.postprocess"],
        ],
        input=output,
        capture_output=True,
        text=True,
        check=True,
        timeout=5,
    )

    value = result.stdout.strip()
    return float(value) if value else None


class CPUFrequencyTests(unittest.TestCase):

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / "cpufreq"
        self.root.mkdir()
        self.config = read_template()

    def write_frequency(self, name, value):
        (self.root / name).write_text(f"{value}\n")

    def test_legacy_cpuinfo_source(self):
        self.write_frequency("cpuinfo_cur_freq", 1500000)
        self.assertEqual(
            collect_frequency(self.config, self.root), 1500.0
        )

    def test_pi5_scaling_fallback(self):
        self.write_frequency("scaling_cur_freq", 2400000)
        self.assertEqual(
            collect_frequency(self.config, self.root), 2400.0
        )

    def test_prefer_cpuinfo_when_both_exist(self):
        self.write_frequency("cpuinfo_cur_freq", 1800000)
        self.write_frequency("scaling_cur_freq", 2400000)
        self.assertEqual(
            collect_frequency(self.config, self.root), 1800.0
        )

    def test_no_frequency_source(self):
        self.assertIsNone(
            collect_frequency(self.config, self.root)
        )

    def test_existing_metric_contract(self):
        self.assertEqual(
            self.config["dynamic.1.name"], "cpu_frequency"
        )
        self.assertEqual(self.config["dynamic.1.rrd"], "")
        self.assertEqual(
            self.config["snmp.cpu_frequency.id"], "7"
        )
        self.assertEqual(
            self.config["dynamic.1.postprocess"], "$1/1000"
        )


if __name__ == "__main__":
    unittest.main()
