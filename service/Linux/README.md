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
mkdir -p ~/.config/systemd/user/
cp broadlink-remote.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now broadlink-remote.service
```

## Configuration

The application currently expects the Broadlink REST API to be available. (Note: Ensure your REST service host/port matches the defaults in the code or set them via environment variables if implemented).

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
