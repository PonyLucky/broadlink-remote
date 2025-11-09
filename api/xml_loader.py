import os
import time
from typing import Dict, Any, List, Optional
import xml.etree.ElementTree as ET

from models import Command, Group, Device, Controller


class Config:
    def __init__(self, xml_path: str) -> None:
        self.xml_path = xml_path
        self._mtime = 0.0
        self.controllers: Dict[str, Controller] = {}
        self.reload_if_changed(force=True)

    def reload_if_changed(self, force: bool = False) -> None:
        try:
            mtime = os.path.getmtime(self.xml_path)
        except OSError:
            mtime = time.time()
        if force or mtime != self._mtime:
            self.controllers = self._parse_xml(self.xml_path)
            self._mtime = mtime

    def _parse_xml(self, path: str) -> Dict[str, Controller]:
        tree = ET.parse(path)
        root = tree.getroot()
        controllers: Dict[str, Controller] = {}
        for c_el in root.findall('controler'):
            c_name = c_el.get('name')
            if not c_name:
                continue
            controller = Controller(
                name=c_name,
                friendly_name=c_el.get('friendly-name'),
                ip=c_el.get('ip') or '0.0.0.0',
                port=int(c_el.get('port') or '80'),
                devtype_hex=c_el.get('type') or '0x0000',
                mac=c_el.get('mac') or '',
                model=c_el.get('model'),
                devices={}
            )
            for d_el in c_el.findall('device'):
                d_name = d_el.get('name')
                if not d_name:
                    continue
                device = Device(
                    name=d_name,
                    friendly_name=d_el.get('friendly-name'),
                    type=d_el.get('type') or 'ir',
                    manufacturer=d_el.get('manifacturer') or d_el.get('manufacturer'),
                    model=d_el.get('model'),
                    commands={},
                    groups={}
                )
                # Top-level commands
                for cmd_el in d_el.findall('command'):
                    cmd_name = cmd_el.get('name')
                    if not cmd_name:
                        continue
                    disabled = (cmd_el.get('disabled') or '').lower() == 'true'
                    payload = (cmd_el.text or '').strip()
                    device.commands[cmd_name] = Command(cmd_name, payload, disabled)
                # Groups (may be nested)
                for g_el in d_el.findall('group'):
                    grp = self._parse_group(g_el)
                    device.groups[grp.name] = grp
                controller.devices[device.name] = device
            controllers[controller.name] = controller
        return controllers

    def _parse_group(self, g_el: ET.Element) -> Group:
        g_name = g_el.get('name') or 'group'
        g_disabled = (g_el.get('disabled') or '').lower() == 'true'
        group = Group(name=g_name, disabled=g_disabled)
        for cmd_el in g_el.findall('command'):
            cmd_name = cmd_el.get('name')
            if not cmd_name:
                continue
            disabled = (cmd_el.get('disabled') or '').lower() == 'true'
            payload = (cmd_el.text or '').strip()
            group.commands[cmd_name] = Command(cmd_name, payload, disabled)
        for sg_el in g_el.findall('group'):
            subgroup = self._parse_group(sg_el)
            group.subgroups[subgroup.name] = subgroup
        return group

    # Lookup helpers
    def get_controller(self, name: str) -> Optional[Controller]:
        self.reload_if_changed()
        return self.controllers.get(name)

    def get_device(self, c_name: str, d_name: str) -> Optional[Device]:
        ctrl = self.get_controller(c_name)
        if not ctrl:
            return None
        return ctrl.devices.get(d_name)

    def list_commands(self, device: Device) -> Dict[str, Any]:
        def group_to_dict(g: Group) -> Dict[str, Any]:
            return {
                'name': g.name,
                'disabled': g.disabled,
                'commands': [
                    {'name': n, 'disabled': cmd.disabled}
                    for n, cmd in sorted(g.commands.items())
                ],
                'groups': [group_to_dict(sg) for _, sg in sorted(g.subgroups.items())]
            }

        return {
            'device': device.name,
            'friendly_name': device.friendly_name,
            'commands': [
                {'name': n, 'disabled': cmd.disabled}
                for n, cmd in sorted(device.commands.items())
            ],
            'groups': [group_to_dict(g) for _, g in sorted(device.groups.items())]
        }

    def resolve_command(self, device: Device, path: List[str]) -> Optional[Command]:
        # Try direct command name first (no group)
        if len(path) == 1 and path[0] in device.commands:
            return device.commands[path[0]]
        # Traverse groups
        def traverse(gmap: Dict[str, Group], idx: int) -> Optional[Command]:
            if idx >= len(path):
                return None
            g = gmap.get(path[idx])
            if not g:
                return None
            # If last segment -> must be command
            if idx == len(path) - 1:
                return g.commands.get(path[idx])  # command same name as group unlikely
            # If one segment left -> command name inside this group
            if idx == len(path) - 2:
                return g.commands.get(path[idx + 1])
            # Otherwise go deeper into subgroups
            return traverse(g.subgroups, idx + 1)

        return traverse(device.groups, 0)
