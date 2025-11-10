# BroadlinkRemote API

A lightweight Flask-based API and single-page UI to control Broadlink IR/RF controllers using a simple XML configuration (`broadlink.xml`).

- Backend: Flask blueprint under the prefix `/api`
- Frontend: static assets served from `/static` and a home page at `/`
- Config: `broadlink.xml` defines controllers, devices, and commands
- Tools: interactive TUI to discover controllers, add devices, and learn IR/RF commands

## Quick start

Prerequisites:
- Python 3.9+
- Network access to your Broadlink controller(s)

Install dependencies (examples):
- If you have a requirements file, run `pip install -r requirements.txt`.
- Otherwise, minimally: `pip install flask python-dotenv broadlink`

Run the app:

```
cd api
python main.py
```

Environment variables (optional):
- `FLASK_HOST` (default: `0.0.0.0`)
- `FLASK_PORT` (default: `5000`)
- `FLASK_DEBUG` (`true`/`false`, default: `false`)
- `FLASK_ENV` (set to `development` to always reload XML on each request)

Once running:
- UI: http://localhost:5000/
- Static files: http://localhost:5000/static/
- OpenAPI: http://localhost:5000/api/doc/openapi.json

## Docker

You can run the API and static UI in a container using the provided `Dockerfile`.

Build the image:

```
# From this directory
docker build -t broadlinkremote-api .
```

Run the container (exposing port 5000 and mounting your XML config read-only):

```
docker run --rm -p 5000:5000 \
  -e FLASK_DEBUG=false \
  -e FLASK_ENV=production \
  -v "$PWD/broadlink.xml":/app/broadlink.xml:ro \
  --name broadlink-api broadlinkremote-api
```

Notes:
- Networking: Default bridge networking works for most setups. If your Broadlink device is only reachable on the host network (especially on Linux), you can use host networking:
  - Linux: `docker run --rm --network host broadlinkremote-api`
- Configuration: The app reads `broadlink.xml` from `/app/broadlink.xml`. Mount your local file into the container as shown above.
- Ports: The container listens on `0.0.0.0:5000` by default. Change the published port if needed, e.g. `-p 8080:5000`.
- Environment variables:
  - `FLASK_HOST` (default: `0.0.0.0`)
  - `FLASK_PORT` (default: `5000`)
  - `FLASK_DEBUG` (`true`/`false`, default: `false`)
  - `FLASK_ENV` (set to `development` to always reload XML on each request)
- Deterministic installs: If you add a `requirements.txt` later, consider updating the `Dockerfile` to copy it first and run `pip install -r requirements.txt` for better layer caching.

## API overview

The API is mounted at `/api`. Key endpoints:

- `GET /api/doc/openapi.json` — Generated OpenAPI document
- `GET /api/controller` — List known controllers from `broadlink.xml`
- `GET /api/<controller>/device` — List devices under a controller
- `GET /api/<controller>/<device>` — List available commands for a device
- `POST /api/<controller>/<device>/<command.path>` — Send a command (dot-separated nested names allowed)
- `GET /api/<controller>/scripts` — List scripts for a controller
- `GET /api/<controller>/scripts/<scriptlet>` — Show content (steps) of a scriptlet
- `POST /api/<controller>/scripts/<scriptlet>` — Run a scriptlet (sequentially executes its steps)

Example usages:

```
# List controllers
curl http://localhost:5000/api/controller

# List devices under controller named "bedroom"
curl http://localhost:5000/api/bedroom/device

# List commands for device "tv"
curl http://localhost:5000/api/bedroom/tv

# Send a command to turn the TV on (example: command path "power.on")
curl -X POST http://localhost:5000/api/bedroom/tv/power.on
```

Responses are JSON. Error responses use proper HTTP status codes with a short description.

## Configuration: broadlink.xml

The XML file at `api/broadlink.xml` holds:
- Controllers (`<controller>`): IP, port, dev type, MAC, model, friendly name
- Devices (`<device>`): type (e.g., `ir` or `rf`), manufacturer, model, friendly name
- Commands: hex payloads to be sent by the controller
- Scripts: reusable sequences of steps under each controller

You can manage this file manually or via the TUI tool (see below).

### Scripts and scriptlets

Under a controller you can now define a `<scripts>` section with one or more `<scriptlet>` entries. Each scriptlet contains steps executed sequentially. Supported steps:
- `<send device="<device_name>" command="<command_or_path>"/>` — send a command to a device. `command` can be a simple command name or a dot-separated path into groups (same as the `POST /<controller>/<device>/<command.path>` endpoint).
- `<wait time="<milliseconds>"/>` — pause execution for the specified amount of time (required `time` attribute, integer milliseconds).

Example:

```
<controller ...>
  ...
  <scripts>
    <scriptlet name="music" friendly-name="Musique">
      <send device="amp" command="power"/>
      <wait time="500"/>
      <send device="network_player" command="power"/>
    </scriptlet>
  </scripts>
</controller>
```

Run this scriptlet with:

```
curl -X POST http://localhost:5000/api/bedroom/scripts/music
```

Inspect scripts:

```
# List scripts for a controller
curl http://localhost:5000/api/bedroom/scripts

# Show a scriptlet's content
curl http://localhost:5000/api/bedroom/scripts/music
```

## Tools

- `tools/tui.py` — Text UI to:
  - Discover controllers on the network and add them
  - Add controllers manually
  - Add devices to a controller
  - Learn IR or RF commands and store them under a device
  - List stored controllers and devices

- `tools/pos_picker.html` — Simple browser utility to inspect pixel coordinates on an image/page; useful when mapping UI click areas.

See `tools/README.md` for detailed usage.

## Static frontend

- `static/index.html`, `static/styles/`, `static/scripts/`
- Served at `/` (index) and `/static/*` (assets)
- Frontend calls the API endpoints above to populate UI and trigger commands

See `static/README.md` for details.

## Development notes

- App factory: `app_factory.create_app()` wires up the blueprint and static serving.
- The backend automatically reloads `broadlink.xml` when it changes (and always in development mode) before serving relevant requests.
- The Broadlink Python library is optional at import time; sending commands requires it at runtime.
