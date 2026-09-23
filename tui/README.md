# Broadlink Remote TUI

A terminal UI for controlling Broadlink remote devices. Implements the same API as the Linux system tray tool.

## Features

- Browse and control Broadlink devices from the terminal
- Navigate device command trees
- Execute commands and scripts
- Configuration shared with the system tray tool (`~/.config/broadlink-remote/config.json`)

## Usage

```bash
# Build
cargo build --release

# Run
./target/release/broadlink-remote-tui
```

## Controls

| Key | Action |
|-----|--------|
| ↑/↓ | Navigate |
| Enter | Select/Execute |
| Backspace/B | Go back |
| R | Refresh devices |
| Q/Esc | Quit |

## Configuration

Same config file as the system tray tool: `~/.config/broadlink-remote/config.json`

```json
{
  "host": "192.168.1.143",
  "port": 6676,
  "selected_controllers": [],
  "tray_icon": "preferences-desktop-peripherals",
  "mpris": {
    "enable": false,
    "controller": "",
    "device": "",
    "commands": {
      "play-pause": "",
      "previous": "",
      "next": ""
    }
  }
}
```