#!/usr/bin/env python3
"""Regression tests for boot filesystem detection."""

import os
import subprocess
import tempfile
import unittest
from pathlib import Path

TEMPLATE = (
    Path(__file__).resolve().parents[1]
    / "src/etc/rpimonitor/template/sdcard.conf"
)

PERL_EXTRACT = r"""
use strict;
use warnings;

my ($regexp, $postprocess) = @ARGV;
my $value;

while (<STDIN>) {
    if (/$regexp/) {
        $value = $1;
    }
}

if (defined($value) && length($postprocess)) {
    $value =~ /(.*)/;
    $value = eval($postprocess);
    die $@ if $@;
}

print $value if defined($value);
"""

MOCK_MOUNTPOINT = r"""#!/bin/sh
[ "$1" = "-q" ] || exit 2

case ":${RPIMON_TEST_MOUNTS}:" in
    *":$2:"*) exit 0 ;;
esac

exit 1
"""

MOCK_DF = r"""#!/bin/sh
target=
for arg do
    target=$arg
done

printf 'Filesystem 1024-blocks Used Available Capacity Mounted on\n'

case "$target" in
    /boot/firmware)
        case ":${RPIMON_TEST_MOUNTS}:" in
            *":/boot/firmware:"*)
                echo "${RPIMON_TEST_DEVICE:-/dev/test1} 512000 80000 432000 16% /boot/firmware"
                exit 0
                ;;
        esac
        ;;
    /boot)
        case ":${RPIMON_TEST_MOUNTS}:" in
            *":/boot:"*)
                echo "${RPIMON_TEST_DEVICE:-/dev/test2} 262144 32768 229376 13% /boot"
                exit 0
                ;;
        esac
        ;;
esac

echo "/dev/root 1048576 524288 524288 50% /"
"""


def read_template():
    config = {}

    for line in TEMPLATE.read_text().splitlines():
        if line and not line.startswith("#") and "=" in line:
            key, value = line.split("=", 1)
            config[key] = value

    return config


class BootMountpointTests(unittest.TestCase):

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)

        self.bin_dir = Path(self.temp.name) / "bin"
        self.bin_dir.mkdir()

        for name, content in (
            ("mountpoint", MOCK_MOUNTPOINT),
            ("df", MOCK_DF),
        ):
            path = self.bin_dir / name
            path.write_text(content)
            path.chmod(0o755)

        self.config = read_template()

    def collect(self, key, mounts, device=None):
        source = self.config[f"{key}.source"]

        env = os.environ.copy()
        env["PATH"] = (
            str(self.bin_dir) + os.pathsep + env.get("PATH", "")
        )
        env["RPIMON_TEST_MOUNTS"] = ":".join(mounts)
        if device is not None:
            env["RPIMON_TEST_DEVICE"] = device
        else:
            env.pop("RPIMON_TEST_DEVICE", None)

        command = subprocess.run(
            source,
            shell=True,
            text=True,
            capture_output=True,
            timeout=5,
            check=True,
            env=env,
        )

        result = subprocess.run(
            [
                "perl",
                "-e",
                PERL_EXTRACT,
                self.config[f"{key}.regexp"],
                self.config[f"{key}.postprocess"],
            ],
            input=command.stdout,
            text=True,
            capture_output=True,
            timeout=5,
            check=True,
        )

        value = result.stdout.strip()
        return float(value) if value else None

    def boot_values(self, mounts, device=None):
        return (
            self.collect("static.2", mounts, device),
            self.collect("dynamic.2", mounts, device),
        )

    def test_legacy_boot_mount(self):
        self.assertEqual(
            self.boot_values(["/boot"]),
            (256.0, 32.0),
        )

    def test_bookworm_boot_firmware(self):
        self.assertEqual(
            self.boot_values(["/boot/firmware"]),
            (500.0, 78.125),
        )

    def test_prefer_boot_firmware_when_both_mounted(self):
        self.assertEqual(
            self.boot_values(["/boot", "/boot/firmware"]),
            (500.0, 78.125),
        )

    def test_no_separate_boot_mount(self):
        self.assertEqual(
            self.boot_values([]),
            (None, None),
        )

    def test_storage_device_independence(self):
        devices = (
            "/dev/mmcblk0p1",
            "/dev/nvme0n1p1",
            "/dev/sda1",
            "/dev/md0",
            "/dev/mapper/boot",
            "server:/export/boot",
        )

        layouts = (
            ("/boot", (256.0, 32.0)),
            ("/boot/firmware", (500.0, 78.125)),
        )

        for mount, expected in layouts:
            for device in devices:
                with self.subTest(mount=mount, device=device):
                    self.assertEqual(
                        self.boot_values([mount], device),
                        expected,
                    )

    def test_metric_contract(self):
        self.assertEqual(
            self.config["static.2.name"],
            "sdcard_boot_total",
        )
        self.assertEqual(
            self.config["dynamic.2.name"],
            "sdcard_boot_used",
        )
        self.assertEqual(
            self.config["static.2.postprocess"],
            "$1/1024",
        )
        self.assertEqual(
            self.config["dynamic.2.postprocess"],
            "$1/1024",
        )
        self.assertEqual(
            self.config["dynamic.2.rrd"],
            "GAUGE",
        )


if __name__ == "__main__":
    unittest.main()
