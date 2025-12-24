# Broadlink Remote (Linux)

A lightweight Linux port of the Broadlink Remote application, originally written for macOS. This version is written in Rust and integrates natively with Linux desktop environments using DBus.

## Features

- **System Tray (SNI)**: Native system tray icon with dynamic menus for controlling your devices and running scripts.
- **MPRIS2 Support**: Exposed as a virtual media player. You can control your remote devices using system-wide media keys or desktop media widgets.
- **Dynamic Command Tree**: Automatically fetches your device configurations, groups, and commands from the Broadlink REST API.
- **Asynchronous & Efficient**: Built on `tokio` and `zbus` for high performance and low resource usage.

## Prerequisites

- **Rust**: [Install Rust](https://www.rust-lang.org/tools/install) (latest stable version recommended).
- **DBus Development Headers**: Required for communication with the system bus.
  - Ubuntu/Debian: `sudo apt install libdbus-1-dev`
  - Fedora: `sudo dnf install dbus-devel`
  - Arch: `sudo pacman -S dbus`

## Installation

### Option A: Download from Release (Recommended)

1. Download the latest binary and `.service` file from the [Releases](http://192.168.1.143:3000/PonyLucky/BroadlinkRemote/releases/latest) page.
2. Install the binary:
   ```bash
   sudo make install-binary BINARY=./broadlink-remote-linux
   ```
   Or:
   ```bash
   # Variables
   PREFIX ?= /usr/local
   BINNAME = broadlink-remote-linux
   BINARY = ./broadlink-remote-linux  # name of the binary to install
   # Install command
   install -D -m 755 $(BINARY) $(DESTDIR)$(PREFIX)/bin/$(BINNAME)
   ```
3. Install the service:
   ```bash
   make install-service
   ```
   Or:
   ```bash
   mkdir -p $(HOME)/.config/systemd/user/
   cp broadlink-remote.service $(HOME)/.config/systemd/user/
   systemctl --user daemon-reload
   systemctl --user enable --now broadlink-remote.service
   ```

### Option B: Build from source

### 1. Build the project
```bash
make build
```
Or directly via cargo:
```bash
cargo build --release
```

### 2. Install the binary
By default, it installs to `/usr/local/bin`.
```bash
sudo make install
```

### 3. Setup Systemd Service (User session)
To have the application start automatically when you log in:

```bash
make install-service
```

## Update

To update the application to the latest version:

```bash
make update
```

This will pull the latest changes, rebuild the binary, reinstall it, and restart the systemd service.

## Configuration

The application stores its configuration in a JSON file located at:
`~/.config/broadlink-remote/config.json`

This file is automatically created with default values on the first run.

### Configuration Fields

- **`host`**: The IP address or hostname of your Broadlink REST API service (Default: `192.168.1.143`).
- **`port`**: The port number of the Broadlink REST API service (Default: `6676`).
- **`selected_controllers`**: A list of controller names that should be visible in the tray menu.
- **`tray_icon`**: The name of the icon to use in the system tray (Default: `preferences-desktop-peripherals`).
- **`mpris`**: Settings for MPRIS2 integration:
    - **`enable`**: Boolean to enable/disable the virtual media player.
    - **`controller`**: The Broadlink controller to use for MPRIS commands.
    - **`device`**: The device to control via MPRIS.
    - **`commands`**: Mapping for media keys (`play-pause`, `previous`, `next`) to specific device commands.

### Example Configuration

```json
{
  "host": "192.168.1.143",
  "port": 6676,
  "selected_controllers": [
    "Living Room"
  ],
  "tray_icon": "preferences-desktop-peripherals",
  "mpris": {
    "enable": true,
    "controller": "Living Room",
    "device": "TV",
    "commands": {
      "play-pause": "Power",
      "previous": "Volume Down",
      "next": "Volume Up"
    }
  }
}
```

## Developer Notes

### Useful Commands

- **Run in development mode**:
  ```bash
  cargo run
  ```

- **Run with debug logging**:
  ```bash
  RUST_LOG=debug cargo run
  ```

- **Check for compilation errors**:
  ```bash
  cargo check
  ```

- **Run tests**:
  ```bash
  cargo test
  ```

- **Update dependencies**:
  ```bash
  cargo update
  ```

### Project Structure

- `src/api_client`: Handles communication with the Broadlink REST API.
- `src/mpris`: Implementation of the MPRIS2 DBus interface.
- `src/tray`: Implementation of the StatusNotifierItem (System Tray).
- `src/state`: Centralized application state and synchronization logic.
- `src/main.rs`: Entry point and async runtime orchestration.

### DBus Troubleshooting
You can monitor MPRIS messages using `playerctl` or `dbus-monitor`:
```bash
# Monitor metadata updates
playerctl --player=broadlink_remote metadata

# Listen to DBus signals
dbus-monitor "sender='org.mpris.MediaPlayer2.broadlink_remote'"
```
