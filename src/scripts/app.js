import { initNotes } from './notes.js';
import { initReminders } from './reminders.js';
import { initSettings } from './settings.js';
import { api } from './api.js';

const tabs = document.querySelectorAll('.tab');
const views = document.querySelectorAll('.view');

async function applyTheme() {
  try {
    const config = await api.getConfig();
    document.body.className = config.theme;
  } catch (e) {
    console.error('Failed to load config:', e);
  }
}

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

// Escape to hide window
document.addEventListener('keydown', async (e) => {
  if (e.key === 'Escape') {
    try {
      await window.__TAURI_INTERNALS__.invoke('plugin:window|set_visible', { visible: false });
    } catch (err) {
      // Ignore
    }
  }
});

// Click outside to hide
document.addEventListener('mousedown', async (e) => {
  const appEl = document.getElementById('app');
  if (!appEl.contains(e.target)) {
    try {
      await window.__TAURI_INTERNALS__.invoke('plugin:window|set_visible', { visible: false });
    } catch (err) {
      // Ignore
    }
  }
});

applyTheme();
initNotes();
