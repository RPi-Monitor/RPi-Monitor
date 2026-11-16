# Installation and upgrade

## Supported installation

The recommended way to install **RPi-Monitor** is the unified installation
script. It downloads the latest stable release from GitHub, checks the
published SHA-256 checksum, installs the required packages, and configures the
systemd service.

On Debian, Raspberry Pi OS, or Ubuntu, run:

```bash
curl -fsSL https://raw.githubusercontent.com/RPi-Monitor/RPi-Monitor/main/install.sh | sudo bash
```

The script requires Bash, root privileges, an interactive terminal for
confirmation, and `curl`, `jq`, and `unzip`. It installs the latest stable
release by default. If **RPi-Monitor** is already installed by this script,
running it again updates the installation and preserves the configuration in
`/etc/rpimonitor`. The web interface is available on port `8888` by default;
the listening port can be changed in `daemon.conf`.

> [!IMPORTANT]
> Run the script with Bash, not `sh`. If the legacy Debian package is
> installed, remove it first with `sudo apt remove rpimonitor`; the package
> removal keeps `/etc/rpimonitor`. Back up your configuration before changing
> installation methods.

## Installation options

Pass options after the script URL with `bash -s --`:

```bash
curl -fsSL https://raw.githubusercontent.com/RPi-Monitor/RPi-Monitor/main/install.sh | sudo bash -s -- --help
```

| Option | Description |
| --- | --- |
| `-h`, `--help` | Show help and exit. |
| `-y`, `--yes` | Confirm automatically; required when no interactive terminal is available. |
| `--version VERSION` | Install a specific published release, such as `2.13.2`. A leading `v` is optional. |
| `--pre-release` | Select the newest published release, including pre-releases. |
| `--force` | Reinstall even when the requested version is already installed. |
| `--dry-run` | Preview system commands without applying installation changes. Root privileges are still required. |
| `--no-service` | Do not enable, start, or restart the systemd service. |

For example, to install a specific release without prompting:

```bash
curl -fsSL https://raw.githubusercontent.com/RPi-Monitor/RPi-Monitor/main/install.sh | sudo bash -s -- --version 2.13.2 --yes
```

Options can be combined. `--pre-release` should be used only when you are
prepared to run a version that may not be stable.

## Upgrade

To upgrade an installation managed by this script, run the same command again.
It selects the latest stable release, preserves configuration files, and
restarts the service if needed. New templates are added without overwriting
local configuration; if an installed template differs from the release copy,
the new version is saved with a `.new` suffix. Use `--version VERSION` to
select a particular release or `--force` to reinstall the current version.

The daily package-status update is handled automatically. To refresh the
upgradable-packages information immediately, run:

```bash
sudo /usr/share/rpimonitor/scripts/updatePackagesStatus.pl
```

> [!NOTE]
> `TARGET_DIR` can direct deployed files to an alternate root, for example
> `sudo TARGET_DIR=/tmp/rpi-root bash ./install.sh --dry-run`. It does not
> isolate host-level operations such as package management or systemd.
