import { initNotes } from './notes.js';
import { initReminders } from './reminders.js';
import { initSettings } from './settings.js';
import { api } from './api.js';

const { getCurrentWindow } = window.__TAURI__.window;

const tabs = document.querySelectorAll('.tab');
const views = document.querySelectorAll('.view');

async function applyTheme() {
  const config = await api.getConfig();
  document.body.className = config.theme;
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

document.addEventListener('keydown', async (e) => {
  if (e.key === 'Escape') {
    const win = getCurrentWindow();
    await win.hide();
  }
});

document.addEventListener('mousedown', async (e) => {
  const app = document.getElementById('app');
  if (!app.contains(e.target)) {
    const win = getCurrentWindow();
    await win.hide();
  }
});

applyTheme();
initNotes();
