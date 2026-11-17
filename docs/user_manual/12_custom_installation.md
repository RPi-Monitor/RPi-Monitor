# Custom installation

## Other platforms and package integrations

The official installation script targets Debian, Raspberry Pi OS, and Ubuntu.
The former third-party Debian repository and manual `.deb` instructions are no
longer the recommended installation path. For a supported system, follow
[Installation and upgrade](11_installation.md).

Community integrations for other distributions may be available, but are
maintained independently and may not track the current release:

### Gentoo
**Gentoo** users can find the [ebuild](https://github.com/srcshelton/gentoo-ebuilds/tree/master/www-apps/rpi-monitor) created by Stuart Shelton.

### Arch Linux
**Arch Linux** users can find the [aur package](https://aur.archlinux.org/packages/rpimonitor/) created by ajs124. 

Development branch package is located [here](https://aur.archlinux.org/packages/rpimonitor-dev-git/)

### Installation from sources

The release archive uses a root-based filesystem layout. The installation
script handles deployment, dependencies, configuration preservation, and
systemd integration; manual installation on other distributions requires
adapting those steps to the target system. See
[Packaging](41_contributing.md#packaging) for the installed file layout.

## Docker (community image)

### Docker in few words
Source: [Wikipedia](https://en.wikipedia.org/wiki/Docker_(software))

Docker is a tool that can package an application and its dependencies in a virtual 
container that can run on any Linux server. This helps enable flexibility and 
portability on where the application can run, whether on premises, 
public cloud, private cloud, bare metal, etc.

Docker technology is providing operating-system-level virtualization also 
known as containers. [...] Docker uses the resource isolation features of the 
Linux kernel [...] to allow independent "containers" to run within a 
single Linux instance, avoiding the overhead of starting and maintaining virtual machines (VMs).

### RPi-Monitor running in Docker container
As specified in Wikipedia's description, the goal of Docker is to isolate 
programs in containers. It has the advantage to keep the host system clean 
since programs and dependencies are installed inside a container but this advantage 
could become a problem if your goal is to perform monitoring of host.

The solution provided by Michael is to share the part of host system with 
**RPi-Monitor**'s container to allow access to KPI.

```text
/opt/vc
/boot
/sys
/etc
/proc
/usr/lib
```

All volumes are mapped as read-only to ensure the container can't modify the 
data on the docker host. Additionally access to the Raspberry Pi's `vchiq` and `vcsm`
device needs to be mapped to the container to access hardware sensors, like CPU Temperature, e.g.

The following command does the job:

```text
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
```

All the detail are gathered in [DockerHub](https://hub.docker.com/r/michaelmiklis/rpi-monitor/) pages of the 
[Docker-RPi-Monitor](https://github.com/RPi-Monitor/Docker-RPi-Monitor) project.

> [!NOTE]
> This docker image is based on [resin/rpi-raspbian:latest](https://hub.docker.com/r/resin/rpi-raspbian/) image and is
> then designed to run on ARM based architecture such as Raspberry Pi.

## Yocto (community layer)

`meta-rpiexperiences` is available on [github](https://github.com/RPi-Monitor/meta-rpiexperiences) 
gathering the recipes to add **RPi-Monitor** and all its dependencies to your project.
[meta-rpiexperiences](http://layers.openembedded.org/layerindex/branch/master/layer/meta-rpiexperiences/)
and [rpimonitor](http://layers.openembedded.org/layerindex/recipe/52439/) are referenced on
[layers.openembedded.org](http://layers.openembedded.org/layerindex/branch/master/layers/)

### Yocto in few words
Source: [YoctoProject.org](http://YoctoProject.org)

The Yocto Project is an open source collaboration project that provides templates, 
tools and methods to help you create custom Linux-based systems for embedded products 
regardless of the hardware architecture.

### meta-rpiexperiences
This layer provides **RPi-Monitor** moniritoring tools and its dependencies for Yocto.

**Dependencies**

This layer depends on:

```text
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
```

**Adding RPi-Experiences layer to your build**

In order to use this layer, you need to make the build system aware of it.

Assuming the rpi-experiences layer exists at the top-level of your yocto build tree, you can add it to the build system by adding the location of the security layer to bblayers.conf, along with any other layers needed. e.g.:

```text
BBLAYERS ?= " \
  /path/to/meta-openembedded/meta-oe \
  /path/to/meta-openembedded/meta-perl \
  /path/to/layer/meta-rpiexperiences \
  ...
  "
```

**Contents and Help**

In this section the contents of the layer is listed, along with a short help for each package.
 
* **recipes-perl**: This directory contains all the perl recipes needed for **RPi-Monitor**.

* **recipes-rpimonitor**: This directory contains **RPi-Monitor** recipe.

**Maintenance**

Send pull requests, patches, comments or questions to <https://github.com/RPi-Monitor/meta-rpiexperiences/issues>

* **Maintainer**: Xavier Berger

* **rpi4** : need to change /dev/vcsm into /dev/vcsm-cma
