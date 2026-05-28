import { api } from './api.js';
import { formatDate, formatDateTime, escapeHtml, showToast, DEFAULT_CATEGORIES, DEFAULT_TAGS } from './utils.js';

let currentView = 'list';
let currentNote = null;
let allCategories = [...DEFAULT_CATEGORIES];
let allTags = [...DEFAULT_TAGS];
let saveTimeout = null;

export async function initNotes() {
  await renderNotesList();
}

async function renderNotesList() {
  const container = document.getElementById('view-notes');

  container.innerHTML = `
    <div class="search-bar">
      <input type="text" id="note-search" placeholder="Buscar notas...">
      <button class="btn btn-primary" id="btn-new-note" title="Nova nota (Ctrl+N)">+</button>
    </div>
    <div class="filter-row">
      <select id="filter-category"><option value="">Todas categorias</option></select>
      <select id="filter-tag"><option value="">Todas tags</option></select>
    </div>
    <div id="notes-list"></div>
  `;

  // Load categories and tags from API
  try {
    const [apiCats, apiTags] = await Promise.all([api.getCategories(), api.getTags()]);
    apiCats.forEach(c => { if (!allCategories.includes(c)) allCategories.push(c); });
    apiTags.forEach(t => { if (!allTags.includes(t)) allTags.push(t); });
  } catch (e) {}

  const catSelect = document.getElementById('filter-category');
  allCategories.forEach(c => {
    catSelect.innerHTML += `<option value="${escapeHtml(c)}">${escapeHtml(c)}</option>`;
  });

  const tagSelect = document.getElementById('filter-tag');
  allTags.forEach(t => {
    tagSelect.innerHTML += `<option value="${escapeHtml(t)}">${escapeHtml(t)}</option>`;
  });

  document.getElementById('note-search').addEventListener('input', debounce(loadNotes, 200));
  document.getElementById('filter-category').addEventListener('change', loadNotes);
  document.getElementById('filter-tag').addEventListener('change', loadNotes);
  document.getElementById('btn-new-note').addEventListener('click', () => openEditor(null));

  // Ctrl+N shortcut
  document.addEventListener('keydown', (e) => {
    if (e.ctrlKey && e.key === 'n' && currentView === 'list') {
      e.preventDefault();
      openEditor(null);
    }
  });

  await loadNotes();
}

async function loadNotes() {
  const search = document.getElementById('note-search')?.value || '';
  const category = document.getElementById('filter-category')?.value || '';
  const tag = document.getElementById('filter-tag')?.value || '';

  const filter = {};
  if (search) filter.search = search;
  if (category) filter.category = category;
  if (tag) filter.tag = tag;

  const notes = await api.getNotes(Object.keys(filter).length ? filter : null);
  const list = document.getElementById('notes-list');

  if (notes.length === 0) {
    list.innerHTML = '<div class="empty">Nenhuma nota encontrada</div>';
    return;
  }

  list.innerHTML = notes.map(note => `
    <div class="card ${note.pinned ? 'pinned' : ''}" data-id="${note.id}">
      <div class="card-title">${escapeHtml(note.title || 'Sem titulo')}</div>
      <div class="card-meta">
        ${note.category ? `<span>${escapeHtml(note.category)}</span>` : ''}
        ${note.tags.map(t => `<span class="tag">#${escapeHtml(t)}</span>`).join('')}
        <span>${formatDate(note.updated_at)}</span>
      </div>
    </div>
  `).join('');

  list.querySelectorAll('.card').forEach(card => {
    card.addEventListener('click', () => {
      const note = notes.find(n => n.id === card.dataset.id);
      openEditor(note);
    });
  });
}

function openEditor(note) {
  currentView = 'editor';
  currentNote = note;
  const container = document.getElementById('view-notes');

  const selectedCategory = note?.category || '';
  const selectedTags = note?.tags || [];

  container.innerHTML = `
    <div class="header">
      <button class="btn btn-secondary" id="btn-back">Voltar</button>
      <div class="header-actions">
        ${note ? `<button class="btn btn-secondary" id="btn-pin-note" title="Fixar">${note.pinned ? 'Desfixar' : 'Fixar'}</button>` : ''}
        ${note ? `<button class="btn btn-danger" id="btn-delete-note">Excluir</button>` : ''}
      </div>
    </div>
    <div class="quick-note">
      <input type="text" class="note-title" id="note-title" value="${note ? escapeHtml(note.title) : ''}" placeholder="Titulo..." autofocus>
      <div class="form-group" style="margin-top: 8px;">
        <label>Categoria</label>
        <div class="chip-input" id="category-chips">
          ${selectedCategory ? `<span class="chip" data-value="${escapeHtml(selectedCategory)}">${escapeHtml(selectedCategory)}<span class="remove">&times;</span></span>` : ''}
          <input type="text" id="note-category-input" placeholder="${selectedCategory ? '' : 'Selecionar...'}" list="categories-list">
          <datalist id="categories-list">
            ${allCategories.map(c => `<option value="${escapeHtml(c)}">`).join('')}
          </datalist>
        </div>
      </div>
      <div class="form-group">
        <label>Tags</label>
        <div class="chip-input" id="tag-chips">
          ${selectedTags.map(t => `<span class="chip" data-value="${escapeHtml(t)}">#${escapeHtml(t)}<span class="remove">&times;</span></span>`).join('')}
          <input type="text" id="note-tags-input" placeholder="Adicionar tag..." list="tags-list">
          <datalist id="tags-list">
            ${allTags.map(t => `<option value="${escapeHtml(t)}">`).join('')}
          </datalist>
        </div>
      </div>
      <textarea id="note-content" placeholder="Comece a escrever...">${note ? escapeHtml(note.content) : ''}</textarea>
    </div>
    ${note ? `
      <div class="timestamp">Criada: ${formatDateTime(note.created_at)}</div>
      <div class="timestamp">Editada: ${formatDateTime(note.updated_at)}</div>
    ` : ''}
  `;

  // Focus on title or content
  const titleInput = document.getElementById('note-title');
  const contentInput = document.getElementById('note-content');

  if (!note) {
    titleInput.focus();
  } else if (!note.title) {
    titleInput.focus();
  } else {
    contentInput.focus();
  }

  // Auto-save on change (debounced)
  const autoSave = () => {
    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(() => saveNote(true), 1000);
  };

  titleInput.addEventListener('input', autoSave);
  contentInput.addEventListener('input', autoSave);

  // Back button
  document.getElementById('btn-back').addEventListener('click', () => {
    clearTimeout(saveTimeout);
    saveNote(true).then(() => {
      currentView = 'list';
      renderNotesList();
    });
  });

  // Pin button
  if (note) {
    document.getElementById('btn-pin-note').addEventListener('click', async () => {
      await api.updateNote({ id: note.id, pinned: !note.pinned });
      showToast(note.pinned ? 'Nota desfixada' : 'Nota fixada', 'success');
      openEditor({ ...note, pinned: !note.pinned });
    });

    document.getElementById('btn-delete-note').addEventListener('click', async () => {
      await api.deleteNote(note.id);
      showToast('Nota excluida', 'success');
      currentView = 'list';
      renderNotesList();
    });
  }

  // Category chip input
  setupChipInput('category-chips', 'note-category-input', allCategories, false);

  // Tag chip input
  setupChipInput('tag-chips', 'note-tags-input', allTags, true);

  // Ctrl+S to save
  document.addEventListener('keydown', (e) => {
    if (e.ctrlKey && e.key === 's' && currentView === 'editor') {
      e.preventDefault();
      saveNote(false);
    }
  });
}

function setupChipInput(containerId, inputId, suggestions, allowMultiple) {
  const container = document.getElementById(containerId);
  const input = document.getElementById(inputId);

  input.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' || e.key === ',') {
      e.preventDefault();
      const value = input.value.trim().replace(/,$/, '');
      if (value) {
        addChip(container, input, value, allowMultiple);
        input.value = '';
      }
    }
    if (e.key === 'Backspace' && !input.value) {
      const chips = container.querySelectorAll('.chip');
      if (chips.length > 0) {
        chips[chips.length - 1].remove();
      }
    }
  });

  input.addEventListener('change', () => {
    const value = input.value.trim();
    if (value) {
      addChip(container, input, value, allowMultiple);
      input.value = '';
    }
  });
}

function addChip(container, input, value, allowMultiple) {
  // Check if already exists
  const existing = container.querySelectorAll('.chip');
  if (!allowMultiple && existing.length > 0) {
    existing[0].remove();
  } else {
    for (const chip of existing) {
      if (chip.dataset.value === value) return;
    }
  }

  const chip = document.createElement('span');
  chip.className = 'chip';
  chip.dataset.value = value;
  chip.innerHTML = `${allowMultiple ? '#' : ''}${escapeHtml(value)}<span class="remove">&times;</span>`;
  chip.querySelector('.remove').addEventListener('click', () => chip.remove());
  container.insertBefore(chip, input);
  input.placeholder = '';
}

function getChips(containerId) {
  const container = document.getElementById(containerId);
  return Array.from(container.querySelectorAll('.chip')).map(c => c.dataset.value);
}

async function saveNote(silent = false) {
  const title = document.getElementById('note-title')?.value.trim();
  const content = document.getElementById('note-content')?.value;
  const category = getChips('category-chips')[0] || null;
  const tags = getChips('tag-chips');

  if (!title && !content) return;

  try {
    if (currentNote) {
      await api.updateNote({
        id: currentNote.id,
        title: title || 'Sem titulo',
        content: content || '',
        category,
        tags,
      });
    } else {
      const note = await api.createNote({
        title: title || 'Sem titulo',
        content: content || '',
        category,
        tags,
      });
      currentNote = note;
    }

    if (!silent) {
      showToast('Nota salva', 'success');
    }
  } catch (e) {
    showToast('Erro ao salvar: ' + e, 'error');
  }
}

function debounce(fn, ms) {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), ms);
  };
}
