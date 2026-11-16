<img class="wip-ribbon" src="_static/wip.png" alt="Work in progress">

# Authentication and secure access

The purpose of **RPi-Monitor** is to monitor systems. For different reasons (security,
time, project scope, etc.), it does not add any authentication.

Here we will see how to configure a reverse proxy responsible for user authentication
and SSL connections. We will also configure a firewall to make the **RPi-Monitor**
host ready to be directly connected to the Internet.

## Installation

To install nginx, run the following command:

```text
sudo apt-get install nginx
```

## Manage authentication

To manage authentication, we need to create a file containing the username(s) and password(s).
The following script helps generate a new user ID and an encrypted password:

```text
#!/bin/bash

if [ "$(id -u)" != "0" ]; then
echo "This script must be run as root"
exit 1
fi

echo -n "Enter new username: "; read user
echo -n "Enter new password: "; read pass

printf "$user:$(openssl passwd -crypt $pass)\n" >> /etc/nginx/.htpasswd
```

This script is also available on GitHub. Run the following command to use it:

```text
wget http://goo.gl/DO1hw -O addnginxuser.sh
chmod +x addnginxuser.sh
sudo ./addnginxuser.sh
```

Answer the prompts and you are done. If you need to add another user, run the script again.

## Manage secure connections

To enable SSL, we need to create a certificate. In this example, we will create a simple self-signed certificate.

```text
sudo mkdir -p /etc/ssl/localcerts
openssl req -new -x509 -days 3650 -nodes -out \
/etc/ssl/localcerts/RPi-Experiences-cert.pem -keyout \
/etc/ssl/localcerts/RPi-Experiences-key.pem
chmod 600 /etc/ssl/localcerts/*
```

This certificate is self-signed, so you will need to accept the browser warning the first time you connect.

## Reverse proxy configuration

First, disable the default site because we want to use nginx as a reverse proxy only.
To do so, delete the symbolic link from the `sites-enabled` directory:

```text
sudo rm /etc/nginx/sites-enabled/default
```

Create `/etc/nginx/sites-available/reverseproxy` with the following content (also downloadable from GitHub):

```text
access_log off;
add_header Cache-Control public;
server_tokens off;

# HTTP 80
server {
    listen         80;
    # Force the usage of https
    rewrite - https://$host$request_uri? permanent;
}

# HTTPS 443
server {
    listen 443 ssl;
    keepalive_timeout 70;

    # SSL config
    ssl on;
    ssl_certificate /etc/ssl/localcerts/RPi-Experiences-cert.pem;
    ssl_certificate_key /etc/ssl/localcerts/RPi-Experiences-key.pem;

    ssl_session_timeout 5m;
    ssl_protocols SSLv3 TLSv1.2;
    ssl_ciphers RC4:HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;
    ssl_session_cache shared:SSL:10m;

    # Allow use of frames from the same origin
    add_header X-Frame-Options SAMEORIGIN;

    # DDOS protection - tune values or deactivate if needed
    # limit_conn conn_limit_per_ip 20;
    # limit_req zone=req_limit_per_ip burst=20 nodelay;

    # Proxy config
    proxy_redirect          off;
    proxy_set_header        Host            $host;
    proxy_set_header        X-Real-IP       $remote_addr;
    proxy_set_header        X-Forwarded-For $proxy_add_x_forwarded_for;
    client_max_body_size    10m;
    client_body_buffer_size 128k;
    proxy_connect_timeout   90;
    proxy_send_timeout      90;
    proxy_read_timeout      90;
    proxy_buffers           32 4k;

    # Define the default site
    location / {
        rewrite - /rpimonitor/ permanent;
    }

    location /rpimonitor/ {
        proxy_pass http://localhost:8888;
        auth_basic            "Access Restricted";
        auth_basic_user_file  "/etc/nginx/.htpasswd";
        access_log /var/log/nginx/rpimonitor.access.log;
        error_log /var/log/nginx/rpimonitor.error.log;
    }

    location /shellinabox/ {
        proxy_pass http://localhost:4200;
        auth_basic            "Access Restricted";
        auth_basic_user_file  "/etc/nginx/.htpasswd";
        access_log /var/log/nginx/shellinabox.access.log;
        error_log /var/log/nginx/shellinabox.error.log;
    }
}
```

Activate the reverse proxy site and restart nginx with the following commands:

```text
sudo ln -s /etc/nginx/sites-available/reverseproxy /etc/nginx/sites-enabled/
sudo service nginx restart
```

You can now test your configuration by opening <http://raspberrypi.local/>. You will be redirected automatically to <https://raspberrypi.local/rpimonitor/>.

## Configure the firewall

To finish the protection, we will configure a few basic firewall rules to reject all traffic except HTTP (redirected to HTTPS), HTTPS, and SSH. The following lines do the job:

```text
sudo iptables -F
sudo iptables -A INPUT -i lo -p all -j ACCEPT
sudo iptables -A OUTPUT -o lo -p all -j ACCEPT
sudo iptables -A INPUT -i eth0 -m state --state ESTABLISHED,RELATED -j ACCEPT
sudo iptables -A INPUT -p tcp --dport ssh -j ACCEPT
sudo iptables -A INPUT -p tcp --dport http -j ACCEPT
sudo iptables -A INPUT -p tcp --dport https -j ACCEPT
sudo iptables -P INPUT DROP
```

Explanation:

* line 1: clean previously existing rules
* lines 2 and 3: allow full access on the `lo` interface (only accessible locally and used by the reverse proxy to reach RPi-Monitor and Shellinabox)
* line 4: continue to accept established connections on `eth0`
* line 5: accept connections to port SSH (22)
* line 6: accept connections to port HTTP (80)
* line 7: accept connections to port HTTPS (443)
* line 8: drop everything else

The commands above apply the firewall configuration immediately, but without persistence. This means the firewall configuration disappears after reboot. To make it persistent, install an additional package:

```text
sudo apt-get install iptables-persistent
```

When the installation program asks whether you want to save the current IPv4 rules, answer `yes` and you are done (you can skip saving the IPv6 rules). The configuration will then be stored in `/etc/iptables/rules.v4` and reapplied at startup.

## Conclusion

Now your host is protected. You can try to access **RPi-Monitor** directly at <http://raspberrypi.local:8888/>, and you will get an error. If you access it through the reverse proxy at <http://raspberrypi.local/>, you will be prompted to authenticate before being redirected to the secure HTTPS connection.

This gives you a server that can safely be exposed on the Internet.
