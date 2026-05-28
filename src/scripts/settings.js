import { api } from './api.js';
import { showToast } from './utils.js';

export async function initSettings() {
  const container = document.getElementById('view-settings');
  const config = await api.getConfig();

  container.innerHTML = `
    <h3 style="margin-bottom: 16px;">Configuracoes</h3>
    <div class="form-group">
      <label>Tema</label>
      <div class="radio-group">
        <label><input type="radio" name="theme" value="dark" ${config.theme === 'dark' ? 'checked' : ''}> Dark</label>
        <label><input type="radio" name="theme" value="light" ${config.theme === 'light' ? 'checked' : ''}> Light</label>
      </div>
    </div>
    <div class="form-group">
      <label>Atalho global</label>
      <div class="shortcut-capture" id="shortcut-display" tabindex="0" title="Clique para gravar">
        ${formatShortcut(config.shortcut)}
      </div>
      <input type="hidden" id="setting-shortcut" value="${config.shortcut}">
      <div style="font-size: 11px; color: var(--text-secondary); margin-top: 4px;">
        Clique e pressione a combinacao desejada
      </div>
    </div>
    <div class="form-group" style="display:flex; justify-content:space-between; align-items:center;">
      <label style="margin:0">Iniciar com o Windows</label>
      <label class="toggle">
        <input type="checkbox" id="setting-autostart" ${config.autostart ? 'checked' : ''}>
        <span class="slider"></span>
      </label>
    </div>
    <div class="form-group" style="display:flex; justify-content:space-between; align-items:center;">
      <label style="margin:0">Verificar atualizacoes</label>
      <label class="toggle">
        <input type="checkbox" id="setting-updates" ${config.check_updates ? 'checked' : ''}>
        <span class="slider"></span>
      </label>
    </div>
    <div class="form-group">
      <button class="btn btn-secondary" id="btn-check-updates">Verificar atualizacoes agora</button>
    </div>
    <div style="margin-top: 24px;">
      <button class="btn btn-primary" id="btn-save-settings">Salvar</button>
    </div>
    <div class="timestamp" style="margin-top: 16px;">Versao: 0.1.0</div>
  `;

  // Theme toggle live preview
  document.querySelectorAll('input[name="theme"]').forEach(radio => {
    radio.addEventListener('change', () => {
      document.body.className = radio.value;
    });
  });

  // Shortcut capture
  setupShortcutCapture();

  // Save button
  document.getElementById('btn-save-settings').addEventListener('click', async () => {
    const theme = document.querySelector('input[name="theme"]:checked').value;
    const shortcut = document.getElementById('setting-shortcut').value;
    const autostart = document.getElementById('setting-autostart').checked;
    const check_updates = document.getElementById('setting-updates').checked;

    await api.updateConfig({
      theme,
      shortcut,
      autostart,
      check_updates,
      window_width: config.window_width,
      window_height: config.window_height,
    });

    // Update the global shortcut
    try {
      await window.__TAURI_INTERNALS__.invoke('update_shortcut', { shortcutStr: shortcut });
    } catch (e) {
      console.error('Failed to update shortcut:', e);
    }

    document.body.className = theme;
    showToast('Configuracoes salvas', 'success');
  });

  // Check updates button
  document.getElementById('btn-check-updates').addEventListener('click', async () => {
    try {
      const update = await window.__TAURI_INTERNALS__.invoke('plugin:updater|check');
      if (update && update.available) {
        showToast(`Atualizacao disponivel: v${update.version}`, 'info');
      } else {
        showToast('Nenhuma atualizacao disponivel', 'info');
      }
    } catch (e) {
      showToast('Erro ao verificar atualizacoes', 'error');
    }
  });
}

function setupShortcutCapture() {
  const display = document.getElementById('shortcut-display');
  const input = document.getElementById('setting-shortcut');
  let isRecording = false;

  display.addEventListener('click', () => {
    isRecording = true;
    display.classList.add('recording');
    display.textContent = 'Pressione a combinacao...';
    display.focus();
  });

  display.addEventListener('keydown', (e) => {
    if (!isRecording) return;
    e.preventDefault();
    e.stopPropagation();

    // Escape cancels recording
    if (e.key === 'Escape') {
      isRecording = false;
      display.classList.remove('recording');
      display.textContent = formatShortcut(input.value);
      return;
    }

    // Need at least one modifier
    if (!e.ctrlKey && !e.altKey && !e.shiftKey && !e.metaKey) {
      return;
    }

    // Ignore modifier-only keydowns
    if (['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) {
      return;
    }

    // Build shortcut string
    const parts = [];
    if (e.ctrlKey) parts.push('Ctrl');
    if (e.altKey) parts.push('Alt');
    if (e.shiftKey) parts.push('Shift');
    if (e.metaKey) parts.push('Super');
    parts.push(e.key.length === 1 ? e.key.toUpperCase() : e.key);

    const shortcut = parts.join('+');
    input.value = shortcut;
    display.textContent = formatShortcut(shortcut);
    display.classList.remove('recording');
    isRecording = false;

    showToast(`Atalho definido: ${formatShortcut(shortcut)}`, 'success');
  });

  display.addEventListener('blur', () => {
    if (isRecording) {
      isRecording = false;
      display.classList.remove('recording');
      display.textContent = formatShortcut(input.value);
    }
  });
}

function formatShortcut(shortcut) {
  if (!shortcut) return 'Nenhum';
  return shortcut.split('+').map(part => {
    switch (part.toLowerCase()) {
      case 'ctrl': return 'Ctrl';
      case 'alt': return 'Alt';
      case 'shift': return 'Shift';
      case 'super': return 'Win';
      default: return part.toUpperCase();
    }
  }).join(' + ');
}
