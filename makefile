default: run

build:
	cargo build --all

run: build
	sleep .1 && WAYLAND_DISPLAY=wayland-1 ./target/debug/awesome &
	trap 'kill %1' SIGINT
	./target/debug/way-cooler

awesome:
	./target/debug/awesome

way_cooler:
	./target/debug/way-cooler

man:
	./makedocs.sh -m manpages target/man

html:
	./makedocs.sh -h manpages target/html

docs: man html