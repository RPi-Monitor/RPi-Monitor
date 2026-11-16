:github_url: https://github.com/XavierBerger/RPi-Monitor/blob/develop/docs/source/11_first_installation.rst

Installation and upgrade
========================

Installation from repository
----------------------------

**RPi-Monitor** is providing a debian repository. This repository makes 
installation and update accessible with the command ``apt``, ``apt-get`` or ``aptitude``.

To use this repository follow the instruction bellow:

Install **RPi-Monitor**'s public key to trust **RPi-Monitor** repository:

::

  sudo apt-get install dirmngr
  sudo apt-key adv --recv-keys --keyserver keyserver.ubuntu.com 43A579636E330A99A8336C14E4E362DE2C0D3C0F

Execute the following command to add **RPi-Monitor** into your list of repository: 

::

  echo "deb https://www.giteduberger.fr rpimonitor/" | sudo tee /etc/apt/sources.list.d/rpimonitor.list


To install **RPi-Monitor**, execute the following command:

::

  sudo apt-get update
  sudo apt-get install rpimonitor

.. hint:: **RPi-Monitor** is designed to start automatically and collect metrics.
          The web interface is available on address http://raspberrypi.local:8888.

.. note:: You may notice that **RPiMonitor** repository is hosted on 
          `http://giteduberger.fr/ <https://giteduberger.fr/en>`_


First step
----------

After first installation you will see the following message:

.. figure:: _static/firststep001.png
   :align: center

Execute the following command to update packages information

::

  sudo /etc/init.d/rpimonitor update


Upgrade
-------

If you have performed an `installation from repository <11_first_installation.html#installation-from-repository>`_
you can upgrade **RPi-Monitor** with the following command:

::

  sudo apt-get update
  sudo apt-get upgrade


After installation you should excute the following command to update information 
about upgradable packages:

::

  sudo /etc/init.d/rpimonitor update
