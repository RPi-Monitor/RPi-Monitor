**RPi-Monitor** est un programme conçu à l'origine pour surveiller le système d'un Raspberry-Pi et les valeurs disponibles sur les interfaces GPIO. Depuis sa création il y a deux ans, une vingtaine de versions sont venues apporter de nouvelles fonctionnalités et corriger des bugs. Son champ d'application s'est lui aussi étendu à d'autres architectures et d'autres systèmes d'exploitation.
 
Vous découvrirez dans la suite de cette dépêche comment **RPi-Monitor** est venu au jour, les apports de l'_open-source_ à son développement et bien sûr, vous trouverez une explication sur ce qu'est **RPi-Monitor** et comment l'utiliser. 

----
 [RPi-Experiences - Blog officiel de RPi-Monitor](http://rpi-experiences.blogspot.fr/)
 [RPi-Monitor sur github](https://github.com/RPi-Monitor/Rpi-Monitor)
 [RPi-Monitor sur eLinux](http://elinux.org/RPi-Monitor) 
 [RPi-Monitor-peeble](http://rpi-experiences.blogspot.fr/2014/10/rpi-monitor-peeble-rpi-monitor-is-now.html#more)
 
----

# Les débuts de RPi-Monitor #
Il y a quelques temps de ça, j'ai décidé de me perfectionner en Perl et ayant récemment acheté un Raspberry-Pi, je me suis dit, pourquoi ne pas faire moi-même l'outil de supervision que je n'arrivais pas à trouver sur Internet.


C'est comme ça qu'est né **RPi-Monitor** et c'est aussi ce qui explique le nom de cette application. Le préfixe _RPi_ est aujourd'hui quelque peu réducteur puisque, basé sur Perl et sur les technologies web, **RPi-Monitor** est capable de tourner sur un grand nombre de machines et de distributions.


# Une aventure dans le monde du Libre #
**RPi-Monitor** utilise plusieurs bibliothèques libres comme [Bootstrap](http://twitter.github.io/bootstrap/), [jQuery](http://jquery.com/), [jsqrencode](https://code.google.com/p/jsqrencode/), [justgage](http://justgage.com/), [JavascriptRRD](http://javascriptrrd.sourceforge.net/) et [flot](http://www.flotcharts.org/). 
Il était donc tout naturel de publier **RPi-Monitor** sous licence libre (_GPLv3_ en l'occurrence). Les sources sont disponibles sur github: [https://github.com/RPi-Monitor](https://github.com/RPi-Monitor)/[RPi-Monitor](https://github.com/RPi-Monitor/RPi-Monitor). 


Pour faire connaître mon logiciel, j'ai fait une [annonce](https://www.raspberrypi.org/forums/viewtopic.php?p=345924#p345924) (en Français et en Anglais) sur les forums de [raspberrypi.org](http://raspberrypi.org). J'ai aussi créé une page sur [eLinux.org](http://elinux.org/RPi-Monitor) et un blog: [RPi-Experiences](http://rpi-experiences.blogspot.fr/) sur laquelle j'annonce la publication de nouvelles versions et explique comment se servir et personnaliser le logiciel.


Quelques temps après cette première publication, j'ai eu des retours constructifs sur des axes d'amélioration, des remarques sur certains bugs. Ces retours ont permis d'améliorer **RPi-Monitor** tout au cours des 20 versions déjà publiées.


Je fournis un paquet Debian disponible sur Github transformé pour l'occasion en dépôt (cf. [RPi-Monitor-deb](https://github.com/RPi-Monitor/RPi-Monitor-deb) pour voir comment mettre en œuvre cette astuce). D'autres personnes ont pris l'initiative de créer les paquets d'installation de **RPi-Monitor** pour [ArchLinux](https://aur.archlinux.org/packages/rpimonitor/) et même [Gentoo](https://github.com/srcshelton/gentoo-ebuilds/tree/master/www-apps/rpi-monitor).


Un peu plus tard, j'ai reçu un courriel en provenance de Russie me demandant l'autorisation d'utiliser le nom **RPi-Monitor** pour une application tournant sur les montres [Pebble](https://getpebble.com/): [RPi-Monitor-Pebble](http://rpi-experiences.blogspot.fr/2014/10/rpi-monitor-peeble-rpi-monitor-is-now.html#more). J'ai bien évidemment accepté et même publié un article sur mon blog pour faire connaître cette application permettant d'afficher les informations issues de **RPi-Monitor** sur une _smartwatch_. Plus récemment, j'ai reçu des propositions de modification de code pour afficher les informations issues sur plusieurs colonnes, bout de code ajouté dans la branche _devel_ de github et qui fera parti de la prochaine version. Ce ne sont que quelques exemples de la force du partage du monde libre car l'aventure n'est pas terminée.

# RPi-Monitor, qu'est ce que c'est? #


**RPi-Monitor** c'est un programme qui est capable d'extraire des informations d'un système, de les stocker et de les présenter via une interface web.


**RPi-Monitor** possède un service d'extraction de données entièrement configurable (cf. chapitre suivant). Ces données sont extraites périodiquement et stockées dans une base de données [RRD](https://oss.oetiker.ch/rrdtool/). Il embarque aussi un serveur web (désactivable par configuration) permettant de fournir les informations nécessaires au navigateur en charge de la présentation des données.


L'architecture de **RPi-Monitor** a été conçue dès les débuts dans un but de minimiser l'utilisation du CPU du système surveillé (dans un esprit "embarqué"). Ainsi, les rendus des graphiques et la présentation ont été déportés dans le navigateur de la personne visualisant les pages. Le serveur web se contentant de fournir le contenu. Le développement du système de rendu tire profit des capacités du HTML5 pour stocker des données sur le navigateur et éviter ainsi des requêtes inutiles vers le serveur.

Voici quelques copies d'écran qui parleront certainement plus qu'un long discours.


La page d'accueil avec un exemple d'utilisation de [JustGage](http://justgage.com/) et le QRCode permettant d'afficher la page courante sur un mobile:


![RPi-Monitor_1ColumnStatusPage](https://raw.githubusercontent.com/RPi-Monitor/RPi-Monitor/master/screenshots/RPi-Monitor_1ColumnStatusPage.png)


La page d'accueil de **RPi-Monitor** tournant sur Ubuntu 14.04.2 LTS sur un grand écran permettant d'afficher 3 colonnes. L'addon _top3_ a été ajouté à cette page (dans la section CPU):

![RPi-Monitor_3ColumnsStatusPage](https://raw.githubusercontent.com/RPi-Monitor/RPi-Monitor/master/screenshots/RPi-Monitor_3ColumnsStatusPage.png)


L'interface de **RPi-Monitor** est _responsive_ et s'adapte à la taille de l'écran. Voici à quoi ressemble la page d'accueil sur un mobile.


![RPi-Monitor_SmartphoneStatusPage](https://raw.githubusercontent.com/RPi-Monitor/RPi-Monitor/master/screenshots/RPi-Monitor_SmartphoneStatusPage.png)


La page des statistiques présente des graphiques interactifs générés par le navigateur grâce à [JavascriptRRD](http://javascriptrrd.sourceforge.net/) qui intègrent une fonction zoom.


![RPi-Monitor_StatisticsPage](https://raw.githubusercontent.com/RPi-Monitor/RPi-Monitor/master/screenshots/RPi-Monitor_StatisticsPage.png)



Il est possible d'ajouter des pages d'addons intégrant par exemple [ShellInABox](https://github.com/shellinabox/shellinabox) directement dans l'interface de **RPi-Monitor**. 


![RPi-Monitor_ShellInABoxAddOn](https://raw.githubusercontent.com/RPi-Monitor/RPi-Monitor/master/screenshots/RPi-Monitor_ShellInABoxAddOn.png)


Un autre exemple est l'intégration d'un addon interagissant avec [Hawkeye](https://github.com/ipartola/hawkeye) et affichant l'image d'un _selfy_ capturé par une webcam directement dans l'interface de **RPi-Monitor**. (On ne se serait pas déjà vu quelque part?)


![RPi-Monitor_HawkeyeSelfie](https://raw.githubusercontent.com/RPi-Monitor/RPi-Monitor/master/screenshots/RPi-Monitor_HawkeyeSelfie.png)


Les copies d'écrans pourraient se multiplier à l'infini car...


#Rien n'est imposé, tout est configurable#


**Rpi-Monitor** fourni une configuration par défaut permettant de surveiller un système Linux mais rien n'est imposé et tout est configurable via des fichiers de configuration.


Un fichier de configuration ressemble à ceci:


```ini
########################################################################
# Extract Memory information
# Page: 1
# Information Status Statistics
# - memory total - yes - yes
# - memory free - yes - yes
# - memory available - yes - yes
########################################################################
static.5.name=memory_total
static.5.source=/proc/meminfo
static.5.regexp=MemTotal:\s+(\d+)
static.5.postprocess=$1/1024

dynamic.9.name=memory_free
dynamic.9.source=/proc/meminfo
dynamic.9.regexp=MemFree:\s+(\d+)
dynamic.9.postprocess=$1/1024
dynamic.9.rrd=GAUGE

dynamic.15.name=memory_available
dynamic.15.source=/usr/bin/free -mk
dynamic.15.regexp=^-\/ buffers\/cache:\s+\d+\s+(\d+)
dynamic.15.postprocess=$1/1024
dynamic.15.rrd=GAUGE

web.status.1.content.5.name=Memory
web.status.1.content.5.icon=memory.png
web.status.1.content.5.line.1="Used: <b>" + KMG(data.memory_total-data.memory_available,'M') + "</b> (<b>" + Percent(data.memory_total-data.memory_available,data.memory_total,'M') + "</b>) Available: <b>" + KMG(data.memory_available,'M') + "</b> Total: <b>" + KMG(data.memory_total,'M') + "</b>"
web.status.1.content.5.line.2=ProgressBar(data.memory_total-data.memory_available,data.memory_total)

web.statistics.1.content.6.name=Memory
web.statistics.1.content.6.graph.1=memory_total
web.statistics.1.content.6.graph.2=memory_free
web.statistics.1.content.6.graph.3=memory_available
web.statistics.1.content.6.ds_graph_options.memory_total.label=Total Memory(MB)
web.statistics.1.content.6.ds_graph_options.memory_free.label=Free Memory (MB)
web.statistics.1.content.6.ds_graph_options.memory_free.color="#7777FF"
web.statistics.1.content.6.ds_graph_options.memory_available.label=Available Memory (MB)
```



Les informations déclarées comme **static** sont des informations ne changeant pas au cours du temps. Elles sont extraites une seule fois au démarrage de l'application.


Les informations déclarées comme **dynamic** sont des informations évoluant au cours du temps. Elles sont extraites périodiquement (toutes les 10 secondes par défaut).


Les informations **web** peuvent représenter un état: **status** ou des statistiques: **statistics**. Ces sections définissent comment les informations extraites devront être présentées à l'utilisateur.


Plein d'autres possibilités sont offertes par les _addons_ comme préenté dans les copies d'écran ci-dessus. Il est possible grâce à des étapes simples de configuration ou un peu plus complexes de programmation d'ajouter des fonctionnalités à **RPi-Monitor**, la seule limite devient alors votre imagination.


Tout ceci peut paraître un peu compliqué au premier abord mais vous pourrez trouver de l'aide dans les _manpages_, en utilisant le _configuration helper_ accessible avec l'option **-i** ou en lisant les [articles](http://rpi-experiences.blogspot.fr/p/rpi-monitor-articles.html) du blog RPi-Experience décrivant comment faire des personnalisations et comment utiliser les _templates_ fournis lors de l'installation du logiciel. Le mot clé **include**, dans un fichier de configuration, permet d'organiser ou de réorganiser plus facilement les configurations et l'affichage en découlant. 


#Les compagnons et le futur de RPi-Monitor#


**RPi-Monitor** ne vient pas seul. Il est accompagné d'autres logiciels avec qui il peut travailler de concert. 


* **[RPi-Monitor-peeble](http://rpi-experiences.blogspot.fr/2014/10/rpi-monitor-peeble-rpi-monitor-is-now.html#more)**: Déjà cité plus haut permettant d'accéder aux données de **RPi-Monitor** depuis un monter _Pebble_.
* **[RPi-Monitor-deb](https://github.com/RPi-Monitor/RPi-Monitor-deb)**: Dépôt debian permettant de faire une installation et une mise à jour avec les commandes _apt-get_ ou _aptitude_.
* **[RPi-Monitor-LCD](https://github.com/RPi-Monitor/RPi-Monitor-LCD)**: Machine d'état programmée en Python permettant d'afficher les données de **RPi-Monitor** sur un écran LCD piloté par un Raspberry-Pi et d'interagir avec quelques boutons connectés au port GPIO d'un Raspberry-Pi.
* **RPi-Monitor-SNMP**: Projet d'interface pour **RPi-Monitor** pouvant remplacer ou ajoutant une interface SNMP. Cette interface permettra d'utiliser le moteur d'extraction de **RPi-Monitor** dans le but d'une intégration dans un système de surveillance plus puissant.
* **RPi-Monitor-Notifier**: Projet de programme permettant d'émettre des alertes par mail, SMS ou d'autre moyens suivant des règles définies par l'utilisateur.
* **meta-rpiexperiences/recipe-rpimonitor...**: Recettes pour Yocto permettant installer **RPi-Monitor** et ses compagnons sur systèmes embarqués. Note: Il me reste à comprendre comment décrire les dépendances vers les modules Perl dans mes recettes...
