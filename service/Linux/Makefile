PREFIX ?= /usr/local
BINNAME = broadlink-remote-linux

all: build

build:
	cargo build --release

install:
	install -D -m 755 target/release/$(BINNAME) $(DESTDIR)$(PREFIX)/bin/$(BINNAME)

clean:
	cargo clean

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/bin/$(BINNAME)

.PHONY: all build install clean uninstall
