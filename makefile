default: build

# Debug build options
awesome:
	cargo build --package awesome

way_cooler:
	cargo build --package way-cooler

build:
	cargo build --all

run: build
	sleep .1 && WAYLAND_DISPLAY=wayland-1 ./target/debug/awesome &
	trap 'kill %1' SIGINT
	./target/debug/way-cooler

# Release build options
awesome_release:
	cargo build --release --package awesome

way_cooler_release:
	cargo build --release --package way-cooler

release:
	cargo build --all --release

run_release: release
	sleep .1 && WAYLAND_DISPLAY=wayland-1 ./target/release/awesome &
	trap 'kill %1' SIGINT
	./target/release/way-cooler

# Docs
man:
	./makedocs.sh -m manpages target/man

html:
	./makedocs.sh -h manpages target/html

docs: man html

# Tests
test: build
	./tests/awesome/tests/run_wayland.sh
