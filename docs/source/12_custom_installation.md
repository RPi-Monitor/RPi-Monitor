:github_url: https://github.com/XavierBerger/RPi-Monitor/blob/develop/docs/source/12_custom_installation.rst

Custom installation
===================

Installation from Debian package
--------------------------------
Download the latest package from the official repository in `Github
<https://github.com/XavierBerger/RPi-Monitor-deb/tree/master/packages>`_:

::

    wget https://goo.gl/yDYFhy -O rpimonitor_latest.deb

If you have issue with GnuTLS: A TLS warning alert coming from Github you can download it with the following command:

::

    curl -L https://goo.gl/yDYFhy -o rpimonitor_latest.deb

Install the dependencies by executing the following command (use command 
``apt-get`` or ``apt`` if ``aptitude`` is not available on your target system):

::

  sudo aptitude install librrds-perl libhttp-daemon-perl libjson-perl \
  libipc-sharelite-perl libfile-which-perl libsnmp-extension-passpersist-perl

Installation or upgrade can be done with the command:

::

  sudo dpkg -i rpimonitor_latest.deb

Complete the installation with the command:

::

  sudo /usr/share/rpimonitor/scripts/updatePackagesStatus.pl

Installing on Gentoo
--------------------
**Gentoo** users can find the `ebuild <https://github.com/srcshelton/gentoo-ebuilds/tree/master/www-apps/rpi-monitor>`_ created by Stuart Shelton.

Installing on Arch Linux
------------------------
**Arch Linux** users can find the `aur package <https://aur.archlinux.org/packages/rpimonitor/>`_ created by ajs124. 

Development branch package is located `here <https://aur.archlinux.org/packages/rpimonitor-dev-git/>`_

Installation from sources
-------------------------

For manual installation, refer to `Packaging <32_contributing.html#packaging>`_ 
instruction which describes how and where install files. 

Docker
------

Docker in few words
^^^^^^^^^^^^^^^^^^^
Source: `Wikipedia <https://en.wikipedia.org/wiki/Docker_(software)>`_

Docker is a tool that can package an application and its dependencies in a virtual 
container that can run on any Linux server. This helps enable flexibility and 
portability on where the application can run, whether on premises, 
public cloud, private cloud, bare metal, etc.

Docker technology is providing operating-system-level virtualization also 
known as containers. [...] Docker uses the resource isolation features of the 
Linux kernel [...] to allow independent "containers" to run within a 
single Linux instance, avoiding the overhead of starting and maintaining virtual machines (VMs).

RPi-Monitor running in Docker container
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
As specified in Wikipedia's description, the goal of Docker is to isolate 
programs in containers. It has the advantage to keep the host system clean 
since programs and dependencies are installed inside a container but this advantage 
could become a problem if your goal is to perform monitoring of host.

The solution provided by Michael is to share the part of host system with 
**RPi-Monitor**'s container to allow access to KPI.

::

    /opt/vc
    /boot
    /sys
    /etc
    /proc
    /usr/lib

All volumes are mapped as read-only to ensure the container can't modify the 
data on the docker host. Additionally access to the Raspberry Pi's ``vchiq`` and ``vcsm``
device needs to be mapped to the container to access hardware sensors, like CPU Temperature, e.g.

The following command does the job:

::

    docker run --device=/dev/vchiq \
    --device=/dev/vcsm \
    --volume=/opt/vc:/opt/vc \
    --volume=/boot:/boot \
    --volume=/sys:/dockerhost/sys:ro \
    --volume=/etc:/dockerhost/etc:ro \
    --volume=/proc:/dockerhost/proc:ro \
    --volume=/usr/lib:/dockerhost/usr/lib:ro \
    -p=8888:8888 \
    --name="rpi-monitor" \
    -d \
    michaelmiklis/rpi-monitor:latest

All the detail are gathered in `DockerHub <https://hub.docker.com/r/michaelmiklis/rpi-monitor/>`_ pages of the 
`Docker-RPi-Monitor <https://github.com/XavierBerger/Docker-RPi-Monitor>`_ project.

.. note:: This docker image is based on `resin/rpi-raspbian:latest <https://hub.docker.com/r/resin/rpi-raspbian/>`_ image and is 
          then designed to run on ARM based architecture such as Raspberry Pi.

Yocto
-----

``meta-rpiexperiences`` is available on `github <https://github.com/XavierBerger/meta-rpiexperiences>`_ 
gathering the recipes to add **RPi-Monitor** and all its dependencies to your project.
`meta-rpiexperiences <http://layers.openembedded.org/layerindex/branch/master/layer/meta-rpiexperiences/>`_  
and `rpimonitor <http://layers.openembedded.org/layerindex/recipe/52439/>`_ are referenced on
`layers.openembedded.org <http://layers.openembedded.org/layerindex/branch/master/layers/>`_

Yocto in few words
^^^^^^^^^^^^^^^^^^
Source: `YoctoProject.org <http://YoctoProject.org>`_

The Yocto Project is an open source collaboration project that provides templates, 
tools and methods to help you create custom Linux-based systems for embedded products 
regardless of the hardware architecture.

meta-rpiexperiences
^^^^^^^^^^^^^^^^^^^
This layer provides **RPi-Monitor** moniritoring tools and its dependencies for Yocto.

**Dependencies**

This layer depends on:

::

    URI: git://git.openembedded.org/openembedded-core
    branch: master
    revision: HEAD
    prio: default

    URI: git://git.openembedded.org/meta-openembedded/meta-oe
    branch: master
    revision: HEAD
    prio: default

    URI: git://git.openembedded.org/meta-openembedded/meta-perl
    branch: master
    revision: HEAD
    prio: default

**Adding RPi-Experiences layer to your build**

In order to use this layer, you need to make the build system aware of it.

Assuming the rpi-experiences layer exists at the top-level of your yocto build tree, you can add it to the build system by adding the location of the security layer to bblayers.conf, along with any other layers needed. e.g.:

::

    BBLAYERS ?= " \
      /path/to/meta-openembedded/meta-oe \
      /path/to/meta-openembedded/meta-perl \
      /path/to/layer/meta-rpiexperiences \
      ...
      "

**Contents and Help**

In this section the contents of the layer is listed, along with a short help for each package.
 
* **recipes-perl**: This directory contains all the perl recipes needed for **RPi-Monitor**.

* **recipes-rpimonitor**: This directory contains **RPi-Monitor** recipe.

**Maintenance**

Send pull requests, patches, comments or questions to https://github.com/XavierBerger/meta-rpiexperiences/issues

* **Maintainer**: Xavier Berger

* **rpi4** : need to change /dev/vcsm into /dev/vcsm-cma
