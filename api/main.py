import os

# Optional env loader
try:
    from dotenv import load_dotenv  # type: ignore
    load_dotenv()
except Exception:
    pass

from src.app_factory import create_app


def main():
    app = create_app()
    host = os.getenv('FLASK_HOST', '0.0.0.0')
    port = int(os.getenv('FLASK_PORT', '6676'))
    debug = os.getenv('FLASK_DEBUG', 'false').lower() == 'true'
    app.run(host=host, port=port, debug=debug)


if __name__ == '__main__':
    main()