# <img src="http://i.imgur.com/OGeL1nN.png" width="60"> Way Cooler
[![Crates.io](https://img.shields.io/crates/v/way-cooler.svg)](https://crates.io/crates/way-cooler)
[![Downloads](https://img.shields.io/crates/d/way-cooler.svg)](https://crates.io/crates/way-cooler)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/way-cooler/way-cooler/)

Way Cooler is the spiritual successor of [AwesomeWM][] for [Wayland][]. It is
written in [Rust][] and C. It uses [wlroots][].

## Building
To build Way Cooler, ensure you have meson installed (as well as wlroots, or use
the `subprojects/` directory and build it locally).

Then, execute:

```bash
meson build
ninja -C build
```

This will build a debug version of the compositor and the client. The compositor
binary will be placed in `build/compositor/way-cooler` and the client will be
placed in `build/client/debug/way-cooler/client`. To execute them both, you can
pass the client program (or any program for that matter) as an argument to
`way-cooler`:

```bash
./build/compositor/way-cooler -c ./build/client/debug/way-cooler-client
```

Both pieces are designed to run stand-alone, though neither is really useful
(yet) without the other. 

The compositor is a bare-bones Wayland compositor. 

The client is a Wayland client that implements exactly what the old AwesomeWM
program used to do but with Wayland instead of X11.

## Development

Way Cooler is under active development (again). If you would like to contribute
you can contact me best on [IRC][] (I also hang out on freenode).

**Master is not usable for production**. There are old versions of Way Cooler that do work, however:
* They use an old framework, [wlc][], and thus are very limited and buggy.
* Was not designed to emulate Awesome, but instead has [i3][] tiling and its own (very incomplete) Lua libraries.

[Rust]: https://www.rust-lang.org
[Wayland]: https://wayland.freedesktop.org/
[wlc]: https://github.com/Cloudef/wlc
[AwesomeWM]: https://awesomewm.org/
[wlroots]: https://github.com/swaywm/wlroots
[IRC]: https://webchat.oftc.net/?channels=awesome&uio=d4
[i3]: https://i3wm.org
