:github_url: https://github.com/XavierBerger/RPi-Monitor/blob/develop/docs/source/34_autentication.rst
:wip:

Authentication and secure access
================================

The purpose of **RPi-Monitor** is to... monitor. For different reason (security, 
time, scope of project...) it do not add any authentication. 

We will see here how to configure a reverse proxy which will be in charge of 
user authentication and ssl connections. We will also configure a firewall to 
make the **RPi-Monitor** host ready to be directly connected to Internet.

Installation
-------------
To install nginx execute the following command:

::

  sudo apt-get install nginx


Manage authentication
---------------------

To manage authentication we need to create a file gathering the username and 
passwords. The following script will help you to generate new users id 
and encrypted password.

::

    #!/bin/bash

    if [ "$(id -u)" != "0" ]; then
    echo "This script must be run as root"
    exit 1
    fi

    echo -n "Enter new username: "; read user
    echo -n "Enter new password: "; read pass

    printf "$user:$(openssl passwd -crypt $pass)\n" >> /etc/nginx/.htpasswd

This script is also downloadable from Github. Execute the following command to use it:

:: 

    wget http://goo.gl/DO1hw -O addnginxuser.sh
    chmod +x addnginxuser.sh
    sudo ./addnginxuser.sh

Answer to the question and you are done. If you need to enter additionnal user, 
execute the script again.

Manage secured connection
-------------------------

To activate SSL connection we need to create certificate. In this post we 
will create a simple self signed certificate.

::

    sudo mkdir -p /etc/ssl/localcerts
    openssl req -new -x509 -days 3650 -nodes -out \
    /etc/ssl/localcerts/RPi-Experiences-cert.pem -keyout \
    /etc/ssl/localcerts/RPi-Experiences-key.pem
    chmod 600 /etc/ssl/localcerts/*

This certificate is self signed it will then be required to accept it when the 
browser will raise the certificate warning.

Reverse proxy configuration
---------------------------

Let's first deactivate the default site since we want to use ngnix as a reverse 
proxy only. To do so, delete the symbolic link from sites-enabled directory:

::

    sudo rm /etc/nginx/sites-enabled/default

Create the file ``/etc/nginx/sites-available/reverseproxy`` with the following 
content (also downloadable from Github):

::

    access_log off;
    add_header Cache-Control public;
    server_tokens off;

    # HTTP 80
    server {
        listen         80;
        #Force the usage of https
        rewrite - https://$host$request_uri? permanent;
    }

    # HTTPS 443
    server  {
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

        # Allow to use frame from same origin
        add_header X-Frame-Options SAMEORIGIN;

        # DDOS protection - Tune Values or deactivate in case of issue
        # limit_conn conn_limit_per_ip 20;
        # limit_req zone=req_limit_per_ip burst=20 nodelay;

        # Proxy Config
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

Activate the reverse proxy site and retart nginx with the following commands:

::

    sudo ln -s /etc/nginx/sites-available/reverseproxy /etc/nginx/sites-enabled/
    sudo service nginx restart

You can now start to test to access your configuration by browsing 
http://raspberrypi.local/. You will be automatically redirected to https://raspberrypi.local/rpimonitor/.

Configure the firewall
----------------------

To finish our protection, we will then configure some basic firewall rules to 
reject every traffic but http (redirected to https), https and ssh. The 
following lines are doing the job:

:: 

    sudo iptables -F
    sudo iptables -A INPUT -i lo -p all -j ACCEPT
    sudo iptables -A OUTPUT -o lo -p all -j ACCEPT
    sudo iptables -A INPUT -i eth0 -m state --state ESTABLISHED,RELATED -j ACCEPT
    sudo iptables -A INPUT -p tcp --dport ssh -j ACCEPT
    sudo iptables -A INPUT -p tcp --dport http -j ACCEPT
    sudo iptables -A INPUT -p tcp --dport https -j ACCEPT
    sudo iptables -P INPUT DROP

Explanation:

* line 1 : clean previously existing rules
* lines 2 and 3 : Add a full access to lo interface (which can only be accessed locally and which is used by the reverse proxy to reach RPi-Monitor and shellinabox)
* line 4 : continue to accept established connection on interface eth0
* line 5 : accept connection to port ssh (22)
* line 6 : accept connection to port http (80)
* line 7 : accept connection to port https (443)
* line 8 : drop anything else

Executing the command lines described upper will apply the firewall 
configuration but without persistence  this means that the firewall 
configuration will disappear after reboot. To make the firewall persistent 
we need to install an additional package:

::

    sudo apt-get install iptables-persistent

When the installation program ask you to record the actual ipv4 rules, answer 
``yes`` and the job is done (you can skip ipv6 rules recording). The 
configuration is now stored into ``/etc/iptables/rules.v4`` and will be 
reapplied at start-up.

Conclusion
----------

Now your host is protected. You can try to access to **RPi-Monitor** directly 
http://raspberrypi.local:8888/ and you will have an error. If you try to access to 
it through the revers proxy http://raspberrypi.local/ you will have to authenticate 
before accessing to the server and once authenticated, you will be connected 
through a secured https connection. 

Here it is we have a server which is now able to be connected on the internet.
