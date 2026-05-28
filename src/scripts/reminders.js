import { api } from './api.js';
import { formatDateTime, formatRelativeDate, escapeHtml, showToast } from './utils.js';

export async function initReminders() {
  await renderRemindersList();
}

async function renderRemindersList() {
  const container = document.getElementById('view-reminders');

  container.innerHTML = `
    <div class="toolbar">
      <h3>Lembretes</h3>
      <button class="btn btn-primary" id="btn-new-reminder">+ Novo</button>
    </div>
    <div id="reminders-list"></div>
  `;

  document.getElementById('btn-new-reminder').addEventListener('click', () => openReminderForm(null));
  await loadReminders();
}

async function loadReminders() {
  const reminders = await api.getReminders();
  const list = document.getElementById('reminders-list');

  if (reminders.length === 0) {
    list.innerHTML = '<div class="empty">Nenhum lembrete</div>';
    return;
  }

  list.innerHTML = reminders.map(r => {
    const statusClass = r.status === 'pending' ? 'badge-pending' :
                        r.status === 'fired' ? 'badge-fired' : 'badge-dismissed';
    const statusLabel = r.status === 'pending' ? 'Pendente' :
                        r.status === 'fired' ? 'Disparado' : 'Dispensado';
    const repeatLabel = r.repeat ? ` · ${r.repeat === 'daily' ? 'Diario' : r.repeat === 'weekly' ? 'Semanal' : 'Mensal'}` : '';

    return `
      <div class="card" data-id="${r.id}">
        <div class="card-title">${escapeHtml(r.title)}</div>
        <div class="card-meta">
          <span class="badge ${statusClass}">${statusLabel}</span>
          <span>${formatDateTime(r.trigger_at)}${repeatLabel}</span>
          ${r.status === 'pending' ? `<span>${formatRelativeDate(r.trigger_at)}</span>` : ''}
        </div>
        ${r.description ? `<div class="card-meta">${escapeHtml(r.description)}</div>` : ''}
        ${r.status === 'fired' ? `<button class="btn btn-secondary btn-sm" data-dismiss="${r.id}">Dispensar</button>` : ''}
      </div>
    `;
  }).join('');

  list.querySelectorAll('.card').forEach(card => {
    card.addEventListener('click', (e) => {
      if (e.target.dataset.dismiss) return;
      const reminder = reminders.find(r => r.id === card.dataset.id);
      openReminderForm(reminder);
    });
  });

  list.querySelectorAll('[data-dismiss]').forEach(btn => {
    btn.addEventListener('click', async (e) => {
      e.stopPropagation();
      await api.dismissReminder(btn.dataset.dismiss);
      showToast('Lembrete dispensado', 'success');
      await loadReminders();
    });
  });
}

function openReminderForm(reminder) {
  const container = document.getElementById('view-reminders');

  container.innerHTML = `
    <div class="header">
      <button class="btn btn-secondary" id="btn-back-reminders">Voltar</button>
      <div class="header-actions">
        ${reminder ? `<button class="btn btn-danger" id="btn-delete-reminder">Excluir</button>` : ''}
        <button class="btn btn-primary" id="btn-save-reminder">Salvar</button>
      </div>
    </div>
    <div class="form-group">
      <label>Titulo</label>
      <input type="text" id="reminder-title" value="${reminder ? escapeHtml(reminder.title) : ''}" placeholder="Titulo do lembrete" autofocus>
    </div>
    <div class="form-group">
      <label>Descricao</label>
      <input type="text" id="reminder-desc" value="${reminder?.description ? escapeHtml(reminder.description) : ''}" placeholder="Detalhes opcionais">
    </div>
    <div class="form-group">
      <label>Tipo</label>
      <div class="radio-group">
        <label><input type="radio" name="reminder-type" value="datetime" checked> Data/hora exata</label>
        <label><input type="radio" name="reminder-type" value="recurrence" ${reminder?.repeat ? 'checked' : ''}> Recorrencia</label>
        <label><input type="radio" name="reminder-type" value="timer" ${reminder?.relative_minutes ? 'checked' : ''}> Timer relativo</label>
      </div>
    </div>
    <div id="reminder-datetime-fields" class="form-group">
      <label>Data e hora</label>
      <input type="datetime-local" id="reminder-datetime">
    </div>
    <div id="reminder-recurrence-fields" class="form-group" style="display:none">
      <label>Repetir</label>
      <select id="reminder-repeat">
        <option value="daily">Diario</option>
        <option value="weekly">Semanal</option>
        <option value="monthly">Mensal</option>
      </select>
    </div>
    <div id="reminder-timer-fields" class="form-group" style="display:none">
      <label>Em quantos minutos?</label>
      <input type="number" id="reminder-minutes" min="1" value="30">
    </div>
  `;

  if (reminder?.trigger_at) {
    const d = new Date(reminder.trigger_at);
    const local = new Date(d.getTime() - d.getTimezoneOffset() * 60000);
    document.getElementById('reminder-datetime').value = local.toISOString().slice(0, 16);
  } else {
    const now = new Date();
    now.setHours(now.getHours() + 1, 0, 0, 0);
    const local = new Date(now.getTime() - now.getTimezoneOffset() * 60000);
    document.getElementById('reminder-datetime').value = local.toISOString().slice(0, 16);
  }

  if (reminder?.repeat) {
    document.getElementById('reminder-repeat').value = reminder.repeat;
  }

  document.querySelectorAll('input[name="reminder-type"]').forEach(radio => {
    radio.addEventListener('change', () => {
      document.getElementById('reminder-datetime-fields').style.display = radio.value === 'datetime' || radio.value === 'recurrence' ? 'block' : 'none';
      document.getElementById('reminder-recurrence-fields').style.display = radio.value === 'recurrence' ? 'block' : 'none';
      document.getElementById('reminder-timer-fields').style.display = radio.value === 'timer' ? 'block' : 'none';
    });
  });

  const checkedType = document.querySelector('input[name="reminder-type"]:checked').value;
  document.getElementById('reminder-datetime-fields').style.display = checkedType === 'datetime' || checkedType === 'recurrence' ? 'block' : 'none';
  document.getElementById('reminder-recurrence-fields').style.display = checkedType === 'recurrence' ? 'block' : 'none';
  document.getElementById('reminder-timer-fields').style.display = checkedType === 'timer' ? 'block' : 'none';

  document.getElementById('btn-back-reminders').addEventListener('click', renderRemindersList);
  document.getElementById('btn-save-reminder').addEventListener('click', saveReminder);

  if (reminder) {
    document.getElementById('btn-delete-reminder').addEventListener('click', async () => {
      await api.deleteReminder(reminder.id);
      showToast('Lembrete excluido', 'success');
      renderRemindersList();
    });
  }

  document.getElementById('reminder-title').focus();
}

async function saveReminder() {
  const title = document.getElementById('reminder-title').value.trim();
  if (!title) {
    showToast('Titulo e obrigatorio', 'warning');
    return;
  }

  const description = document.getElementById('reminder-desc').value.trim() || null;
  const type = document.querySelector('input[name="reminder-type"]:checked').value;

  const input = { title, description };

  if (type === 'datetime') {
    const dt = document.getElementById('reminder-datetime').value;
    if (!dt) { showToast('Selecione data e hora', 'warning'); return; }
    input.trigger_at = new Date(dt).toISOString();
  } else if (type === 'recurrence') {
    const dt = document.getElementById('reminder-datetime').value;
    if (!dt) { showToast('Selecione data e hora', 'warning'); return; }
    input.trigger_at = new Date(dt).toISOString();
    input.repeat = document.getElementById('reminder-repeat').value;
  } else {
    const minutes = parseInt(document.getElementById('reminder-minutes').value);
    if (!minutes || minutes < 1) { showToast('Informe os minutos', 'warning'); return; }
    input.relative_minutes = minutes;
  }

  await api.createReminder(input);
  showToast('Lembrete criado', 'success');
  renderRemindersList();
}
