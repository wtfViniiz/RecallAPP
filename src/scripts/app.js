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
  } catch (e) {}
}

// Pin window button
const pinBtn = document.getElementById('pin-window');
const pinIcon = pinBtn.querySelector('.pin-icon');

function updatePinVisual() {
  if (isPinned) {
    pinBtn.classList.add('active');
    pinIcon.innerHTML = '&#128204;'; // Filled pin
    pinBtn.style.color = 'var(--accent)';
    pinBtn.style.background = 'var(--bg-primary)';
  } else {
    pinBtn.classList.remove('active');
    pinIcon.innerHTML = '&#128204;'; // Outline pin
    pinBtn.style.color = 'var(--text-secondary)';
    pinBtn.style.background = 'none';
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
  if (e.key === 'Escape' && !isPinned) {
    try {
      await window.__TAURI_INTERNALS__.invoke('plugin:window|set_visible', { visible: false });
    } catch (err) {}
  }
});

// Click outside to hide (only if not pinned)
document.addEventListener('mousedown', async (e) => {
  const appEl = document.getElementById('app');
  if (!appEl.contains(e.target) && !isPinned) {
    try {
      await window.__TAURI_INTERNALS__.invoke('plugin:window|set_visible', { visible: false });
    } catch (err) {}
  }
});

applyTheme();
initNotes();
