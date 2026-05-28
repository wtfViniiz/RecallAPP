import { initNotes } from './notes.js';
import { initReminders } from './reminders.js';
import { initSettings } from './settings.js';
import { api } from './api.js';

const tabs = document.querySelectorAll('.tab');
const views = document.querySelectorAll('.view');
let isPinned = false;

async function applyTheme() {
  try {
    const config = await api.getConfig();
    document.body.className = config.theme;
  } catch (e) {
    console.error('Failed to load config:', e);
  }
}

// Pin window button
const pinBtn = document.getElementById('pin-window');
pinBtn.addEventListener('click', async () => {
  isPinned = !isPinned;
  pinBtn.classList.toggle('active', isPinned);
  pinBtn.innerHTML = isPinned ? '&#128204;' : '&#128204;';
  try {
    await api.setAlwaysOnTop(isPinned);
  } catch (e) {
    console.error('Failed to set always on top:', e);
  }
});

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
  if (e.key === 'Escape' && !isPinned) {
    try {
      await window.__TAURI_INTERNALS__.invoke('plugin:window|set_visible', { visible: false });
    } catch (err) {
      // Ignore
    }
  }
});

// Click outside to hide (only if not pinned)
document.addEventListener('mousedown', async (e) => {
  const appEl = document.getElementById('app');
  if (!appEl.contains(e.target) && !isPinned) {
    try {
      await window.__TAURI_INTERNALS__.invoke('plugin:window|set_visible', { visible: false });
    } catch (err) {
      // Ignore
    }
  }
});

applyTheme();
initNotes();
