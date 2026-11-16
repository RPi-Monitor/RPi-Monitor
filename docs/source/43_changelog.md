:github_url: https://github.com/XavierBerger/RPi-Monitor/blob/develop/docs/source/43_changelog.rst
:wip:

Release note
============

v2.13-beta7
-----------
  .. important :: Target version v2.13-r0 not yet published yet

  * New: Implement sortable status items written by iaa (#166 #197)
  * New: Create documentation in https://xavierberger.github.io/RPi-Monitor-docs/index.html
  * New: Generate manpage from documentation
  * New: Add snmp agent "pass_persist" extension
  * New: Add readonly option disabling disk write
  * New: Add LinkToGraph function (#201)
  * New: Add defaultinterval to make interval setup easier
  * Change: 'name' keyword of web interface definition become 'title' and value has to be a valid javascript code (using "") 
  * Change: Improve multiple KPI extraction, postprocessing and default values
  * Change: Improve logfile management (#169)
  * Fix: Serverstatus is not refreshed (#172)
  * Various bug fixes and cleanup code

v2.12-r1
--------
  * Fix interactive configuration mode to accept commands with parameters
  * Fix various bug occuring with raspbian stretch
  * Fix info updated packages after upgrade
  * Fix typos and other bugs
  * Add configuration and tools (``/usr/share/rpimonitor/scripts/``) for OrangePi and Allwinner_H3

v2.11-r5
--------
  * Add possibility to use result of KPI into source with keyword 'data.<kpi>'
  * Add multicolumn view in status page for large screen (Enhancement #77)
  * Add alert feature able to execute script when necessary (Recovery action, SMS, e-mail, etc.)
  * Add possibility to set a default value for KPI when source is not existing
  * Add possibility to set interval to reduce the number of data extraction frequency
  * Add Makefile for installation and packege building
  * Add tools dedicated to generate manpages
  * Add arch configuration files
  * Set Status page as default landing page.
  * Logfile destination is configurable (Default: ``/var/log/rpimonitor.log``)
  * Allow to use addons when RPi-Monitor is protected by a reverse proxy.
  * Rounding temperation to 2 digit as proposed in issue #86
  * Update startup script
  * Bug fixes and cleanup code

v2.10-1
-------
  * Use only data.conf and daemon.conf to avoid duplicate statistics + status when .conf placed in ``/etc/rpimonitor`` (Fix issue # 68 )
  * Add addons allowing to create custom pages (Enhancement issue #27 #28 #47)
  * Add suport of UTF-8 (Fix issue #43)
  * Add BananaPi configuration from tkaiser (http://forum.lemaker.org/forum.php?mod=viewthread&tid=8137)
  * Add mysql status in services.conf example file

v2.9.1-1
--------
 * Fix issue #63 - Multiple pages index not correctly managed

v2.9-1
------
  * Migrate style to bootstrap 3.2.0 
  * Add color into progressbar orange for warning and red for danger
  * Move qrcode to about menu (link is related to the page display)
  * Add justgage generating colored gauges
  * Add visibility parameter defining if a row of status page should be shown 
  * Add example configuration files showing rpimonitor rendering features
  * Sign repository and add gpg public key (to remove "unauthenticated package" installaton warning)

v2.8-1
------
  * Add 'Badge' function for status page (similar to Label) 
  * Simplify and fix startup scripts
  * Manage configuration indexes unicity per file (to make configuration easier)
  * Add 'include' function to load configuration additionnal files
  * Split configuration files and use ``include`` in default configuration file
  * Change ownership of cron script to avoid issue with logs
  * Add parameters to customise menu logo, title and page title
  * Add debian repository using github as repo
  * Check if localStorage is activated and warn user is not
  * Improve upgradable packages detection using aptitude for debian or pacman
    for archLinux (Thanks ajs124 for archLinux implementation)

v2.7-1
------
  * Change directory storing configuration to ``/etc/rpimonitor/``
  * Change configuration file name to better names
  * Create a template directory for configuration files
  * Change location of RRD storage
  * Add xbian support and distribution automatic detection in debian package.
  * Cleanup shellinabox menu display and configuration
  * Change ownership of empty.rrd to activate on demand regeneration Fixing issue #23
  * Update raspbian.conf to make it compatible with NOOBS installation
  * Update jquery (v2.1.1) and bootstrap (v2.3.2)
  * Update disk usage formula for better compatibility
  * Add 'Label' function for status page

v2.6-1
------
  * Add ``version.json``, ``menu.json`` and ``friends.json`` generation to fix issue #25
  * Check is file is executable (instead of existing) to define if it should be executed
  * Generate ``empty.rrd`` at each request to fix issue #23
  * Generate ``empty.rrd`` at each start workaround for issue #23 when using an external web server
  * Improve shellinabox management: ``<IP>/shellinabox`` forwards to https://<IP>:4200
  * Add RPi-Monitor Interactive Configuration Helper

v2.5.1-1
--------
  * Update rrd graph with unkown data if fetched information is not a valid number
  * Fix issue #22 : ``empty.rrd`` file not properly generated

v2.5-1
------
  * Fix issue #16: (v2.4) Network Graph displaying Bytes/s not in bits/s (wrong Legend) (From deMattin)
  * Update configuration to be compatible with NOOBS as proposed in issue #17 (From deMattin)
  * Implement modification for small screen proposed in issue #18 (From deMattin)
  * Change footer to better fit with small screens
  * Implement pull request #20: Counting buffers and cache as available memory (From Harbulot)
  * Add the possibility to set Min and Max values for rrd graphs in configuration file (issue #19 #21)

v2.4-1
------
  * Change shared memory management to fix issue #12 and define the shmkey in configuration file
  * Added the possibility to define the default timeline to be display in statistics page
  * Add -s (show) option to show configuration as loaded (usefull for advanced customization) 
  * Added WiFi icon

v2.3-1
------
  * Add small improvement given by a feedback in RPi-Experiences blog
  * Use javascript to resize correctly shellinabox iframe even in Firefox
  * Fix issue #9: Wrong calculation of used memory
  * Fix issue #11: adding friends link missing colon

v2.2-1
------
  * Add the possibility to create multiple status and statistics pages
  * Add process respawner to improve reliability of rpimonitord
  * Add HTML5 cache for json data to speedup page display and offload RPi
  * bug fix: Embedded server won't start on 443 or 80 ports (issue #8)
  * bug fix: cleanup code according to comment described in issue #7 

v2.1-1
------
  * Precheck result before adding in RRD to improve graphs
  * Add the possibility to customize axis with graph_options parameter
  * Add timeout for KPI reading to avoid possible hang when reading external sensors
  * Change keywork ds_graph_opts by ds_graph_options to be consistant with javascriptrrd and flot documentation
  * Fix Issue #6: a stranger rpimonitor.conf

v2.0-1
------
  * Add the possibility to draw static values
  * Remove update package status update after ``apt-get`` command. Update command
    will have to be executed manually.
  * Add redirection for shellinabox for url /shellinabox
  * Add a manpage dedicated to configuration: ``man rpimonitord.conf``
  * Change in configuration files to make status and statistic pages fully configurable
  * Add post-process formula to have more readable values
  * Curves displayed in graph are now fully configurable
  * Remove Legend position and timezone from graphs
  * Make legend of graph configurable

v1.5-1
------
  * Fix bugs highligthed by validator.w3.org
  * Fix bugs avoiding **RPi-Monitor** to work properly with an external web server

v1.4-1
------
  * Fix bugs 
  * Add package to be upgraded popover listing
  * Add firmware version
  * Increase shared memory to 8kb to support bigger json due to package upgrade status list
  * Add warning before closing or refreshing sheelinabox page to avoid unwanted connection closing
  * Remove graph without real meaning (voltage, cpu_frequency)
  * Remember last graph displayed (to make update easier just by clicking Statistics menu)

v1.3-1
------
  * Bugs fix related to dependencies of package v1.2.
  * Added package status improvement.
  * Improve embedded server to work with a secure reverse proxy.
  * Add governor info in CPU status.

v1.2-1
------
  * Bugs fix related to uptime.
  * Adding preload spinner for statistics.
  * Added package status into status page.
  * Prepare embedded server to work with a secure reverse proxy.
  * Remove SSL embedded capability (which is not working in RPi) to reduce dependencies.

v1.1-1
------
  * Add 'Friends' to add links to **RPi-Monitor** from different RPi.
  * Improve uptime display in status page and add RPi clock.
  * Fixed some bugs.

v1.0-1
------
  * v1.0-1 Initial release.
