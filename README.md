# <img src="http://i.imgur.com/OGeL1nN.png" width="60"> Way Cooler [![Crates.io](https://img.shields.io/crates/v/way-cooler.svg)](https://crates.io/crates/way-cooler) [![Downloads](https://img.shields.io/crates/d/way-cooler.svg)](https://crates.io/crates/way-cooler) [![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/way-cooler/way-cooler/)

Way Cooler is the spiritual successor of [AwesomeWM][] for [Wayland][]. It is written in [Rust][] and uses [wlroots][].


# Building
To build Way Cooler, make sure to init the submodules correctly:

```bash
git submodule update --init --recursive
```

To build a debug build use `make`. `make run` will run a debug version of `way-cooler` and `awesome`. 

Use `make release` to build a release version and `make run_release` to run the release version of Way Cooler.

To run the Awesome tests use `make test`.

To get the docs, run `make docs`. Man pages will be in `target/man` and web pages in `target/html`

# Development

Way Cooler is currently not being developed right now. I have decided to focus on wlroots-rs for now and may come back to this project later.
I am still accepting patches and willing to mentor anyone seeking to contribute. You can contact me on [IRC][].

Master is not usable for production. There are old versions of Way Cooler that do work, however:
* They use an old framework, [wlc][], and thus are very limited and buggy.
* Was not designed to emulate Awesome, but instead has [i3][] tiling and its own (very incomplete) Lua libraries.

[Rust]: https://www.rust-lang.org
[Wayland]: https://wayland.freedesktop.org/
[wlc]: https://github.com/Cloudef/wlc
[AwesomeWM]: https://awesomewm.org/
[submit an issue]: https://github.com/Immington-Industries/way-cooler/issues/new
[wlroots]: https://github.com/swaywm/wlroots
[IRC]: https://webchat.oftc.net/?channels=awesome&uio=d4
[i3]: https://i3wm.org
