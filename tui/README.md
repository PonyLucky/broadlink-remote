# Broadlink Remote TUI

A terminal UI for controlling Broadlink remote devices. Implements the same API as the Linux system tray tool.

## Features

- Browse controllers, devices, and commands in a tree structure
- Friendly names displayed for controllers, devices, commands, and scripts (falls back to ID)
- Execute commands by selecting them and pressing Enter
- Run scripts
- Multi-controller support with tab switching
- Vim-like keyboard navigation (h/j/k/l)
- Configuration shared with the system tray tool (`~/.config/broadlink-remote/config.json`)

## Usage

```bash
# Build
make build

# Run
make start

# Development (run without building first)
make dev

# Install for current user
make install

# Run using installed binary
broadlink-remote-tui
# or use the short alias
br
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