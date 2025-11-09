import os
from flask import Flask, send_from_directory

from xml_loader import Config
from api import create_api_blueprint


XML_PATH = os.path.join(os.path.dirname(__file__), 'broadlink.xml')


def create_app() -> Flask:
    app = Flask(__name__)
    cfg = Config(XML_PATH)
    api_bp = create_api_blueprint(cfg)
    app.register_blueprint(api_bp)
    return app
