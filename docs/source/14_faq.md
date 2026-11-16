# Frequently Asked Questions

**RPi-Monitor** is displaying `NaN` instead of value in my "unsupported" distro. Can you fixe this issue?
: The installation script targets Debian, Raspberry Pi OS, and Ubuntu. The
  default metrics depend on the operating system and hardware; Raspberry Pi OS
  Bookworm is supported by the current templates. On another distribution or
  device, a missing metric may need a customized source or template. Use the
  [configuration helper](23_configuration_helper.md) to inspect sources and
  generate a configuration example.

<br>

**RPi-Monitor**'s graphs are not clean and frequently drop down to zero as in screenshot bellow. How can I have a clean curve?
: <figure>
  <img src="_static/faq001.png" width="400">
  </figure>

  **RPi-Monitor** is periodically getting data from program of files. 
  If the program is not answering or if the file is empty, the value is unknown. 
  This unknown value will be drawn as a zero in graphs. If, as in the upper 
  example, the program reading the sensor isn't giving reliable values, it is 
  advised to create a script writing the temperature result into a file and to 
  configure **RPi-Monitor** to read this file. In this case, if the file is not 
  updated, the previous value will be read and drawn in graphs.

<br>

**RPi-Monitor**'s web interface is not working, I've only white pages?
: **RPi-Monitor** requires HTML5 local storage and cookies to work properly. 
  You should check that these both functionalities are activated into your browser 
  and check that no plug-ins is blocking them.

<br>

Is **RPi-Monitor** supporting distribution XXX? Can you make **RPi-Monitor** compatible with distribution XXX?
: The installation script is intended for Debian, Raspberry Pi OS, and Ubuntu.
  Other Linux distributions may work with a community package or manual
  adaptation, but are not covered by the official installer. The metrics
  themselves are configurable; see the [configuration templates](27_configuration_templates.md).

<br>

I have a problems running `rpimonitord`:  failed "Can't locate _______.pm  in @INC ...
: Check that the installation completed and that the required Perl modules are
  installed. When using the official installer, review its output for missing
  packages and rerun it after installing them.

<br>

I would like to use my favorite web server instead of the one embedded into **RPi-Monitor**. Is it possible?
: **Yes**, this is possible but not recommended. **RPi-Monitor** embedded web server is 
  optimized to reduce the usage of the SDCard (using shared memory instead of writing data on disk), 
  reduce the resources used into the system. The Web interface is requiring some specific feature that can only 
  be delivered by **RPi-Monitor** embedded web server (like empty.rrd file generation 
  ensuring a clean display of graphs).

<br>

Can you add login/password and authentication to RPI-Monitor web interface?
: **No**. I'll never add such a feature to **RPi-Monitor**. Why? Just because it is 
  something complex to do in a clean and fully secured way. Some software are 
  already design to do the job and they do it well. 

  A [solution is proposed in this documentation using nginx](34_autentication.md#authentication-and-secure-access)
  frontend.

<br>

Can I do customize **RPi-Monitor**, change data extracted, add images, change texts...
: **Yes**, in **RPi-Monitor** almost everything is configurable. This can be done by 
  configuration file. Everything is explained in this documentation.

<br>

I tried to install **RPi-Monitor** into an OS not officially supported and some data are missing. How can I fix it?
: As said previously, in **RPi-Monitor** almost everything is configurable. You 
  should find the command to execute or the file to read to get the missing 
  value directly in the shell. Once the command is found you will need to 
  update the configuration file. Have a look to **RPi-Monitor** configuration chapter and to
  Interactive Configuration Helper (`rpimonitord -i`).

<br>

I would like to monitor a cluster of raspberries with your tool. Is there a way to gather the whole data in one node? 
: It could have any answer to such a question. One possibility would be to use 
  ssh and configure **RPi-Monitor** to execute commands on each remote nodes of 
  your cluster and process the data locally.

  To do so, you should create one configuration file per node to monitor and 
  use all of them in **RPi-Monitor**. Using pages feature could be very usefull. 
  You can use the main page to display some consolidate data in addition to one 
  page per cluster.

  You can also use the snmp capabilities to share data from node to central point.

<br>

I found a bug into **RPi-Monitor**. How can I share my discovery?
: You should open an issue in [github](https://github.com/RPi-Monitor/RPi-Monitor/issues) 
  and describe you problem and how to reproduce it.

<br>

Download bandwidth is shown as negative values. It's bug or feature?
: This is a feature :-). But, remember, everything is customizable in **RPI-Monitor**...

<br>

I want to create a package for my favorite distribution which is not supported by defalut. How can I contribute to this project?
: **RPi-Monitor** source code is hosted in github. Refer to 
  [Packaging section of contributing chapter](41_contributing.md#packaging) for details.

<br>

I've some issue with my installation or customization of **RPi-Monitor** and I want to reinstall it from scratch. How can I do it?
: Back up `/etc/rpimonitor` first. For an installation managed by the
  official script, rerun the installer with `--force` to reinstall the current
  release while preserving configuration. If the Debian package is installed,
  remove it with `sudo apt remove rpimonitor` before switching to the script;
  this keeps the configuration directory.

<br>

Some data are not extracting correctly at boot but when I restart `rpimonitord` everything becomes OK until I reboot. How can I fix thi issue?
: If you reach such situation, you certainly configure your source using a 
  command whitout the full path. For example `gettemp.sh` instead of 
  `/usr/bin/gettemp.sh`. The first command is working in your shell because the 
  variable $PATH is set and the executable is reachable (that why it work when 
  you restart `rpimonitord`).
  When you start your computer, the $PATH variable is not set so `gettemp.sh` 
  will not work. The solution is to use the full path command `/usr/bin/gettemp.sh` 

<br>

I just update my distribution and **RPi-Monitor** is still saying that packages required to be updated. Why?
: The upgradable-packages information is refreshed daily. To update it now,
  run `sudo /usr/share/rpimonitor/scripts/updatePackagesStatus.pl`. The script
  checks for `aptitude`, `apt`, or `checkupdates`.

  See [service and package status](13_execution.md#service-and-package-status).

<br>

I required better resolutions of the reading provided by **RPi-Monitor**. Can we increase the frequency with which **RPi-Monitor** collects the data?
: **Yes**, it is possible to define the RRD structure per data extracted. 
  Refer to documentation and man page to know how use this advanced feature.

I want to uninstall **RPi-Monitor**, How can I do it?
: If it was installed as the legacy Debian package, use
  `sudo apt remove rpimonitor`. The script-based installer does not currently
  provide an uninstaller; back up your configuration and data before removing
  installed files or disabling the systemd service.
