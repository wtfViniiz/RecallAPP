import { api } from './api.js';
import { formatDate, formatDateTime, escapeHtml, showToast, showConfirm, renderMarkdown, htmlToMarkdown, markdownToHtml, DEFAULT_CATEGORIES, DEFAULT_TAGS, NOTE_TEMPLATES } from './utils.js';

let currentView = 'list';
let currentNote = null;
let allCategories = [...DEFAULT_CATEGORIES];
let allTags = [...DEFAULT_TAGS];
let saveTimeout = null;
let keyListenersAttached = false;
let notesOffset = 0;
const NOTES_PAGE_SIZE = 30;

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

function renderNoteCards(notes, searchQuery) {
  return notes.map(note => `
    <div class="card ${note.pinned ? 'pinned' : ''}" data-id="${note.id}" draggable="true">
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
}

function attachNoteCardHandlers(list, notes) {
  let draggedCard = null;

  list.querySelectorAll('.card').forEach(card => {
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
      for (let i = 0; i < updatedCards.length; i++) {
        const id = updatedCards[i].dataset.id;
        try {
          await api.updateNote({ id, position: i });
        } catch (err) {}
      }

      showToast('Nota reposicionada', 'success');
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
        showToast('Erro ao mover para lixeira', 'error');
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

  const result = await api.getNotes(filter);
  const notes = result.items;
  const total = result.total;
  const list = document.getElementById('notes-list');

  if (!append && notes.length === 0) {
    list.innerHTML = '<div class="empty">Nenhuma nota encontrada</div>';
    return;
  }

  const cardsHtml = renderNoteCards(notes, search);

  if (append) {
    const existingBtn = list.querySelector('.load-more-container');
    if (existingBtn) existingBtn.remove();
    list.insertAdjacentHTML('beforeend', cardsHtml);
  } else {
    list.innerHTML = cardsHtml;
  }

  notesOffset += notes.length;
  attachNoteCardHandlers(list, notes);

  if (notesOffset < total) {
    const loadMoreHtml = '<div class="load-more-container"><button class="btn btn-secondary" id="btn-load-more">Carregar mais</button></div>';
    list.insertAdjacentHTML('beforeend', loadMoreHtml);
    document.getElementById('btn-load-more').addEventListener('click', () => loadNotes(true));
  }
}

async function loadRecentNotes() {
  const result = await api.getNotes({ limit: 10 });
  const recent = result.items;
  const list = document.getElementById('notes-list');

  if (recent.length === 0) {
    list.innerHTML = '<div class="empty">Nenhuma nota recente</div>';
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
      try {
        await api.restoreNote(btn.dataset.restore);
        showToast('Nota restaurada', 'success');
        renderTrashList();
      } catch (err) {
        showToast('Erro ao restaurar nota', 'error');
      }
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

async function showTemplateSelector() {
  const container = document.getElementById('view-notes');
  currentView = 'templates';

  let customTemplates = [];
  try {
    customTemplates = await api.getCustomTemplates();
  } catch (e) {}

  container.innerHTML = `
    <div class="header">
      <button class="btn btn-secondary" id="btn-back-templates">Voltar</button>
      <h3>Nova nota</h3>
      <button class="btn btn-primary btn-sm" id="btn-new-template" title="Criar template">+ Template</button>
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
      ${customTemplates.map(tpl => `
        <div class="template-card" data-template="custom:${tpl.id}">
          <div class="template-icon">${tpl.icon || '&#128221;'}</div>
          <div class="template-name">${escapeHtml(tpl.name)}</div>
          <button class="btn-icon delete-template-btn" data-id="${tpl.id}" title="Excluir template" style="position:absolute;top:4px;right:4px;font-size:12px;">&times;</button>
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
          showToast('Erro ao excluir template', 'error');
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
      <label>Icon (emoji)</label>
      <input type="text" id="tpl-icon" class="form-input" value="${tpl ? escapeHtml(tpl.icon || '') : ''}" placeholder="&#128221;" maxlength="4">
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
    } catch { showToast('Erro ao salvar', 'error'); }
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
        <button class="btn btn-secondary" id="btn-save-template" title="Salvar como template">&#128203;TPL</button>
        ${note ? `<button class="btn btn-secondary" id="btn-versions" title="Historico de versoes">&#128337;</button>` : ''}
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
    <div class="markdown-toolbar" id="markdown-toolbar">
      <button type="button" class="toolbar-btn" data-command="bold" title="Negrito (Ctrl+B)"><strong>B</strong></button>
      <button type="button" class="toolbar-btn" data-command="italic" title="Italico (Ctrl+I)"><em>I</em></button>
      <button type="button" class="toolbar-btn" data-command="strikeThrough" title="Tachado"><s>S</s></button>
      <button type="button" class="toolbar-btn" data-command="formatBlock" data-value="H3" title="Titulo">H</button>
      <button type="button" class="toolbar-btn" data-command="insertUnorderedList" title="Lista">&#8226;</button>
      <button type="button" class="toolbar-btn" data-command="formatBlock" data-value="BLOCKQUOTE" title="Citacao">&#8220;</button>
      <button type="button" class="toolbar-btn" id="btn-insert-code" title="Codigo">&lt;/&gt;</button>
      <button type="button" class="toolbar-btn" id="btn-insert-link" title="Link">&#128279;</button>
    </div>
    <div class="editor-wrapper" id="editor-wrapper">
      <div class="editor-container">
        <div id="note-content" class="wysiwyg-editor" contenteditable="true">${note ? markdownToHtml(note.content) : ''}</div>
      </div>
    </div>
    <div class="markdown-preview" id="markdown-preview" style="display:none"></div>
    <div class="versions-panel" id="versions-panel" style="display:none">
      <div class="versions-header">
        <span>Historico de versoes</span>
        <button class="btn btn-sm btn-secondary" id="btn-close-versions">&times;</button>
      </div>
      <div class="versions-list" id="versions-list"></div>
    </div>
    ${note ? `
      <div class="timestamp">Criada: ${formatDateTime(note.created_at)}</div>
      <div class="timestamp">Editada: ${formatDateTime(note.updated_at)}</div>
    ` : ''}
    <div id="save-status" class="timestamp"></div>
  `;

  const titleInput = document.getElementById('note-title');
  const contentInput = document.getElementById('note-content');

  // --- WYSIWYG toolbar via execCommand ---
  document.getElementById('markdown-toolbar').addEventListener('click', (e) => {
    const btn = e.target.closest('.toolbar-btn');
    if (!btn) return;

    const command = btn.dataset.command;
    if (command) {
      e.preventDefault();
      contentInput.focus();
      if (command === 'formatBlock') {
        document.execCommand(command, false, btn.dataset.value);
      } else {
        document.execCommand(command, false, null);
      }
      return;
    }

    // Code inline
    if (btn.id === 'btn-insert-code') {
      e.preventDefault();
      contentInput.focus();
      const sel = window.getSelection();
      if (sel.rangeCount > 0) {
        const range = sel.getRangeAt(0);
        const selected = range.toString();
        const code = document.createElement('code');
        code.textContent = selected || 'codigo';
        range.deleteContents();
        range.insertNode(code);
        // Move cursor after
        range.setStartAfter(code);
        range.collapse(true);
        sel.removeAllRanges();
        sel.addRange(range);
      }
    }

    // Link
    if (btn.id === 'btn-insert-link') {
      e.preventDefault();
      contentInput.focus();
      const sel = window.getSelection();
      if (sel.rangeCount > 0) {
        const range = sel.getRangeAt(0);
        const selected = range.toString();
        const url = prompt('URL:', 'https://');
        if (url) {
          const a = document.createElement('a');
          a.href = url;
          a.textContent = selected || 'link';
          a.target = '_blank';
          range.deleteContents();
          range.insertNode(a);
          range.setStartAfter(a);
          range.collapse(true);
          sel.removeAllRanges();
          sel.addRange(range);
        }
      }
    }
  });

  // Keyboard shortcuts for formatting
  contentInput.addEventListener('keydown', (e) => {
    if (e.ctrlKey || e.metaKey) {
      if (e.key === 'b') { e.preventDefault(); document.execCommand('bold', false, null); }
      if (e.key === 'i') { e.preventDefault(); document.execCommand('italic', false, null); }
      if (e.key === 's') { e.preventDefault(); document.execCommand('strikeThrough', false, null); }
    }
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
            const img = document.createElement('img');
            img.src = path;
            img.style.maxWidth = '100%';
            const sel = window.getSelection();
            if (sel.rangeCount > 0) {
              const range = sel.getRangeAt(0);
              range.deleteContents();
              range.insertNode(img);
              range.setStartAfter(img);
              range.collapse(true);
              sel.removeAllRanges();
              sel.addRange(range);
            } else {
              contentInput.appendChild(img);
            }
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

  // Event delegation for chip removal
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
    const content = htmlToMarkdown(contentInput.innerHTML);
    const title = titleInput.value;
    const fullText = title ? `${title}\n\n${content}` : content;
    try {
      await navigator.clipboard.writeText(fullText);
      showToast('Nota copiada', 'success');
    } catch (e) {
      showToast('Erro ao copiar', 'error');
    }
  });

  // Save as template
  document.getElementById('btn-save-template').addEventListener('click', async () => {
    const title = titleInput.value.trim();
    const content = htmlToMarkdown(contentInput.innerHTML);
    if (!title && !content) {
      showToast('Nota vazia, nada para salvar como template', 'warning');
      return;
    }
    const name = prompt('Nome do template:', title || 'Meu template');
    if (!name) return;
    try {
      await api.saveCustomTemplate({
        id: crypto.randomUUID ? crypto.randomUUID() : Date.now().toString(36),
        name,
        title: title || '',
        content: content || '',
        icon: null,
      });
      showToast('Template salvo', 'success');
    } catch (err) {
      showToast('Erro ao salvar template', 'error');
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
      const md = htmlToMarkdown(contentInput.innerHTML);
      previewDiv.innerHTML = renderMarkdown(md);
      toggleBtn.innerHTML = '&#9998;';
      toggleBtn.title = 'Editar';
    } else {
      editorWrapper.style.display = 'flex';
      previewDiv.style.display = 'none';
      toggleBtn.innerHTML = '&#128065;';
      toggleBtn.title = 'Visualizar';
    }
  });

  // Version history
  if (note) {
    document.getElementById('btn-versions').addEventListener('click', async () => {
      const panel = document.getElementById('versions-panel');
      const listDiv = document.getElementById('versions-list');
      if (panel.style.display === 'none') {
        panel.style.display = 'block';
        listDiv.innerHTML = '<div class="empty">Carregando...</div>';
        try {
          const versions = await api.listNoteVersions(note.id);
          if (versions.length === 0) {
            listDiv.innerHTML = '<div class="empty">Nenhuma versao anterior</div>';
          } else {
            listDiv.innerHTML = versions.map(v => `
              <div class="version-item" data-version-id="${v.id}">
                <div class="version-info">
                  <span class="version-date">${formatDateTime(v.created_at)}</span>
                  <span class="version-title">${escapeHtml(v.title || 'Sem titulo')}</span>
                </div>
                <button class="btn btn-sm btn-secondary btn-restore-version" data-version-id="${v.id}">Restaurar</button>
              </div>
            `).join('');

            listDiv.querySelectorAll('.btn-restore-version').forEach(btn => {
              btn.addEventListener('click', async () => {
                showConfirm('Restaurar esta versao? O estado atual sera salvo como versao.', async () => {
                  try {
                    const restored = await api.restoreNoteVersion(note.id, btn.dataset.versionId);
                    currentNote = restored;
                    titleInput.value = restored.title;
                    contentInput.innerHTML = markdownToHtml(restored.content);
                    panel.style.display = 'none';
                    showToast('Versao restaurada', 'success');
                  } catch (err) {
                    showToast('Erro ao restaurar versao', 'error');
                  }
                });
              });
            });
          }
        } catch (err) {
          listDiv.innerHTML = '<div class="empty">Erro ao carregar versoes</div>';
        }
      } else {
        panel.style.display = 'none';
      }
    });

    document.getElementById('btn-close-versions').addEventListener('click', () => {
      document.getElementById('versions-panel').style.display = 'none';
    });

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

  document.querySelectorAll('#category-chips .remove, #tag-chips .remove').forEach(btn => {
    btn.addEventListener('click', () => btn.parentElement.remove());
  });
}

async function toggleVersions(note) {
  if (!note?.id) { showToast('Salve a nota primeiro', 'info'); return; }
  const panel = document.getElementById('versions-panel');
  const list = document.getElementById('versions-list');
  if (panel.style.display === 'none') {
    panel.style.display = 'block';
    list.innerHTML = '<div class="versions-loading">Carregando...</div>';
    try {
      const versions = await api.listNoteVersions(note.id);
      if (versions.length === 0) {
        list.innerHTML = '<div class="versions-empty">Nenhuma versao anterior</div>';
      } else {
        list.innerHTML = versions.map(v => `
          <div class="version-item">
            <div class="version-info">
              <span class="version-date">${formatDateTime(v.created_at)}</span>
              <span class="version-title">${escapeHtml(v.title)}</span>
            </div>
            <button class="btn btn-sm btn-secondary restore-version" data-version-id="${v.id}">Restaurar</button>
          </div>
        `).join('');

        list.querySelectorAll('.restore-version').forEach(btn => {
          btn.addEventListener('click', async () => {
            try {
              const restored = await api.restoreNoteVersion(note.id, btn.dataset.versionId);
              showToast('Versao restaurada', 'success');
              openEditor(restored);
            } catch { showToast('Erro ao restaurar', 'error'); }
          });
        });
      }
    } catch {
      list.innerHTML = '<div class="versions-empty">Erro ao carregar</div>';
    }
  } else {
    panel.style.display = 'none';
  }
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
  const contentEl = document.getElementById('note-content');
  const content = contentEl ? htmlToMarkdown(contentEl.innerHTML) : '';
  const category = getChips('category-chips')[0] || null;
  const tags = getChips('tag-chips');

  if (!title && !content) return;

  try {
    if (currentNote && currentNote.id) {
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
