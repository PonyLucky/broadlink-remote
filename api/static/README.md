# Static Frontend

This directory contains the single-page UI served by Flask:

- `index.html` — entry page served at `/`
- `scripts/index.js` — UI logic and API calls
- `styles/index.css` — styling for the app
- `images/` — image assets used by the UI

## How it works

- The Flask app serves `index.html` at `/` and static assets under `/static/*`.
- The UI calls the backend API mounted at `/api` to fetch controllers, devices, and commands, and to send actions.
- Local state (selected view, filters) is persisted in `localStorage` to keep preferences between sessions.

### Main UI features

- Scripts panel: lists scriptlets for the selected controller with actions to View steps or Run the scriptlet.
- View toggle: "list" and "fancy" views for commands
- Filters modal:
  - Select controller
  - Multi-select devices (with Select All and partial states)
  - Toggle visibility of disabled commands
- Commands panel lists available commands for the selected controller/devices. Clicking a command sends it via the API.

#### Scripts panel usage

- Select a controller via Filters.
- The Scripts panel shows all scriptlets defined under that controller in `broadlink.xml`.
- Click "View" to see the steps (send/wait) in a modal; click "Run" to execute synchronously.

### Key API endpoints used

- `GET /api/controller` — list controllers
- `GET /api/<controller>/device` — list devices for a controller
- `GET /api/<controller>/<device>` — list device commands
- `POST /api/<controller>/<device>/<command.path>` — send command
- `GET /api/<controller>/scripts` — list scripts
- `GET /api/<controller>/scripts/<scriptlet>` — show scriptlet content
- `POST /api/<controller>/scripts/<scriptlet>` — run a scriptlet

## Development tips

- Launch the Flask app (from `api/`): `python main.py`, then open `http://localhost:5000/`.
- You can open DevTools console to see logs or network requests made by `scripts/index.js`.
- CSS variables at the top of `styles/index.css` control colors, spacing, and corners.
- If you change static files, simply refresh the browser. In some environments, you might need a hard refresh to bust the cache.

## Accessibility and UX

- The UI attempts to use semantic roles for modal dialogs and controls.
- Keyboard and focus behavior can be improved further; contributions are welcome.

## Folder structure

```
static/
  index.html
  scripts/
    index.js
  styles/
    index.css
  images/
    bedroom/
      amp.png
      fan.png
      network_player.png
      velux.png
```
