import { initNotes } from './notes.js';
import { initReminders } from './reminders.js';
import { initSettings } from './settings.js';
import { api } from './api.js';

const { getCurrentWindow } = window.__TAURI__.window;
const win = getCurrentWindow();

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
  if (e.key === 'Escape' && !isPinned) {
    try {
      await win.hide();
    } catch (err) {}
  }
});

// Click outside to hide (only if not pinned)
document.addEventListener('mousedown', async (e) => {
  const appEl = document.getElementById('app');
  if (!appEl.contains(e.target) && !isPinned) {
    try {
      await win.hide();
    } catch (err) {}
  }
});

applyTheme();
initNotes();
