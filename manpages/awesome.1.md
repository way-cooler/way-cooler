awesome(1)
==========

NAME
----

awesome - Lua controller for Way-Cooler

SYNOPSIS
--------

*awesome* [*--version*]

DESCRIPTION
-----------

*way-cooler* is a tiling window manager for WayLand based on AwesomeWM.

*awesome* controls *way-cooler*'s behavior by exposing Lua and DBUS APIs. At startup, it reads an RC file and configure *way-cooler* accordingly.

The Lua API is (will be) the same as AwesomeWM, so any documentation should be interchangeable. The configuration options are discussed more in length in the *awesomerc* man page.

OPTIONS
-------
*--version*:
    Print version information to standard output, then exit.

CUSTOMIZATION
-------------
Create a rc file in '$HOME/.config/way-cooler/rc.lua'. It will be read on launch and will perform all the specified customization.

SEE ALSO
--------
*way-cooler*(1) *awesomerc*(5)

BUGS
----
Please feel free to report them to https://github.com/way-cooler/way-cooler

AUTHORS
-------
Preston Carpenter (a.k.a. Timidger) and others.

WWW
---
https://way-cooler.org
