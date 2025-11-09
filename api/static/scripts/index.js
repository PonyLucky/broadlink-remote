const $ = (sel) => document.querySelector(sel);
const controllerSel = $('#controller');
const devicesTrigger = $('#devicesTrigger');
const devicesMenu = $('#devicesMenu');
const devicesList = $('#devicesList');
const devicesSelectAll = $('#devicesSelectAll');
const statusEl = $('#status');
const commandsEl = $('#commands');
const showDisabledChk = $('#showDisabled');
const API_PATH = '/api';
const API_PORT = 5000;

let allDevices = [];
let selectedDevices = new Set();

function updateDevicesTriggerLabel() {
  const total = allDevices.length;
  const selectedCount = selectedDevices.size;
  if (total === 0) {
    devicesTrigger.textContent = 'No devices';
    devicesTrigger.disabled = true;
    return;
  }
  devicesTrigger.disabled = false;
  if (selectedCount === 0) {
    devicesTrigger.textContent = 'No devices';
  } else if (selectedCount === total) {
    devicesTrigger.textContent = 'All devices';
  } else {
    // show first 2 friendly names and +N
    const map = new Map(allDevices.map(d => [d.name, (d.friendly_name || d.name) + ' (' + d.name + ')']));
    const names = Array.from(selectedDevices).slice(0, 2).map(n => map.get(n) || n);
    const more = selectedCount - names.length;
    devicesTrigger.textContent = names.join(', ') + (more > 0 ? ` +${more}` : '');
  }
}

function resetDevicesDropdown() {
  allDevices = [];
  selectedDevices = new Set();
  if (devicesList) devicesList.innerHTML = '';
  if (devicesSelectAll) {
    devicesSelectAll.checked = true;
    devicesSelectAll.indeterminate = false;
  }
  devicesTrigger && (devicesTrigger.textContent = 'All devices');
  if (devicesTrigger) devicesTrigger.disabled = true;
  closeDevicesMenu();
}

function setDevicesInDropdown(devices) {
  // devices: array with {name, friendly_name}
  allDevices = devices.map(d => ({ name: d.name, friendly_name: d.friendly_name || d.name }));
  selectedDevices = new Set(allDevices.map(d => d.name));
  // build list
  devicesList.innerHTML = '';
  for (const d of allDevices) {
    const label = document.createElement('label');
    label.className = 'option';
    const cb = document.createElement('input');
    cb.type = 'checkbox';
    cb.checked = true;
    cb.setAttribute('data-device', d.name);
    const span = document.createElement('span');
    span.textContent = `${d.friendly_name} (${d.name})`;
    label.appendChild(cb);
    label.appendChild(span);
    devicesList.appendChild(label);
  }
  // set select all states
  devicesSelectAll.checked = true;
  devicesSelectAll.indeterminate = false;
  updateDevicesTriggerLabel();
  bindDeviceItemHandlers();
  if (devicesTrigger) devicesTrigger.disabled = false;
}

function getSelectedDevices() {
  return Array.from(selectedDevices);
}

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
  resetDevicesDropdown();
  commandsEl.innerHTML = '';
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
  resetDevicesDropdown();
  commandsEl.innerHTML = '';
  if (!c) return;
  setStatus(`Loading devices for ${c}...`, 'muted');
  try {
    const devices = await fetchJSON(`/${encodeURIComponent(c)}/device`);
    if (!devices.length) {
      setStatus('No devices found for controller.', 'warn');
      return;
    }
    setDevicesInDropdown(devices);
    await loadSelectedDevicesCommands();
    setStatus('Devices loaded.', 'ok');
  } catch (e) {
    setStatus(e.message, 'err');
  }
}

function renderCommandsTreeInto(container, data, cName, dName) {
  // Title
  const title = document.createElement('h3');
  title.className = 'muted';
  title.textContent = `${data.friendly_name || data.device || dName}`;
  container.appendChild(title);

  const ul = document.createElement('ul');
  ul.className = 'tree';

  for (const cmd of data.commands || []) {
    ul.appendChild(renderCommandItem(cName, dName, [], cmd));
  }
  for (const g of data.groups || []) {
    ul.appendChild(renderGroupItem(cName, dName, [], g));
  }

  container.appendChild(ul);
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

  for (const cmd of group.commands || []) {
    inner.appendChild(renderCommandItem(cName, dName, path, cmd));
  }
  for (const sg of group.groups || []) {
    inner.appendChild(renderGroupItem(cName, dName, path, sg));
  }

  details.appendChild(inner);
  li.appendChild(details);
  return li;
}

async function loadSelectedDevicesCommands() {
  const c = decodeURIComponent(controllerSel.value || '');
  commandsEl.innerHTML = '';
  if (!c) return;
  const selected = getSelectedDevices();
  if (!selected.length) {
    setStatus('No device selected. Select one or more devices.', 'warn');
    return;
  }
  try {
    for (const d of selected) {
      setStatus(`Loading commands for ${c}/${d}...`, 'muted');
      const data = await fetchJSON(`/${encodeURIComponent(c)}/${encodeURIComponent(d)}`);
      const section = document.createElement('section');
      section.className = 'device-commands';
      renderCommandsTreeInto(section, data, c, d);
      commandsEl.appendChild(section);
    }
    applyDisabledFilter();
    setStatus('Commands loaded.', 'ok');
  } catch (e) {
    setStatus(e.message, 'err');
  }
}

function applyDisabledFilter() {
  const showDisabled = !!showDisabledChk.checked;
  const items = commandsEl.querySelectorAll('.cmd, details');
  items.forEach(item => {
    const isDisabled = item.querySelector('.badge.disabled');
    if (isDisabled) {
      const li = item.closest('li');
      if (li) li.style.display = showDisabled ? '' : 'none';
    }
  });
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
$('#refresh').addEventListener('click', loadControllers);
showDisabledChk.addEventListener('change', applyDisabledFilter);

// Dropdown interactions
function openDevicesMenu() {
  devicesMenu.hidden = false;
  devicesTrigger.setAttribute('aria-expanded', 'true');
}
function closeDevicesMenu() {
  devicesMenu.hidden = true;
  devicesTrigger.setAttribute('aria-expanded', 'false');
}
devicesTrigger?.addEventListener('click', (e) => {
  e.stopPropagation();
  const isOpen = devicesTrigger.getAttribute('aria-expanded') === 'true';
  if (isOpen) closeDevicesMenu(); else openDevicesMenu();
});
document.addEventListener('click', (e) => {
  if (!devicesMenu.hidden && !devicesMenu.contains(e.target) && e.target !== devicesTrigger) {
    closeDevicesMenu();
  }
});
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') closeDevicesMenu();
});

devicesSelectAll?.addEventListener('change', () => {
  const check = devicesSelectAll.checked;
  selectedDevices = new Set(check ? allDevices.map(d => d.name) : []);
  // reflect in UI
  devicesList.querySelectorAll('input[type="checkbox"][data-device]').forEach(cb => {
    cb.checked = check;
  });
  updateDevicesTriggerLabel();
  loadSelectedDevicesCommands();
});

// helper to bind item change
function bindDeviceItemHandlers() {
  devicesList.querySelectorAll('input[type="checkbox"][data-device]').forEach(cb => {
    cb.addEventListener('change', () => {
      const name = cb.getAttribute('data-device');
      if (cb.checked) selectedDevices.add(name); else selectedDevices.delete(name);
      // update select-all indeterminate state
      const total = allDevices.length;
      const selectedCount = selectedDevices.size;
      devicesSelectAll.indeterminate = selectedCount > 0 && selectedCount < total;
      devicesSelectAll.checked = selectedCount === total;
      updateDevicesTriggerLabel();
      loadSelectedDevicesCommands();
    });
  });
}

// Initial load
loadControllers();
