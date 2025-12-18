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

update:
	git pull
	$(MAKE) build
	sudo $(MAKE) install
	systemctl --user restart broadlink-remote.service

# Binary Installation (from Releases)
# If you downloaded the binary and .service file from:
# http://192.168.1.143:3000/PonyLucky/BroadlinkRemote/releases/latest
# You can install them using:
#   sudo make install-binary BINARY=./broadlink-remote-linux
#   make install-service

install-binary:
	install -D -m 755 $(BINARY) $(DESTDIR)$(PREFIX)/bin/$(BINNAME)

install-service:
	mkdir -p $(HOME)/.config/systemd/user/
	cp broadlink-remote.service $(HOME)/.config/systemd/user/
	systemctl --user daemon-reload
	systemctl --user enable --now broadlink-remote.service

.PHONY: all build install clean uninstall update install-binary install-service
