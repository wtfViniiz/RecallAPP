import { api } from './api.js';
import { formatDateTime, formatRelativeDate, escapeHtml, showToast, showConfirm } from './utils.js';

let currentReminder = null;

export async function initReminders() {
  await renderRemindersList();
}

async function renderRemindersList() {
  currentReminder = null;
  const container = document.getElementById('view-reminders');

  container.innerHTML = `
    <div class="toolbar">
      <h3>Lembretes</h3>
      <div>
        <button class="btn btn-secondary" id="btn-calendar-view" title="Calendario">&#128197;</button>
        <button class="btn btn-primary" id="btn-new-reminder">+ Novo</button>
      </div>
    </div>
    <div id="reminders-list"></div>
  `;

  document.getElementById('btn-calendar-view').addEventListener('click', renderCalendarView);

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
        ${r.status === 'fired' ? `
          <div class="card-actions">
            <button class="btn btn-secondary btn-sm" data-snooze="${r.id}" data-minutes="5">+5min</button>
            <button class="btn btn-secondary btn-sm" data-snooze="${r.id}" data-minutes="15">+15min</button>
            <button class="btn btn-secondary btn-sm" data-snooze="${r.id}" data-minutes="60">+1h</button>
            <button class="btn btn-secondary btn-sm" data-dismiss="${r.id}">Dispensar</button>
          </div>
        ` : ''}
      </div>
    `;
  }).join('');

  list.querySelectorAll('.card').forEach(card => {
    card.addEventListener('click', (e) => {
      if (e.target.dataset.dismiss || e.target.dataset.snooze) return;
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

  list.querySelectorAll('[data-snooze]').forEach(btn => {
    btn.addEventListener('click', async (e) => {
      e.stopPropagation();
      const id = btn.dataset.snooze;
      const minutes = parseInt(btn.dataset.minutes);
      await api.snoozeReminder(id, minutes);
      showToast(`Lembrete adiado ${minutes} minutos`, 'success');
      await loadReminders();
    });
  });
}

async function openReminderForm(reminder) {
  currentReminder = reminder;
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
      <label>Vincular nota (opcional)</label>
      <select id="reminder-note">
        <option value="">Nenhuma</option>
      </select>
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

  // Populate note selector
  try {
    const notes = await api.getNotes(null);
    const noteSelect = document.getElementById('reminder-note');
    notes.forEach(note => {
      const option = document.createElement('option');
      option.value = note.id;
      option.textContent = note.title || 'Sem titulo';
      if (reminder?.note_id === note.id) option.selected = true;
      noteSelect.appendChild(option);
    });
  } catch (e) {}

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
    document.getElementById('btn-delete-reminder').addEventListener('click', () => {
      showConfirm('Excluir este lembrete?', async () => {
        await api.deleteReminder(reminder.id);
        showToast('Lembrete excluido', 'success');
        renderRemindersList();
      });
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

  // If editing, update all fields
  if (currentReminder) {
    const updateData = { id: currentReminder.id, title, description };

    const type = document.querySelector('input[name="reminder-type"]:checked')?.value;
    if (type === 'datetime') {
      const dt = document.getElementById('reminder-datetime').value;
      if (dt) updateData.trigger_at = new Date(dt).toISOString();
    } else if (type === 'recurrence') {
      const dt = document.getElementById('reminder-datetime').value;
      if (dt) {
        updateData.trigger_at = new Date(dt).toISOString();
        updateData.repeat = document.getElementById('reminder-repeat').value;
      }
    } else {
      const minutes = parseInt(document.getElementById('reminder-minutes').value);
      if (minutes && minutes >= 1) updateData.relative_minutes = minutes;
    }

    try {
      await api.updateReminder(updateData);
      showToast('Lembrete atualizado', 'success');
      renderRemindersList();
    } catch (e) {
      showToast('Erro ao atualizar', 'error');
    }
    return;
  }

  // Creating new reminder
  const type = document.querySelector('input[name="reminder-type"]:checked').value;
  const noteId = document.getElementById('reminder-note').value || null;
  const input = { title, description, note_id: noteId };

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

  try {
    await api.createReminder(input);
    showToast('Lembrete criado', 'success');
  } catch (e) {
    showToast('Erro ao criar lembrete', 'error');
  }
  renderRemindersList();
}

async function renderCalendarView() {
  const container = document.getElementById('view-reminders');
  const allReminders = await api.getReminders();
  const reminders = allReminders.filter(r => r.status === 'pending');

  const now = new Date();
  const year = now.getFullYear();
  const month = now.getMonth();
  const firstDay = new Date(year, month, 1);
  const lastDay = new Date(year, month + 1, 0);
  const startDay = firstDay.getDay();
  const daysInMonth = lastDay.getDate();

  const monthNames = ['Janeiro', 'Fevereiro', 'Marco', 'Abril', 'Maio', 'Junho',
    'Julho', 'Agosto', 'Setembro', 'Outubro', 'Novembro', 'Dezembro'];
  const dayNames = ['Dom', 'Seg', 'Ter', 'Qua', 'Qui', 'Sex', 'Sab'];

  // Get reminders for this month
  const monthReminders = reminders.filter(r => {
    const d = new Date(r.trigger_at);
    return d.getMonth() === month && d.getFullYear() === year;
  });

  // Group by day
  const remindersByDay = {};
  monthReminders.forEach(r => {
    const day = new Date(r.trigger_at).getDate();
    if (!remindersByDay[day]) remindersByDay[day] = [];
    remindersByDay[day].push(r);
  });

  let calendarHtml = `
    <div class="header">
      <button class="btn btn-secondary" id="btn-back-calendar">Voltar</button>
      <h3>${monthNames[month]} ${year}</h3>
      <div></div>
    </div>
    <div class="calendar">
      <div class="calendar-header">
        ${dayNames.map(d => `<div class="calendar-day-name">${d}</div>`).join('')}
      </div>
      <div class="calendar-body">
  `;

  // Empty cells before first day
  for (let i = 0; i < startDay; i++) {
    calendarHtml += '<div class="calendar-day empty"></div>';
  }

  // Days of the month
  for (let day = 1; day <= daysInMonth; day++) {
    const isToday = day === now.getDate();
    const hasReminders = remindersByDay[day]?.length > 0;
    const dots = hasReminders ? remindersByDay[day].map(() => '<span class="reminder-dot"></span>').join('') : '';

    calendarHtml += `
      <div class="calendar-day ${isToday ? 'today' : ''} ${hasReminders ? 'has-reminders' : ''}" data-day="${day}">
        <span class="day-number">${day}</span>
        <div class="reminder-dots">${dots}</div>
      </div>
    `;
  }

  calendarHtml += '</div></div>';

  container.innerHTML = calendarHtml;

  document.getElementById('btn-back-calendar').addEventListener('click', renderRemindersList);

  // Click on day to show reminders
  container.querySelectorAll('.calendar-day:not(.empty)').forEach(dayEl => {
    dayEl.addEventListener('click', () => {
      const day = parseInt(dayEl.dataset.day);
      const dayReminders = remindersByDay[day] || [];
      if (dayReminders.length > 0) {
        const titles = dayReminders.map(r => r.title).join(', ');
        showToast(`${day}/${month + 1}: ${titles}`, 'info', 5000);
      }
    });
  });
}
