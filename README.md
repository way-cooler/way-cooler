# <img src="http://i.imgur.com/OGeL1nN.png" width="60"> Way Cooler [![Crates.io](https://img.shields.io/crates/v/way-cooler.svg)](https://crates.io/crates/way-cooler) [![Downloads](https://img.shields.io/crates/d/way-cooler.svg)](https://crates.io/crates/way-cooler) [![Build Status](https://travis-ci.org/way-cooler/way-cooler.svg?branch=master)](https://travis-ci.org/way-cooler/way-cooler) [![Gitter](https://badges.gitter.im/Immington-Industries/way-cooler.svg)](https://gitter.im/Immington-Industries/way-cooler?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge) [![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/way-cooler/way-cooler/)

Way Cooler is a customizable tiling window manager written in [Rust][] for [Wayland][wayland] and configurable using [Lua][].

It is heavily inspired by the tiling of [i3][] and the extensibility of [awesome][].

While Lua is used for the runtime configuration (like in [awesome][]), extensions for Way Cooler are implemented as totally separate client programs using [D-Bus][]. Currently we support 3 official extensions:
* [way-cooler-bg](https://github.com/way-cooler/way-cooler-bg): Draws the background for Way Cooler.
* [wc-grab](https://github.com/way-cooler/way-cooler-grab): Allows the user to take pictures of a screen.
* [wc-lock](https://github.com/way-cooler/way-cooler-lock): Locks Way Cooler and requires their password to be entered to unlock.

# Development

Way Cooler is currently in beta. The core features have been added and it is in a usable state, but there will be backwards-incompatible changes in future versions that might require some user intervention. 

Once Way Cooler reaches 1.0, we will guarantee backwards compatibilty for both the configuration files and the D-Bus interfaces.

Here are some pictures of what Way Cooler looks like today:

[![](http://i.imgur.com/UQAmli3.png)](http://i.imgur.com/UQAmli3.png)
[![](http://i.imgur.com/e89P4hw.png)](http://i.imgur.com/e89P4hw.png)
[![](http://imgur.com/A3V5x28.png)](http://imgur.com/A3V5x28.png)


## Motivation

We wanted to get experience with Rust and we found current X11 window managers to not have all the features we wanted.

Currently there are very few fully-featured tiling window managers in the Wayland ecosystem, as most of the effort has been porting Gnome and KDE over. Although Wayland is still in early-stage development
and is not backwards compatible with existing X11 tools, we wanted to put our stake in and provide for current tiling window manager users in the future.


## Current Features
- i3-style tiling
  * Horizontal/Vertical layouts
  * Tabbed/Stacked layouts
  * Nest containers with different layouts
  * Floating windows per workspace
- Client application support via the D-Bus IPC
  * See an example application [here](https://github.com/way-cooler/Way-Cooler-Example-Clients). It displays the tree in a somewhat organized format, and is actually really helpful for both debugging the tree and understanding how subcontainers work.
  * Enables dynamic configuration at runtime, without having to reload a configuration file
  * Allows extensions of the window manager to exist as separate programs talking over the IPC
- A Lua environment designed to make extending Way Cooler simple and easy
  * Lua is the configuration format, allowing the user to enhance their window manager in any way they want.
  * Utilities library included to aid communicating with Way Cooler
- X programs supported through XWayland
- Borders around containers
- Gaps between containers
- Basic X11 bar support (e.g [lemonbar][], [polybar][])
- Screen grabber / screen shot taker
- Lock screen

## Planned Features

- Notification support
- AwesomeWM compatibilty (see [this issue for more info](https://github.com/way-cooler/way-cooler/issues/338)
- A dedicated status bar
  * The status bar should be swappable, such that a user can implement their own or use a different one.
- More customization settings

Follow the development of these features in our [issues section] or checkout our [contribution guidelines](#Contributing) if you want to help out.

# Installation

## On the AUR

@vinipsmaker and @timidger maintain our AUR packages:

[way-cooler][way-cooler-aur]

[way-cooler-git][way-cooler-git-aur]

## NixOS

@miltador mantains our [NixOS package](https://github.com/NixOS/nixpkgs/blob/master/pkgs/applications/window-managers/way-cooler/default.nix).

## openSUSE

@jubalh maintains our [openSUSE package](https://build.opensuse.org/package/show/X11:windowmanagers/way-cooler).
Install with:

```
zypper ar -f obs://X11:windowmanagers windowmanagers
zypper in way-cooler
```

## Installation Script

For users who are not using the above mentioned Linux distributions, we have provided a simple install script that you can run in the terminal in order to install Way Cooler.

Please go to the [download page on our site](http://way-cooler.org/download) in order to download Way Cooler.

## Build from source

You will need the following dependencies installed on your machine to install Way Cooler:
- Wayland
  * Including the server and client libraries
- wlc
  * Installation instructions can be found on [their github page](https://github.com/Cloudef/wlc)
- Weston (optional)
  * The init file defaults to using `weston-terminal` as the default terminal emulator
- Cargo
  * The package manager / build system used by Rust
- Cairo

Finally, to install Way Cooler simply run the following cargo command:

```shell
cargo install way-cooler
```

You can try it out while running in an X environment, or switch to a TTY and run it as a standalone.

# Init File

All keyboard shortcuts are configurable through the init file. The recommended strategy is to copy the [default configuration file](https://github.com/way-cooler/way-cooler/blob/master/config/init.lua) to `$XDG_CONFIG_HOME/way-cooler/init.lua` and edit from there.

# Contributors
Way Cooler was started by @Timidger and @SnirkImmington, but these fine people have helped us:

- @vinipsmaker created (and maintains) AUR packages
- @miltador created NixOS package
- @starfys created way-cooler desktop file
- @toogley fixed a link
- @paulmenzel fixed a typo
- @thefarwind made kill way-cooler command rebindable
- @bluss for updating our use of `PetGraph` to use `StableGraph`
- @Abdillah
  + fixed background program to have solid colors be variable size at initilization
  + [added modes to the background program (fill, fit, stretch, title)](https://github.com/way-cooler/way-cooler-bg/pull/6#pullrequestreview-32859779)
- @hedning fixed an unsigned underflow in the tiling code
- @jubalh created the openSUSE package

And of course, thanks to the Rust community and the developers of [wlc].

# Contributing
Check out [Contributing](Contributing.md) for more information.

If you find bugs or have questions about the code, please [submit an issue] or [ping us on gitter][gitter].

[Rust]: https://www.rust-lang.org
[wayland]: https://wayland.freedesktop.org/
[Lua]: https://lua.org/
[wlc]: https://github.com/Cloudef/wlc
[i3]: https://i3wm.org
[D-Bus]: https://www.freedesktop.org/wiki/Software/dbus/
[awesome]: https://awesomewm.org/
[polybar]: https://github.com/jaagr/polybar
[lemonbar]: https://github.com/LemonBoy/bar
[issues section]: https://github.com/Immington-Industries/way-cooler/issues
[submit an issue]: https://github.com/Immington-Industries/way-cooler/issues/new
[gitter]: https://gitter.im/Immington-Industries/way-cooler?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge
[way-cooler-aur]: https://aur.archlinux.org/packages/way-cooler/
[way-cooler-git-aur]: https://aur.archlinux.org/packages/way-cooler-git/
