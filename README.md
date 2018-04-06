# <img src="http://i.imgur.com/OGeL1nN.png" width="60"> Way Cooler [![Crates.io](https://img.shields.io/crates/v/way-cooler.svg)](https://crates.io/crates/way-cooler) [![Downloads](https://img.shields.io/crates/d/way-cooler.svg)](https://crates.io/crates/way-cooler) [![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/way-cooler/way-cooler/)

Way Cooler is a clone of [AwesomeWM][] written in [Rust][] for [Wayland][] using [wlroots][].


# Development

Way Cooler is under very active development right now. Currently, master is not usable for production. There are old versions of Way Cooler that do work, however:
* They use an old framework, [wlc][], and thus are very limited and buggy.
* Was not designed to emulate Awesome, but instead has [i3][] tiling and its own (very incomplete) Lua libraries.

There will be another release this year, v0.9.0. It will be the last release before 1.0. 1.0 is defined to be 100% compatible with AwesomeWM.

# Contributing
Check out [Contributing](Contributing.md) for more information.

If you find bugs or have questions about the code, please [submit an issue][] or ask us on [IRC][].

[Rust]: https://www.rust-lang.org
[Wayland]: https://wayland.freedesktop.org/
[wlc]: https://github.com/Cloudef/wlc
[AwesomeWM]: https://awesomewm.org/
[submit an issue]: https://github.com/Immington-Industries/way-cooler/issues/new
[wlroots]: https://github.com/swaywm/wlroots
[IRC]: https://webchat.freenode.net/?channels=awesome&uio=d4
