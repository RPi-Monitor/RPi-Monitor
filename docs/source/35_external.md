:github_url: https://github.com/XavierBerger/RPi-Monitor/blob/develop/docs/source/35_external.rst
:wip:

Non standard configuration
==========================

Use it with my own Web Server
-----------------------------

In this example I will use the **nginx** server. You could use the same tactic with 
your preferred web server.


Let's first update the configuration file to disable the embedded web server. 
Edit the file ``/etc/rpimonitor/daemon.conf`` and set ``daemon.noserver=1``.


Then restart RPi-Monitor with the command:

::

    sudo service rpimonitor restart


The embedded server is no longer running so we will need to configure another server to access the data.


For nginx we will create the file ``/etc/nginx/sites-enabled/rpimonitor`` with the following content:

::

    server {     
        listen 80;     
        index index.html;     
        root /usr/share/rpimonitor/web; 
    }

Restart the server with the command:

::

    sudo service nginx restart

And add the symlink:

::

    sudo ln -s /var/lib/rpimonitor/stat /usr/share/rpimonitor/web/stat

Now you can reach **RPi-Monitor** with your favorite browser on your favorite web server.


RPi-Monitor on sunxi
---------------------
**Author**: Thomas Kaiser 

Overview
^^^^^^^^

**RPi-Monitor** is a self monitoring application designed to run on Raspberry Pi.

With a few adjustments it can also be used on numerous `sunxi devices <http://linux-sunxi.org/Main_Page>`_ 
that are using the `A10 <http://linux-sunxi.org/Category:A10_Boards>`_, 
`A13 <http://linux-sunxi.org/Category:A13_Boards>`_ or 
`A20 <http://linux-sunxi.org/Category:A20_Boards>`_ SoC since they all feature 
the same power management unit `AXP209 <http://linux-sunxi.org/AXP209>`_.

Installation/Configuration
^^^^^^^^^^^^^^^^^^^^^^^^^^

All you have to do is to relink RPi-Monitor's `data.conf` to `sunxi_axp209.conf`, 
ensure that `/usr/share/rpimonitor/scripts/sunxi-temp-daemon.sh` is running as 
root and restart the *rpimonitor* service/daemon afterwards:

::

	# stop rpimonitor
	ln -f -s /etc/rpimonitor/template/sunxi_axp209.conf /etc/rpimonitor/data.conf
	nohup /usr/share/rpimonitor/scripts/sunxi-temp-daemon.sh &
	# start rpimonitor

You should ensure that the `sunxi-temp-daemon.sh` daemon is started at boot, eg. 
by adding `/usr/share/rpimonitor/scripts/sunxi-temp-daemon.sh &` to 
`/etc/rc.local` or creating an appropriate systemd service. We chose a daemon 
approach since otherwise it wasn't possible to gather thermal values of connected 
SATA disks and the SoC's temperature under high load. Now the daemon collects 
these thermal values in a loop and writes them to 3 files below /tmp/ where 
they can be fetched by rpimonitor.

Monitoring disks
^^^^^^^^^^^^^^^^

If you use a SATA disk and want its temperature to be monitored you'll need to 
install the *hddtemp* package since the daemon relies on the `/usr/sbin/hddtemp` 
binary. If you've a USB disk instead that is 
[able to be queried using S.M.A.R.T.](https://www.smartmontools.org/wiki/Supported_USB-Devices) 
then you might install the *smartmontools* package and replace the `hddtemp` call in 
`sunxi-temp-daemon.sh` with an appropriate `smartctl -d` call:

::

	/usr/sbin/smartctl -d sat|usbsunplus|usbcypress|usbjmicron|usbprolific -a /dev/sda | awk -F" " '/Temperature_Cel/ {print $10}'

You've to try out which expressions work for your combination of USB enclosure 
and disk since different enclosures can be accessed differently and temperature 
S.M.A.R.T. attributes aren't standardized. Still using `hddtemp` 
is recommended since it doesn't wake up disks that are in standby/sleep mode. 
Using `smartctl` you can create data sources for any other
[relevant S.M.A.R.T. attribute](https://en.wikipedia.org/wiki/S.M.A.R.T.#Known_ATA_S.M.A.R.T._attributes) 
you're interested in, especially those related to drive health. But this is left 
as an excercise for the reader and a better job for 
[smartd](https://www.smartmontools.org/browser/trunk/smartmontools/smartd.8.in).

Caveats
^^^^^^^

Most informations regarding the power management unit rely on the ability to
query the AXP209 via I2C/sysfs. A driver for kernel 3.4 provides internal 
PMU informations below `/sys/devices/platform/sunxi-i2c.0/i2c-0/0-0034/`
but unfortunately this is still missing in mainline kernel.

Informations regarding [cpufreq settings](http://linux-sunxi.org/Cpufreq) 
(CPU clock speed, governor) can only be queried if cpufreq support is built 
into the kernel. This should apply to all 3.4.x kernels used nowadays and
with mainline kernel starting with 4.0 for A10/A13/A20 too.

If you use mainline kernel 4.0 or above the SoC's temperature will be read 
out using `/sys/class/thermal/thermal_zone0/temp`. If you've applied the 
patches at the bottom of [this page](http://sunxi.montjoie.ovh) and installed 
the lm-sensors package it should be possible to read out at least the AXP209's 
internal thermal sensor (that is more reliable than the SoC's) when using 
mainline kernel.


Use it into another Linux distribution 
--------------------------------------

RPi-Monitor has been designed to run into a Raspberry Pi but as it is using only 
standard Linux resources, it is not hardware dependent. A simple configuration 
update can make it run on Ubuntu, CentOS or any other distribution.


Ubuntu is a Debian based distribution as Raspbian. The installation can then be 
done using the deb package available for each releases. Download and install 
the package as described in this previous post.


With CentOS and withe any non Debian based distribution it will required to 
perform a manual installation.

First install the perl dependencies: HTTP::Daemon (``perl-libwww-perl``), RRD (``rrdtool-perl``) and JSON(``perl-JSON``)


Connect to github  and select the latest stable branch on the top left dropdown list.

Then download the code as zip file from the link "Download zip" visible at the bottom of the right menu.


Unpack it.

::

    unzip Version-2.x.zip


Finally install rpimonitor manually:

::

    su -
    mv Version-2.x/rpimonitor /usr/local


You can now start RPi-Monitor with the following commands:

::

    cd /usr/local/rpimonitor
    ./rpimonitird -c rpimonitor.conf -c default.conf


Note: I will not describe here how to configure the auto startup since each 
distribution has its own way to do so. An upstart script is available into 
**RPi-Monitor** github tools directory, it may help you in such an action.

Once the installation is done you can start RPi-Monitor and connect to it 
with your favorite browser.

You may notice that some values are undefined or displayed as NaN (Not a Number). 
To fix these issues, you will have to update the configuration file 
(rpimonitord.conf or default.conf in /etc/ + /etc/rpimonitord.conf.d or 
/usr/local/rpimonitor/ depending on your installation).
