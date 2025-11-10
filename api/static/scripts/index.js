const $ = (sel) => document.querySelector(sel);
const controllerSel = $('#controller');
const devicesTrigger = $('#devicesTrigger');
const devicesMenu = $('#devicesMenu');
const devicesList = $('#devicesList');
const devicesSelectAll = $('#devicesSelectAll');
const commandsEl = $('#commands');
const showDisabledChk = $('#showDisabled');

// Scripts elements
const scriptsPanel = $('#scriptsPanel');
const scriptsEl = $('#scripts');

// View toggle elements
const viewFancyBtn = $('#viewFancyBtn');
const viewListBtn = $('#viewListBtn');
const commandsPanel = $('#commandsPanel');
const commandsView = $('#commandsView');
const viewContainer = $('#viewContainer');

// Modal elements
const filtersBtn = $('#filtersBtn');
const filtersOverlay = $('#filtersOverlay');
const filtersModal = $('#filtersModal');
const filtersClose = $('#filtersClose');
const filtersApply = $('#filtersApply');

// Script details modal
const scriptOverlay = $('#scriptOverlay');
const scriptModal = $('#scriptModal');
const scriptClose = $('#scriptClose');
const scriptBody = $('#scriptBody');
const scriptRunBtn = $('#scriptRunBtn');

const API_PATH = '/api';
const API_PORT = 5000;

// LocalStorage keys and helpers
const LS_KEYS = {
  view: 'blr.viewMode',
  showDisabled: 'blr.showDisabled',
  controller: 'blr.controller',
  devicesByController: 'blr.devicesByController',
};
function lsGet(key, def = null) {
  try {
    const v = localStorage.getItem(key);
    return v === null ? def : v;
  } catch (_) {
    return def;
  }
}
function lsSet(key, val) {
  try {
    localStorage.setItem(key, val);
  } catch (_) {}
}
function lsGetJSON(key, def = {}) {
  const raw = lsGet(key, null);
  if (raw == null) return def;
  try { return JSON.parse(raw); } catch (_) { return def; }
}
function lsSetJSON(key, obj) {
  try { localStorage.setItem(key, JSON.stringify(obj)); } catch (_) {}
}

let allDevices = [];
let selectedDevices = new Set();

function currentControllerName() {
  return decodeURIComponent(controllerSel?.value || '');
}

function persistDevicesSelectionForCurrentController() {
  const c = currentControllerName();
  if (!c) return;
  const map = lsGetJSON(LS_KEYS.devicesByController, {});
  map[c] = Array.from(selectedDevices);
  lsSetJSON(LS_KEYS.devicesByController, map);
}

function restoreDevicesSelectionForCurrentController() {
  const c = currentControllerName();
  if (!c) return;
  const map = lsGetJSON(LS_KEYS.devicesByController, {});
  const saved = Array.isArray(map[c]) ? map[c] : null;
  const names = new Set(allDevices.map(d => d.name));
  if (saved && saved.length) {
    const intersect = saved.filter(n => names.has(n));
    if (intersect.length) {
      selectedDevices = new Set(intersect);
      // reflect in UI checkboxes
      devicesList.querySelectorAll('input[type="checkbox"][data-device]').forEach(cb => {
        const name = cb.getAttribute('data-device');
        cb.checked = selectedDevices.has(name);
      });
      // update select-all states
      const total = allDevices.length;
      const selectedCount = selectedDevices.size;
      devicesSelectAll.indeterminate = selectedCount > 0 && selectedCount < total;
      devicesSelectAll.checked = selectedCount === total;
      updateDevicesTriggerLabel();
    }
  }
}

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
  // set select all states (default all)
  devicesSelectAll.checked = true;
  devicesSelectAll.indeterminate = false;
  // Try restore saved selection for this controller
  restoreDevicesSelectionForCurrentController();
  updateDevicesTriggerLabel();
  bindDeviceItemHandlers();
  if (devicesTrigger) devicesTrigger.disabled = false;
}

function getSelectedDevices() {
  return Array.from(selectedDevices);
}

function setStatus(msg, cls = 'muted') {
  switch (cls) {
    case 'muted':
      console.log(msg);
      break;
    case 'ok':
      console.log(`OK: ${msg}`);
      break;
    case 'warn':
      console.warn(`WARN: ${msg}`);
      break;
    case 'err':
      console.error(`ERROR: ${msg}`);
      break;
    default:
      console.log(msg);
      break;
  }
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
    let detail;
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
    // Restore previously selected controller if available
    const savedCtrl = lsGet(LS_KEYS.controller, null);
    if (savedCtrl) {
      const enc = encodeURIComponent(savedCtrl);
      const opt = Array.from(controllerSel.options).find(o => o.value === enc);
      if (opt) controllerSel.value = enc;
    }
    // Persist current selection
    lsSet(LS_KEYS.controller, decodeURIComponent(controllerSel.value || ''));
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
  if (!c) {
    if (scriptsEl) scriptsEl.innerHTML = '<p class="muted">Select a controller to see scripts.</p>';
    return;
  }
  setStatus(`Loading devices for ${c}...`, 'muted');
  try {
    const devices = await fetchJSON(`/${encodeURIComponent(c)}/device`);
    if (!devices.length) {
      setStatus('No devices found for controller.', 'warn');
    } else {
      setDevicesInDropdown(devices);
      await loadSelectedDevicesCommands();
      setStatus('Devices loaded.', 'ok');
    }
  } catch (e) {
    setStatus(e.message, 'err');
  }
  // Always (re)load scripts after devices attempt
  await loadScripts();
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

controllerSel.addEventListener('change', () => {
  // persist controller selection
  lsSet(LS_KEYS.controller, decodeURIComponent(controllerSel.value || ''));
  loadDevices();
  if (isFancyActive()) updateFancyView();
});
showDisabledChk.addEventListener('change', () => {
  lsSet(LS_KEYS.showDisabled, showDisabledChk.checked ? '1' : '0');
  applyDisabledFilter();
  if (isFancyActive()) updateFancyView();
});

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
  persistDevicesSelectionForCurrentController();
  loadSelectedDevicesCommands();
  afterDevicesSelectionChanged();
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
      persistDevicesSelectionForCurrentController();
      loadSelectedDevicesCommands();
      afterDevicesSelectionChanged();
    });
  });
}

// Modal open/close helpers
let lastFocusedEl = null;
function isModalOpen() {
  return !filtersModal?.hasAttribute('hidden');
}
function openFiltersModal() {
  if (!filtersModal || !filtersOverlay) return;
  lastFocusedEl = document.activeElement;
  filtersOverlay.hidden = false;
  filtersModal.hidden = false;
  document.body.classList.add('modal-open');
  // focus first focusable element inside modal
  const focusable = filtersModal.querySelectorAll('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])');
  if (focusable.length) {
    /** @type {HTMLElement} */(focusable[0]).focus();
  } else {
    filtersClose?.focus();
  }
}
function closeFiltersModal() {
  if (!filtersModal || !filtersOverlay) return;
  filtersOverlay.hidden = true;
  filtersModal.hidden = true;
  document.body.classList.remove('modal-open');
  if (lastFocusedEl && typeof lastFocusedEl.focus === 'function') {
    lastFocusedEl.focus();
  } else {
    filtersBtn?.focus();
  }
}

// Bind modal events
filtersBtn?.addEventListener('click', openFiltersModal);
filtersClose?.addEventListener('click', closeFiltersModal);
filtersApply?.addEventListener('click', closeFiltersModal);
filtersOverlay?.addEventListener('click', closeFiltersModal);

// Script details modal helpers
let scriptModalLastFocus = null;
function isScriptModalOpen() {
  return !!(scriptModal && !scriptModal.hasAttribute('hidden'));
}
function openScriptModal() {
  if (!scriptModal || !scriptOverlay) return;
  scriptModalLastFocus = document.activeElement;
  scriptOverlay.hidden = false;
  scriptModal.hidden = false;
  document.body.classList.add('modal-open');
  const focusable = scriptModal.querySelectorAll('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])');
  if (focusable.length) /** @type {HTMLElement} */(focusable[0]).focus();
}
function closeScriptModal() {
  if (!scriptModal || !scriptOverlay) return;
  scriptOverlay.hidden = true;
  scriptModal.hidden = true;
  document.body.classList.remove('modal-open');
  if (scriptModalLastFocus && typeof scriptModalLastFocus.focus === 'function') scriptModalLastFocus.focus();
}
scriptClose?.addEventListener('click', closeScriptModal);
scriptOverlay?.addEventListener('click', closeScriptModal);

// Close on Escape and trap focus inside modal when open
document.addEventListener('keydown', (e) => {
  // Filters modal
  if (isModalOpen()) {
    if (e.key === 'Escape') {
      e.preventDefault();
      closeFiltersModal();
      return;
    }
    if (e.key === 'Tab') {
      const focusable = Array.from(filtersModal.querySelectorAll('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'));
      if (!focusable.length) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      const current = document.activeElement;
      if (e.shiftKey) {
        if (current === first || !filtersModal.contains(current)) {
          e.preventDefault();
          /** @type {HTMLElement} */(last).focus();
        }
      } else {
        if (current === last) {
          e.preventDefault();
          /** @type {HTMLElement} */(first).focus();
        }
      }
    }
  }
  // Script details modal
  if (isScriptModalOpen()) {
    if (e.key === 'Escape') {
      e.preventDefault();
      closeScriptModal();
      return;
    }
    if (e.key === 'Tab') {
      const focusable = Array.from(scriptModal.querySelectorAll('button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'));
      if (!focusable.length) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      const current = document.activeElement;
      if (e.shiftKey) {
        if (current === first || !scriptModal.contains(current)) {
          e.preventDefault();
          /** @type {HTMLElement} */(last).focus();
        }
      } else {
        if (current === last) {
          e.preventDefault();
          /** @type {HTMLElement} */(first).focus();
        }
      }
    }
  }
});

// Scripts API and rendering
async function loadScripts() {
  const c = decodeURIComponent(controllerSel?.value || '');
  if (!scriptsEl) return;
  scriptsEl.innerHTML = '';
  if (!c) {
    scriptsEl.innerHTML = '<p class="muted">Select a controller to see scripts.</p>';
    return;
  }
  try {
    const items = await fetchJSON(`/${encodeURIComponent(c)}/scripts`);
    renderScriptsList(c, items);
  } catch (e) {
    scriptsEl.innerHTML = `<p class="err">${e.message}</p>`;
  }
}

function renderScriptsList(controller, items) {
  if (!items || !items.length) {
    scriptsEl.innerHTML = '<p class="muted">No scripts defined for this controller.</p>';
    return;
  }
  const list = document.createElement('div');
  list.className = 'scripts-grid';
  items.forEach(sc => {
    const card = document.createElement('div');
    card.className = 'script-card';
    const title = document.createElement('div');
    title.className = 'script-title';
    title.textContent = `${sc.friendly_name || sc.name} (${sc.name})`;
    const actions = document.createElement('div');
    actions.className = 'script-actions';
    const btnRun = document.createElement('button');
    btnRun.className = 'primary';
    btnRun.textContent = 'Run';
    btnRun.addEventListener('click', () => runScriptlet(controller, sc.name, btnRun));
    const btnView = document.createElement('button');
    btnView.textContent = 'View';
    btnView.addEventListener('click', () => viewScript(controller, sc.name));
    actions.appendChild(btnView);
    actions.appendChild(btnRun);
    card.appendChild(title);
    card.appendChild(actions);
    list.appendChild(card);
  });
  scriptsEl.appendChild(list);
}

async function viewScript(controller, name) {
  try {
    const sc = await fetchJSON(`/${encodeURIComponent(controller)}/scripts/${encodeURIComponent(name)}`);
    // Build steps list
    scriptBody.innerHTML = '';
    const h = document.getElementById('scriptTitle');
    if (h) h.textContent = `${sc.friendly_name || sc.name} (${sc.name})`;
    const div = document.createElement('div');
    (sc.steps || []).forEach((st, idx) => {
      const wrap = document.createElement('div');
      wrap.className = 'cmd';
      const name = document.createElement('span');
      name.className = 'name';
      if (st.type === 'wait') {
        name.textContent = `wait ${st.time} ms`;
      } else if (st.type === 'send') {
        name.textContent = `send ${st.device}.${st.command}`;
      } else {
        name.textContent = JSON.stringify(st);
      }
      const badge = document.createElement('span');
      badge.className = 'badge';
      badge.textContent = `#${idx+1}`;
      const space = document.createElement('div');
      space.className = 'space';
      wrap.appendChild(badge);
      wrap.appendChild(name);
      wrap.appendChild(space);
      div.appendChild(wrap);
    });
    scriptBody.appendChild(div);
    scriptRunBtn.onclick = () => runScriptlet(controller, sc.name, scriptRunBtn);
    openScriptModal();
  } catch (e) {
    scriptsEl.innerHTML = `<p class="err">${e.message}</p>`;
  }
}

async function runScriptlet(controller, name, btn) {
  const prev = btn?.textContent;
  if (btn) { btn.disabled = true; btn.textContent = 'Running...'; }
  setStatus(`Running script ${controller}/${name}...`, 'muted');
  try {
    const res = await fetchJSON(`/${encodeURIComponent(controller)}/scripts/${encodeURIComponent(name)}`, { method: 'POST' });
    setStatus(`OK: ${res.status || 'ok'} — ${res.controller}/scripts/${res.scriptlet}`, 'ok');
  } catch (e) {
    setStatus(e.message, 'err');
  } finally {
    if (btn) { btn.disabled = false; btn.textContent = prev || 'Run'; }
    if (isScriptModalOpen()) closeScriptModal();
  }
}

// View switching logic
function setActiveView(mode) {
  const toFancy = mode === 'fancy';
  // persist mode
  lsSet(LS_KEYS.view, toFancy ? 'fancy' : 'list');
  if (viewFancyBtn && viewListBtn) {
    viewFancyBtn.classList.toggle('active', toFancy);
    viewFancyBtn.setAttribute('aria-pressed', toFancy ? 'true' : 'false');
    viewListBtn.classList.toggle('active', !toFancy);
    viewListBtn.setAttribute('aria-pressed', !toFancy ? 'true' : 'false');
  }
  if (commandsPanel && commandsView) {
    commandsPanel.hidden = toFancy;
    commandsView.hidden = !toFancy;
  }
  if (toFancy) updateFancyView();
}

function isFancyActive() {
  return !!(viewFancyBtn && viewFancyBtn.classList.contains('active'));
}

function updateFancyView() {
  if (!commandsView || !viewContainer) return;
  const controller = decodeURIComponent(controllerSel?.value || '');
  const devices = getSelectedDevices();
  viewContainer.innerHTML = '';
  if (!controller || !devices.length) {
    const p = document.createElement('p');
    p.className = 'muted';
    p.textContent = 'Select a controller and one or more devices to see the fancy view.';
    viewContainer.appendChild(p);
    return;
  }

  // Helper to flatten commands with coords
  function eachPositionedCommand(data, cb, groupPath = []) {
    const pushCmd = (cmd, name) => {
      const hasAll = ['x','y','width','height'].every(k => Object.prototype.hasOwnProperty.call(cmd, k));
      if (!hasAll) return;
      cb({
        path: [...groupPath, name].join('.'),
        disabled: !!cmd.disabled,
        x: cmd.x, y: cmd.y, width: cmd.width, height: cmd.height,
      });
    };
    (data.commands || []).forEach(c => pushCmd(c, c.name));
    (data.groups || []).forEach(g => {
      (g.commands || []).forEach(c => pushCmd(c, c.name));
      (g.groups || []).forEach(sg => {
        // recurse two+ levels
        eachPositionedGroup(sg, [...groupPath, g.name]);
      });
    });
    function eachPositionedGroup(group, gpath) {
      (group.commands || []).forEach(c => pushCmd(c, c.name));
      (group.groups || []).forEach(sg => eachPositionedGroup(sg, [...gpath, group.name]));
    }
  }

  const showDisabled = !!showDisabledChk.checked;

  // Render each device panel
  devices.forEach(async (dName) => {
    const panel = document.createElement('div');
    panel.className = 'device-view';
    const header = document.createElement('div');
    header.className = 'device-title';
    header.textContent = dName;
    panel.appendChild(header);

    const canvas = document.createElement('div');
    canvas.className = 'device-canvas';
    canvas.setAttribute('data-device', dName);

    const img = document.createElement('img');
    img.alt = dName;
    img.className = 'device-image';
    canvas.appendChild(img);

    const overlay = document.createElement('div');
    overlay.className = 'device-overlay';
    canvas.appendChild(overlay);

    viewContainer.appendChild(panel);
    panel.appendChild(canvas);

    try {
      const data = await fetchJSON(`/${encodeURIComponent(controller)}/${encodeURIComponent(dName)}`);
      header.textContent = `${data.friendly_name || data.device || dName}`;
      if (!data.image) {
        const note = document.createElement('p');
        note.className = 'muted';
        note.textContent = 'No image defined for this device. Fancy view is unavailable.';
        canvas.appendChild(note);
        return;
      }
      img.src = data.image;

      const positioned = [];
      // Walk tree and collect with coords
      const collect = [];
      const walk = (node, path=[]) => {
        (node.commands || []).forEach(c => {
          const hasAll = ['x','y','width','height'].every(k => Object.prototype.hasOwnProperty.call(c, k));
          if (hasAll) collect.push({ path: [...path, c.name].join('.'), disabled: !!c.disabled, x: c.x, y: c.y, width: c.width, height: c.height });
        });
        (node.groups || []).forEach(g => walk(g, [...path, g.name]));
      };
      walk(data, []);

      collect.forEach(c => { if (showDisabled || !c.disabled) positioned.push(c); });

      if (!positioned.length) {
        const note = document.createElement('p');
        note.className = 'muted';
        note.textContent = 'No positioned commands for this device.';
        canvas.appendChild(note);
        return;
      }

      function renderButtons() {
        // clear overlay
        overlay.innerHTML = '';
        const naturalW = img.naturalWidth || img.width; // fallback
        const naturalH = img.naturalHeight || img.height;
        if (!naturalW || !naturalH) return;
        const scaleX = img.clientWidth / naturalW;
        const scaleY = img.clientHeight / naturalH;
        positioned.forEach(cmd => {
          const btn = document.createElement('button');
          btn.className = 'hit' + (cmd.disabled ? ' disabled' : '');
          const left = Math.round(cmd.x * scaleX);
          const top = Math.round(cmd.y * scaleY);
          const w = Math.round(cmd.width * scaleX);
          const h = Math.round(cmd.height * scaleY);
          btn.style.left = left + 'px';
          btn.style.top = top + 'px';
          btn.style.width = w + 'px';
          btn.style.height = h + 'px';
          btn.title = cmd.path + (cmd.disabled ? ' (disabled)' : '');
          btn.disabled = !!cmd.disabled;
          btn.addEventListener('click', () => sendCommand(controller, dName, cmd.path));
          overlay.appendChild(btn);
        });
      }

      if (img.complete) {
        renderButtons();
      }
      img.addEventListener('load', () => renderButtons());
      window.addEventListener('resize', renderButtons);

    } catch (e) {
      const err = document.createElement('p');
      err.className = 'error';
      err.textContent = e.message;
      panel.appendChild(err);
    }
  });
}

// Bind view buttons
viewFancyBtn?.addEventListener('click', () => setActiveView('fancy'));
viewListBtn?.addEventListener('click', () => setActiveView('list'));

// Keep fancy view in sync with selection changes
controllerSel.addEventListener('change', () => { if (isFancyActive()) updateFancyView(); });
showDisabledChk.addEventListener('change', () => { if (isFancyActive()) updateFancyView(); });

// When devices selection changes, load list and refresh fancy view
function afterDevicesSelectionChanged() {
  if (isFancyActive()) updateFancyView();
}

// Initialize UI state from localStorage, then initial load
(async function initFromLocalStorage() {
  // showDisabled
  const sd = lsGet(LS_KEYS.showDisabled, '0');
  if (showDisabledChk) showDisabledChk.checked = sd === '1';
  // view mode
  const mode = lsGet(LS_KEYS.view, 'list');
  // Initial load
  await loadControllers();
  // Set view mode after controllers/devices are loaded
  if (mode === 'fancy') {
    setActiveView('fancy');
  } else {
    setActiveView('list');
  }
})();
