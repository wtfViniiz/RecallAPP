import { api } from './api.js';
import { formatDate, formatDateTime, escapeHtml, showToast, showConfirm, renderMarkdown, DEFAULT_CATEGORIES, DEFAULT_TAGS, NOTE_TEMPLATES } from './utils.js';

let currentView = 'list';
let currentNote = null;
let allCategories = [...DEFAULT_CATEGORIES];
let allTags = [...DEFAULT_TAGS];
let saveTimeout = null;
let keyListenersAttached = false;

export async function initNotes() {
  setupGlobalKeyListeners();
  await renderNotesList();
}

export async function flushPendingSave() {
  if (saveTimeout) {
    clearTimeout(saveTimeout);
    saveTimeout = null;
    if (currentNote && currentView === 'editor') {
      await saveNote(true);
    }
  }
}

function setupGlobalKeyListeners() {
  if (keyListenersAttached) return;
  keyListenersAttached = true;

  document.addEventListener('keydown', (e) => {
    if (e.ctrlKey && e.key === 'n' && currentView === 'list') {
      e.preventDefault();
      openEditor(null);
    }
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
      <button class="btn btn-secondary btn-sm" id="btn-recent" title="Recentes">&#128336; Recent</button>
      <button class="btn btn-secondary btn-sm" id="btn-trash" title="Lixeira">&#128465; Lixeira</button>
    </div>
    <div id="notes-list"></div>
  `;

  document.getElementById('btn-trash').addEventListener('click', renderTrashList);

  try {
    const result = await api.getCategoriesAndTags();
    result.categories.forEach(c => { if (!allCategories.includes(c)) allCategories.push(c); });
    result.tags.forEach(t => { if (!allTags.includes(t)) allTags.push(t); });
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
  document.getElementById('btn-new-note').addEventListener('click', () => showTemplateSelector());
  document.getElementById('btn-recent').addEventListener('click', () => loadRecentNotes());

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

  const searchQuery = search;

  list.innerHTML = notes.map(note => `
    <div class="card ${note.pinned ? 'pinned' : ''}" data-id="${note.id}">
      <div class="card-header">
        <div class="card-title">${highlightMatch(note.title || 'Sem titulo', searchQuery)}</div>
        <button class="btn-icon delete-note-btn" data-id="${note.id}" title="Excluir">&#128465;</button>
      </div>
      <div class="card-meta">
        ${note.category ? `<span>${escapeHtml(note.category)}</span>` : ''}
        ${note.tags.map(t => `<span class="tag">#${escapeHtml(t)}</span>`).join('')}
        <span>${formatDate(note.updated_at)}</span>
      </div>
    </div>
  `).join('');

  list.querySelectorAll('.card').forEach(card => {
    card.addEventListener('click', (e) => {
      if (e.target.closest('.delete-note-btn')) return;
      const note = notes.find(n => n.id === card.dataset.id);
      openEditor(note);
    });
  });

  list.querySelectorAll('.delete-note-btn').forEach(btn => {
    btn.addEventListener('click', async (e) => {
      e.stopPropagation();
      const id = btn.dataset.id;
      await api.trashNote(id);
      showToast('Nota movida para lixeira', 'success');
      await loadNotes();
    });
  });
}

async function loadRecentNotes() {
  const notes = await api.getNotes(null);
  const recent = notes.slice(0, 10); // Already sorted by updated_at desc
  const list = document.getElementById('notes-list');

  if (recent.length === 0) {
    list.innerHTML = '<div class="empty">Nenhuma nota recente</div>';
    return;
  }

  list.innerHTML = recent.map(note => `
    <div class="card ${note.pinned ? 'pinned' : ''}" data-id="${note.id}">
      <div class="card-header">
        <div class="card-title">${escapeHtml(note.title || 'Sem titulo')}</div>
        <button class="btn-icon delete-note-btn" data-id="${note.id}" title="Excluir">&#128465;</button>
      </div>
      <div class="card-meta">
        ${note.category ? `<span>${escapeHtml(note.category)}</span>` : ''}
        ${note.tags.map(t => `<span class="tag">#${escapeHtml(t)}</span>`).join('')}
        <span>${formatDate(note.updated_at)}</span>
      </div>
    </div>
  `).join('');

  list.querySelectorAll('.card').forEach(card => {
    card.addEventListener('click', (e) => {
      if (e.target.closest('.delete-note-btn')) return;
      const note = recent.find(n => n.id === card.dataset.id);
      openEditor(note);
    });
  });

  list.querySelectorAll('.delete-note-btn').forEach(btn => {
    btn.addEventListener('click', async (e) => {
      e.stopPropagation();
      await api.trashNote(btn.dataset.id);
      showToast('Nota movida para lixeira', 'success');
      loadRecentNotes();
    });
  });
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
      const count = await api.emptyTrash();
      showToast(`${count} notas excluidas permanentemente`, 'success');
      renderTrashList();
    });
  });

  const notes = await api.getTrashedNotes();
  const list = document.getElementById('trash-list');

  if (notes.length === 0) {
    list.innerHTML = '<div class="empty">Lixeira vazia</div>';
    return;
  }

  list.innerHTML = notes.map(note => `
    <div class="card" data-id="${note.id}">
      <div class="card-header">
        <div class="card-title">${escapeHtml(note.title || 'Sem titulo')}</div>
        <div class="header-actions">
          <button class="btn btn-secondary btn-sm" data-restore="${note.id}">Restaurar</button>
          <button class="btn btn-danger btn-sm" data-delete="${note.id}">Excluir</button>
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
      await api.restoreNote(btn.dataset.restore);
      showToast('Nota restaurada', 'success');
      renderTrashList();
    });
  });

  list.querySelectorAll('[data-delete]').forEach(btn => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      showConfirm('Excluir permanentemente?', async () => {
        await api.deleteNote(btn.dataset.delete);
        showToast('Nota excluida permanentemente', 'success');
        renderTrashList();
      });
    });
  });
}

function highlightMatch(text, query) {
  if (!query) return escapeHtml(text);
  const escaped = escapeHtml(text);
  const regex = new RegExp(`(${query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'gi');
  return escaped.replace(regex, '<mark>$1</mark>');
}

function showTemplateSelector() {
  const container = document.getElementById('view-notes');
  currentView = 'templates';

  container.innerHTML = `
    <div class="header">
      <button class="btn btn-secondary" id="btn-back-templates">Voltar</button>
      <h3>Nova nota</h3>
    </div>
    <div class="template-grid">
      <div class="template-card" data-template="blank">
        <div class="template-icon">&#128221;</div>
        <div class="template-name">Em branco</div>
      </div>
      ${Object.entries(NOTE_TEMPLATES).map(([key, tpl]) => `
        <div class="template-card" data-template="${key}">
          <div class="template-icon">${key === 'reuniao' ? '&#128101;' : key === 'tarefa' ? '&#9745;' : key === 'diario' ? '&#128214;' : '&#127891;'}</div>
          <div class="template-name">${tpl.name}</div>
        </div>
      `).join('')}
    </div>
  `;

  document.getElementById('btn-back-templates').addEventListener('click', renderNotesList);

  container.querySelectorAll('.template-card').forEach(card => {
    card.addEventListener('click', () => {
      const templateKey = card.dataset.template;
      if (templateKey === 'blank') {
        openEditor(null);
      } else {
        const tpl = NOTE_TEMPLATES[templateKey];
        openEditor({ title: tpl.title, content: tpl.content, tags: [], category: null });
      }
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
        <button class="btn btn-secondary" id="btn-toggle-preview" title="Visualizar">&#128065;</button>
        <button class="btn btn-secondary" id="btn-copy-all" title="Copiar tudo">&#128203;</button>
        ${note ? `<button class="btn btn-secondary" id="btn-pin-note" title="Fixar na lista">${note.pinned ? '&#9733;' : '&#9734;'}</button>` : ''}
        ${note ? `<button class="btn btn-danger" id="btn-delete-note">Excluir</button>` : ''}
      </div>
    </div>
    <input type="text" class="note-title" id="note-title" value="${note ? escapeHtml(note.title) : ''}" placeholder="Titulo..." autofocus>
    <div class="form-row">
      <div class="form-group" style="flex:1">
        <label>Categoria</label>
        <div class="chip-input" id="category-chips">
          ${selectedCategory ? `<span class="chip" data-value="${escapeHtml(selectedCategory)}">${escapeHtml(selectedCategory)}<span class="remove">&times;</span></span>` : ''}
          <input type="text" id="note-category-input" placeholder="${selectedCategory ? '' : 'Selecionar...'}" list="categories-list">
          <datalist id="categories-list">
            ${allCategories.map(c => `<option value="${escapeHtml(c)}">`).join('')}
          </datalist>
        </div>
      </div>
      <div class="form-group" style="flex:1">
        <label>Tags</label>
        <div class="chip-input" id="tag-chips">
          ${selectedTags.map(t => `<span class="chip" data-value="${escapeHtml(t)}">#${escapeHtml(t)}<span class="remove">&times;</span></span>`).join('')}
          <input type="text" id="note-tags-input" placeholder="Adicionar tag..." list="tags-list">
          <datalist id="tags-list">
            ${allTags.map(t => `<option value="${escapeHtml(t)}">`).join('')}
          </datalist>
        </div>
      </div>
    </div>
    <div class="editor-wrapper" id="editor-wrapper">
      <div class="line-numbers" id="line-numbers"></div>
      <div class="editor-container">
        <textarea id="note-content" placeholder="Comece a escrever... (Ctrl+V para colar imagens)">${note ? escapeHtml(note.content) : ''}</textarea>
      </div>
    </div>
    <div class="markdown-preview" id="markdown-preview" style="display:none"></div>
    ${note ? `
      <div class="timestamp">Criada: ${formatDateTime(note.created_at)}</div>
      <div class="timestamp">Editada: ${formatDateTime(note.updated_at)}</div>
    ` : ''}
    <div id="save-status" class="timestamp"></div>
  `;

  const titleInput = document.getElementById('note-title');
  const contentInput = document.getElementById('note-content');
  const lineNumbers = document.getElementById('line-numbers');

  function updateLineNumbers() {
    const lines = contentInput.value.split('\n');
    lineNumbers.innerHTML = lines.map((_, i) => {
      const isEven = i % 2 === 0;
      return `<div class="line-number ${isEven ? 'even' : 'odd'}" data-line="${i}">${i + 1}</div>`;
    }).join('');
  }

  updateLineNumbers();
  contentInput.addEventListener('input', updateLineNumbers);
  contentInput.addEventListener('scroll', () => {
    lineNumbers.scrollTop = contentInput.scrollTop;
  });

  const autoSave = () => {
    clearTimeout(saveTimeout);
    const status = document.getElementById('save-status');
    if (status) {
      status.textContent = 'Salvando...';
      status.style.color = 'var(--text-secondary)';
    }
    saveTimeout = setTimeout(async () => {
      await saveNote(true);
      if (status) {
        status.textContent = 'Salvo';
        status.style.color = 'var(--success)';
        setTimeout(() => { if (status) status.textContent = ''; }, 2000);
      }
    }, 1000);
  };

  titleInput.addEventListener('input', autoSave);
  contentInput.addEventListener('input', autoSave);

  // Image paste
  contentInput.addEventListener('paste', async (e) => {
    const items = e.clipboardData?.items;
    if (!items) return;

    for (const item of items) {
      if (item.type.startsWith('image/')) {
        e.preventDefault();
        const file = item.getAsFile();
        if (!file) continue;

        const reader = new FileReader();
        reader.onload = async () => {
          const base64 = reader.result.split(',')[1];
          try {
            const noteId = currentNote?.id || 'new';
            const path = await api.saveImage(base64, noteId);
            const cursor = contentInput.selectionStart;
            const text = contentInput.value;
            const imgTag = `\n![Imagem](${path})\n`;
            contentInput.value = text.slice(0, cursor) + imgTag + text.slice(cursor);
            updateLineNumbers();
            showToast('Imagem colada', 'success');
            autoSave();
          } catch (err) {
            showToast('Erro ao colar imagem', 'error');
          }
        };
        reader.readAsDataURL(file);
        return;
      }
    }
  });

  // Event delegation for chip removal (works for pre-rendered and new chips)
  document.getElementById('category-chips').addEventListener('click', (e) => {
    if (e.target.classList.contains('remove')) {
      e.target.parentElement.remove();
      document.getElementById('note-category-input').placeholder = 'Selecionar...';
    }
  });
  document.getElementById('tag-chips').addEventListener('click', (e) => {
    if (e.target.classList.contains('remove')) {
      e.target.parentElement.remove();
    }
  });

  if (!note || !note.title) {
    titleInput.focus();
  } else {
    contentInput.focus();
  }

  document.getElementById('btn-back').addEventListener('click', () => {
    clearTimeout(saveTimeout);
    saveNote(true).then(() => {
      currentView = 'list';
      renderNotesList();
    });
  });

  document.getElementById('btn-copy-all').addEventListener('click', async () => {
    const content = contentInput.value;
    const title = titleInput.value;
    const fullText = title ? `${title}\n\n${content}` : content;
    try {
      await navigator.clipboard.writeText(fullText);
      showToast('Nota copiada', 'success');
    } catch (e) {
      showToast('Erro ao copiar', 'error');
    }
  });

  // Preview toggle
  let isPreviewMode = false;
  const editorWrapper = document.getElementById('editor-wrapper');
  const previewDiv = document.getElementById('markdown-preview');
  const toggleBtn = document.getElementById('btn-toggle-preview');

  toggleBtn.addEventListener('click', () => {
    isPreviewMode = !isPreviewMode;
    if (isPreviewMode) {
      editorWrapper.style.display = 'none';
      previewDiv.style.display = 'block';
      previewDiv.innerHTML = renderMarkdown(contentInput.value);
      toggleBtn.innerHTML = '&#9998;'; // Edit icon
      toggleBtn.title = 'Editar';
    } else {
      editorWrapper.style.display = 'flex';
      previewDiv.style.display = 'none';
      toggleBtn.innerHTML = '&#128065;'; // Eye icon
      toggleBtn.title = 'Visualizar';
    }
  });

  if (note) {
    document.getElementById('btn-pin-note').addEventListener('click', async () => {
      await api.updateNote({ id: note.id, pinned: !note.pinned });
      showToast(note.pinned ? 'Desfixada' : 'Fixada', 'success');
      openEditor({ ...note, pinned: !note.pinned });
    });

    document.getElementById('btn-delete-note').addEventListener('click', () => {
      showConfirm('Mover para lixeira?', async () => {
        await api.trashNote(note.id);
        showToast('Nota movida para lixeira', 'success');
        currentView = 'list';
        renderNotesList();
      });
    });
  }

  setupChipInput('category-chips', 'note-category-input', allCategories, false);
  setupChipInput('tag-chips', 'note-tags-input', allTags, true);

  // Attach click handlers to pre-existing chip remove buttons
  document.querySelectorAll('#category-chips .remove, #tag-chips .remove').forEach(btn => {
    btn.addEventListener('click', () => btn.parentElement.remove());
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
    showToast('Erro ao salvar', 'error');
  }
}

function debounce(fn, ms) {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => fn(...args), ms);
  };
}
