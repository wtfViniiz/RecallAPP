import { api } from './api.js';
import { formatDate, escapeHtml, showToast, showConfirm, DEFAULT_CATEGORIES, DEFAULT_TAGS, NOTE_TEMPLATES, debounce } from './utils.js';
import { icons } from './icons.js';
import { openEditor, flushPendingSave, saveNote } from './editor.js';

let currentView = 'list';
let draggedCard = null;
let allLoadedNotes = [];
let allCategories = [...DEFAULT_CATEGORIES];
let allTags = [...DEFAULT_TAGS];
let keyListenersAttached = false;
let notesOffset = 0;
const NOTES_PAGE_SIZE = 30;

export { initNotes, flushPendingSave, openEditorDirect, renderNotesList, getCurrentView, setCurrentView, getAllCategories, getAllTags };

function getCurrentView() { return currentView; }
function setCurrentView(view) { currentView = view; }
function getAllCategories() { return allCategories; }
function getAllTags() { return allTags; }

async function initNotes() {
  console.log('[Notes] initNotes called');
  setupGlobalKeyListeners();
  await renderNotesList();
  console.log('[Notes] renderNotesList done');
}

function openEditorDirect() {
  openEditor(null);
}

function setupGlobalKeyListeners() {
  if (keyListenersAttached) return;
  keyListenersAttached = true;

  document.addEventListener('keydown', (e) => {
    if (e.ctrlKey && e.key === 's' && currentView === 'editor') {
      e.preventDefault();
      saveNote(false);
    }
  });
}

async function renderNotesList() {
  currentView = 'list';
  const container = document.getElementById('view-notes');

  container.innerHTML = `
    <div class="search-bar">
      <input type="text" id="note-search" placeholder="Buscar notas...">
      <button class="btn btn-primary" id="btn-new-note" title="Nova nota (Ctrl+N)">+</button>
    </div>
    <div class="filter-row">
      <select id="filter-category"><option value="">Todas categorias</option></select>
      <select id="filter-tag"><option value="">Todas tags</option></select>
      <button class="btn btn-secondary btn-sm" id="btn-recent" title="Recentes">${icons.clock(14)} Recent</button>
      <button class="btn btn-secondary btn-sm" id="btn-trash" title="Lixeira">${icons.trash(14)} Lixeira</button>
    </div>
    <div id="notes-list"></div>
  `;

  document.getElementById('btn-trash').addEventListener('click', renderTrashList);

  try {
    const result = await api.getCategoriesAndTags();
    result.categories.forEach(c => { if (!allCategories.includes(c)) allCategories.push(c); });
    result.tags.forEach(t => { if (!allTags.includes(t)) allTags.push(t); });
  } catch (e) { console.warn('[Recall] Failed to load categories/tags:', e); }

  const catSelect = document.getElementById('filter-category');
  let catOptionsHtml = '';
  allCategories.forEach(c => {
    catOptionsHtml += `<option value="${escapeHtml(c)}">${escapeHtml(c)}</option>`;
  });
  catSelect.innerHTML += catOptionsHtml;

  const tagSelect = document.getElementById('filter-tag');
  let tagOptionsHtml = '';
  allTags.forEach(t => {
    tagOptionsHtml += `<option value="${escapeHtml(t)}">${escapeHtml(t)}</option>`;
  });
  tagSelect.innerHTML += tagOptionsHtml;

  document.getElementById('note-search').addEventListener('input', debounce(() => loadNotes(), 200));
  document.getElementById('filter-category').addEventListener('change', () => loadNotes());
  document.getElementById('filter-tag').addEventListener('change', () => loadNotes());
  document.getElementById('btn-new-note').addEventListener('click', () => showTemplateSelector());
  document.getElementById('btn-recent').addEventListener('click', () => loadRecentNotes());

  await loadNotes();
}

function renderNoteCards(notes, searchQuery) {
  return notes.map(note => `
    <div class="card ${note.pinned ? 'pinned' : ''}" data-id="${escapeHtml(note.id)}" draggable="true">
      <div class="card-header">
        <div class="card-title">${highlightMatch(note.title || 'Sem titulo', searchQuery)}</div>
        <button class="btn-icon delete-note-btn" data-id="${escapeHtml(note.id)}" title="Excluir">${icons.trash(14)}</button>
      </div>
      <div class="card-meta">
        ${note.category ? `<span>${escapeHtml(note.category)}</span>` : ''}
        ${note.tags.map(t => `<span class="tag">#${escapeHtml(t)}</span>`).join('')}
        <span>${formatDate(note.updated_at)}</span>
      </div>
    </div>
  `).join('');
}

function attachNoteCardHandlers(list, notes, append = false) {
  const cards = append
    ? list.querySelectorAll('.card:not([data-handler])')
    : list.querySelectorAll('.card');
  cards.forEach(card => {
    card.dataset.handler = 'true';
    card.addEventListener('click', (e) => {
      if (e.target.closest('.delete-note-btn')) return;
      if (card.classList.contains('drag-over')) return;
      const note = notes.find(n => n.id === card.dataset.id);
      openEditor(note);
    });

    card.addEventListener('dragstart', (e) => {
      draggedCard = card;
      card.classList.add('dragging');
      e.dataTransfer.effectAllowed = 'move';
      e.dataTransfer.setData('text/plain', card.dataset.id);
    });

    card.addEventListener('dragover', (e) => {
      e.preventDefault();
      e.dataTransfer.dropEffect = 'move';
      if (card !== draggedCard) {
        card.classList.add('drag-over');
      }
    });

    card.addEventListener('dragleave', () => {
      card.classList.remove('drag-over');
    });

    card.addEventListener('drop', async (e) => {
      e.preventDefault();
      card.classList.remove('drag-over');
      if (!draggedCard || draggedCard === card) return;

      const allCards = Array.from(list.querySelectorAll('.card'));
      const targetIndex = allCards.indexOf(card);
      const draggedIndex = allCards.indexOf(draggedCard);

      if (draggedIndex < targetIndex) {
        card.after(draggedCard);
      } else {
        card.before(draggedCard);
      }

      const updatedCards = Array.from(list.querySelectorAll('.card'));
      const results = await Promise.allSettled(
        updatedCards.map((card, i) => api.updateNote({ id: card.dataset.id, position: i }))
      );
      const failures = results.filter(r => r.status === 'rejected');
      if (failures.length === 0) {
        showToast('Nota reposicionada', 'success');
      } else {
        showToast(`${failures.length} notas nao atualizadas`, 'warning');
      }
    });

    card.addEventListener('dragend', () => {
      card.classList.remove('dragging');
      list.querySelectorAll('.card').forEach(c => c.classList.remove('drag-over'));
      draggedCard = null;
    });
  });

  list.querySelectorAll('.delete-note-btn').forEach(btn => {
    btn.addEventListener('click', async (e) => {
      e.stopPropagation();
      try {
        await api.trashNote(btn.dataset.id);
        showToast('Nota movida para lixeira', 'success');
        await loadNotes();
      } catch (err) {
        showToast(err.message || 'Erro ao mover para lixeira', 'error');
      }
    });
  });
}

async function loadNotes(append = false) {
  const search = document.getElementById('note-search')?.value || '';
  const category = document.getElementById('filter-category')?.value || '';
  const tag = document.getElementById('filter-tag')?.value || '';

  const filter = { limit: NOTES_PAGE_SIZE };
  if (search) filter.search = search;
  if (category) filter.category = category;
  if (tag) filter.tag = tag;

  if (!append) {
    notesOffset = 0;
  }
  filter.offset = notesOffset;

  let result;
  try {
    result = await api.getNotes(filter);
  } catch (err) {
    console.error('Erro ao carregar notas:', err);
    showToast(err.message || 'Erro ao carregar notas', 'error');
    return;
  }
  const notes = result.items;
  const total = result.total;
  const list = document.getElementById('notes-list');

  if (!append && notes.length === 0) {
    list.innerHTML = `<div class="empty">${icons['file-text'](32)}<p>Nenhuma nota encontrada</p></div>`;
    allLoadedNotes = [];
    return;
  }

  const cardsHtml = renderNoteCards(notes, search);

  if (append) {
    const existingBtn = list.querySelector('.load-more-container');
    if (existingBtn) existingBtn.remove();
    list.insertAdjacentHTML('beforeend', cardsHtml);
    allLoadedNotes = allLoadedNotes.concat(notes);
  } else {
    list.innerHTML = cardsHtml;
    allLoadedNotes = notes;
  }

  notesOffset += notes.length;
  attachNoteCardHandlers(list, allLoadedNotes, append);

  if (notesOffset < total) {
    const loadMoreHtml = '<div class="load-more-container"><button class="btn btn-secondary" id="btn-load-more">Carregar mais</button></div>';
    list.insertAdjacentHTML('beforeend', loadMoreHtml);
    document.getElementById('btn-load-more').addEventListener('click', () => loadNotes(true));
  }
}

async function loadRecentNotes() {
  let result;
  try {
    result = await api.getNotes({ limit: 10 });
  } catch (err) {
    showToast(err.message || 'Erro ao carregar notas recentes', 'error');
    return;
  }
  const recent = result.items;
  const list = document.getElementById('notes-list');

  if (recent.length === 0) {
    list.innerHTML = `<div class="empty">${icons.clock(32)}<p>Nenhuma nota recente</p></div>`;
    return;
  }

  list.innerHTML = renderNoteCards(recent, '');
  attachNoteCardHandlers(list, recent);
}

async function renderTrashList() {
  currentView = 'trash';
  const container = document.getElementById('view-notes');

  container.innerHTML = `
    <div class="header">
      <button class="btn btn-secondary" id="btn-back-notes">Voltar</button>
      <div class="header-actions">
        <button class="btn btn-danger" id="btn-empty-trash">Esvaziar lixeira</button>
      </div>
    </div>
    <div id="trash-list"></div>
  `;

  document.getElementById('btn-back-notes').addEventListener('click', renderNotesList);
  document.getElementById('btn-empty-trash').addEventListener('click', () => {
    showConfirm('Excluir permanentemente todas as notas da lixeira?', async () => {
      try {
        const count = await api.emptyTrash();
        showToast(`${count} notas excluidas permanentemente`, 'success');
      } catch (err) {
        showToast(err.message || 'Erro ao esvaziar lixeira', 'error');
      }
      renderTrashList();
    });
  });

  const notes = await api.getTrashedNotes();
  const list = document.getElementById('trash-list');

  if (notes.length === 0) {
    list.innerHTML = `<div class="empty">${icons.trash(32)}<p>Lixeira vazia</p></div>`;
    return;
  }

  list.innerHTML = notes.map(note => `
    <div class="card" data-id="${escapeHtml(note.id)}">
      <div class="card-header">
        <div class="card-title">${escapeHtml(note.title || 'Sem titulo')}</div>
        <div class="header-actions">
          <button class="btn btn-secondary btn-sm" data-restore="${escapeHtml(note.id)}">Restaurar</button>
          <button class="btn btn-danger btn-sm" data-delete="${escapeHtml(note.id)}">Excluir</button>
        </div>
      </div>
      <div class="card-meta">
        <span>${formatDate(note.updated_at)}</span>
      </div>
    </div>
  `).join('');

  list.querySelectorAll('[data-restore]').forEach(btn => {
    btn.addEventListener('click', async (e) => {
      e.stopPropagation();
      try {
        await api.restoreNote(btn.dataset.restore);
        showToast('Nota restaurada', 'success');
        renderTrashList();
      } catch (err) {
        showToast(err.message || 'Erro ao restaurar nota', 'error');
      }
    });
  });

  list.querySelectorAll('[data-delete]').forEach(btn => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      showConfirm('Excluir permanentemente?', async () => {
        try {
          await api.deleteNote(btn.dataset.delete);
          showToast('Nota excluida permanentemente', 'success');
        } catch (err) {
          showToast(err.message || 'Erro ao excluir', 'error');
        }
        renderTrashList();
      });
    });
  });
}

function highlightMatch(text, query) {
  if (!query) return escapeHtml(text);
  const escapedQuery = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const regex = new RegExp(`(${escapedQuery})`, 'gi');
  const parts = text.split(regex);
  return parts.map((part, i) => {
    if (i % 2 === 1) {
      return `<mark>${escapeHtml(part)}</mark>`;
    }
    return escapeHtml(part);
  }).join('');
}

async function showTemplateSelector() {
  const container = document.getElementById('view-notes');
  currentView = 'templates';

  let customTemplates = [];
  try {
    customTemplates = await api.getCustomTemplates();
  } catch (e) { console.warn('[Recall] Failed to load custom templates:', e); }

  container.innerHTML = `
    <div class="header">
      <button class="btn btn-secondary" id="btn-back-templates">Voltar</button>
      <h3>Nova nota</h3>
      <button class="btn btn-primary btn-sm" id="btn-new-template" title="Criar template">+ Template</button>
    </div>
    <div class="template-grid">
      <div class="template-card" data-template="blank">
        <div class="template-icon">${icons['file-plus'](24)}</div>
        <div class="template-name">Em branco</div>
      </div>
      ${Object.entries(NOTE_TEMPLATES).map(([key, tpl]) => `
        <div class="template-card" data-template="${key}">
          <div class="template-icon">${key === 'reuniao' ? icons.users(24) : key === 'tarefa' ? icons['check-square'](24) : key === 'diario' ? icons['book-open'](24) : icons['graduation-cap'](24)}</div>
          <div class="template-name">${tpl.name}</div>
        </div>
      `).join('')}
      ${customTemplates.map(tpl => `
        <div class="template-card" data-template="custom:${tpl.id}">
          <div class="template-icon">${icons['file-text'](24)}</div>
          <div class="template-name">${escapeHtml(tpl.name)}</div>
          <button class="btn-icon delete-template-btn" data-id="${escapeHtml(tpl.id)}" title="Excluir template" style="position:absolute;top:4px;right:4px;font-size:12px;">${icons.x(12)}</button>
        </div>
      `).join('')}
    </div>
  `;

  // Make custom template cards position relative for the delete button
  container.querySelectorAll('.template-card[data-template^="custom:"]').forEach(card => {
    card.style.position = 'relative';
  });

  document.getElementById('btn-back-templates').addEventListener('click', renderNotesList);
  document.getElementById('btn-new-template').addEventListener('click', () => openTemplateEditor(null));

  container.querySelectorAll('.template-card').forEach(card => {
    card.addEventListener('click', (e) => {
      if (e.target.closest('.delete-template-btn')) return;
      const templateKey = card.dataset.template;
      if (templateKey === 'blank') {
        openEditor(null);
      } else if (templateKey.startsWith('custom:')) {
        const id = templateKey.replace('custom:', '');
        const tpl = customTemplates.find(t => t.id === id);
        if (tpl) {
          openEditor({ title: tpl.title, content: tpl.content, tags: [], category: null });
        }
      } else {
        const tpl = NOTE_TEMPLATES[templateKey];
        openEditor({ title: tpl.title, content: tpl.content, tags: [], category: null });
      }
    });
  });

  container.querySelectorAll('.delete-template-btn').forEach(btn => {
    btn.addEventListener('click', async (e) => {
      e.stopPropagation();
      showConfirm('Excluir este template?', async () => {
        try {
          await api.deleteCustomTemplate(btn.dataset.id);
          showToast('Template excluido', 'success');
          showTemplateSelector();
        } catch (err) {
          showToast(err.message || 'Erro ao excluir template', 'error');
        }
      });
    });
  });
}

function openTemplateEditor(tpl) {
  const container = document.getElementById('view-notes');
  currentView = 'template-editor';

  container.innerHTML = `
    <div class="header">
      <button class="btn btn-secondary" id="btn-back-te">Voltar</button>
      <h3>${tpl ? 'Editar' : 'Criar'} Template</h3>
    </div>
    <div class="form-group">
      <label>Nome</label>
      <input type="text" id="tpl-name" class="form-input" value="${tpl ? escapeHtml(tpl.name) : ''}" placeholder="Ex: Minha Reuniao">
    </div>
    <div class="form-group">
      <label>Icone (emoji ou texto)</label>
      <input type="text" id="tpl-icon" class="form-input" value="${tpl ? escapeHtml(tpl.icon || '') : ''}" placeholder="Opcional" maxlength="4">
    </div>
    <div class="form-group">
      <label>Titulo padrao</label>
      <input type="text" id="tpl-title" class="form-input" value="${tpl ? escapeHtml(tpl.title) : ''}" placeholder="Ex: Reuniao - ">
    </div>
    <div class="form-group">
      <label>Conteudo padrao</label>
      <textarea id="tpl-content" class="form-input" rows="6" placeholder="Conteudo do template...">${tpl ? escapeHtml(tpl.content) : ''}</textarea>
    </div>
    <div class="form-actions">
      <button class="btn btn-primary" id="btn-save-tpl">Salvar</button>
    </div>
  `;

  document.getElementById('btn-back-te').addEventListener('click', showTemplateSelector);
  document.getElementById('btn-save-tpl').addEventListener('click', async () => {
    const name = document.getElementById('tpl-name').value.trim();
    if (!name) { showToast('Nome obrigatorio', 'error'); return; }
    const input = {
      id: tpl?.id || crypto.randomUUID(),
      name,
      icon: document.getElementById('tpl-icon').value.trim() || null,
      title: document.getElementById('tpl-title').value.trim(),
      content: document.getElementById('tpl-content').value,
    };
    try {
      await api.saveCustomTemplate(input);
      showToast('Template salvo', 'success');
      showTemplateSelector();
    } catch (err) { showToast(err.message || 'Erro ao salvar', 'error'); }
  });
}
