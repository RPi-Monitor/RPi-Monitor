# Addons configuration

Addons are custom pages that let you free your imagination and customize the
**RPi-Monitor** interface to match your needs.

Addons are HTML pages, JavaScript, and CSS directly integrated into **RPi-Monitor**.
They are installed in `/usr/share/rpimonitor/web/addons/`, and activation is configured in the configuration file.

It is possible to use the same addon several times in a single configuration file
with different parameters if the addon supports it, as shown below with the [custom addon](25_addons.md#custom-addon).

The following sections show examples of addons and highlight the possibilities of this feature.

<figure>
<img src="_static/addons001.png" width="500">
</figure>

## About Addon

The `About` addon is enabled by default and presents the addon feature.

To remove this addon, simply comment out or delete the following lines from `/etc/rpimonitor/data.conf`:

```text
include=/etc/rpimonitor/template/addons.conf
```

## Top3 Addon

Top3 shows how to use addons to add extra information to the status page.

<figure>
<img src="_static/addons002.png" width="400">
</figure>

This addon is designed to periodically generate HTML content. This content can be
made available from the addons menu and/or inserted into the status page using
`InsertHTML()`.

The Top3 addon shows the three processes consuming the most CPU time, in addition to the
consumption of the `rpimonitord` process.

To activate this addon, add the following lines to your configuration file:

```text
web.addons.1.title=Top3
web.addons.1.addons=top3
```

Copy `/usr/share/rpimonitor/web/addons/top3/top3.cron` to `/etc/cron.d/top3` to activate the periodic check.

To add Top3 information to the status page, add the following line to the configuration file:

```text
web.status.1.content.1.line.1=InsertHTML("/addons/top3/top3.html")
```

## Custom Addon

If you are not comfortable with HTML, JavaScript, and CSS, the custom addon may
help you customize **RPi-Monitor** to your needs. This addon implements an
iframe that can display any other web page.

<figure>
<img src="_static/addons003.png" width="500">
</figure>

The custom addon can be configured with the following parameters:

web.addons.\<id\>.title=\<title\>
: `<title>` represents the title displayed in the addon menu or as the addon title.
  This parameter is evaluated by the JavaScript command with `eval()`, so it must be valid JavaScript.

web.addons.\<id\>.addons=custom
: `custom` is the addon that is activated.

web.addons.\<id\>.showtitle=\<title visibility\>
: `<title visibility>` defines whether the title should be displayed.

web.addons.\<id\>.url=\<url\>
: `url` defines the page to be displayed in the iframe. It can be a file reachable from the
  internal **RPi-Monitor** server or a site available on the Internet.

web.addons.\<id\>.allowupdate=\<allow update\>
: `<allow update>` defines whether the `url` can be updated at runtime.

### Shellinabox

[Shellinabox](https://github.com/shellinabox/shellinabox) allows you to access the shell of your
Raspberry Pi through a web interface.

Install Shellinabox with the following command:

```text
apt-get install shellinabox
```

By default, Shellinabox listens on <http://raspberrypi.local:4200/>.

Shellinabox can be integrated into **RPi-Monitor** with a custom addon as follows:

```text
web.addons.1.title=ShelleInABox
web.addons.1.addons=custom
web.addons.1.showtitle=false
web.addons.1.url=https://raspberrypi.local:4200/
web.addons.1.allowupdate=false
```

### Webcam

If you want to view the image from your webcam in your browser, you can use
[Hawkeye](https://github.com/ipartola/hawkeye).
Once `hawkeye` is installed, it is very easy to add it to the **RPi-Monitor** interface using the custom addon. The configuration will then be the following:

```text
web.addons.1.title=Webcam - Hawkeye
web.addons.1.addons=custom
web.addons.1.url=http://raspberrypi.local:8000/
web.addons.1.allowupdate=false
```

The `url` parameter points to the Hawkeye web interface.

> [!CAUTION]
> If you set up such a configuration, keep the security of your images in mind.
> You should use Hawkeye's ability to restrict access to the image using a login and a password.
>
> > **See also:** You can also look at [Authentication and secure access to RPi-Monitor](34_autentication.md).

## Example Addon

If you want to develop your own addon, you can refer to the example addons to see how to implement one.

<figure>
<img src="_static/addons006.png" width="500">
</figure>

The example addon provides an HTML page, JavaScript, and CSS showing how an addon page can interact with **RPi-Monitor**.

The example addon can be enabled by adding the following lines to the configuration file:

```text
web.addons.1.title=Addon example
web.addons.1.addons=example
web.addons.1.showtitle=true
web.addons.1.parameter=parameter_example
```
