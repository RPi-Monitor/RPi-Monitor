:github_url: https://github.com/XavierBerger/RPi-Monitor/blob/develop/docs/source/21_daemon.rst

Daemon configuration
====================

Global configuration
--------------------
``daemon`` is defining the behavior of ``rpimonitord``. 

daemon.sharedmemkey=20130906
  Define the share memory key (Default: 20130906)

daemon.delay=10
  Define the delay in seconds between 2 KPIs extraction (Default: 10)

.. important:: If you want to change the default delay, the rrd files in ``/var/lib/rpimonitor/stat``
               have to be deleted. ``rpimonitord`` will recreate these files at next startup.

daemon.timeout=5
  Define the maximum duration in seconds of KPI extraction. If this timeout is triggered, KPI
  value will depends on default value configured for this KPI. (Default: 5)

daemon.noserver=0
  Tell ``rpimonitord`` to not start web server (Default: 0)

daemon.readonly=1
  Tell ``rpimonitord`` to not write data on disk. (Default: 0)

daemon.addr=0.0.0.0
  Define the address used by the embedded web server (Default: 0.0.0.0)

daemon.port=8888
  Define port of the embedded web server (Default: 8888)

daemon.user=pi
  Define user used to run embedded web server process (Default: pi)
  
  .. note:: If user is not existing, process will run with ``uid=1000``

daemon.group=pi
  Define group used to run embedded web server process (Default: pi)
  
  .. note:: If group is not existing, process will run with ``gid=1000``

  .. important:: Extraction daemon of **RPi-Monitor** running as root.

daemon.webroot=/usr/share/rpimonitor/web
  Define the root directory of the web server (Default: ``/usr/share/rpimonitor/web``)

daemon.datastore=/var/lib/rpimonitor
  Define the data storage directory (Default: ``/var/lib/rpimonitor``).

daemon.logfile=/var/log/rpimonitor.log
  Define directory where logs are stored when ``rpimonitor`` starts un background (Default: ``/dev/null``)

  .. note:: when ``rpimonitord`` is started in foreground, information a written in ``/dev/stderr``

daemon.loglevel=0
  Define log level (Default:0)

SNMP configuration
------------------
  ``snmpagent`` is defining SNMP behavior of ``rpimonitord``.

  snmpagent.rootoid=.1.3.6.1.4.1
    Define root OID for snmp-agent (Default: .1.3.6.1.4.1)

  snmpagent.enterpriseoid=54321
    Define enterprise OID for snmp agent (Default: 54321)

  snmpagent.rpimonitoroid=42
    Define rpimonitor OID for snmp agent (Default: 42)

  snmpagent.mibname=RPIMONITOR-MIB
    Define MIB name (Default: RPIMONITOR-MIB)

  snmpagent.lastupdate=201802030000Z
    Define MIB last update field (Default: 201802030000Z)

  snmpagent.moduleidentity=rpi-experiences
    Define MIB module identity (Default: rpi-experiences)

  snmpagent.organisation=RPi-Monitor
    Define MIB organisation (Default: RPi-Monitor)

  snmpagent.contactionfo=http://rpi-experiences.blogspot.fr/
    Define MIB contact info (Default: http://rpi-experiences.blogspot.fr/)

  snmpagent.description=description
    Define MIB description (Default: description)

  snmpagent.revision=201802030000Z
    Define MIB revision (Default: 201802030000Z)

