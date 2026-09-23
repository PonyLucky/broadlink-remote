.PHONY: build start dev test clean install

build:
	cargo build --release

start: build
	./target/release/broadlink-remote-tui

dev:
	cargo run

test:
	cargo test

clean:
	cargo clean

install: build
	install -D -m 755 ./target/release/broadlink-remote-tui $(DESTDIR)/usr/local/bin/broadlink-remote-tui