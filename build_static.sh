#!/bin/sh

# List out the libraries explicitly
export RUSTFLAGS="-l:libwlc.a -l:libwlc-protos.a -l:libchck-buffer.a -l:libchck-xdg.a -l:libchck-atlas.a -l:libchck-dl.a -l:libchck-fs.a -l:libchck-pool.a -l:libchck-sjis.a -l:libchck-string.a -l:libchck-unicode.a -l:libchck-tqueue.a -l:libchck-lut.a"
cargo build --features=static-wlc --release
