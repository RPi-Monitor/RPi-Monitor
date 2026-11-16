:github_url: https://github.com/XavierBerger/RPi-Monitor/blob/develop/docs/source/13_execution.rst

Understanding program execution
===============================

Command line
------------
``rpimonitord`` can be directly executed with the following parameters:

::

    rpimonitord [-a address][ -b pidfile][[-c configuration_file]...][-d delay][-h][-k][-l logfile]
    [-m][-n][-p port][-s][-t timeout][-v[v[...]]][-V]


Options
^^^^^^^
.. tabularcolumns:: |l{5cm}|c{5cm}|p{10cm}|

-a,--addr address           
      Web server bind address                                               
      Default: 0.0.0.0 

-b,--background pid-file      
      Define the pid file when run in background                            
      Default: not set, application run in foreground   

-c,--conf conf-file     
      Define the configuration file                                         
      Default: ``/etc/rpimonitor/data.conf`` ``/etc/rpimonitor/daemon.conf``

-d,--delay delay
      Delay between check ins seconds                                       
      Default : 10  

-l,--logfile log-file      
      Logfile directory and prefix (ex: ``/var/log/rpimonitor.log``)        
      Default: ``/dev/null``    

-p,--port port          
      Web server port                                                       
      Default: 8888  

-t,--timeout timeout       
      KPI read timeout in seconds                                           
      Default: 5                                                            

.. note:: If you want to change the default ``delay``, the rrd file will have to be deleted. 
          ``rpimonitord`` will recreate database at next startup with the new time slice.

-h,--help        
      Shows this help and exit

-i,--interactive 
      Interactive configuration helper   

-k,--keep        
      Keep log file (Default: logfile is delete at each start)

-m,--mib         
      Get MIB for current configuration   

-n,--noserver    
      Don't start embeded web server   

-r,--readonly    
      Read only mode.                

-s,--show        
      Show configuration as loaded and exit     

-v,--verbose     
      Write debug info on screen     

-V,--Version     
      Show version and exit                                   

Configuration
-------------
Configuration can be defined into ``/etc/rpimonitor/daemon.conf`` and
``/etc/rpimonitor/data.conf`` or in a list of files specified by ``-c`` parameter.

In ``/etc/rpimonitor/template/*.conf``, provided at installation, you can see 
how to customize ``rpimonitord``.

.. note:: * Configuration defined inside a configuration file always overwrite default values.
          * Configuration given as command line option always overwrite values defined in files.

.. warning:: Be sure to use Linux filefeed format with line ending with LF (and not CR/LF like in Windows)

.. seealso:: See `configuration <20_index.html>`_ chapter for details or `usage <30_index.html>`_ chapter for examples.

snmp-agent
----------
``rpimonitord-snmp`` is the snmp agent provided by **RPi-Monitor**. This agent
allow ``snmpd`` to access to data exctracted by ``rpimonitord``.

.. seealso:: See `configuration <20_index.html>`_ chapter for details or `usage <30_index.html>`_ chapter for examples.

Raw data access
---------------
Once ``rpimonitord`` is started the data representing the current status are 
available in json format and can be downloaded from the root of the web interface 
(ex: http://raspberrypi.local:8888)

* ``static.json`` : Static information extracted at startup
* ``dynamic.json`` : Current values of dynamic information extracted periodically
* ``menu.json`` : Description of menus when multiple pages are configured

The web interface configuration can also be downloaded in json format:

* ``statistics.json`` : Description of statistics page
* ``status.json`` : Description of status page
* ``friends.json`` : List of friends
* ``addons.json`` : List of addons

Statistic information are stored into RRD file available in the directory ``/var/lib/rpimonitor/stat/``

Startup script
--------------

**RPi-Monitor** is configured to start automatically. ``/etc/init.d/rpimonitor``
and link in runlevel startup directory (``/etc/rc?.d/``) do the job. 
This script can be executed with the following parameters:

start
  Start ``rpimonitord`` daemon
stop
  Stop ``rpimonitord`` daemon
restart
  Stop and start ``rpimonitord`` daemon
status
  Show ``rpimonitord`` status
update
  Update 'packages to be installed' list. This information is displayed in default 
  configuration of Web interface
install_auto_package_status_update
  Add a script to automatically call package update script when ``apt`` commands are executed
remove_auto_package_status_update
  Remove script automatically call for package update           

Manpages
--------

See also **RPi-Monitor** manpages :

::
 
  man rpimonitor
  man rpimonitor-data
  man rpimonitor-daemon
 
