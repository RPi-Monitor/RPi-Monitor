:github_url: https://github.com/XavierBerger/RPi-Monitor/blob/develop/docs/source/41_contributing.rst

Contributing
=============

If you want to contribute to **RPi-Monitor** and propose a pull request, first 
create a fork from `github <https://github.com/XavierBerger/RPi-Monitor>`_

Contribute to documentation
---------------------------
**RPi-Monitor** documentation is written in reStructured Text using 
`sphinx <http://www.sphinx-doc.org/en/master/>`_ document generator and
`Read the docs theme <https://sphinx-rtd-theme.readthedocs.io/en/latest/index.html>`_ 
(included in project as `git submodule`)

Prepare development environment
^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
Clone your fork and update theme with the following commands:

::

  git clone git@github.com:${GITHUB_ACCOUNT}/RPi-Monitor.git
  cd RPi-Monitor
  git submodule update --init --recursive
  pip install sphinx sphinx-autobuild

Generate documentation
^^^^^^^^^^^^^^^^^^^^^^
The following command update the documentation and start firefox on generated documentation:

::

    cd docs
    make clean && make html && firefox build/html/index.html

Propose a merge request
^^^^^^^^^^^^^^^^^^^^^^^
Once your modifications are done, you can propose a pull request on github.

.. important:: Propose pull requests on **develop** branch only.


Contribute to software
----------------------

Report issue
^^^^^^^^^^^^
Whether you've found an issue with **RPi-Monitor**, here's how to report the problem...

First, check the documentation for solutions to common problems.

To file a bug
"""""""""""""
    * Go to `issue tracker on GitHub <https://github.com/XavierBerger/RPi-Monitor/issues>`_
    * Search for existing issues using the search field at the top of the page
    * File a new issue with all details explaining how to reproduce the bug

Requesting a feature
""""""""""""""""""""
Feel free to file new feature requests as an `issue on GitHub <https://github.com/XavierBerger/RPi-Monitor/issues>`_, just like a bug.

Propose a merge request
^^^^^^^^^^^^^^^^^^^^^^^

GitHub is used to perform versioning of development and also to store the
development progress of future releases.

The branch master contains the latest stable version.

The branch ``develop`` is the unstable development branch (The branch may have bugs 
and partially released code that could avoid RPi-Monitor to run as expected).

Each release is identified by a tag.

You want to contribute to **RPi-Monitor**. I'll be happy to integrate your pull request.

Please note: Pull request perfomed on ``develop`` branch will be integrated as soon 
as possible. Pull request perform on master branche may only be integrated 
when a new version is published (or not may not be integrated at all...)

.. important:: Propose pull requests on **develop** branch only.

Packaging
---------

The following chapter describes how **RPi-Monitor** is packaged and installed.

Dependencies
^^^^^^^^^^^^
**rpimonitord** is a perl script which require the following perl modules to work:

 * perl
 * librrds-perl
 * libhttp-daemon-perl (>= 6.0.0) | libwww-perl (<< 6.0.0)
 * libjson-perl
 * libipc-sharelite-perl
 * libfile-which-perl
 * libsnmp-extension-passpersist-perl
 * aptitude (required by script checking upgradable packages)

Programs
^^^^^^^^

* ``/usr/bin/rpimonitord`` : *Daemon extracting data from the system and embedding the web application server - The help page is used to generate manpage*
* ``/etc/cron.d/rpimonitor`` : *daily extraction the status of package update updating the file* ``updatestatus.txt``
* ``/etc/init.d/rpimonitor`` : *sysVinit startup script*

.. note:: **upstart** and **systemd** script are also avialable in the repository

Configuration
^^^^^^^^^^^^^

* ``/etc/rpimonitor/daemon.conf`` : *configuration of rpimonitord daemon*
* ``/etc/rpimonitor/data.conf`` : *symlink to* ``template/<distribution>.conf`` *configuration of default data to be extracted and presented*
* ``/etc/rpimonitor/*.conf`` : *all other* ``*.conf`` *file will be parsed to look for data to be monitored*
* ``/etc/rpimonitor/template/*.conf`` : *data configuration template customized for different distribution and additionnal examples*

Some configuration files are provided for different distribution (raspbian, xbian, ...).
These files are stored into the subdirectory ``/etc/rpimonitor/templates/``.
For supported distribution, the post installation script of package creates a link, ``data.conf``, pointing to the configuration files dedicated to the distribution.

.. note:: The template directory is also containing some templates that can be used as example to customize configuration.

Manpages
^^^^^^^^

* ``/usr/share/man/man5/rpimonitord.conf.5.gz`` : *Manpage for rpimonitor and data extraction configuration*
* ``/usr/share/man/man1/rpimonitord.1.gz`` : *Manpage for rpimonitord command line usage*

Web Interface
^^^^^^^^^^^^^

* ``/usr/share/rpimonitor/web`` : *Directory containing html*
* ``/usr/share/rpimonitor/web/js`` : *Directory containing javascripts*
* ``/usr/share/rpimonitor/web/css`` : *Directory containing style*
* ``/usr/share/rpimonitor/web/img`` : *Directory containing images*
* ``/usr/share/rpimonitor/web/fonts`` : *Directory containing fonts*
* ``/usr/share/rpimonitor/web/addons`` : *Directory containing addons*

.. note:: When **RPi-Monitor** is configured to not use the internal server, the json are written on the disk into the web root directory ``/usr/share/rpimonitor/web``

Scripts
^^^^^^^

* ``/usr/share/rpimonitor/scripts/updatePackagesStatus.pl`` : *Script periodically executed to update* ``/var/lib/rpimonitor/updatestatus.txt``

Data
^^^^

* ``/var/lib/rpimonitor/web/stat`` : *Directory containing* ``*.rrd``
* ``/var/lib/rpimonitor/updatestatus.txt`` : *Current status of packages update*
