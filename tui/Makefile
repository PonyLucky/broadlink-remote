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
	mkdir -p $(HOME)/.local/bin
	install -m 755 ./target/release/broadlink-remote-tui $(HOME)/.local/bin/broadlink-remote-tui
	ln -sf $(HOME)/.local/bin/broadlink-remote-tui $(HOME)/.local/bin/br