import os
import time
from typing import Dict, Any, List, Optional
import xml.etree.ElementTree as ET

from models import Command, Group, Device, Controller, Scriptlet, ScriptStep, SendStep, WaitStep


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
        for c_el in root.findall('controller'):
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
                    image_src=(d_el.find('image').get('src') if d_el.find('image') is not None else None),
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
                    # Positional attributes for fancy view
                    def to_int(val: Optional[str]) -> Optional[int]:
                        try:
                            return int(val) if val is not None and val != '' else None
                        except ValueError:
                            return None
                    x = to_int(cmd_el.get('x'))
                    y = to_int(cmd_el.get('y'))
                    width = to_int(cmd_el.get('width'))
                    height = to_int(cmd_el.get('height'))
                    device.commands[cmd_name] = Command(cmd_name, payload, disabled, x, y, width, height)
                # Groups (may be nested)
                for g_el in d_el.findall('group'):
                    grp = self._parse_group(g_el)
                    device.groups[grp.name] = grp
                controller.devices[device.name] = device

            # Scripts section
            scripts_el = c_el.find('scripts')
            controller.scripts = {}
            if scripts_el is not None:
                for sc_el in scripts_el.findall('scriptlet'):
                    s_name = sc_el.get('name')
                    if not s_name:
                        continue
                    s_friendly = sc_el.get('friendly-name') or sc_el.get('frendly-name')
                    script = Scriptlet(name=s_name, friendly_name=s_friendly, steps=[])
                    # Steps can be <send> or <wait>
                    for step_el in list(sc_el):
                        tag = step_el.tag.lower()
                        if tag == 'send':
                            dev = step_el.get('device') or ''
                            cmd = step_el.get('command') or ''
                            if not dev or not cmd:
                                continue
                            script.steps.append(SendStep(device=dev, command_path=cmd))
                        elif tag == 'wait':
                            t = step_el.get('time')
                            try:
                                t_ms = int(t) if t is not None else 0
                            except ValueError:
                                t_ms = 0
                            script.steps.append(WaitStep(time_ms=t_ms))
                    controller.scripts[s_name] = script

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
            # Positional attributes for fancy view
            def to_int(val: Optional[str]) -> Optional[int]:
                try:
                    return int(val) if val is not None and val != '' else None
                except ValueError:
                    return None
            x = to_int(cmd_el.get('x'))
            y = to_int(cmd_el.get('y'))
            width = to_int(cmd_el.get('width'))
            height = to_int(cmd_el.get('height'))
            group.commands[cmd_name] = Command(cmd_name, payload, disabled, x, y, width, height)
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

    # Scripts helpers
    def list_scripts(self, c_name: str) -> Optional[List[Dict[str, Any]]]:
        ctrl = self.get_controller(c_name)
        if not ctrl:
            return None
        out: List[Dict[str, Any]] = []
        for name, sc in sorted(ctrl.scripts.items()):
            steps: List[Dict[str, Any]] = []
            for st in sc.steps:
                if isinstance(st, WaitStep):
                    steps.append({'type': 'wait', 'time': st.time_ms})
                elif isinstance(st, SendStep):
                    steps.append({'type': 'send', 'device': st.device, 'command': st.command_path})
            out.append({'name': name, 'friendly_name': sc.friendly_name, 'steps': steps})
        return out

    def get_script(self, c_name: str, s_name: str) -> Optional[Scriptlet]:
        ctrl = self.get_controller(c_name)
        if not ctrl:
            return None
        return ctrl.scripts.get(s_name)

    def list_commands(self, device: Device) -> Dict[str, Any]:
        def cmd_to_dict(n: str, cmd: Command, parent_disabled: bool = False) -> Dict[str, Any]:
            effective_disabled = cmd.disabled or parent_disabled
            data: Dict[str, Any] = {
                'name': n,
                'disabled': effective_disabled,
            }
            # Include positional attributes if present
            if cmd.x is not None and cmd.y is not None and cmd.width is not None and cmd.height is not None:
                data.update({'x': cmd.x, 'y': cmd.y, 'width': cmd.width, 'height': cmd.height})
            return data

        def group_to_dict(g: Group, parent_disabled: bool = False) -> Dict[str, Any]:
            effective_group_disabled = parent_disabled or g.disabled
            return {
                'name': g.name,
                'disabled': effective_group_disabled,
                'commands': [
                    cmd_to_dict(n, cmd, effective_group_disabled)
                    for n, cmd in sorted(g.commands.items())
                ],
                'groups': [group_to_dict(sg, effective_group_disabled) for _, sg in sorted(g.subgroups.items())]
            }

        return {
            'device': device.name,
            'friendly_name': device.friendly_name,
            'image': device.image_src,
            'commands': [
                cmd_to_dict(n, cmd)
                for n, cmd in sorted(device.commands.items())
            ],
            'groups': [group_to_dict(g) for _, g in sorted(device.groups.items())]
        }

    def resolve_command(self, device: Device, path: List[str]) -> Optional[Command]:
        # Try direct command name first (no group)
        if len(path) == 1 and path[0] in device.commands:
            return device.commands[path[0]]

        def clone_with_disabled(cmd: Command, disabled: bool) -> Command:
            # Return a shallow copy with overridden disabled flag
            return Command(
                name=cmd.name,
                payload_hex=cmd.payload_hex,
                disabled=disabled,
                x=cmd.x,
                y=cmd.y,
                width=cmd.width,
                height=cmd.height,
            )

        # Traverse groups, carrying effective disabled state from parent groups
        def traverse(gmap: Dict[str, Group], idx: int, parent_disabled: bool = False) -> Optional[Command]:
            if idx >= len(path):
                return None
            g = gmap.get(path[idx])
            if not g:
                return None
            effective_disabled = parent_disabled or g.disabled
            # If last segment -> could be command with same name as this group (edge case)
            if idx == len(path) - 1:
                cmd = g.commands.get(path[idx])
                if cmd is None:
                    return None
                return clone_with_disabled(cmd, cmd.disabled or effective_disabled)
            # If one segment left -> command name inside this group
            if idx == len(path) - 2:
                cmd = g.commands.get(path[idx + 1])
                if cmd is None:
                    return None
                return clone_with_disabled(cmd, cmd.disabled or effective_disabled)
            # Otherwise go deeper into subgroups
            return traverse(g.subgroups, idx + 1, effective_disabled)

        return traverse(device.groups, 0)
