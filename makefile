default: run

build:
	cargo build --all

run: build
	trap 'kill %1' SIGINT
	./target/debug/way-cooler &
	./target/debug/awesome

awesome:
	./target/debug/awesome

way_cooler:
	./target/debug/way-cooler

man:
	./makedocs.sh -m manpages target/man

html:
	./makedocs.sh -h manpages target/html

docs: man html