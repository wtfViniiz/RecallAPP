import { api } from './api.js';
import { showToast } from './utils.js';

const tabs = document.querySelectorAll('.tab');
const views = document.querySelectorAll('.view');
let isPinned = false;

// Pin window button
const pinBtn = document.getElementById('pin-window');
const pinIcon = pinBtn.querySelector('.pin-icon');

function updatePinVisual() {
  if (isPinned) {
    pinBtn.classList.add('active');
    pinIcon.innerHTML = '&#9733;';
    pinBtn.title = 'Desafixar';
  } else {
    pinBtn.classList.remove('active');
    pinIcon.innerHTML = '&#9734;';
    pinBtn.title = 'Fixar em primeiro plano';
  }
}

pinBtn.addEventListener('click', async () => {
  const newState = !isPinned;
  try {
    await api.setAlwaysOnTop(newState);
    isPinned = newState;
    updatePinVisual();
  } catch (e) {
    showToast('Erro ao fixar janela', 'error');
    // Revert visual on failure
    isPinned = !newState;
    updatePinVisual();
  }
});

updatePinVisual();

// Theme
async function applyTheme() {
  try {
    const config = await api.getConfig();
    document.body.className = config.theme;
  } catch (e) {
    console.error('Theme error:', e);
  }
}

// Tab switching
let notesModule = null;

tabs.forEach(tab => {
  tab.addEventListener('click', async () => {
    const target = tab.dataset.tab;

    // Flush pending saves before leaving notes
    if (notesModule) {
      await notesModule.flushPendingSave();
    }

    tabs.forEach(t => t.classList.remove('active'));
    views.forEach(v => v.classList.remove('active'));
    tab.classList.add('active');
    document.getElementById(`view-${target}`).classList.add('active');

    if (target === 'notes') {
      notesModule = await import('./notes.js');
      notesModule.initNotes();
    }
    if (target === 'reminders') {
      const { initReminders } = await import('./reminders.js');
      initReminders();
    }
    if (target === 'settings') {
      const { initSettings } = await import('./settings.js');
      initSettings();
    }
  });
});

// Escape to hide
document.addEventListener('keydown', async (e) => {
  if (e.key === 'Escape' && !isPinned) {
    try {
      await window.__TAURI_INTERNALS__.invoke('plugin:window|set_visible', { visible: false });
    } catch (err) {}
  }
});

// Global keyboard shortcuts
document.addEventListener('keydown', async (e) => {
  // Ctrl+N: New note
  if (e.ctrlKey && !e.shiftKey && e.key === 'n') {
    e.preventDefault();
    document.querySelector('[data-tab="notes"]').click();
    setTimeout(() => document.getElementById('btn-new-note')?.click(), 100);
    return;
  }

  // Ctrl+P: Focus search
  if (e.ctrlKey && e.key === 'p') {
    e.preventDefault();
    document.querySelector('[data-tab="notes"]').click();
    setTimeout(() => {
      const search = document.getElementById('note-search');
      if (search) { search.focus(); search.select(); }
    }, 100);
    return;
  }

  // Ctrl+,: Settings
  if (e.ctrlKey && e.key === ',') {
    e.preventDefault();
    document.querySelector('[data-tab="settings"]').click();
    return;
  }
});

// Click outside to hide
document.addEventListener('mousedown', async (e) => {
  const appEl = document.getElementById('app');
  if (!appEl.contains(e.target) && !isPinned) {
    try {
      await window.__TAURI_INTERNALS__.invoke('plugin:window|set_visible', { visible: false });
    } catch (err) {}
  }
});

// Tray actions
window.addEventListener('tray-action', (e) => {
  const action = e.detail;
  if (action === 'new-note') {
    document.querySelector('[data-tab="notes"]').click();
    setTimeout(() => document.getElementById('btn-new-note')?.click(), 200);
  } else if (action === 'new-reminder') {
    document.querySelector('[data-tab="reminders"]').click();
    setTimeout(() => document.getElementById('btn-new-reminder')?.click(), 200);
  } else if (action === 'settings') {
    document.querySelector('[data-tab="settings"]').click();
  }
});

// Quick capture
document.addEventListener('keydown', async (e) => {
  if (e.ctrlKey && e.shiftKey && e.key === 'N') {
    e.preventDefault();
    tabs.forEach(t => t.classList.remove('active'));
    views.forEach(v => v.classList.remove('active'));
    document.querySelector('[data-tab="notes"]').classList.add('active');
    document.getElementById('view-notes').classList.add('active');
    document.getElementById('view-notes').innerHTML = `
      <div class="quick-capture">
        <textarea id="quick-capture-input" placeholder="Anotacao rapida..." autofocus></textarea>
        <div class="quick-capture-actions">
          <button class="btn btn-secondary" id="quick-capture-cancel">Cancelar</button>
          <button class="btn btn-primary" id="quick-capture-save">Salvar</button>
        </div>
      </div>
    `;
    const input = document.getElementById('quick-capture-input');
    input.focus();
    document.getElementById('quick-capture-cancel').addEventListener('click', () => {
      document.querySelector('[data-tab="notes"]').click();
    });
    document.getElementById('quick-capture-save').addEventListener('click', async () => {
      const content = input.value.trim();
      if (content) {
        await api.createNote({ title: content.split('\n')[0].slice(0, 50), content, tags: ['rascunho'] });
        showToast('Nota rapida salva', 'success');
      }
      document.querySelector('[data-tab="notes"]').click();
    });
  }
});

applyTheme();
// Init notes tab on load
import('./notes.js').then(({ initNotes }) => initNotes());
