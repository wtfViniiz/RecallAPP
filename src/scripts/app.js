import { api } from './api.js';
import { showToast } from './utils.js';
import { icons } from './icons.js';

const tabs = document.querySelectorAll('.tab');
const views = document.querySelectorAll('.view');
let isPinned = false;
let notesModule = null;

// Pin window button
const pinBtn = document.getElementById('pin-window');
let pinIcon = null;

if (pinBtn) {
  pinIcon = pinBtn.querySelector('.pin-icon');
}

function updatePinVisual() {
  if (!pinBtn || !pinIcon) return;
  if (isPinned) {
    pinBtn.classList.add('active');
    pinIcon.innerHTML = icons['star-filled'](16);
    pinBtn.title = 'Desafixar';
  } else {
    pinBtn.classList.remove('active');
    pinIcon.innerHTML = icons.star(16);
    pinBtn.title = 'Fixar em primeiro plano';
  }
}

if (pinBtn) {
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
}

updatePinVisual();

// Theme + config
async function applyTheme() {
  try {
    const config = await api.getConfig();
    document.body.className = config.theme;
    // Fix L8: Persist pin state from config
    if (config.always_on_top) {
      isPinned = true;
      updatePinVisual();
    }
  } catch (e) {
    console.error('Theme error:', e);
  }
}

// Tab switching

// Initialize the default active tab on load
(async () => {
  notesModule = await import('./notes.js');
  notesModule.initNotes();
  applyTheme();
})();

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
    // Fix M1: null check on view element
    document.getElementById(`view-${target}`)?.classList.add('active');

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

// Fix M3: Consolidated single keydown handler
document.addEventListener('keydown', async (e) => {
  // Ctrl+Shift+N — quick capture (highest priority)
  // Fix H2: flush pending saves before destroying view
  if (e.ctrlKey && e.shiftKey && e.key === 'N') {
    e.preventDefault();
    if (notesModule) {
      await notesModule.flushPendingSave();
    }
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
    return;
  }

  // Ctrl+N: New note
  // Fix M2: Use MutationObserver to wait for btn-new-note
  if (e.ctrlKey && !e.shiftKey && e.key === 'n') {
    e.preventDefault();
    document.querySelector('[data-tab="notes"]').click();
    const waitForBtn = () => {
      const btn = document.getElementById('btn-new-note');
      if (btn) {
        btn.click();
      } else {
        requestAnimationFrame(waitForBtn);
      }
    };
    requestAnimationFrame(waitForBtn);
    return;
  }

  // Ctrl+P: Focus search
  if (e.ctrlKey && e.key === 'p') {
    e.preventDefault();
    document.querySelector('[data-tab="notes"]').click();
    const waitForSearch = () => {
      const search = document.getElementById('note-search');
      if (search) {
        search.focus();
        search.select();
      } else {
        requestAnimationFrame(waitForSearch);
      }
    };
    requestAnimationFrame(waitForSearch);
    return;
  }

  // Ctrl+,: Settings
  if (e.ctrlKey && e.key === ',') {
    e.preventDefault();
    document.querySelector('[data-tab="settings"]').click();
    return;
  }

  // Escape to hide
  if (e.key === 'Escape' && !isPinned) {
    try {
      await window.__TAURI_INTERNALS__.invoke('plugin:window|set_visible', { visible: false });
    } catch (err) {}
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

// Prevent double-click on title bar from maximizing window
const headerBar = document.getElementById('header-bar');
if (headerBar) {
  headerBar.addEventListener('dblclick', (e) => {
    e.preventDefault();
    e.stopPropagation();
  });
}

// Tray actions
window.addEventListener('tray-action', (e) => {
  const action = e.detail;
  if (action === 'new-note') {
    document.querySelector('[data-tab="notes"]').click();
    setTimeout(() => document.getElementById('btn-new-note')?.click(), 200);
  } else if (action === 'direct-new-note') {
    // Open editor directly without template selector
    document.querySelector('[data-tab="notes"]').click();
    setTimeout(async () => {
      if (notesModule) {
        await notesModule.openEditorDirect();
      }
    }, 200);
  } else if (action === 'new-reminder') {
    document.querySelector('[data-tab="reminders"]').click();
    setTimeout(() => document.getElementById('btn-new-reminder')?.click(), 200);
  } else if (action === 'settings') {
    document.querySelector('[data-tab="settings"]').click();
  }
});
