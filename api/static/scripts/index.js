const $ = (sel) => document.querySelector(sel);
const controllerSel = $('#controller');
const deviceSel = $('#device');
const statusEl = $('#status');
const commandsEl = $('#commands');
const deviceTitle = $('#deviceTitle');
const toggleDisabledBtn = $('#toggleDisabled');
let showDisabled = false;
const API_PATH = '/api';
const API_PORT = 5000;

function setStatus(msg, cls = 'muted') {
  statusEl.className = 'status ' + cls;
  statusEl.textContent = msg || '';
}

function getAPIUrl(path) {
  let url = window.location.toString();
  if (url.endsWith('/')) url = url.slice(0, -1);
  url = url.replace(/:(\d+)(?=\/|$)/, '');  // Remove port number
  url += ':' + API_PORT.toString() + API_PATH;
  return url + path;
}

async function fetchJSON(path, options) {
  const res = await fetch(getAPIUrl(path), options);
  if (!res.ok) {
    let detail = '';
    try {
      const err = await res.json();
      detail = err.description || err.message || JSON.stringify(err);
    } catch (_) {
      detail = await res.text();
    }
    throw new Error(`${res.status} ${res.statusText}${detail ? ' — ' + detail : ''}`);
  }
  return res.json();
}

async function loadControllers() {
  setStatus('Loading controllers...', 'muted');
  controllerSel.innerHTML = '';
  deviceSel.innerHTML = '';
  commandsEl.innerHTML = '';
  deviceTitle.textContent = 'Select a controller and device.';
  try {
    const controllers = await fetchJSON('/controller');
    if (!controllers.length) {
      controllerSel.innerHTML = '<option value="">(none)</option>';
      setStatus('No controllers found in XML.', 'warn');
      return;
    }
    controllerSel.innerHTML = controllers.map(c => `<option value="${encodeURIComponent(c.name)}">${c.friendly_name || c.name} (${c.name})</option>`).join('');
    await loadDevices();
    setStatus('Controllers loaded.', 'ok');
  } catch (e) {
    setStatus(e.message, 'err');
  }
}

async function loadDevices() {
  const c = decodeURIComponent(controllerSel.value || '');
  deviceSel.innerHTML = '';
  commandsEl.innerHTML = '';
  deviceTitle.textContent = 'Select a controller and device.';
  if (!c) return;
  setStatus(`Loading devices for ${c}...`, 'muted');
  try {
    const devices = await fetchJSON(`/${encodeURIComponent(c)}/device`);
    if (!devices.length) {
      deviceSel.innerHTML = '<option value="">(none)</option>';
      setStatus('No devices found for controller.', 'warn');
      return;
    }
    deviceSel.innerHTML = devices.map(d => `<option value="${encodeURIComponent(d.name)}">${d.friendly_name || d.name} (${d.name})</option>`).join('');
    await loadCommands();
    setStatus('Devices loaded.', 'ok');
  } catch (e) {
    setStatus(e.message, 'err');
  }
}

function renderCommandsTree(data, cName, dName) {
  commandsEl.innerHTML = '';
  deviceTitle.textContent = `${data.friendly_name || data.device} — Commands`;

  const ul = document.createElement('ul');
  ul.className = 'tree';

  // Top-level commands
  for (const cmd of data.commands || []) {
    ul.appendChild(renderCommandItem(cName, dName, [], cmd));
  }

  // Groups
  for (const g of data.groups || []) {
    ul.appendChild(renderGroupItem(cName, dName, [], g));
  }

  commandsEl.appendChild(ul);
}

function renderCommandItem(cName, dName, groupPath, cmd) {
  const li = document.createElement('li');
  const wrap = document.createElement('div');
  wrap.className = 'cmd';

  const name = document.createElement('span');
  name.className = 'name';
  name.textContent = [...groupPath, cmd.name].join('.')

  const badge = document.createElement('span');
  badge.className = 'badge' + (cmd.disabled ? ' disabled' : '');
  badge.textContent = cmd.disabled ? 'disabled' : 'command';

  const space = document.createElement('div');
  space.className = 'space';

  const btn = document.createElement('button');
  btn.textContent = 'Send';
  btn.disabled = !!cmd.disabled;
  btn.onclick = () => sendCommand(cName, dName, [...groupPath, cmd.name].join('.'));

  wrap.appendChild(name);
  wrap.appendChild(badge);
  wrap.appendChild(space);
  wrap.appendChild(btn);
  li.appendChild(wrap);
  return li;
}

function renderGroupItem(cName, dName, parentPath, group) {
  const li = document.createElement('li');
  const details = document.createElement('details');
  details.open = true;
  const summary = document.createElement('summary');
  const title = document.createElement('span');
  title.textContent = group.name;

  const badge = document.createElement('span');
  badge.className = 'badge' + (group.disabled ? ' disabled' : '');
  badge.textContent = group.disabled ? 'group disabled' : 'group';

  summary.appendChild(title);
  summary.appendChild(document.createTextNode(' '));
  summary.appendChild(badge);
  details.appendChild(summary);

  const path = [...parentPath, group.name];

  const inner = document.createElement('ul');
  inner.className = 'tree';

  // Commands inside the group
  for (const cmd of group.commands || []) {
    inner.appendChild(renderCommandItem(cName, dName, path, cmd));
  }
  // Sub-groups
  for (const sg of group.groups || []) {
    inner.appendChild(renderGroupItem(cName, dName, path, sg));
  }

  details.appendChild(inner);
  li.appendChild(details);
  return li;
}

async function loadCommands() {
  const c = decodeURIComponent(controllerSel.value || '');
  const d = decodeURIComponent(deviceSel.value || '');
  commandsEl.innerHTML = '';
  if (!c || !d) return;
  setStatus(`Loading commands for ${c}/${d}...`, 'muted');
  try {
    const data = await fetchJSON(`/${encodeURIComponent(c)}/${encodeURIComponent(d)}`);
    renderCommandsTree(data, c, d);
      // Initially hide disabled items
      const items = commandsEl.querySelectorAll('.cmd, details');
      items.forEach(item => {
          const isDisabled = item.querySelector('.badge.disabled');
          if (isDisabled) {
              item.closest('li').style.display = showDisabled ? '' : 'none';
          }
      });
      setStatus('Commands loaded.', 'ok');
  } catch (e) {
    setStatus(e.message, 'err');
  }
}

async function sendCommand(cName, dName, cmdPath) {
  const url = `/${encodeURIComponent(cName)}/${encodeURIComponent(dName)}/${encodeURIComponent(cmdPath)}`;
  setStatus(`Sending: ${cName}/${dName}/${cmdPath} ...`, 'muted');
  try {
    const res = await fetchJSON(url, { method: 'POST' });
    setStatus(`OK: ${res.status || 'ok'} — ${res.controller}/${res.device}/${res.command}`, 'ok');
  } catch (e) {
    setStatus(e.message, 'err');
  }
}

controllerSel.addEventListener('change', loadDevices);
deviceSel.addEventListener('change', loadCommands);
$('#refresh').addEventListener('click', loadControllers);
toggleDisabledBtn.addEventListener('click', () => {
    showDisabled = !showDisabled;
    toggleDisabledBtn.textContent = showDisabled ? 'Hide disabled' : 'Show disabled';
    const items = commandsEl.querySelectorAll('.cmd, details');
    items.forEach(item => {
        const isDisabled = item.querySelector('.badge.disabled');
        if (isDisabled) {
            item.closest('li').style.display = showDisabled ? '' : 'none';
        }
    });
});

// Initial load
loadControllers();
