import os
import time
from typing import Dict, Any, List, Optional
from dataclasses import dataclass, field
from flask import Flask, jsonify, abort
import xml.etree.ElementTree as ET

# Optional env loader
try:
    from dotenv import load_dotenv  # type: ignore
    load_dotenv()
except Exception:
    pass

# Optional broadlink import; we'll fail gracefully if missing
try:
    import broadlink  # type: ignore
except Exception:  # pragma: no cover
    broadlink = None  # type: ignore


XML_PATH = os.path.join(os.path.dirname(__file__), 'broadlink.xml')


@dataclass
class Command:
    name: str
    payload_hex: str
    disabled: bool = False


@dataclass
class Group:
    name: str
    disabled: bool = False
    commands: Dict[str, Command] = field(default_factory=dict)
    subgroups: Dict[str, 'Group'] = field(default_factory=dict)


@dataclass
class Device:
    name: str
    friendly_name: Optional[str]
    type: str
    manufacturer: Optional[str]
    model: Optional[str]
    commands: Dict[str, Command] = field(default_factory=dict)
    groups: Dict[str, Group] = field(default_factory=dict)


@dataclass
class Controller:
    name: str
    friendly_name: Optional[str]
    ip: str
    port: int
    devtype_hex: str
    mac: str
    model: Optional[str]
    devices: Dict[str, Device] = field(default_factory=dict)


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
                return g.commands.get(path[idx])  # command having same name as group unlikely
            # If one segment left -> command name inside this group
            if idx == len(path) - 2:
                return g.commands.get(path[idx + 1])
            # Otherwise go deeper into subgroups
            return traverse(g.subgroups, idx + 1)

        return traverse(device.groups, 0)


def create_app() -> Flask:
    app = Flask(__name__)
    cfg = Config(XML_PATH)

    @app.get('/doc/openapi.json')
    def openapi_doc():
        doc = build_openapi()
        return jsonify(doc)

    @app.get('/controller')
    def list_controllers():
        cfg.reload_if_changed()
        return jsonify([
            {
                'name': c.name,
                'friendly_name': c.friendly_name,
                'ip': c.ip,
                'port': c.port,
                'type': c.devtype_hex,
                'mac': c.mac,
                'model': c.model,
                'devices': sorted(c.devices.keys())
            }
            for c in sorted(cfg.controllers.values(), key=lambda x: x.name)
        ])

    @app.get('/<c_name>/device')
    def list_devices(c_name: str):
        ctrl = cfg.get_controller(c_name)
        if not ctrl:
            abort(404, description=f'Controller {c_name} not found')
        return jsonify([
            {
                'name': d.name,
                'friendly_name': d.friendly_name,
                'type': d.type,
                'manufacturer': d.manufacturer,
                'model': d.model
            }
            for d in sorted(ctrl.devices.values(), key=lambda x: x.name)
        ])

    @app.get('/<c_name>/<d_name>')
    def list_device_commands(c_name: str, d_name: str):
        device = cfg.get_device(c_name, d_name)
        if not device:
            abort(404, description=f'Device {d_name} on controller {c_name} not found')
        return jsonify(cfg.list_commands(device))

    @app.post('/<c_name>/<d_name>/<path:cmd_name>')
    def send_command(c_name: str, d_name: str, cmd_name: str):
        device = cfg.get_device(c_name, d_name)
        if not device:
            abort(404, description=f'Device {d_name} on controller {c_name} not found')
        parts = cmd_name.split('.') if cmd_name else []
        if not parts:
            abort(400, description='cmd_name required')
        cmd = cfg.resolve_command(device, parts)
        if not cmd:
            abort(404, description=f'Command {cmd_name} not found')
        if cmd.disabled:
            abort(403, description=f'Command {cmd_name} is disabled')

        ctrl = cfg.get_controller(c_name)
        if not ctrl:
            abort(404, description=f'Controller {c_name} not found')
        if broadlink is None:
            abort(500, description='broadlink library not available')

        # Prepare and send
        payload_hex = cmd.payload_hex.replace('\n', '').replace('\r', '').strip()
        if not payload_hex:
            abort(400, description='Empty command payload')
        try:
            payload = bytes.fromhex(payload_hex)
        except ValueError:
            abort(400, description='Invalid hex payload')

        devtype = int(ctrl.devtype_hex, 16) if isinstance(ctrl.devtype_hex, str) else int(ctrl.devtype_hex)
        mac_bytes = mac_str_to_bytes(ctrl.mac)
        ok, err = False, None
        try:
            ok = send_via_broadlink(devtype, ctrl.ip, ctrl.port, mac_bytes, payload)
        except Exception as e:  # pragma: no cover
            ok, err = False, str(e)
        if not ok:
            # try reversed mac as fallback
            try:
                ok = send_via_broadlink(devtype, ctrl.ip, ctrl.port, mac_bytes[::-1], payload)
            except Exception as e2:  # pragma: no cover
                ok, err = False, str(e2)
        if not ok:
            abort(502, description=f'Failed to send command: {err or "unknown error"}')

        return jsonify({'status': 'ok', 'controller': c_name, 'device': d_name, 'command': cmd_name})

    return app


def mac_str_to_bytes(mac: str) -> bytes:
    # Accept forms like 'AA:BB:CC:DD:EE:FF' or 'AABBCCDDEEFF'
    m = mac.replace(':', '').replace('-', '').strip()
    if len(m) != 12:
        return b"\x00\x00\x00\x00\x00\x00"
    return bytes.fromhex(m)


def send_via_broadlink(devtype: int, ip: str, port: int, mac: bytes, payload: bytes) -> bool:
    dev = broadlink.gendevice(devtype, (ip, port), mac)
    if not dev.auth():
        return False
    # RM devices typically use send_data
    if hasattr(dev, 'send_data'):
        dev.send_data(payload)
        return True
    # Fallback: try set_power for plugs, not applicable here
    return False


def build_openapi() -> Dict[str, Any]:
    return {
        'openapi': '3.0.3',
        'info': {
            'title': 'Broadlink XML Controller API',
            'version': '1.0.0'
        },
        'paths': {
            '/doc/openapi.json': {
                'get': {'summary': 'Get OpenAPI document', 'responses': {'200': {'description': 'OK'}}}
            },
            '/controller': {
                'get': {'summary': 'List controllers', 'responses': {'200': {'description': 'OK'}}}
            },
            '/{c_name}/device': {
                'get': {
                    'summary': 'List devices of a controller',
                    'parameters': [{'name': 'c_name', 'in': 'path', 'required': True, 'schema': {'type': 'string'}}],
                    'responses': {'200': {'description': 'OK'}, '404': {'description': 'Not found'}}
                }
            },
            '/{c_name}/{d_name}': {
                'get': {
                    'summary': 'List commands and groups for a device',
                    'parameters': [
                        {'name': 'c_name', 'in': 'path', 'required': True, 'schema': {'type': 'string'}},
                        {'name': 'd_name', 'in': 'path', 'required': True, 'schema': {'type': 'string'}}
                    ],
                    'responses': {'200': {'description': 'OK'}, '404': {'description': 'Not found'}}
                }
            },
            '/{c_name}/{d_name}/{cmd_name}': {
                'post': {
                    'summary': 'Send a command to a device',
                    'parameters': [
                        {'name': 'c_name', 'in': 'path', 'required': True, 'schema': {'type': 'string'}},
                        {'name': 'd_name', 'in': 'path', 'required': True, 'schema': {'type': 'string'}},
                        {'name': 'cmd_name', 'in': 'path', 'required': True, 'schema': {'type': 'string'}},
                    ],
                    'responses': {
                        '200': {'description': 'OK'},
                        '400': {'description': 'Bad request'},
                        '403': {'description': 'Forbidden'},
                        '404': {'description': 'Not found'},
                        '502': {'description': 'Failed to send'}
                    }
                }
            }
        }
    }


def main():
    app = create_app()
    host = os.getenv('FLASK_HOST', '0.0.0.0')
    port = int(os.getenv('FLASK_PORT', '5000'))
    debug = os.getenv('FLASK_DEBUG', 'false').lower() == 'true'
    app.run(host=host, port=port, debug=debug)


if __name__ == '__main__':
    main()