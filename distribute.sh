#!/bin/sh
set -euo pipefail
IFS=$'\n\t'
mkdir dist
echo "Building Way Cooler..."
cargo build --release --features static-wlc
cp target/release/way-cooler dist
echo "Building Way Cooler Background..."
(cd ../way-cooler-bg;
 cargo build --release
 cp target/release/way-cooler-bg ../way-cooler/dist)
echo "Bundling install script..."
cp install.sh dist
echo "Zipping folder"
mv dist way-cooler
tar -cvzf way-cooler.gz way-cooler
echo "Cleaning up..."
rm -r way-cooler
