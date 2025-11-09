from dataclasses import dataclass, field
from typing import Dict, Optional


@dataclass
class Command:
    name: str
    payload_hex: str
    disabled: bool = False
    # Fancy view positional attributes (optional)
    x: Optional[int] = None
    y: Optional[int] = None
    width: Optional[int] = None
    height: Optional[int] = None


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
    image_src: Optional[str] = None
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
