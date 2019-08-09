# <img src="http://i.imgur.com/OGeL1nN.png" width="60"> Way Cooler
[![Crates.io](https://img.shields.io/crates/v/way-cooler.svg)](https://crates.io/crates/way-cooler)
[![Downloads](https://img.shields.io/crates/d/way-cooler.svg)](https://crates.io/crates/way-cooler)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/way-cooler/way-cooler/)

Way Cooler is the spiritual successor of [AwesomeWM][] for [Wayland][]. It uses [wlroots][].

## Building

To build Way Cooler, ensure you have meson installed (as well as wlroots, or use
the `subprojects/` directory and build it locally).

Then, execute:

```bash
meson build
ninja -C build
```

To run the compositor simply execute `build/way-cooler/way-cooler` in a TTY or
any existing window manager.

It can run with [this patched version of the Awesome
client](https://github.com/way-cooler/awesome). The simplest way to execute both
is to run `way-cooler -c /path/to/patched/awesome>`.

Though technically they can run standalone, the compositor is not usable by
itself and the client will fail out in other compositor due to the custom
protocols not being present.

## Development

Way Cooler is under active development. If you would like to contribute you can
contact me best on [IRC][] (I also hang out on freenode).

**Master is not usable for production**. There are old versions of Way Cooler
that do work, however:

* They use an old framework, [wlc][], and thus are very limited and buggy.
* Was not designed to emulate Awesome, but instead has [i3][] tiling and its own
  (very incomplete) Lua libraries.

[Wayland]: https://wayland.freedesktop.org/
[wlc]: https://github.com/Cloudef/wlc
[AwesomeWM]: https://awesomewm.org/
[wlroots]: https://github.com/swaywm/wlroots
[IRC]: https://webchat.oftc.net/?channels=awesome&uio=d4
[i3]: https://i3wm.org
