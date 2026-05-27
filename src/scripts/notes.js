import { api } from './api.js';
import { formatDate, formatDateTime, escapeHtml } from './utils.js';

let currentView = 'list';
let currentNote = null;
let allCategories = [];
let allTags = [];

export async function initNotes() {
  await renderNotesList();
}

async function renderNotesList() {
  const container = document.getElementById('view-notes');

  container.innerHTML = `
    <div class="search-bar">
      <input type="text" id="note-search" placeholder="Buscar notas...">
      <button class="btn btn-primary" id="btn-new-note">+</button>
    </div>
    <div class="filter-row">
      <select id="filter-category"><option value="">Todas categorias</option></select>
      <select id="filter-tag"><option value="">Todas tags</option></select>
    </div>
    <div id="notes-list"></div>
  `;

  allCategories = await api.getCategories();
  allTags = await api.getTags();

  const catSelect = document.getElementById('filter-category');
  allCategories.forEach(c => {
    catSelect.innerHTML += `<option value="${escapeHtml(c)}">${escapeHtml(c)}</option>`;
  });

  const tagSelect = document.getElementById('filter-tag');
  allTags.forEach(t => {
    tagSelect.innerHTML += `<option value="${escapeHtml(t)}">${escapeHtml(t)}</option>`;
  });

  document.getElementById('note-search').addEventListener('input', debounce(loadNotes, 300));
  document.getElementById('filter-category').addEventListener('change', loadNotes);
  document.getElementById('filter-tag').addEventListener('change', loadNotes);
  document.getElementById('btn-new-note').addEventListener('click', () => openEditor(null));

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
      <div class="card-title">${escapeHtml(note.title)}</div>
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

  container.innerHTML = `
    <div class="header">
      <button class="btn btn-secondary" id="btn-back">Voltar</button>
      <div class="header-actions">
        ${note ? `<button class="btn btn-danger" id="btn-delete-note">Excluir</button>` : ''}
        <button class="btn btn-primary" id="btn-save-note">Salvar</button>
      </div>
    </div>
    <div class="form-group">
      <label>Titulo</label>
      <input type="text" id="note-title" value="${note ? escapeHtml(note.title) : ''}" placeholder="Titulo da nota">
    </div>
    <div class="form-group">
      <label>Categoria</label>
      <input type="text" id="note-category" value="${note?.category ? escapeHtml(note.category) : ''}" placeholder="Ex: Trabalho, Pessoal" list="categories-list">
      <datalist id="categories-list">
        ${allCategories.map(c => `<option value="${escapeHtml(c)}">`).join('')}
      </datalist>
    </div>
    <div class="form-group">
      <label>Tags (separadas por virgula)</label>
      <input type="text" id="note-tags" value="${note ? note.tags.join(', ') : ''}" placeholder="Ex: urgente, reuniao">
    </div>
    <div class="form-group">
      <label>Conteudo</label>
      <textarea id="note-content" rows="8" placeholder="Escreva aqui...">${note ? escapeHtml(note.content) : ''}</textarea>
    </div>
    ${note ? `
      <div class="timestamp">Criada: ${formatDateTime(note.created_at)}</div>
      <div class="timestamp">Editada: ${formatDateTime(note.updated_at)}</div>
    ` : ''}
  `;

  document.getElementById('btn-back').addEventListener('click', () => {
    currentView = 'list';
    renderNotesList();
  });

  document.getElementById('btn-save-note').addEventListener('click', saveNote);

  if (note) {
    document.getElementById('btn-delete-note').addEventListener('click', async () => {
      if (confirm('Excluir esta nota?')) {
        await api.deleteNote(note.id);
        currentView = 'list';
        renderNotesList();
      }
    });
  }

  document.getElementById('note-title').focus();
}

async function saveNote() {
  const title = document.getElementById('note-title').value.trim();
  if (!title) {
    alert('Titulo e obrigatorio');
    return;
  }

  const content = document.getElementById('note-content').value;
  const category = document.getElementById('note-category').value.trim() || null;
  const tagsStr = document.getElementById('note-tags').value;
  const tags = tagsStr ? tagsStr.split(',').map(t => t.trim()).filter(Boolean) : [];

  if (currentNote) {
    await api.updateNote({
      id: currentNote.id,
      title,
      content,
      category,
      tags,
    });
  } else {
    await api.createNote({ title, content, category, tags });
  }

  currentView = 'list';
  await renderNotesList();
}

function debounce(fn, ms) {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), ms);
  };
}
