import os
from flask import Flask, send_from_directory

from src.xml_loader import Config
from src.api import create_api_blueprint


XML_PATH = os.path.join(os.path.dirname(__file__), '..', 'broadlink.xml')
if not os.path.exists(XML_PATH):
    raise FileNotFoundError("Could not find broadlink.xml")


def create_app() -> Flask:
    app = Flask(__name__, static_folder='../static', static_url_path='/static')

    cfg = Config(XML_PATH)
    api_bp = create_api_blueprint(cfg, '/api')
    cfg.reload_if_changed(
        force=os.getenv('FLASK_ENV') == 'development'
    )
    app.register_blueprint(api_bp)

    @app.get('/')
    def index():
        return send_from_directory(app.static_folder, 'index.html')

    @app.get('/static/<path:path>')
    def static_file(path):
        return send_from_directory(app.static_folder, path)

    return app
