:github_url: https://github.com/XavierBerger/RPi-Monitor/blob/develop/docs/source/26_alert.rst

Alert configuration
===================
**RPi-Monitor** is able to detect if value exceed a defined value or come back to the normal.

Overview
--------
Alert/Cancel are sent only when the state is stable to avoid messages
flooding when limit is about to be exceeded as shown in schema bellow:

.. code-block:: text

                          send alert
                              ^
   th      th <------dh------>|        th
    _____   __________________|______   _______           send cancel
    |   |   |                       |   |     |              ^
    |   |   |                       |   |     |              |
  __|   |___|                       |___|     |______________|_____
       tl                          tl        tl<-----dl----->
                              |------------------------------|
                                      Alert is raised

If alert is still active after resend period, alerte is sent again:

.. code-block:: text

                    send alert                                    resend alert
                        ^                                               ^
      th <-----dh------>|<---------------------dr---------------------->|
      __________________|_______________________________________________|_____
      |
      |
  ____|
     tl                 |----------------------------------------------------->
                                      Alert is raised

Configuration
-------------

Each alert is identified by its ``<alert name>``. 

alert.<alert name>.active=<activation condition>
  <activation condition> defines the alert pre-condition. If this formula
  returns false, the trigger will not be checked. Default value is 1.
  This command could refer to KPI using keyword ``data.<kpi name>``

alert.<alert name>.kpi=<trigger>
  ``<trigger>`` is defining the formula to evaluate. If formula returns true, the
  alert is detected. This command should refer to KPI using keyword ``data.<kpi name>``

alert.<alert name>.maxalertduration=<duration before raise>
  The alert will be dectected immediatly but alert raised commande will
  be executed only afer a defined ``<duration before raise>`` in seconds
  (``dh`` in the upper schema)

alert.<alert name>.cancelvalidation=<duration before cancel>
  As for alert raising, cancellation command will be executed only after
  ``<duration before cancel>`` seconds (``dl`` in the upper schema)

alert.<alert name>.resendperiod=<period>
  If alert is still active after ``<period>`` seconds, the raise command
  will be executed again (``dr`` in the upper schema).

alert.<alert name>.raisecommand=<Raise Command>
  ``<Raise Command>`` is the command to executed when an alert is raised.
  This command could refer to KPI using keyword ``data.<kpi name>``

alert.<alert name>.cancelcommand=<Cancel Command>
  ``<Cancel Command>`` is the command to executed when an alert is canceled
  This command could refer to KPI using keyword ``data.<kpi name>``

.. seealso:: Example of implementation in `Alert example <27_configuration_templates.html#alerts>`_