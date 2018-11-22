# rustfmt
rustfmt is used to format this project automatically. The CI is set up so that any code that doesn't follow the formatting rules is denied.

Please run `rustfmt` before committing, as it's very annoying to find git commits that simply format code. To aid in this, we humbly suggest you add this as a `pre-commit` git hook:

```bash
function checkfmt() {
    expected="rustfmt 1.0.0"
    version=$(rustfmt --version)
    if [ "${version#*$expected}" == "$version" ]; then
       echo "Bad version of rustfmt: $(rustfmt --version)"
       echo "Expected $expected"
       return 1
    fi
    formatted=$(cargo fmt -- --check)
    [ -z "$formatted" ] && return 0

    echo >&2 "Please format your files with cargo fmt"

    return 1
}

checkfmt || exit 1
```

Note that we also lock it to 1.0.0 to avoid any changes between versions.

# Line Length
Try to keep your lines under 80 columns, but you can go up to 100 if it improves readability. Don't break lines indiscriminately, try to find nice breaking points so your code is easy to read.

rustfmt will normally take care of this for you. When it comes to strings use `\` to break up multiple lines like so:

```
"This is a very long line of text, because the author did not realize that \
brevity is the soul of wit"
```

Note you don't need to have the next line start at the beginning of the line like in e.g. Python.

# Tabs vs Spaces
Use 4 spaces for indentation. Tabs have nice characteristics, but most of the Rust ecosystem uses 4 spaces.

# Importing
Imports should be separated into 3 sections: standard library, crate imports, local lib/binary imports. Within these groups the imports should be sorted in alphabetical order. Each of these groups must be separated by a single empty line.

Standard library imports should be one import statement using the nested `{}` to separate different modules. If using an older version of Rust where this feature is disallowed multiple lines per module import group is allowed.

Crate imports should be one use expression per crate, unless an older version of Rust is used where that's disallowed.

The crate imports should not be prefixed with the leading `::`. E.g. `libc::c_void` instead of `::libc::c_void`. This is to avoid confusion with the local imports.

Local imports should always be prefixed with `::`. E.g. `::common::Signal`.

Using `self::` is discouraged unless it's necessary.

The following example is taken from a (simplified) import section in Way Cooler:

```rust
use std::{env, mem, path::PathBuf, process::exit, io::{self, Write},
          os::unix::io::RawFd};

use exec::Command;
use rlua::{LightUserData, Lua, Table};

use ::lua::{LUA, NEXT_LUA};
use ::wayland_protocols::xdg_shell::xdg_wm_base;
```

# Unused Parameters
If a parameter of a function is unused (and cannot be removed because it's conforming to an interface) the variable name shall be `_` if it is a non-std type. If it is a standard or simple type (such as `String` or `bool`) then a descriptive variable name should be provided prefixed with `_`.

If it is a non-std type but still ambigious then you may give it a name. This is best evaluated on a case-by-case basis.

If the code is part of an example in `examples/` it is strongly encouraged to give it a descriptive name with a `_` prefixed.

An example using rlua:

```rust
fn process_flags(_: &rlua::Lua, (obj, _activated): (rlua::Table, bool))
                 -> rlua::Result<()> {
    assert!(obj.get::<_, bool>("activated")?);
    Ok(())
}
```
