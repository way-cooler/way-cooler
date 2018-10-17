default: run

build:
	cargo build --all

run: build way_cooler awesome

awesome:
	./target/debug/awesome

way_cooler:
	./target/debug/way-cooler

man:
	./makedocs.sh -m manpages target/man

html:
	./makedocs.sh -h manpages target/html

docs: man html