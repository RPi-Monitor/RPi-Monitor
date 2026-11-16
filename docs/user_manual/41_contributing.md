# Contributing

If you want to contribute to **RPi-Monitor** and propose a pull request, first 
create a fork from [github](https://github.com/RPi-Monitor/RPi-Monitor)

## Contribute to documentation
**RPi-Monitor** documentation is written in Markdown using
[mdBook](https://rust-lang.github.io/mdBook/).

### Prepare development environment
Clone your fork and install mdBook with the following commands:

```text
git clone git@github.com:${GITHUB_ACCOUNT}/RPi-Monitor.git
cd RPi-Monitor
cargo install mdbook --locked
```

### Generate documentation
Build the HTML book from the repository root with:

```text
make -C docs html
```

The result is in `docs/build/html/`. To preview changes while editing, run
`mdbook serve docs` from the repository root.

Man pages can be generated with `make -C docs man` (requires Pandoc).

### Propose a merge request
Once your modifications are done, you can propose a pull request on github.

> [!IMPORTANT]
> Propose pull requests on **develop** branch only.


## Contribute to software

### Report issue
Whether you've found an issue with **RPi-Monitor**, here's how to report the problem...

First, check the documentation for solutions to common problems.

#### To file a bug
* Go to [issue tracker on GitHub](https://github.com/RPi-Monitor/RPi-Monitor/issues)
* Search for existing issues using the search field at the top of the page
* File a new issue with all details explaining how to reproduce the bug

#### Requesting a feature
Feel free to file new feature requests as an [issue on GitHub](https://github.com/RPi-Monitor/RPi-Monitor/issues), just like a bug.

### Propose a merge request

GitHub is used to perform versioning of development and also to store the
development progress of future releases.

The `main` branch contains the current release line. The `develop` branch is
used for integration work and may contain changes that are not yet released.

Each release is identified by a tag.

You want to contribute to **RPi-Monitor**. I'll be happy to integrate your pull request.

Please target pull requests at `develop`, as requested by the project README.

> [!IMPORTANT]
> Propose pull requests on **develop** branch only.

## Packaging

The following chapter describes how **RPi-Monitor** is packaged and installed.

### Dependencies
The release is distributed as a root-based ZIP archive and installed by
`install.sh`. On a fresh supported Debian-based system, the installer provides
the runtime dependencies:

 * perl
 * librrds-perl
 * libhttp-daemon-perl
 * libjson-perl
 * libipc-sharelite-perl
 * libfile-which-perl
 * libsnmp-extension-passpersist-perl
 * aptitude (installed by the installer; package-status detection also supports `apt` or `checkupdates`)

### Programs

* `/usr/bin/rpimonitord` : *Daemon extracting data from the system and embedding the web application server - The help page is used to generate manpage*
* `/etc/cron.d/rpimonitor` : *daily refresh of upgradable-package status*
* `/usr/lib/systemd/system/rpimonitord.service` : *systemd service unit*
* `/usr/lib/systemd/system/rpimonitord-upgradable.{service,timer}` : *systemd timer and service for package status refresh*

### Configuration

* `/etc/rpimonitor/daemon.conf` : *configuration of rpimonitord daemon*
* `/etc/rpimonitor/data.conf` : *symlink to* `template/raspbian.conf` *configuration of default data to be extracted and presented*
* `/etc/rpimonitor/*.conf` : *all other* `*.conf` *file will be parsed to look for data to be monitored*
* `/etc/rpimonitor/template/*.conf` : *data configuration template customized for different distribution and additionnal examples*

Additional platform and example configurations are provided in
`/etc/rpimonitor/template/`. The installer preserves existing configuration;
when an updated template differs, it saves the new copy with a `.new` suffix
instead of overwriting local changes.

> [!NOTE]
> The template directory is also containing some templates that can be used as example to customize configuration.

### Manpages

* `/usr/share/man/man1/rpimonitor.1` : *Manpage for rpimonitord command line usage*
* `/usr/share/man/man5/rpimonitor-daemon.conf.5` : *Manpage for daemon configuration*
* `/usr/share/man/man5/rpimonitor-data.conf.5` : *Manpage for data extraction configuration*
* `/usr/share/man/man5/rpimonitor-alert.conf.5` : *Manpage for alert configuration*

### Web Interface

* `/usr/share/rpimonitor/web` : *Directory containing html*
* `/usr/share/rpimonitor/web/js` : *Directory containing javascripts*
* `/usr/share/rpimonitor/web/css` : *Directory containing style*
* `/usr/share/rpimonitor/web/img` : *Directory containing images*
* `/usr/share/rpimonitor/web/fonts` : *Directory containing fonts*
* `/usr/share/rpimonitor/web/addons` : *Directory containing addons*

> [!NOTE]
> When **RPi-Monitor** is configured to not use the internal server, the json are written on the disk into the web root directory `/usr/share/rpimonitor/web`

### Scripts

* `/usr/share/rpimonitor/scripts/updatePackagesStatus.pl` : *Script periodically executed to update* `/var/lib/rpimonitor/updatestatus.txt`

### Data

* `/var/lib/rpimonitor/stat` : *Directory containing* `*.rrd`
* `/var/lib/rpimonitor/updatestatus.txt` : *Current status of packages update*
