# Tools

This directory contains helper tools for working with the BroadlinkRemote API and its XML configuration.

## `tui.py` — Text User Interface

An interactive terminal utility to manage the `broadlink.xml` configuration and to learn IR/RF commands from Broadlink controllers.

Features:
- Discover Broadlink controllers on the local network and add them to the XML.
- Add a controller manually (name, IP, port, device type, MAC, etc.).
- Add devices (e.g., TV, fan, amp) under a controller.
- Learn IR or RF commands and store them as hex payloads under a device.
- List stored controllers and devices.

Usage:

```
cd api/tools
python tui.py
```

Typical flow:
1. Discover controllers (or add manually).
2. Add a device under a controller (type `ir`/`rf`).
3. Learn an IR command, or learn an RF command (optionally specifying frequency), and save it.
4. Use the API or UI to trigger the stored commands.

Requirements:
- Python 3.9+
- `broadlink` Python package: `pip install broadlink`
- Network access to the Broadlink device(s)

Notes:
- The tool reads and writes the shared XML file at `api/broadlink.xml`.
- In case learning fails (e.g., due to RF frequency or timing), retry the action; the tool prints helpful messages.
- MAC address endianness nuances are handled when sending via the API, not required during learning.

## `pos_picker.html` — Position Picker

A tiny browser utility helpful when mapping click areas on images for the UI. It displays the mouse cursor coordinates relative to the page/image so you can copy exact positions.

Usage:
- Open the file in your browser: `api/tools/pos_picker.html`.
- Move your cursor over the image/page and copy the displayed coordinates.

This file is standalone and does not require a server.
