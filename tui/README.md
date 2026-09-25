# Broadlink Remote TUI

A terminal UI for controlling Broadlink remote devices. Implements the same API as the Linux system tray tool.

## Features

- Browse controllers, devices, and commands in a tree structure
- Friendly names displayed for controllers, devices, commands, and scripts (falls back to ID)
- Execute commands by selecting them and pressing Enter
- Run scripts
- Multi-controller support with tab switching
- Controller selection persisted across sessions
- Vim-like keyboard navigation (h/j/k/l)
- Configuration in `~/.config/broadlink-remote/config-tui.json` (auto-created)

## Usage

```bash
# Build
make build

# Run
make start

# Development (run without building first)
make dev

# Clean build artifacts
make clean

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

The TUI uses its own config file at `~/.config/broadlink-remote/config-tui.json`, which is automatically created on first run if it doesn't exist. The currently selected controller is saved when you switch controllers (Tab or controller view selection).

```json
{
  "host": "192.168.1.143",
  "port": 6676,
  "selected_controllers": []
}
```

The service's main config file (`~/.config/broadlink-remote/config.json`) is separate and contains additional settings like MPRIS and tray icon.