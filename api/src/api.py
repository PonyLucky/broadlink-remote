from flask import Blueprint, jsonify, abort
from typing import List
import time

from src.openapi_spec import build_openapi
from src.xml_loader import Config
from src.bl_client import mac_str_to_bytes, send_via_broadlink, is_broadlink_available
from src.models import SendStep, WaitStep


def create_api_blueprint(cfg: Config, prefix: str) -> Blueprint:
    api = Blueprint('api', __name__, url_prefix=prefix)

    @api.get('/doc/openapi.json')
    def openapi_doc():
        doc = build_openapi()
        return jsonify(doc)

    @api.get('/controller')
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

    @api.get('/<c_name>/device')
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

    @api.get('/<c_name>/<d_name>')
    def list_device_commands(c_name: str, d_name: str):
        device = cfg.get_device(c_name, d_name)
        if not device:
            abort(404, description=f'Device {d_name} on controller {c_name} not found')
        return jsonify(cfg.list_commands(device))

    def _send_device_command(ctrl_name: str, device_name: str, command_path: str) -> None:
        device = cfg.get_device(ctrl_name, device_name)
        if not device:
            abort(404, description=f'Device {device_name} on controller {ctrl_name} not found')
        parts: List[str] = command_path.split('.') if command_path else []
        if not parts:
            abort(400, description='cmd_name required')
        cmd = cfg.resolve_command(device, parts)
        if not cmd:
            abort(404, description=f'Command {command_path} not found')
        if cmd.disabled:
            abort(403, description=f'Command {command_path} is disabled')

        ctrl = cfg.get_controller(ctrl_name)
        if not ctrl:
            abort(404, description=f'Controller {ctrl_name} not found')
        if not is_broadlink_available():
            abort(500, description='broadlink library not available')

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
            try:
                ok = send_via_broadlink(devtype, ctrl.ip, ctrl.port, mac_bytes[::-1], payload)
            except Exception as e2:  # pragma: no cover
                ok, err = False, str(e2)
        if not ok:
            abort(502, description=f'Failed to send command: {err or "unknown error"}')

    @api.post('/<c_name>/<d_name>/<path:cmd_name>')
    def send_command(c_name: str, d_name: str, cmd_name: str):
        _send_device_command(c_name, d_name, cmd_name)
        return jsonify({'status': 'ok', 'controller': c_name, 'device': d_name, 'command': cmd_name})

    # Scripts routes
    @api.get('/<c_name>/scripts')
    def list_scripts(c_name: str):
        items = cfg.list_scripts(c_name)
        if items is None:
            abort(404, description=f'Controller {c_name} not found')
        return jsonify(items)

    @api.get('/<c_name>/scripts/<s_name>')
    def show_script(c_name: str, s_name: str):
        sc = cfg.get_script(c_name, s_name)
        if not sc:
            abort(404, description=f'Scriptlet {s_name} on controller {c_name} not found')
        # Serialize steps
        steps = []
        for st in sc.steps:
            if isinstance(st, WaitStep):
                steps.append({'type': 'wait', 'time': st.time_ms})
            elif isinstance(st, SendStep):
                steps.append({'type': 'send', 'device': st.device, 'command': st.command_path})
        return jsonify({'name': sc.name, 'friendly_name': sc.friendly_name, 'steps': steps})

    @api.post('/<c_name>/scripts/<s_name>')
    def run_script(c_name: str, s_name: str):
        sc = cfg.get_script(c_name, s_name)
        if not sc:
            abort(404, description=f'Scriptlet {s_name} on controller {c_name} not found')
        for st in sc.steps:
            if isinstance(st, WaitStep):
                delay = max(0, st.time_ms) / 1000.0
                if delay > 0:
                    time.sleep(delay)
            elif isinstance(st, SendStep):
                _send_device_command(c_name, st.device, st.command_path)
        return jsonify({'status': 'ok', 'controller': c_name, 'scriptlet': s_name})

    return api
