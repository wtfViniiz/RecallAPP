import { api } from './api.js';
import { formatDateTime, escapeHtml, sanitizeUrl, toAssetUrl, showToast, showConfirm, renderMarkdown, htmlToMarkdown, markdownToHtml } from './utils.js';
import { icons } from './icons.js';
import { renderNotesList, getCurrentView, setCurrentView, getAllCategories, getAllTags } from './notes.js';

let currentNote = null;
let saveTimeout = null;

export { openEditor, flushPendingSave, saveNote };

async function flushPendingSave() {
  if (saveTimeout) {
    clearTimeout(saveTimeout);
    saveTimeout = null;
    if (currentNote && getCurrentView() === 'editor') {
      await saveNote(true);
    }
  }
}

function openEditor(note) {
  setCurrentView('editor');
  currentNote = note;
  const container = document.getElementById('view-notes');

  const categories = getAllCategories();
  const tags = getAllTags();
  const selectedCategory = note?.category || '';
  const selectedTags = note?.tags || [];

  container.innerHTML = `
    <div class="header">
      <button class="btn btn-secondary" id="btn-back">Voltar</button>
      <div class="header-actions">
        <button class="btn btn-secondary" id="btn-toggle-preview" title="Visualizar">${icons.eye(16)}</button>
        <button class="btn btn-secondary" id="btn-copy-all" title="Copiar tudo">${icons.clipboard(16)}</button>
        <button class="btn btn-secondary" id="btn-save-template" title="Salvar como template">${icons['file-text'](16)}</button>
        ${note ? `<button class="btn btn-secondary" id="btn-versions" title="Historico de versoes">${icons.history(16)}</button>` : ''}
        ${note ? `<button class="btn btn-secondary" id="btn-pin-note" title="Fixar na lista">${note.pinned ? icons['star-filled'](16) : icons.star(16)}</button>` : ''}
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
            ${categories.map(c => `<option value="${escapeHtml(c)}">`).join('')}
          </datalist>
        </div>
      </div>
      <div class="form-group" style="flex:1">
        <label>Tags</label>
        <div class="chip-input" id="tag-chips">
          ${selectedTags.map(t => `<span class="chip" data-value="${escapeHtml(t)}">#${escapeHtml(t)}<span class="remove">&times;</span></span>`).join('')}
          <input type="text" id="note-tags-input" placeholder="Adicionar tag..." list="tags-list">
          <datalist id="tags-list">
            ${tags.map(t => `<option value="${escapeHtml(t)}">`).join('')}
          </datalist>
        </div>
      </div>
    </div>
    <div class="markdown-toolbar" id="markdown-toolbar">
      <button type="button" class="toolbar-btn" data-command="bold" title="Negrito (Ctrl+B)">${icons.bold(14)}</button>
      <button type="button" class="toolbar-btn" data-command="italic" title="Italico (Ctrl+I)">${icons.italic(14)}</button>
      <button type="button" class="toolbar-btn" data-command="strikeThrough" title="Tachado">${icons.strikethrough(14)}</button>
      <button type="button" class="toolbar-btn" data-command="formatBlock" data-value="H3" title="Titulo">${icons.heading(14)}</button>
      <button type="button" class="toolbar-btn" data-command="insertUnorderedList" title="Lista">${icons.list(14)}</button>
      <button type="button" class="toolbar-btn" data-command="formatBlock" data-value="BLOCKQUOTE" title="Citacao">${icons.quote(14)}</button>
      <button type="button" class="toolbar-btn" id="btn-insert-code" title="Codigo">${icons.code(14)}</button>
      <button type="button" class="toolbar-btn" id="btn-insert-link" title="Link">${icons.link(14)}</button>
    </div>
    <div class="editor-wrapper" id="editor-wrapper">
      <div class="editor-container">
        <div id="note-content" class="wysiwyg-editor" contenteditable="true" data-placeholder="Comece a escrever...">${note ? markdownToHtml(note.content) : ''}</div>
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
        const urlContainer = document.createElement('span');
        urlContainer.className = 'inline-url-input';
        urlContainer.innerHTML = `<input type="text" class="inline-input" placeholder="https://..." value="https://">`;
        range.deleteContents();
        range.insertNode(urlContainer);
        const urlInput = urlContainer.querySelector('input');
        urlInput.focus();
        urlInput.select();
        let finished = false;
        const finishLink = () => {
          if (finished) return;
          finished = true;
          const url = sanitizeUrl(urlInput.value.trim());
          if (url && url !== 'about:blank') {
            const a = document.createElement('a');
            a.href = url;
            a.textContent = selected || 'link';
            a.target = '_blank';
            a.rel = 'noopener noreferrer';
            urlContainer.replaceWith(a);
          } else {
            urlContainer.replaceWith(document.createTextNode(selected || 'link'));
          }
        };
        urlInput.addEventListener('blur', finishLink);
        urlInput.addEventListener('keydown', (evt) => {
          if (evt.key === 'Enter') { evt.preventDefault(); finishLink(); }
          if (evt.key === 'Escape') { evt.stopPropagation(); finished = true; urlContainer.replaceWith(document.createTextNode(selected || 'link')); }
        });
      }
    }
  });

  // Keyboard shortcuts for formatting
  contentInput.addEventListener('keydown', (e) => {
    if (e.ctrlKey || e.metaKey) {
      if (e.key === 'b') { e.preventDefault(); document.execCommand('bold', false, null); }
      if (e.key === 'i') { e.preventDefault(); document.execCommand('italic', false, null); }
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
    const success = await saveNote(true);
    if (status) {
      if (success) {
        status.textContent = 'Salvo';
        status.style.color = 'var(--success)';
      } else {
        status.textContent = 'Erro ao salvar';
        status.style.color = 'var(--error)';
      }
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
            img.src = toAssetUrl(path);
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
            showToast(err.message || 'Erro ao colar imagem', 'error');
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
      setCurrentView('list');
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
      showToast(e.message || 'Erro ao copiar', 'error');
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
    const defaultName = title || 'Meu template';
    const toastEl = showToast(`<div class="confirm-toast">
      <div style="font-size:13px;margin-bottom:6px">Nome do template:</div>
      <div style="display:flex;gap:8px">
        <input type="text" id="tpl-name-input" class="inline-input" value="${escapeHtml(defaultName)}" style="flex:1">
        <button class="btn btn-primary btn-sm" id="tpl-save-btn">Salvar</button>
        <button class="btn btn-secondary btn-sm" id="tpl-cancel-btn">&times;</button>
      </div>
    </div>`, 'info', 30000, true);
    setTimeout(() => {
      const nameInput = document.getElementById('tpl-name-input');
      const saveBtn = document.getElementById('tpl-save-btn');
      const cancelBtn = document.getElementById('tpl-cancel-btn');
      if (!nameInput) return;
      nameInput.focus();
      nameInput.select();
      const doSave = async () => {
        const name = nameInput.value.trim();
        if (!name) return;
        if (toastEl?.parentElement) toastEl.remove();
        try {
          await api.saveCustomTemplate({
            id: crypto.randomUUID(),
            name,
            title: title || '',
            content: content || '',
            icon: null,
          });
          showToast('Template salvo', 'success');
        } catch (err) { showToast(err.message || 'Erro ao salvar', 'error'); }
      };
      saveBtn?.addEventListener('click', doSave);
      cancelBtn?.addEventListener('click', () => { if (toastEl?.parentElement) toastEl.remove(); });
      nameInput.addEventListener('keydown', (evt) => {
        if (evt.key === 'Enter') doSave();
        if (evt.key === 'Escape') { if (toastEl?.parentElement) toastEl.remove(); }
      });
    }, 50);
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
      toggleBtn.innerHTML = icons.edit(16);
      toggleBtn.title = 'Editar';
    } else {
      editorWrapper.style.display = 'flex';
      previewDiv.style.display = 'none';
      toggleBtn.innerHTML = icons.eye(16);
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
            listDiv.innerHTML = `<div class="empty">${icons.history(32)}<p>Nenhuma versao anterior</p></div>`;
          } else {
            listDiv.innerHTML = versions.map(v => `
              <div class="version-item" data-version-id="${escapeHtml(v.id)}">
                <div class="version-info">
                  <span class="version-date">${formatDateTime(v.created_at)}</span>
                  <span class="version-title">${escapeHtml(v.title || 'Sem titulo')}</span>
                </div>
                <button class="btn btn-sm btn-secondary btn-restore-version" data-version-id="${escapeHtml(v.id)}">Restaurar</button>
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
                    // Update category chips
                    const catContainer = document.getElementById('category-chips');
                    catContainer.querySelectorAll('.chip').forEach(c => c.remove());
                    if (restored.category) {
                      const chip = document.createElement('span');
                      chip.className = 'chip';
                      chip.dataset.value = restored.category;
                      chip.innerHTML = `${escapeHtml(restored.category)}<span class="remove">&times;</span>`;
                      catContainer.insertBefore(chip, catContainer.querySelector('input'));
                    }
                    // Update tag chips
                    const tagContainer = document.getElementById('tag-chips');
                    tagContainer.querySelectorAll('.chip').forEach(c => c.remove());
                    restored.tags.forEach(t => {
                      const chip = document.createElement('span');
                      chip.className = 'chip';
                      chip.dataset.value = t;
                      chip.innerHTML = `#${escapeHtml(t)}<span class="remove">&times;</span>`;
                      tagContainer.insertBefore(chip, tagContainer.querySelector('input'));
                    });
                    panel.style.display = 'none';
                    showToast('Versao restaurada', 'success');
                  } catch (err) {
                    showToast(err.message || 'Erro ao restaurar versao', 'error');
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
      try {
        await api.updateNote({ id: note.id, pinned: !note.pinned });
        showToast(note.pinned ? 'Desfixada' : 'Fixada', 'success');
        openEditor({ ...note, pinned: !note.pinned });
      } catch (err) {
        showToast(err.message || 'Erro ao fixar nota', 'error');
      }
    });

    document.getElementById('btn-delete-note').addEventListener('click', () => {
      showConfirm('Mover para lixeira?', async () => {
        clearTimeout(saveTimeout);
        try {
          await api.trashNote(note.id);
          showToast('Nota movida para lixeira', 'success');
        } catch (err) {
          showToast(err.message || 'Erro ao mover para lixeira', 'error');
        }
        setCurrentView('list');
        renderNotesList();
      });
    });
  }

  setupChipInput('category-chips', 'note-category-input', categories, false);
  setupChipInput('tag-chips', 'note-tags-input', tags, true);
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
  const titleEl = document.getElementById('note-title');
  const contentEl = document.getElementById('note-content');
  if (!titleEl || !contentEl) return;
  const title = titleEl.value.trim();
  const content = htmlToMarkdown(contentEl.innerHTML);
  const category = getChips('category-chips')[0] || null;
  const tags = getChips('tag-chips');

  if (!title && !content) return true;

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
    return true;
  } catch (e) {
    showToast(e.message || 'Erro ao salvar', 'error');
    return false;
  }
}
