import { api } from './api.js';
import { showToast } from './utils.js';

export async function initSettings() {
  const container = document.getElementById('view-settings');
  let config;
  try {
    config = await api.getConfig();
  } catch (err) {
    showToast(err.message || 'Erro ao carregar configuracoes', 'error');
    return;
  }
  const version = await api.getAppVersion().catch(() => '0.1.0');

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
    <div class="form-group">
      <label>Atalho nova nota direta</label>
      <div class="shortcut-capture" id="new-note-shortcut-display" tabindex="0" title="Clique para gravar">
        ${config.new_note_shortcut ? formatShortcut(config.new_note_shortcut) : 'Nenhum'}
      </div>
      <input type="hidden" id="setting-new-note-shortcut" value="${config.new_note_shortcut || ''}">
      <div style="font-size: 11px; color: var(--text-secondary); margin-top: 4px;">
        Abre editor direto sem seletor de template. Clique para gravar.
      </div>
    </div>
    <div class="form-group" style="display:flex; justify-content:space-between; align-items:center;">
      <label style="margin:0">Iniciar com o Windows</label>
      <label class="toggle">
        <input type="checkbox" id="setting-autostart" ${config.autostart ? 'checked' : ''}>
        <span class="slider"></span>
      </label>
    </div>
    <div class="form-group">
      <label>Backup</label>
      <div style="display: flex; gap: 8px;">
        <button class="btn btn-secondary" id="btn-export">Exportar dados</button>
        <button class="btn btn-secondary" id="btn-import">Importar dados</button>
      </div>
      <input type="file" id="import-file" accept=".json" style="display:none">
    </div>
    <div style="margin-top: 24px;">
      <button class="btn btn-primary" id="btn-save-settings">Salvar</button>
    </div>
    <div class="timestamp" style="margin-top: 16px;">Versao: ${version}</div>
  `;

  // Theme toggle live preview
  document.querySelectorAll('input[name="theme"]').forEach(radio => {
    radio.addEventListener('change', () => {
      document.body.className = radio.value;
    });
  });

  // Shortcut capture
  setupShortcutCapture();
  setupNewNoteShortcutCapture();

  // Save button
  document.getElementById('btn-save-settings').addEventListener('click', async () => {
    const theme = document.querySelector('input[name="theme"]:checked').value;
    const shortcut = document.getElementById('setting-shortcut').value;
    const newNoteShortcut = document.getElementById('setting-new-note-shortcut').value;
    const autostart = document.getElementById('setting-autostart').checked;

    // Save config first (independent of shortcut)
    try {
      await api.updateConfig({
        theme,
        shortcut,
        new_note_shortcut: newNoteShortcut,
        autostart,
        check_updates: config.check_updates,
        window_width: config.window_width,
        window_height: config.window_height,
      });
    } catch (e) {
      console.error('Failed to save settings:', e);
      showToast(e.message || 'Erro ao salvar configuracoes', 'error');
      return;
    }

    // Then update shortcuts (may fail if invalid/in-use)
    try {
      await api.updateShortcut(shortcut);
    } catch (e) {
      console.error('Failed to update shortcut:', e);
      showToast(e.message || 'Config salvas, mas atalho invalido ou ja em uso', 'warning');
    }

    try {
      await api.updateNewNoteShortcut(newNoteShortcut);
    } catch (e) {
      console.error('Failed to update new note shortcut:', e);
      showToast(e.message || 'Config salvas, mas atalho de nova nota invalido', 'warning');
    }

    // Toggle autostart registration
    try {
      if (autostart) {
        await window.__TAURI_INTERNALS__.invoke('plugin:autostart|enable');
      } else {
        await window.__TAURI_INTERNALS__.invoke('plugin:autostart|disable');
      }
    } catch (e) {
      console.error('Autostart toggle error:', e);
    }

    document.body.className = theme;
    showToast('Configuracoes salvas', 'success');
  });

  // Export button
  document.getElementById('btn-export').addEventListener('click', async () => {
    try {
      const data = await api.exportData();
      const blob = new Blob([data], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `recall-backup-${new Date().toISOString().slice(0,10)}.json`;
      a.click();
      // Revoke after delay to ensure download starts
      setTimeout(() => URL.revokeObjectURL(url), 5000);
      showToast('Dados exportados', 'success');
    } catch (e) {
      showToast(e.message || 'Erro ao exportar', 'error');
    }
  });

  // Import button
  document.getElementById('btn-import').addEventListener('click', () => {
    document.getElementById('import-file').click();
  });

  document.getElementById('import-file').addEventListener('change', async (e) => {
    const file = e.target.files[0];
    if (!file) return;

    try {
      const text = await file.text();
      const result = await api.importData(text);
      showToast(result, 'success');
    } catch (err) {
      showToast('Erro ao importar: ' + (err.message || err), 'error');
    }
    e.target.value = '';
  });
}

function setupShortcutCaptureFor(displayId, inputId, toastPrefix) {
  const display = document.getElementById(displayId);
  const input = document.getElementById(inputId);
  if (!display || !input) return;
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

    if (e.key === 'Escape') {
      isRecording = false;
      display.classList.remove('recording');
      display.textContent = input.value ? formatShortcut(input.value) : 'Nenhum';
      return;
    }

    if (!e.ctrlKey && !e.altKey && !e.shiftKey && !e.metaKey) {
      return;
    }

    if (['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) {
      return;
    }

    const key = normalizeShortcutKey(e);
    if (!key) {
      return;
    }

    const parts = [];
    if (e.ctrlKey) parts.push('Ctrl');
    if (e.altKey) parts.push('Alt');
    if (e.shiftKey) parts.push('Shift');
    if (e.metaKey) parts.push('Super');
    parts.push(key);

    const shortcut = parts.join('+');
    input.value = shortcut;
    display.textContent = formatShortcut(shortcut);
    display.classList.remove('recording');
    isRecording = false;

    showToast(`${toastPrefix}: ${formatShortcut(shortcut)}`, 'success');
  });

  display.addEventListener('blur', () => {
    if (isRecording) {
      isRecording = false;
      display.classList.remove('recording');
      display.textContent = input.value ? formatShortcut(input.value) : 'Nenhum';
    }
  });
}

function setupShortcutCapture() {
  setupShortcutCaptureFor('shortcut-display', 'setting-shortcut', 'Atalho definido');
}

function setupNewNoteShortcutCapture() {
  setupShortcutCaptureFor('new-note-shortcut-display', 'setting-new-note-shortcut', 'Atalho nova nota definido');
}

function normalizeShortcutKey(e) {
  const code = e.code || '';
  const supportedCodes = new Set([
    'Backquote', 'Backslash', 'BracketLeft', 'BracketRight', 'Comma',
    'Equal', 'Minus', 'Period', 'Quote', 'Semicolon', 'Slash',
    'Backspace', 'CapsLock', 'Delete', 'End', 'Enter', 'Escape',
    'Home', 'Insert', 'PageDown', 'PageUp', 'Pause', 'PrintScreen',
    'ScrollLock', 'Space', 'Tab', 'ArrowDown', 'ArrowLeft',
    'ArrowRight', 'ArrowUp', 'NumLock', 'NumpadAdd', 'NumpadDecimal',
    'NumpadDivide', 'NumpadEnter', 'NumpadEqual', 'NumpadMultiply',
    'NumpadSubtract',
  ]);

  if (/^Key[A-Z]$/.test(code) || /^Digit[0-9]$/.test(code) || /^Numpad[0-9]$/.test(code)) {
    return code;
  }

  if (/^F([1-9]|1[0-9]|2[0-4])$/.test(code) || supportedCodes.has(code)) {
    return code;
  }

  const keyAliases = {
    ' ': 'Space',
    Esc: 'Escape',
    Del: 'Delete',
    Up: 'ArrowUp',
    Down: 'ArrowDown',
    Left: 'ArrowLeft',
    Right: 'ArrowRight',
  };

  if (keyAliases[e.key]) {
    return keyAliases[e.key];
  }

  return e.key.length === 1 ? e.key.toUpperCase() : e.key;
}

function formatShortcut(shortcut) {
  if (!shortcut) return 'Nenhum';
  return shortcut.split('+').map(formatShortcutPart).join(' + ');
}

function formatShortcutPart(part) {
  const normalized = part.trim();
  const lower = normalized.toLowerCase();

  if (/^key[a-z]$/i.test(normalized)) {
    return normalized.slice(3).toUpperCase();
  }

  if (/^digit[0-9]$/i.test(normalized)) {
    return normalized.slice(5);
  }

  if (/^numpad[0-9]$/i.test(normalized)) {
    return `Num ${normalized.slice(6)}`;
  }

  if (/^f([1-9]|1[0-9]|2[0-4])$/i.test(normalized)) {
    return normalized.toUpperCase();
  }

  const labels = {
    arrowup: 'Up',
    arrowdown: 'Down',
    arrowleft: 'Left',
    arrowright: 'Right',
    backquote: '`',
    backslash: '\\',
    bracketleft: '[',
    bracketright: ']',
    comma: ',',
    digit0: '0',
    digit1: '1',
    digit2: '2',
    digit3: '3',
    digit4: '4',
    digit5: '5',
    digit6: '6',
    digit7: '7',
    digit8: '8',
    digit9: '9',
    equal: '=',
    minus: '-',
    period: '.',
    quote: "'",
    semicolon: ';',
    slash: '/',
    pagedown: 'Page Down',
    pageup: 'Page Up',
    printscreen: 'Print Screen',
    scrolllock: 'Scroll Lock',
    capslock: 'Caps Lock',
    numlock: 'Num Lock',
    numpadadd: 'Num +',
    numpaddecimal: 'Num .',
    numpaddivide: 'Num /',
    numpadenter: 'Num Enter',
    numpadequal: 'Num =',
    numpadmultiply: 'Num *',
    numpadsubtract: 'Num -',
  };

  if (labels[lower]) {
    return labels[lower];
  }

  switch (lower) {
    case 'ctrl': return 'Ctrl';
    case 'alt': return 'Alt';
    case 'shift': return 'Shift';
    case 'super': return 'Win';
    default: return normalized.toUpperCase();
  }
}
