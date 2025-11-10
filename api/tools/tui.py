#!/usr/bin/env python3
"""
Interactive TUI (text UI) to:
- Discover Broadlink controllers on the network and store them
- Learn new IR/RF commands for new or existing devices

All data is stored in an XML file compatible with the application's
broadlink.xml format. The XML is saved under the local tools directory
as tools/broadlink.xml. If it doesn't exist, it will be created.

Note: Positional attributes (x, y, width, height) are NOT handled yet,
      as requested. Commands will only include the name and payload.
"""
import os
import sys
import time
import base64
import xml.etree.ElementTree as ET
from typing import Optional, List

try:
    import broadlink
    from broadlink.exceptions import ReadError, StorageError
except Exception:  # pragma: no cover
    print("This tool requires the 'broadlink' package.\nInstall with: pip install broadlink")
    sys.exit(1)

# Where to save the XML (in the local tools directory)
SCRIPTS_DIR = os.path.dirname(os.path.abspath(__file__))
XML_PATH = os.path.join(SCRIPTS_DIR, "broadlink.xml")

TIMEOUT = 30  # seconds for learning


# ------------------------- XML helpers -------------------------

def _indent(elem: ET.Element, level: int = 0) -> None:
    """Pretty-print helper: indents children and aligns closing tags correctly.

    Rules:
    - For elements with children: opening tag on its own line, children indented one level.
    - Only the last child sets its tail to the parent's indent (so the closing tag unindents).
    - All previous siblings keep their tail at the child indent (so the next sibling stays indented).
    - Leaf elements keep inline text and only adjust tail indentation if needed.
    """
    parent_indent = "\n" + level * "    "
    child_indent = parent_indent + "    "
    if len(elem):
        if not elem.text or not elem.text.strip():
            elem.text = child_indent
        children = list(elem)
        last_index = len(children) - 1
        for idx, child in enumerate(children):
            _indent(child, level + 1)
            # Determine desired tail: child indent for all but the last child
            desired_tail = parent_indent if idx == last_index else child_indent
            if not child.tail or not child.tail.strip():
                child.tail = desired_tail
            else:
                # If tail exists but is only whitespace with wrong indent, normalize it
                if child.tail.strip() == "":
                    child.tail = desired_tail
    else:
        if level and (not elem.tail or not elem.tail.strip()):
            elem.tail = parent_indent


def load_or_create_xml(path: str) -> ET.ElementTree:
    if not os.path.exists(path):
        root = ET.Element('broadlink')
        tree = ET.ElementTree(root)
        save_xml(tree, path)
        return tree
    return ET.parse(path)


def save_xml(tree: ET.ElementTree, path: str) -> None:
    root = tree.getroot()
    _indent(root)
    tree.write(path, encoding='utf-8', xml_declaration=True)


def find_controller(root: ET.Element, name: str) -> Optional[ET.Element]:
    for c in root.findall('controler'):
        if c.get('name') == name:
            return c
    return None


def upsert_controller(root: ET.Element, *, name: str, ip: str, port: int,
                      devtype_hex: str, mac: str, model: Optional[str],
                      friendly_name: Optional[str]) -> ET.Element:
    c = find_controller(root, name)
    if c is None:
        c = ET.SubElement(root, 'controler')  # Keep tag name consistent with existing file
    # Update attributes
    c.set('name', name)
    if friendly_name:
        c.set('friendly-name', friendly_name)
    c.set('ip', ip)
    c.set('port', str(port))
    c.set('type', devtype_hex)
    c.set('mac', mac)
    if model:
        c.set('model', model)
    return c


def find_device(ctrl_el: ET.Element, dev_name: str) -> Optional[ET.Element]:
    for d in ctrl_el.findall('device'):
        if d.get('name') == dev_name:
            return d
    return None


def upsert_device(ctrl_el: ET.Element, *, name: str, dev_type: str = 'ir',
                  manufacturer: Optional[str] = None, model: Optional[str] = None,
                  friendly_name: Optional[str] = None) -> ET.Element:
    d = find_device(ctrl_el, name)
    if d is None:
        d = ET.SubElement(ctrl_el, 'device')
    d.set('name', name)
    d.set('type', dev_type or 'ir')
    if friendly_name:
        d.set('friendly-name', friendly_name)
    if manufacturer:
        # Preserve existing misspelling for compatibility (manifacturer)
        d.set('manifacturer', manufacturer)
    if model:
        d.set('model', model)
    return d


def _find_command_in_group(group_el: ET.Element, name: str) -> Optional[ET.Element]:
    # Look for command in this group
    for cmd in group_el.findall('command'):
        if cmd.get('name') == name:
            return cmd
    # Recurse into subgroups
    for sg in group_el.findall('group'):
        found = _find_command_in_group(sg, name)
        if found is not None:
            return found
    return None


def find_command_recursive(dev_el: ET.Element, name: str) -> Optional[ET.Element]:
    """Find a command element by name anywhere under a device element.

    This searches both top-level <command> elements and recursively inside
    nested <group> elements. Returns the first match found, or None.
    """
    # Top-level commands
    for cmd in dev_el.findall('command'):
        if cmd.get('name') == name:
            return cmd
    # Groups
    for g in dev_el.findall('group'):
        found = _find_command_in_group(g, name)
        if found is not None:
            return found
    return None


def add_or_replace_command(dev_el: ET.Element, *, name: str, payload_hex: str) -> ET.Element:
    """Add a new command or replace the payload of an existing one.

    If a command with the same name already exists anywhere under the device
    (including inside nested groups), only its text payload is updated so that
    any existing positional attributes (x, y, width, height) and other flags
    (like disabled) are preserved. If not found, a new top-level command is
    created.
    """
    existing = find_command_recursive(dev_el, name)
    if existing is not None:
        existing.text = payload_hex
        return existing
    # Not found: create a new top-level command (no positional attributes)
    cmd = ET.SubElement(dev_el, 'command')
    cmd.set('name', name)
    cmd.text = payload_hex
    return cmd


# ------------------------- Broadlink helpers -------------------------

def discover_devices() -> List["broadlink.Device"]:
    print("Discovering... (this may take a few seconds)")
    devices = broadlink.discover()
    ok = []
    for d in devices:
        try:
            if d.auth():
                ok.append(d)
        except Exception:
            pass
    return ok


def mac_bytes_to_str(mac_bytes: bytes) -> str:
    # Discovery gives a bytes-like in little endian order for the library usage.
    # For display and storage, we show a conventional MAC (big-endian, colon separated).
    # Reverse the bytes when printing to human format.
    b = bytes(mac_bytes)[::-1]
    return ":".join(f"{x:02X}" for x in b)


# ------------------------- Learning helpers -------------------------

def learn_ir_packet(dev):
    print("Entering IR learning mode... Press the remote button now.")
    dev.enter_learning()
    start = time.time()
    while time.time() - start < TIMEOUT:
        time.sleep(1)
        try:
            data = dev.check_data()
        except (ReadError, StorageError):
            continue
        else:
            print("Packet received!")
            return data
    print("No data received in time.")
    return None


def rf_find_and_learn_packet(dev: "broadlink.rm", frequency: Optional[float] = None):
    if frequency is None:
        print("Sweeping radio frequency... Hold the RF button to lock frequency.")
        dev.sweep_frequency()
        start = time.time()
        while time.time() - start < TIMEOUT:
            time.sleep(1)
            locked, freq = dev.check_frequency()
            if locked:
                frequency = freq
                break
        else:
            print("Failed to detect RF frequency in time.")
            try:
                dev.cancel_sweep_frequency()
            except Exception:
                pass
            return None
        print(f"RF locked at {frequency} MHz. Release the button, then press it briefly.")
        input("Press Enter to continue...")
    else:
        print(f"Using provided RF frequency: {frequency} MHz")

    dev.find_rf_packet(frequency)
    start = time.time()
    while time.time() - start < TIMEOUT:
        time.sleep(1)
        try:
            data = dev.check_data()
        except (ReadError, StorageError):
            continue
        else:
            print("RF packet received!")
            return data
    print("No RF data received in time.")
    return None


# ------------------------- Menu actions -------------------------

def action_discover_and_add(tree: ET.ElementTree) -> None:
    root = tree.getroot()
    devices = discover_devices()
    if not devices:
        print("No devices found.")
        return

    for idx, d in enumerate(devices, 1):
        mac_str = mac_bytes_to_str(d.mac)
        print(f"[{idx}] type={hex(d.devtype)} ip={d.host[0]} mac={mac_str}")

    while True:
        sel = input("Select a controller to add (number, empty to cancel): ").strip()
        if not sel:
            return
        if sel.isdigit() and 1 <= int(sel) <= len(devices):
            d = devices[int(sel) - 1]
            break
        print("Invalid selection.")

    name = input("Controller name (identifier): ").strip() or f"controller_{int(time.time())}"
    friendly = input("Friendly name (display, optional): ").strip() or None
    model = getattr(d, 'model', None)

    ctrl_el = upsert_controller(
        root,
        name=name,
        ip=d.host[0],
        port=d.host[1] if isinstance(d.host, (list, tuple)) and len(d.host) > 1 else 80,
        devtype_hex=hex(d.devtype),
        mac=mac_bytes_to_str(d.mac),
        model=model,
        friendly_name=friendly,
    )

    # Ensure XML persists
    save_xml(tree, XML_PATH)
    print(f"Controller '{name}' saved to {XML_PATH}.")

    # Offer to add a first device container
    if input("Add a device under this controller now? [y/N]: ").strip().lower() == 'y':
        action_add_device(tree, ctrl_el)
        save_xml(tree, XML_PATH)


def action_add_controller_manual(tree: ET.ElementTree) -> None:
    root = tree.getroot()
    name = input("Controller name (identifier): ").strip()
    if not name:
        print("Name is required.")
        return
    ip = input("Controller IP: ").strip()
    port_str = input("Port [80]: ").strip() or '80'
    devtype_hex = input("Device type (hex, e.g., 0x520b): ").strip() or '0x0000'
    mac = input("MAC address (e.g., AA:BB:CC:DD:EE:FF): ").strip()
    model = input("Model (optional): ").strip() or None
    friendly = input("Friendly name (optional): ").strip() or None

    try:
        port = int(port_str)
    except ValueError:
        print("Invalid port.")
        return

    upsert_controller(root, name=name, ip=ip, port=port, devtype_hex=devtype_hex,
                      mac=mac, model=model, friendly_name=friendly)
    save_xml(tree, XML_PATH)
    print(f"Controller '{name}' saved to {XML_PATH}.")


def pick_controller(root: ET.Element) -> Optional[ET.Element]:
    ctrls = root.findall('controler')
    if not ctrls:
        print("No controllers found in XML. Add one first (discover or manual).")
        return None
    for i, c in enumerate(ctrls, 1):
        print(f"[{i}] {c.get('name')} ({c.get('friendly-name') or ''}) ip={c.get('ip')}")
    while True:
        sel = input("Pick controller (number, empty to cancel): ").strip()
        if not sel:
            return None
        if sel.isdigit() and 1 <= int(sel) <= len(ctrls):
            return ctrls[int(sel) - 1]
        print("Invalid selection.")


def action_add_device(tree: ET.ElementTree, ctrl_el: Optional[ET.Element] = None) -> None:
    root = tree.getroot()
    if ctrl_el is None:
        ctrl_el = pick_controller(root)
        if ctrl_el is None:
            return
    name = input("Device name (identifier): ").strip()
    if not name:
        print("Device name is required.")
        return
    friendly = input("Friendly name (optional): ").strip() or None
    dev_type = input("Device type [ir]: ").strip() or 'ir'
    manufacturer = input("Manufacturer (optional): ").strip() or None
    model = input("Model (optional): ").strip() or None

    upsert_device(ctrl_el, name=name, dev_type=dev_type, manufacturer=manufacturer,
                  model=model, friendly_name=friendly)
    save_xml(tree, XML_PATH)
    print(f"Device '{name}' saved under controller '{ctrl_el.get('name')}'.")


def _connect_controller(ctrl_el: ET.Element):
    """Try to get a device handle using IP via broadlink.hello()."""
    ip = ctrl_el.get('ip') or ''
    if not ip:
        print("Controller has no IP configured.")
        return None
    try:
        dev = broadlink.hello(ip)
        if not dev.auth():
            print("Auth failed with controller.")
            return None
        return dev
    except Exception as e:
        print(f"Connection failed: {e}")
        return None


def action_learn_command(tree: ET.ElementTree, rf: bool = False) -> None:
    root = tree.getroot()
    ctrl_el = pick_controller(root)
    if ctrl_el is None:
        return

    # Ensure at least one device exists or create one
    devs = ctrl_el.findall('device')
    if not devs:
        print("No devices under this controller. Let's create one.")
        action_add_device(tree, ctrl_el)
        devs = ctrl_el.findall('device')
        if not devs:
            return

    for i, d in enumerate(devs, 1):
        print(f"[{i}] {d.get('name')} ({d.get('friendly-name') or ''}) type={d.get('type')}")
    while True:
        sel = input("Pick device (number), or 'n' to add a new one, or empty to cancel: ").strip().lower()
        if not sel:
            return
        if sel == 'n':
            action_add_device(tree, ctrl_el)
            devs = ctrl_el.findall('device')
            for i, d in enumerate(devs, 1):
                print(f"[{i}] {d.get('name')} ({d.get('friendly-name') or ''}) type={d.get('type')}")
            continue
        if sel.isdigit() and 1 <= int(sel) <= len(devs):
            dev_el = devs[int(sel) - 1]
            break
        print("Invalid selection.")

    # Connect to controller
    device_handle = _connect_controller(ctrl_el)
    if device_handle is None:
        return

    print("Learning mode will start. You have 30 seconds to press the button.")
    if rf:
        inp = input("Provide RF frequency in MHz (optional, Enter to auto-detect): ").strip()
        freq = float(inp) if inp else None
        data = rf_find_and_learn_packet(device_handle, frequency=freq)
    else:
        data = learn_ir_packet(device_handle)

    if not data:
        return

    # Show in multiple formats and pick payload to store (hex like the provided XML)
    raw_hex = data.hex()
    base64_fmt = base64.b64encode(data).decode('ascii')
    print(f"Raw hex: {raw_hex}")
    print(f"Base64:  {base64_fmt}")

    cmd_name = input("Command name to store: ").strip()
    if not cmd_name:
        print("Command name is required. Aborting.")
        return

    add_or_replace_command(dev_el, name=cmd_name, payload_hex=raw_hex)
    save_xml(tree, XML_PATH)
    print(f"Command '{cmd_name}' saved under device '{dev_el.get('name')}'.")


def action_list_controllers(tree: ET.ElementTree) -> None:
    root = tree.getroot()
    ctrls = root.findall('controler')
    if not ctrls:
        print("No controllers stored yet.")
        return
    for c in ctrls:
        print(f"- {c.get('name')} ({c.get('friendly-name') or ''}) ip={c.get('ip')} type={c.get('type')} mac={c.get('mac')}")
        devs = c.findall('device')
        for d in devs:
            print(f"    * {d.get('name')} ({d.get('friendly-name') or ''}) type={d.get('type')}")


# ------------------------- Main loop -------------------------

def main():
    tree = load_or_create_xml(XML_PATH)

    actions = {
        '1': ("Discover controllers and add", action_discover_and_add),
        '2': ("List stored controllers and devices", action_list_controllers),
        '3': ("Add controller manually", action_add_controller_manual),
        '4': ("Add a device to a controller", action_add_device),
        '5': ("Learn IR command for a device", lambda t: action_learn_command(t, rf=False)),
        '6': ("Learn RF command for a device", lambda t: action_learn_command(t, rf=True)),
        '0': ("Exit", None),
    }

    while True:
        print("\n=== Broadlink TUI ===")
        for k in sorted(actions.keys()):
            print(f"{k}. {actions[k][0]}")
        choice = input("Select an option: ").strip()
        if choice == '0' or choice.lower() in ('q', 'quit', 'exit'):
            print("Bye.")
            break
        action = actions.get(choice)
        if not action:
            print("Unknown choice.")
            continue
        fn = action[1]
        if fn is None:
            break
        try:
            # Some actions expect controller element (action_add_device), handle that by passing tree only
            if fn is action_add_device:
                fn(tree, None)  # type: ignore[arg-type]
            else:
                fn(tree)  # type: ignore[misc]
        except KeyboardInterrupt:
            print("\nCancelled by user.")
        except Exception as e:
            print(f"Error: {e}")


if __name__ == '__main__':
    main()
