# Key features of RPi-Monitor

At the beginning, **RPi-Monitor** was designed to run on a **Raspberry Pi**.
This explains the name of the software. In reality, **RPi-Monitor** can run on
any computer running `perl`.

Since the beginning, **RPi-Monitor** has been designed for embedded devices, and
performance remains at the core of the development strategy.

The software is divided into two main parts:

* A lightweight daemon running on the device and collecting data
* A powerful web application running in the browser and responsible for rendering the data

<figure>
<img src="_static/features001.png" width="500">
</figure>

## Collecting metrics
`rpimonitord` is a daemon that periodically gathers, processes, and stores metrics.
In **RPi-Monitor**, metrics are named **KPI** for **K**ey **P**erformance **I**ndicator.

## Storing metrics
Metrics are stored in a [Round Robin Database](https://oss.oetiker.ch/rrdtool/) (RRD).
This ensures a fixed size for the data storage and keeps data for one year by default.

## Presenting collected metrics
**RPi-Monitor** embeds a web server designed to present collected metrics.
Instantaneous data are shown on the status page, while metrics stored in the RRD are
shown on the statistics pages.

The web interface allows users to reorganize status information by drag-and-drop.

Statistics pages allow zooming in on graphs to inspect metric details.

The embedded web server can be disabled, and pages can be served by an external web server.

## Configurable
In **RPi-Monitor**, everything is configurable:

- Define which metrics to extract and how to extract them
- Define how to store metrics
- Define how to present and share metrics and statistics
- Define how to detect alerts and what to do when they appear or disappear
- Define ... anything you want to do with **RPi-Monitor**

## Processing metrics and raising alerts
**RPi-Monitor** embeds a fully configurable alert engine able to determine whether
metrics are outside expected ranges and raise alerts.

When an alert is detected, **RPi-Monitor** can execute an action, for example sending an email or SMS.

The criteria for alert detection and the action to perform when an alert is triggered are fully configurable.

## Read-only mode
If writing data to disk, eMMC, SSD, SD card, and so on is not possible,
**RPi-Monitor** can be run in **read-only** mode.

Metrics can be retrieved either through the web interface in JSON format or through SNMP.

## Sharing metrics
Metrics can be downloaded in JSON format (using the embedded web server), read from disk,
or accessed remotely through SNMP.

## Extending the capabilities
**RPi-Monitor** is extensible. It provides addons that allow customization of the
interface. For example: [Shellinabox](https://github.com/shellinabox/shellinabox) or
[Hawkeye](https://github.com/ipartola/hawkeye).

**RPi-Monitor-Peeble** allows you to view the monitoring status on a [Pebble](https://www.pebble.com/) smart watch.

**RPi-Monitor** is also available for [Docker](https://www.docker.com/) and for embedded devices created
with the [Yocto Project](https://www.yoctoproject.org/).

## Screenshots

See the [screenshots](02_screenshots.md) page for more details.
