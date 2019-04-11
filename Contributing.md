# Style Guide

## General Guidelines

### Git

[Use good git commit messages](https://chris.beams.io/posts/git-commit/). Please
squash/rebase liberally.

## C

Way Cooler follows the [wlroots guidelines for its C
code](https://github.com/swaywm/wlroots/blob/master/CONTRIBUTING.md#style-reference).

## Rust

rustfmt is used to format this project automatically. The CI is set up so that
any code that doesn't follow the formatting rules is denied.

To install rustfmt execute `rustup component add rustfmt-preview --toolchain nightly`.

Please run `cargo fmt` before committing, as it's very annoying to find git
commits that simply format code. To aid in this, we humbly suggest you add this
as a `pre-commit` git hook:

```bash
function checkfmt() {
    expected="rustfmt 1.0.3-nightly"
    version=$(rustfmt --version)
    if [ "${version#*$expected}" == "$version" ]; then
       echo "Bad version of rustfmt: $(rustfmt --version)"
       echo "Expected $expected"
       return 1
    fi
    formatted=$(cd client; cargo fmt -- --check)
    [ -z "$formatted" ] && return 0

    echo >&2 "Please format your files with cargo fmt"

    return 1
}

checkfmt || exit 1
```

### Import Groups

Imports should be separated into 3 sections: standard library, 3rd party
imports, and local lib/binary imports. Within these groups the imports should be
sorted in alphabetical order. Each of these groups must be separated by a single
empty line.

Import statements should be grouped using the nested `{}` syntax.

Here is an example:

```rust
use std::{env, mem, path::PathBuf, process::exit, io::{self, Write},
          os::unix::io::RawFd};

use exec::Command;
use rlua::{LightUserData, Lua, Table};

use lua::{LUA, NEXT_LUA};
use wayland_protocols::xdg_shell::xdg_wm_base;
```

### Unused Parameters

If a parameter of a function is unused (and cannot be removed because it's
conforming to an interface) the variable name shall be `_` if it is a non-std
type. If it is a standard or simple type (such as `String` or `bool`) then a
descriptive variable name should be provided prefixed with `_`.

If it is a non-std type but still ambiguous then you may give it a name. This is
best evaluated on a case-by-case basis.

If the code is part of an example in `examples/` it is strongly encouraged to
give it a descriptive name with a `_` prefixed.

An example using rlua:

```rust
fn process_flags(_: &rlua::Lua, (obj, _activated): (rlua::Table, bool))
                 -> rlua::Result<()> {
    assert!(obj.get::<_, bool>("activated")?);
    Ok(())
}
```
