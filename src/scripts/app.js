import { initNotes } from './notes.js';
import { initReminders } from './reminders.js';
import { initSettings } from './settings.js';
import { api } from './api.js';
import { showToast } from './utils.js';

let win = null;
try {
  win = window.__TAURI__?.window?.getCurrentWindow?.() || null;
} catch (e) {}

const tabs = document.querySelectorAll('.tab');
const views = document.querySelectorAll('.view');
let isPinned = false;

async function applyTheme() {
  try {
    const config = await api.getConfig();
    document.body.className = config.theme;
  } catch (e) {}
}

// Pin window button
const pinBtn = document.getElementById('pin-window');
const pinIcon = pinBtn.querySelector('.pin-icon');

function updatePinVisual() {
  if (isPinned) {
    pinBtn.classList.add('active');
    pinIcon.innerHTML = '&#9733;'; // Filled star
    pinBtn.title = 'Desafixar';
  } else {
    pinBtn.classList.remove('active');
    pinIcon.innerHTML = '&#9734;'; // Outline star
    pinBtn.title = 'Fixar em primeiro plano';
  }
}

pinBtn.addEventListener('click', async () => {
  isPinned = !isPinned;
  updatePinVisual();
  try {
    await api.setAlwaysOnTop(isPinned);
  } catch (e) {}
});

updatePinVisual();

tabs.forEach(tab => {
  tab.addEventListener('click', () => {
    const target = tab.dataset.tab;
    tabs.forEach(t => t.classList.remove('active'));
    views.forEach(v => v.classList.remove('active'));
    tab.classList.add('active');
    document.getElementById(`view-${target}`).classList.add('active');

    if (target === 'notes') initNotes();
    if (target === 'reminders') initReminders();
    if (target === 'settings') initSettings();
  });
});

// Escape to hide window (only if not pinned)
document.addEventListener('keydown', async (e) => {
  if (e.key === 'Escape' && !isPinned && win) {
    try {
      await win.hide();
    } catch (err) {}
  }
});

// Click outside to hide (only if not pinned)
document.addEventListener('mousedown', async (e) => {
  const appEl = document.getElementById('app');
  if (!appEl.contains(e.target) && !isPinned && win) {
    try {
      await win.hide();
    } catch (err) {}
  }
});

applyTheme();
initNotes();

// Tray action handler
window.addEventListener('tray-action', (e) => {
  const action = e.detail;
  if (action === 'new-note') {
    document.querySelector('[data-tab="notes"]').click();
    setTimeout(() => document.getElementById('btn-new-note')?.click(), 100);
  } else if (action === 'new-reminder') {
    document.querySelector('[data-tab="reminders"]').click();
    setTimeout(() => document.getElementById('btn-new-reminder')?.click(), 100);
  } else if (action === 'settings') {
    document.querySelector('[data-tab="settings"]').click();
  }
});

// Quick capture (Ctrl+Shift+N)
document.addEventListener('keydown', async (e) => {
  if (e.ctrlKey && e.shiftKey && e.key === 'N') {
    e.preventDefault();
    showQuickCapture();
  }
});

function showQuickCapture() {
  // Switch to notes tab
  tabs.forEach(t => t.classList.remove('active'));
  views.forEach(v => v.classList.remove('active'));
  document.querySelector('[data-tab="notes"]').classList.add('active');
  document.getElementById('view-notes').classList.add('active');

  const container = document.getElementById('view-notes');
  container.innerHTML = `
    <div class="quick-capture">
      <textarea id="quick-capture-input" placeholder="Anotacao rapida... (Enter para salvar, Esc para cancelar)" autofocus></textarea>
      <div class="quick-capture-actions">
        <button class="btn btn-secondary" id="quick-capture-cancel">Cancelar</button>
        <button class="btn btn-primary" id="quick-capture-save">Salvar</button>
      </div>
    </div>
  `;

  const input = document.getElementById('quick-capture-input');
  input.focus();

  document.getElementById('quick-capture-cancel').addEventListener('click', () => {
    initNotes();
  });

  document.getElementById('quick-capture-save').addEventListener('click', async () => {
    const content = input.value.trim();
    if (content) {
      const title = content.split('\n')[0].slice(0, 50);
      await api.createNote({ title, content, tags: ['rascunho'] });
      showToast('Nota rapida salva', 'success');
    }
    initNotes();
  });

  input.addEventListener('keydown', async (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      document.getElementById('quick-capture-save').click();
    }
    if (e.key === 'Escape') {
      initNotes();
    }
  });
}
