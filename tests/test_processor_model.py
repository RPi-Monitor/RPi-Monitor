#!/usr/bin/env python3
"""Regression tests for processor identification in version.conf."""

import subprocess
import unittest
from pathlib import Path

TEMPLATE = (
    Path(__file__).resolve().parents[1]
    / "src/etc/rpimonitor/template/version.conf"
)

# Match the Perl line-by-line extraction used by rpimonitord.
PERL_EXTRACT = r"""
use strict;
use warnings;

my $regexp = shift @ARGV;
my $value;

while (<STDIN>) {
    if (/$regexp/) {
        $value = $1;
    }
}

print $value if defined($value);
"""


def read_template():
    config = {}

    for line in TEMPLATE.read_text().splitlines():
        if not line or line.startswith("#") or "=" not in line:
            continue

        key, value = line.split("=", 1)
        config[key] = value

    return config


def extract_processor(config, cpuinfo):
    result = subprocess.run(
        [
            "perl",
            "-e",
            PERL_EXTRACT,
            config["static.4.regexp"],
        ],
        input=cpuinfo,
        text=True,
        capture_output=True,
        check=True,
        timeout=5,
    )

    return result.stdout or None


class ProcessorModelTests(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.config = read_template()

    def test_legacy_processor(self):
        cpuinfo = "Processor\t: ARMv7 Processor rev 4\n"
        self.assertEqual(
            extract_processor(self.config, cpuinfo),
            "ARMv7 Processor rev 4",
        )

    def test_model_name(self):
        cpuinfo = "model name\t: ARM Cortex-A72\n"
        self.assertEqual(
            extract_processor(self.config, cpuinfo),
            "ARM Cortex-A72",
        )

    def test_legacy_hardware(self):
        cpuinfo = "Hardware\t: BCM2835\n"
        self.assertEqual(
            extract_processor(self.config, cpuinfo),
            "BCM2835",
        )

    def test_raspberry_pi_5(self):
        cpuinfo = (
            "processor\t: 0\n"
            "CPU part\t: 0xd0b\n"
            "Revision\t: d04170\n"
            "Model\t: Raspberry Pi 5 Model B Rev 1.0\n"
        )

        self.assertEqual(
            extract_processor(self.config, cpuinfo),
            "Raspberry Pi 5 Model B Rev 1.0",
        )

    def test_missing_identification(self):
        cpuinfo = (
            "processor\t: 0\n"
            "CPU part\t: 0xd0b\n"
        )

        self.assertIsNone(
            extract_processor(self.config, cpuinfo)
        )

    def test_existing_metric_contract(self):
        self.assertEqual(
            self.config["static.4.name"], "processor"
        )
        self.assertEqual(
            self.config["static.4.source"], "/proc/cpuinfo"
        )
        self.assertEqual(
            self.config["static.4.postprocess"], ""
        )
        self.assertEqual(
            self.config["snmp.processor.id"], "4"
        )
        self.assertEqual(
            self.config["snmp.processor.type"], "string"
        )


if __name__ == "__main__":
    unittest.main()
