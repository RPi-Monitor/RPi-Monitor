#!/usr/bin/env python3
"""Regression tests for vcgencmd discovery in RPi-Monitor templates."""

import os
import subprocess
import tempfile
import unittest
from pathlib import Path

TEMPLATES = Path(__file__).resolve().parents[1] / (
    "src/etc/rpimonitor/template"
)

SOURCES = (
    ("cpu.conf", "dynamic.2", "voltage"),
    ("cpu_arch.conf", "dynamic.2", "voltage"),
    ("temperature_xbian.conf", "dynamic.1", "temperature"),
)

PERL_EXTRACT = r"""
use strict;
use warnings;

my ($regexp, $postprocess) = @ARGV;
my @values;

while (<STDIN>) {
    /$regexp/ and (@values = /$regexp/) or next;
}

my $value = $values[0];

if (defined($value) && length($postprocess)) {
    $value =~ /(.*)/;
    $value = eval($postprocess);
    die $@ if $@;
}

print $value if defined($value);
"""


def read_template(filename):
    config = {}

    for line in (TEMPLATES / filename).read_text().splitlines():
        if line and not line.startswith("#") and "=" in line:
            key, value = line.split("=", 1)
            config[key] = value

    return config


class VcgencmdFallbackTests(unittest.TestCase):

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)

        self.root = Path(self.temp.name)
        self.bin_dir = self.root / "bin"
        self.bin_dir.mkdir()

        # Keep command discovery isolated from the real system PATH.
        (self.bin_dir / "sh").symlink_to("/bin/sh")

        self.legacy = self.root / "opt/vc/bin/vcgencmd"
        self.modern = self.root / "usr/bin/vcgencmd"
        self.path_command = self.bin_dir / "vcgencmd"

    def create_command(self, path, voltage, temperature):
        path.parent.mkdir(parents=True, exist_ok=True)

        script = (
            "#!/bin/sh\n"
            'case "$1 $2" in\n'
            f'  "measure_volts core") echo "volt={voltage}V" ;;\n'
            f'  "measure_temp ") echo "temp={temperature}C" ;;\n'
            '  *) exit 1 ;;\n'
            "esac\n"
        )

        path.write_text(script)
        path.chmod(0o755)

    def collect(self, filename, key):
        config = read_template(filename)
        source = config[f"{key}.source"]

        # Redirect absolute legacy/new paths to temporary fixtures.
        source = source.replace(
            "/opt/vc/bin/vcgencmd", str(self.legacy)
        )
        source = source.replace(
            "/usr/bin/vcgencmd", str(self.modern)
        )

        env = os.environ.copy()
        env["PATH"] = str(self.bin_dir)

        command = subprocess.run(
            source,
            shell=True,
            executable="/bin/sh",
            capture_output=True,
            text=True,
            timeout=5,
            env=env,
        )

        # Like rpimonitord, parse stdout regardless of exit code.
        # No matching output means no metric value.

        result = subprocess.run(
            [
                "perl",
                "-e",
                PERL_EXTRACT,
                config[f"{key}.regexp"],
                config[f"{key}.postprocess"],
            ],
            input=command.stdout,
            capture_output=True,
            text=True,
            check=True,
            timeout=5,
        )

        value = result.stdout.strip()
        return float(value) if value else None

    def check_sources(self, voltage, temperature):
        for filename, key, metric in SOURCES:
            expected = (
                voltage if metric == "voltage" else temperature
            )

            with self.subTest(template=filename):
                self.assertEqual(
                    self.collect(filename, key), expected
                )

    def test_legacy_path(self):
        self.create_command(self.legacy, "0.9000", "48.0")
        self.check_sources(0.9, 48.0)

    def test_modern_path(self):
        self.create_command(self.modern, "0.8456", "51.6")
        self.check_sources(0.8456, 51.6)

    def test_prefer_legacy_when_both_available(self):
        self.create_command(self.legacy, "0.9000", "48.0")
        self.create_command(self.modern, "0.8456", "51.6")
        self.check_sources(0.9, 48.0)

    def test_custom_path_fallback(self):
        self.create_command(
            self.path_command, "0.8125", "45.5"
        )
        self.check_sources(0.8125, 45.5)

    def test_prefer_modern_over_custom_path(self):
        self.create_command(
            self.modern, "0.8456", "51.6"
        )
        self.create_command(
            self.path_command, "0.8125", "45.5"
        )
        self.check_sources(0.8456, 51.6)

    def test_missing_vcgencmd(self):
        self.check_sources(None, None)

    def test_executable_failure_has_no_metric(self):
        self.modern.parent.mkdir(parents=True, exist_ok=True)
        self.modern.write_text("#!/bin/sh\nexit 1\n")
        self.modern.chmod(0o755)

        self.check_sources(None, None)

    def test_unrecognized_output_has_no_metric(self):
        self.create_command(
            self.modern, "unavailable", "unavailable"
        )

        self.check_sources(None, None)

    def test_no_leading_whitespace_in_sources(self):
        for filename, key, _ in SOURCES:
            with self.subTest(template=filename):
                source = read_template(filename)[f"{key}.source"]
                self.assertEqual(source, source.lstrip())

    def test_existing_metric_contract(self):
        for filename, key, metric in SOURCES:
            config = read_template(filename)

            with self.subTest(template=filename):
                if metric == "voltage":
                    self.assertEqual(
                        config[f"{key}.name"], "cpu_voltage"
                    )
                    self.assertEqual(
                        config[f"{key}.regexp"], r"(\d+.\d+)V"
                    )
                    self.assertEqual(
                        config[f"{key}.rrd"], ""
                    )
                else:
                    self.assertEqual(
                        config[f"{key}.name"], "soc_temp"
                    )
                    self.assertEqual(
                        config[f"{key}.rrd"], "GAUGE"
                    )

        cpu = read_template("cpu.conf")
        self.assertEqual(cpu["snmp.cpu_voltage.id"], "8")
        self.assertEqual(
            cpu["snmp.cpu_voltage.postprocess"], "$1*100"
        )


if __name__ == "__main__":
    unittest.main()
