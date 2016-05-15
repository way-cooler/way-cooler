# way-cooler

[![Join the chat at https://gitter.im/Immington-Industries/way-cooler](https://badges.gitter.im/Immington-Industries/way-cooler.svg)][gitter]
[![Crates.io](https://img.shields.io/badge/crates.io-v0.2.0-orange.svg)](https://crates.io/crate/way-cooler)
[![Build Status](https://travis-ci.org/Immington-Industries/way-cooler.svg?branch=master)](https://travis-ci.org/Immington-Industries/way-cooler)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Immington-Industries/way-cooler/)

way-cooler is a customizeable tiling window manager written in [Rust][] for [Wayland][wayland]. It uses the Wayland compositor library [wlc][].

# Development

way-cooler is currently in alpha. We have most of our goals in mind, and are working on infrastructure now (nothing screenshot-worthy). 

## Motivation

We wanted to get experience with Rust and we found room for improvement in the window managers we use. Although Wayland gets some flack now for being in development
and not being backwards compatable with existing X11 tools, we wanted to put our stake in and provide for current tiling window manager users in the future.

We take a lot of inspiration from current window managers (namely [i3][] and [awesome][]) but the goal is to exist as a unique alternative.

## Planned Features
We will be tracking these in the [issues section][].

- Workspaces
- i3-style tiling windows (with additional automatic tiling configurations)
- Lua scriptibility
- Control Lua via inter-process channels
- Dynamic configuration
- Compatable versions of existing X tools/setup (`xmodmap`, `xbindkeys`, `xdotool`)
- Extensibility via other programs (i.e. a separate `way-cooler-bar`, `way-cooler-widgets`, etc.)

A lot of the graphical parts of way-cooler (a bar, widgets, etc.) will ultimately be separate programs.

# Installation

You will need Wayland, which should be availble from your distro's package manager.

These are the package names on Arch:
`wayland` for the Wayland protocol
`xorg-server-xwayland` to run X programs on Wayland (recommended)
`weston` provides `weston-terminal` (which can be the default terminal for way-cooler)

If you have Rust, way-cooler can be installed via cargo:

```shell
cargo install way-cooler
```

This will build way-cooler from source. 

For now, you will also need to install our non-Rust dependency, wlc. To do that, see [wlc's GitHub][wlc].

In the future we will have a cargo build script, which will install wlc in the `.cargo` folder.

# Compatability with X11

Wayland is designed to be backwards-compatable with X11 by running X programs using a program called `xwayland`.
Most existing X programs will run in way-cooler this way. 

To only run Wayland programs, set the environment variable `WLC_XWAYLAND` to 0.

# Controls

This alpha version currently supports these hardcoded controls: 

- `Alt+Enter` Launches a terminal defined by the `WAYLAND_TERMINAL` environment variable - 
if unset this defaults to `weston-terminal` which will require installing `weston` (should be available on most distros). 
Note that you can set `WAYLAND_TERMINAL` to be an X11 program, and it will run under `Xwayland`.
- `Alt+d` Open `dmenu` to launch a program
- `Alt+p` Shows off the Lua thread - prints the mouse coordinates to stdout using Lua
- `Alt+Esc` Closes way-cooler
- `Alt-1` through `Alt-9` Switch workspace
- `Ctrl-LeftMouse` Drag the focused window around the workspace
- `Ctrl-RightMouse` Resize the focused window (somewhat buggy at the moment)
- `Ctrl-Shift-LeftMouse` Maximize window

# Contributing
If you would like to contribute code, please feel free to fork and branch off of `development` and submit a pull request.

If you find bugs or have questions about the code, please [submit an issue] or ping us on gitter.

[Rust]: https://www.rust-lang.org
[wayland]: https://wayland.freedesktop.org/
[wlc]: https://github.com/Cloudef/wlc
[i3]: i3wm.org
[awesome]: https://awesomewm.org/
[issues section]: https://github.com/Immington-Industries/way-cooler/labels/features
[submit an issue]: https://github.com/Immington-Industries/way-cooler/issues/new
[gitter]: https://gitter.im/Immington-Industries/way-cooler?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge
