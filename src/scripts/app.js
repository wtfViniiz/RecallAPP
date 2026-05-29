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
    pinIcon.innerHTML = icons['pin-filled'](16);
    pinBtn.title = 'Desafixar';
  } else {
    pinBtn.classList.remove('active');
    pinIcon.innerHTML = icons.pin(16);
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
    // Apply font size from config
    if (config.font_size) {
      document.documentElement.style.fontSize = config.font_size + 'px';
    }
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
    if (!target) return; // Help button has no data-tab

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
        <textarea id="quick-capture-input" placeholder="Anotacao rapida..."></textarea>
        <div class="quick-capture-actions">
          <button class="btn btn-secondary" id="quick-capture-cancel">Cancelar</button>
          <button class="btn btn-primary" id="quick-capture-save">Salvar</button>
        </div>
      </div>
    `;
    const input = document.getElementById('quick-capture-input');
    setTimeout(() => input.focus(), 50);
    document.getElementById('quick-capture-cancel').addEventListener('click', () => {
      document.querySelector('[data-tab="notes"]').click();
    });
    document.getElementById('quick-capture-save').addEventListener('click', async () => {
      const content = input.value.trim();
      if (content) {
        await api.createNote({
          title: content.split('\n')[0].slice(0, 50),
          content,
          tags: ['rascunho'],
          temporary: true,
        });
        showToast('Nota rapida salva (expira em 24h)', 'success');
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

  // Escape to hide (with auto-save)
  if (e.key === 'Escape' && !isPinned) {
    try {
      if (notesModule) {
        await notesModule.flushPendingSave();
      }
      await window.__TAURI_INTERNALS__.invoke('plugin:window|set_visible', { visible: false });
    } catch (err) {}
    return;
  }
});

// Click outside to hide (with auto-save)
document.addEventListener('mousedown', async (e) => {
  const appEl = document.getElementById('app');
  if (!appEl.contains(e.target) && !isPinned) {
    try {
      if (notesModule) {
        await notesModule.flushPendingSave();
      }
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

// Help button
const helpBtn = document.getElementById('btn-help');
if (helpBtn) {
  helpBtn.addEventListener('click', () => {
    if (document.querySelector('.modal-overlay')) return;
    const overlay = document.createElement('div');
    overlay.className = 'modal-overlay';
    const modal = document.createElement('div');
    modal.className = 'modal-confirm';
    modal.style.maxWidth = '440px';
    modal.style.maxHeight = '80vh';
    modal.style.overflowY = 'auto';
    modal.innerHTML = `
      <div class="modal-confirm-message" style="margin-bottom:16px;font-size:16px">Guia do Recall</div>
      <div style="font-size:13px;color:var(--text-secondary);line-height:1.6">
        <div style="margin-bottom:14px">
          <div style="color:var(--text-primary);font-weight:600;margin-bottom:4px">Notas</div>
          <p>Crie notas a partir de templates prontos (Reuniao, Tarefa, Diario, Estudo) ou em branco. Arraste os cards para reorganizar. Fixe notas importantes com o icone de pin.</p>
        </div>
        <div style="margin-bottom:14px">
          <div style="color:var(--text-primary);font-weight:600;margin-bottom:4px">Editor</div>
          <p>Use a barra de formatacao para negrito, italico, listas, citacoes e links. Cole imagens direto do clipboard. Suas notas sao salvas manualmente com Ctrl+S ou o botao de salvar. Ao sair do editor, o salvamento ocorre automaticamente.</p>
        </div>
        <div style="margin-bottom:14px">
          <div style="color:var(--text-primary);font-weight:600;margin-bottom:4px">Busca e Filtros</div>
          <p>Use Ctrl+P para buscar. Filtre por categoria ou tag nos dropdowns acima da lista. Botao "Recentes" mostra as ultimas 10 notas editadas.</p>
        </div>
        <div style="margin-bottom:14px">
          <div style="color:var(--text-primary);font-weight:600;margin-bottom:4px">Lembretes</div>
          <p>Crie lembretes com data/horario, recorrencia (diario, semanal, mensal) ou timer relativo. Vincule lembretes a notas. Quando disparar, snooze ou dismisso.</p>
        </div>
        <div style="margin-bottom:14px">
          <div style="color:var(--text-primary);font-weight:600;margin-bottom:4px">Templates</div>
          <p>Salve qualquer nota como template reutilizavel. Acesse o seletor de templates ao criar uma nova nota. Templates customizados podem ser excluidos.</p>
        </div>
        <div style="margin-bottom:14px">
          <div style="color:var(--text-primary);font-weight:600;margin-bottom:4px">Lixeira</div>
          <p>Notas excluidas vao para a lixeira. Restaure individualmente ou esvazie tudo. Notas temporarias (captura rapida) sao movidas automaticamente apos 24h.</p>
        </div>
        <div style="margin-bottom:14px">
          <div style="color:var(--text-primary);font-weight:600;margin-bottom:4px">Atalhos</div>
          <div style="display:grid;grid-template-columns:auto 1fr;gap:4px 12px;font-size:12px">
            <kbd style="background:var(--bg-tertiary);padding:2px 6px;border-radius:4px;font-family:monospace">Ctrl+N</kbd><span>Nova nota</span>
            <kbd style="background:var(--bg-tertiary);padding:2px 6px;border-radius:4px;font-family:monospace">Ctrl+Shift+N</kbd><span>Captura rapida (24h)</span>
            <kbd style="background:var(--bg-tertiary);padding:2px 6px;border-radius:4px;font-family:monospace">Ctrl+P</kbd><span>Buscar</span>
            <kbd style="background:var(--bg-tertiary);padding:2px 6px;border-radius:4px;font-family:monospace">Ctrl+S</kbd><span>Salvar nota</span>
            <kbd style="background:var(--bg-tertiary);padding:2px 6px;border-radius:4px;font-family:monospace">Ctrl+,</kbd><span>Configuracoes</span>
            <kbd style="background:var(--bg-tertiary);padding:2px 6px;border-radius:4px;font-family:monospace">Escape</kbd><span>Fechar janela</span>
          </div>
        </div>
        <div>
          <div style="color:var(--text-primary);font-weight:600;margin-bottom:4px">Dicas</div>
          <p>O icone na barra de tarefas permite acessar o Recall sem lembrar o atalho. Use o pin no header para manter a janela sempre visivel. Configuracoes permitem personalizar tema, atalhos e tamanho da fonte.</p>
        </div>
      </div>
      <div class="modal-confirm-actions" style="margin-top:16px">
        <button class="btn btn-secondary" id="help-close">Fechar</button>
      </div>
    `;
    overlay.appendChild(modal);
    document.body.appendChild(overlay);
    document.getElementById('help-close').addEventListener('click', () => overlay.remove());
    overlay.addEventListener('click', (e) => { if (e.target === overlay) overlay.remove(); });
    document.addEventListener('keydown', function handler(e) {
      if (e.key === 'Escape') { overlay.remove(); document.removeEventListener('keydown', handler); }
    });
  });
}
