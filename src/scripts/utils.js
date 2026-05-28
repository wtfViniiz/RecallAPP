export function formatDate(isoString) {
  if (!isoString) return '';
  const d = new Date(isoString);
  if (isNaN(d.getTime())) return '';
  const day = String(d.getDate()).padStart(2, '0');
  const month = String(d.getMonth() + 1).padStart(2, '0');
  const year = d.getFullYear();
  return `${day}/${month}/${year}`;
}

export function formatDateTime(isoString) {
  if (!isoString) return '';
  const d = new Date(isoString);
  if (isNaN(d.getTime())) return '';
  const day = String(d.getDate()).padStart(2, '0');
  const month = String(d.getMonth() + 1).padStart(2, '0');
  const year = d.getFullYear();
  const hours = String(d.getHours()).padStart(2, '0');
  const minutes = String(d.getMinutes()).padStart(2, '0');
  return `${day}/${month}/${year} ${hours}:${minutes}`;
}

export function formatRelativeDate(isoString) {
  if (!isoString) return '';
  const now = new Date();
  const target = new Date(isoString);
  if (isNaN(target.getTime())) return '';
  const diffMs = target - now;
  const diffMins = Math.round(diffMs / 60000);

  if (diffMins < 0) return 'Atrasado';
  if (diffMins < 1) return 'Agora';
  if (diffMins < 60) return `Em ${diffMins}min`;
  const diffHours = Math.round(diffMins / 60);
  if (diffHours < 24) return `Em ${diffHours}h`;
  const diffDays = Math.round(diffHours / 24);
  return `Em ${diffDays}d`;
}

export function escapeHtml(str) {
  if (!str) return '';
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

// Toast notification system
export function showToast(message, type = 'info', duration = 3000, isHtml = false) {
  const container = document.getElementById('toast-container');
  const toast = document.createElement('div');
  toast.className = `toast ${type}`;
  if (isHtml) {
    toast.innerHTML = message;
  } else {
    toast.textContent = message;
  }
  container.appendChild(toast);

  setTimeout(() => {
    toast.style.animation = 'slideIn 0.2s ease-out reverse';
    setTimeout(() => toast.remove(), 200);
  }, duration);

  return toast;
}

export function showConfirm(message, onConfirm) {
  const container = document.getElementById('toast-container');
  const toast = document.createElement('div');
  toast.className = 'toast warning confirm-toast';

  const msg = document.createElement('span');
  msg.textContent = message;

  const actions = document.createElement('div');
  actions.className = 'confirm-actions';

  const confirmBtn = document.createElement('button');
  confirmBtn.className = 'btn btn-sm btn-danger';
  confirmBtn.textContent = 'Confirmar';
  confirmBtn.addEventListener('click', () => {
    toast.remove();
    onConfirm();
  });

  const cancelBtn = document.createElement('button');
  cancelBtn.className = 'btn btn-sm btn-secondary';
  cancelBtn.textContent = 'Cancelar';
  cancelBtn.addEventListener('click', () => toast.remove());

  actions.appendChild(cancelBtn);
  actions.appendChild(confirmBtn);
  toast.appendChild(msg);
  toast.appendChild(actions);
  container.appendChild(toast);

  setTimeout(() => {
    if (toast.parentNode) {
      toast.style.animation = 'slideIn 0.2s ease-out reverse';
      setTimeout(() => toast.remove(), 200);
    }
  }, 8000);
}

// Lightweight markdown renderer
function sanitizeUrl(url) {
  const lower = url.toLowerCase().trim();
  if (lower.startsWith('javascript:') || lower.startsWith('data:') || lower.startsWith('vbscript:') || lower.startsWith('blob:')) {
    return '#blocked';
  }
  // Block protocol-relative URLs
  if (lower.startsWith('//')) {
    return '#blocked';
  }
  return url;
}

export function renderMarkdown(text) {
  if (!text) return '';
  let html = escapeHtml(text);

  // Extract code blocks before processing to preserve their content
  const codeBlocks = [];
  html = html.replace(/```([\s\S]*?)```/g, (_, code) => {
    const placeholder = `__CODEBLOCK_${codeBlocks.length}__`;
    codeBlocks.push(`<pre><code>${code}</code></pre>`);
    return placeholder;
  });

  // Extract inline code
  const inlineCodes = [];
  html = html.replace(/`([^`]+)`/g, (_, code) => {
    const placeholder = `__INLINECODE_${inlineCodes.length}__`;
    inlineCodes.push(`<code>${code}</code>`);
    return placeholder;
  });

  // Bold
  html = html.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');
  // Italic
  html = html.replace(/\*(.+?)\*/g, '<em>$1</em>');
  // Strikethrough
  html = html.replace(/~~(.+?)~~/g, '<del>$1</del>');
  // Headings
  html = html.replace(/^### (.+)$/gm, '<h3>$1</h3>');
  html = html.replace(/^## (.+)$/gm, '<h2>$1</h2>');
  html = html.replace(/^# (.+)$/gm, '<h1>$1</h1>');
  // Blockquote
  html = html.replace(/^&gt; (.+)$/gm, '<blockquote>$1</blockquote>');
  // Unordered list items
  html = html.replace(/^- (.+)$/gm, '<li>$1</li>');
  // Horizontal rule
  html = html.replace(/^---$/gm, '<hr>');
  // Links (with URL sanitization)
  html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (_, text, url) => {
    return `<a href="${sanitizeUrl(url)}" target="_blank">${text}</a>`;
  });
  // Images (with URL sanitization)
  html = html.replace(/!\[([^\]]*)\]\(([^)]+)\)/g, (_, alt, url) => {
    return `<img src="${sanitizeUrl(url)}" alt="${alt}" style="max-width:100%;border-radius:6px;margin:4px 0">`;
  });
  // Line breaks (outside code blocks)
  html = html.replace(/\n/g, '<br>');

  // Restore code blocks and inline code
  codeBlocks.forEach((block, i) => {
    html = html.replace(`__CODEBLOCK_${i}__`, block);
  });
  inlineCodes.forEach((code, i) => {
    html = html.replace(`__INLINECODE_${i}__`, code);
  });

  return html;
}

export function htmlToMarkdown(html) {
  if (!html) return '';
  let md = html;

  // Preserve code blocks
  const codeBlocks = [];
  md = md.replace(/<pre><code>([\s\S]*?)<\/code><\/pre>/gi, (_, code) => {
    const placeholder = `__CB_${codeBlocks.length}__`;
    codeBlocks.push(code);
    return placeholder;
  });

  // Inline code
  const inlineCodes = [];
  md = md.replace(/<code>([^<]+)<\/code>/gi, (_, code) => {
    const placeholder = `__IC_${inlineCodes.length}__`;
    inlineCodes.push(code);
    return placeholder;
  });

  // Bold
  md = md.replace(/<strong>([\s\S]*?)<\/strong>/gi, '**$1**');
  md = md.replace(/<b>([\s\S]*?)<\/b>/gi, '**$1**');
  // Italic
  md = md.replace(/<em>([\s\S]*?)<\/em>/gi, '*$1*');
  md = md.replace(/<i>([\s\S]*?)<\/i>/gi, '*$1*');
  // Strikethrough
  md = md.replace(/<del>([\s\S]*?)<\/del>/gi, '~~$1~~');
  md = md.replace(/<s>([\s\S]*?)<\/s>/gi, '~~$1~~');
  // Headings
  md = md.replace(/<h1>([\s\S]*?)<\/h1>/gi, '# $1\n');
  md = md.replace(/<h2>([\s\S]*?)<\/h2>/gi, '## $1\n');
  md = md.replace(/<h3>([\s\S]*?)<\/h3>/gi, '### $1\n');
  // Blockquote
  md = md.replace(/<blockquote>([\s\S]*?)<\/blockquote>/gi, '> $1\n');
  // List items
  md = md.replace(/<li>([\s\S]*?)<\/li>/gi, '- $1\n');
  // Links
  md = md.replace(/<a[^>]*href="([^"]*)"[^>]*>([\s\S]*?)<\/a>/gi, '[$2]($1)');
  // Images
  md = md.replace(/<img[^>]*src="([^"]*)"[^>]*alt="([^"]*)"[^>]*>/gi, '![$2]($1)');
  md = md.replace(/<img[^>]*src="([^"]*)"[^>]*>/gi, '![]($1)');
  // Horizontal rule
  md = md.replace(/<hr\s*\/?>/gi, '\n---\n');
  // Line breaks and paragraphs
  md = md.replace(/<br\s*\/?>/gi, '\n');
  md = md.replace(/<\/div>\s*<div[^>]*>/gi, '\n');
  md = md.replace(/<\/?div[^>]*>/gi, '\n');
  md = md.replace(/<p>([\s\S]*?)<\/p>/gi, '$1\n');
  // Remove remaining HTML tags
  md = md.replace(/<[^>]+>/g, '');
  // Decode HTML entities
  md = md.replace(/&amp;/g, '&').replace(/&lt;/g, '<').replace(/&gt;/g, '>').replace(/&quot;/g, '"').replace(/&#39;/g, "'").replace(/&nbsp;/g, ' ');
  // Clean up extra newlines
  md = md.replace(/\n{3,}/g, '\n\n').trim();

  // Restore code blocks and inline code
  codeBlocks.forEach((block, i) => {
    md = md.replace(`__CB_${i}__`, '```\n' + block + '\n```');
  });
  inlineCodes.forEach((code, i) => {
    md = md.replace(`__IC_${i}__`, '`' + code + '`');
  });

  return md;
}

export function markdownToHtml(md) {
  if (!md) return '';
  return renderMarkdown(md)
    .replace(/<br\s*\/?>/gi, '<br>')
    .replace(/<h([1-3])>/gi, '<h$1>')
    .replace(/<\/h[1-3]>/gi, '</h$1>');
}

// Predefined categories and tags
export const DEFAULT_CATEGORIES = [
  'Pessoal',
  'Trabalho',
  'Estudos',
  'Finanças',
  'Saúde',
  'Projetos',
  'Idéias'
];

export const DEFAULT_TAGS = [
  'urgente',
  'importante',
  'lembrete',
  'ideia',
  'tarefa',
  'referencia',
  'rascunho'
];

export const NOTE_TEMPLATES = {
  'reuniao': {
    name: 'Reuniao',
    title: 'Reuniao - ',
    content: '# Participantes\n\n# Pauta\n\n# Decisoes\n\n# Proximos passos\n'
  },
  'tarefa': {
    name: 'Tarefa',
    title: '',
    content: '- [ ] \n- [ ] \n- [ ] \n'
  },
  'diario': {
    name: 'Diario',
    title: 'Diario - ' + new Date().toLocaleDateString('pt-BR'),
    content: '# Como me sinto\n\n# O que fiz hoje\n\n# Amanha pretendo\n'
  },
  'estudo': {
    name: 'Estudo',
    title: '',
    content: '# Assunto\n\n# Notas\n\n# Referencias\n'
  }
};
