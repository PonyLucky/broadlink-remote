from typing import Optional

# Optional broadlink import; we'll fail gracefully if missing
try:
    import broadlink  # type: ignore
except Exception:  # pragma: no cover
    broadlink = None  # type: ignore


def is_broadlink_available() -> bool:
    return broadlink is not None


def mac_str_to_bytes(mac: str) -> bytes:
    """Accept forms like 'AA:BB:CC:DD:EE:FF' or 'AABBCCDDEEFF'."""
    m = mac.replace(':', '').replace('-', '').strip()
    if len(m) != 12:
        return b"\x00\x00\x00\x00\x00\x00"
    return bytes.fromhex(m)


def send_via_broadlink(devtype: int, ip: str, port: int, mac: bytes, payload: bytes) -> bool:
    if broadlink is None:  # pragma: no cover
        raise RuntimeError('broadlink library not available')

    dev = broadlink.gendevice(devtype, (ip, port), mac)
    # Create device and ensure authentication does not hang forever by setting a timeout.
    # The broadlink library respects the device's `timeout` property (in seconds) for socket ops.
    try:
        dev.timeout = 0.5  # seconds
        if not dev.auth():
            return False
    except Exception:
        return False

    # RM devices typically use send_data
    if hasattr(dev, 'send_data'):
        dev.send_data(payload)
        return True
    # Fallback: try set_power for plugs, not applicable here
    return False
