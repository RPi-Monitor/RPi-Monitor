# Understanding program execution

## Command line
`rpimonitord` can be directly executed with the following parameters:

```text
rpimonitord [-a address][ -b pidfile][[-c configuration_file]...][-d delay][-h][-k][-l logfile]
[-m][-n][-p port][-s][-t timeout][-v[v[...]]][-V]
```


### Options
-a,--addr address
: Web server bind address
  Default: 0.0.0.0 

-b,--background pid-file
: Define the pid file when run in background
  Default: not set, application run in foreground

-c,--conf conf-file
: Define the configuration file
  Default: `/etc/rpimonitor/data.conf` `/etc/rpimonitor/daemon.conf`

-d,--delay delay
: Delay between check ins seconds
  Default : 10

-l,--logfile log-file
: Logfile directory and prefix (ex: `/var/log/rpimonitor.log`)
  Default: `/dev/null`

-p,--port port
: Web server port
  Default: 8888

-t,--timeout timeout
: KPI read timeout in seconds
  Default: 5

> [!NOTE]
> If you want to change the default `delay`, the rrd file will have to be deleted.
> `rpimonitord` will recreate database at next startup with the new time slice.

-h,--help
: Shows this help and exit

-i,--interactive 
: Interactive configuration helper

-k,--keep
: Keep log file (Default: logfile is delete at each start)

-m,--mib
: Get MIB for current configuration

-n,--noserver
: Don't start embeded web server

-r,--readonly
: Read only mode.

-s,--show
: Show configuration as loaded and exit

-v,--verbose
: Write debug info on screen

-V,--Version
: Show version and exit

## Configuration
Configuration can be defined into `/etc/rpimonitor/daemon.conf` and
`/etc/rpimonitor/data.conf` or in a list of files specified by `-c` parameter.

In `/etc/rpimonitor/template/*.conf`, provided at installation, you can see 
how to customize `rpimonitord`.

> [!NOTE]
> * Configuration defined inside a configuration file always overwrite default values.
> * Configuration given as command line option always overwrite values defined in files.

> [!WARNING]
> Be sure to use Linux filefeed format with line ending with LF (and not CR/LF like in Windows)

> **See also:** See [configuration](20_index.md) chapter for details or [usage](30_index.md) chapter for examples.

## snmp-agent
`rpimonitord-snmp` is the snmp agent provided by **RPi-Monitor**. This agent
allow `snmpd` to access to data exctracted by `rpimonitord`.

> **See also:** See [configuration](20_index.md) chapter for details or [usage](30_index.md) chapter for examples.

## Raw data access
Once `rpimonitord` is started the data representing the current status are 
available in json format and can be downloaded from the root of the web interface 
(ex: <http://raspberrypi.local:8888>)

* `static.json` : Static information extracted at startup
* `dynamic.json` : Current values of dynamic information extracted periodically
* `menu.json` : Description of menus when multiple pages are configured

The web interface configuration can also be downloaded in json format:

* `statistics.json` : Description of statistics page
* `status.json` : Description of status page
* `friends.json` : List of friends
* `addons.json` : List of addons

Statistic information are stored into RRD file available in the directory `/var/lib/rpimonitor/stat/`

## Service and package status

The official installation script enables and starts the systemd service
`rpimonitord`. Use `systemctl` to manage it:

```bash
sudo systemctl status rpimonitord
sudo systemctl restart rpimonitord
```

The installer also schedules a daily refresh of the upgradable-packages
information displayed by the default web interface. To refresh it immediately,
run:

```bash
sudo /usr/share/rpimonitor/scripts/updatePackagesStatus.pl
```

The script checks for `aptitude`, `apt`, or `checkupdates`, in that order.
The older `/etc/init.d/rpimonitor` interface belongs to legacy package
installations and is not installed by the current script-based installer.

## Manpages

See also **RPi-Monitor** manpages:

```text
man rpimonitor
man rpimonitor-daemon.conf
man rpimonitor-data.conf
man rpimonitor-alert.conf
```
 
