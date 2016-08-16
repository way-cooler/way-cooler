#Contributing to Way Cooler

We are always looking for more help to improve Way Cooler. Even if you don't know how to write any Rust code, we always need better documentation, detailed bug reports, and more features.

## Documentation
* API documentation makes sense to us, but that's only because we've been staring at it for months. Any improvements or questions about what a function or module does is greatly appreciated.

## Client Programs
* Awesome client programs that work well with Way Cooler. Maybe you need something that spawns a notification when someone tweets at you. Or perhaps you want to open your music app whenever it detects your editor open. Checkout the [client libs](https://github.com/Immington-Industries/way-cooler-client-libs) and our [example clients](https://github.com/Immington-Industries/Way-Cooler-Example-Clients) for more information.
* More language wrappers around our IPC communication would help you or someone else make a great app that interfaces with Way Cooler. See the `docs/` for more information on the protocol.
* We expose a lot of information through the IPC. If you need something else exposed for an app, open an issue! On the other hand, if you think there might be a issue, such as witch security, with the data we expose, open an issue for that instead.

## Way Cooler
* There are a lot of small things that always needed to be done around Way Cooler. Sometimes it's a little clean up of technical debt, perhaps a bug fix or two, or some of the groundwork for a new feature. Ping Timidger or Snirkimmington on [gitter](https://gitter.im/Immington-Industries/way-cooler?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge) to see how you can help out, or feel free to comment in any issue that you think you can help out in.
* One of the things we really need help on is our client bar. There's still a lot to discuss about how it will be designed and the kinds of things users can do with it. Come join the discussion [here](https://gitter.im/Immington-Industries/way-cooler?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge)
* Our tiling system is pretty good, but it's still missing a few features. One of the major things we need to add are status bars (server-side decorations) around all the windows. That's a big blocker for a whole heap of features.
