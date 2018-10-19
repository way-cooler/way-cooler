awesome(1)
==========

NOM
----

awesome - client Wayland clone de AwesomeWM.

SYNOPSIS
--------

*awesome* [*--version*]

DESCRIPTION
-----------

*way-cooler* est un gestionnaire de fenêtres par tuile pour Wayland basé sur AwesomeWM.

*awesome* contrôle *way-cooler* en exposant des APIs Lua et DBUS. Lors du lancement, il exécute son fichier de configuration et configure du même coup *way-cooler*.

L'API Lua est (presque) celui d'AwesomeWM. Ainsi, la documentation devrait être interchangeable. Pour une discussion plus complète sur l'API Lua, veuillez consulter la documentation *awesomerc*.

OPTIONS
-------
*--version*:
    Affiche la version dans la sortie standard, puis termine l'exécution.

PERSONNALISATION
----------------
Créer le fichier de configuration '$HOME/.config/way-cooler/rc.lua'. Lors du lancement, il s'exécutera et aura accès à l'API Lua pour configurer adéquatement way-cooler.

À VOIR
------
*way-cooler*(1) *awesomerc*(5)

BUGS
----
Tous les rapports d'erreur sont les bienvenues. Voir https://github.com/way-cooler/way-cooler

AUTEURS
-------
Preston Carpenter (a.k.a. Timidger) et autres.

WWW
---
https://way-cooler.org