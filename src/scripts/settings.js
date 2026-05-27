import { api } from './api.js';

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
      <input type="text" id="setting-shortcut" value="${config.shortcut}" placeholder="Ctrl+Alt+x">
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

  document.querySelectorAll('input[name="theme"]').forEach(radio => {
    radio.addEventListener('change', () => {
      document.body.className = radio.value;
    });
  });

  document.getElementById('btn-save-settings').addEventListener('click', async () => {
    const theme = document.querySelector('input[name="theme"]:checked').value;
    const shortcut = document.getElementById('setting-shortcut').value.trim();
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

    document.body.className = theme;
    alert('Configuracoes salvas!');
  });

  document.getElementById('btn-check-updates').addEventListener('click', async () => {
    try {
      const { check } = window.__TAURI__.updater;
      const update = await check();
      if (update) {
        alert(`Atualizacao disponivel: v${update.version}`);
      } else {
        alert('Nenhuma atualizacao disponivel.');
      }
    } catch (e) {
      alert('Erro ao verificar atualizacoes: ' + e);
    }
  });
}
