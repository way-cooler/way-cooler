default: build

.PHONY: way_cooler
way_cooler:
	`cd way-cooler; ninja -C build meson build`

.PHONY: awesome
awesome:
	`cd awesome; cargo build`

build: awesome way_cooler

check:
	`cd awesome; cargo +nightly fmt --all -- --check; cargo clippy --all`

# way-cooler should be a shell script that does this
run: build check
	./build/way-cooler-compositor ./awesome/target/debug/way-cooler-awesome

# Docs
man:
	./makedocs.sh -m manpages target/man

html:
	./makedocs.sh -h manpages target/html

docs: man html

# Tests
test: build
	./test_wayland.sh
