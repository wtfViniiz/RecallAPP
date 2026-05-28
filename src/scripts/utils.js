export function formatDate(isoString) {
  if (!isoString) return '';
  const d = new Date(isoString);
  const day = String(d.getDate()).padStart(2, '0');
  const month = String(d.getMonth() + 1).padStart(2, '0');
  const year = d.getFullYear();
  return `${day}/${month}/${year}`;
}

export function formatDateTime(isoString) {
  if (!isoString) return '';
  const d = new Date(isoString);
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
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

// Toast notification system
export function showToast(message, type = 'info', duration = 3000) {
  const container = document.getElementById('toast-container');
  const toast = document.createElement('div');
  toast.className = `toast ${type}`;
  toast.textContent = message;
  container.appendChild(toast);

  setTimeout(() => {
    toast.style.animation = 'slideIn 0.2s ease-out reverse';
    setTimeout(() => toast.remove(), 200);
  }, duration);
}

// Lightweight markdown renderer
export function renderMarkdown(text) {
  if (!text) return '';
  let html = escapeHtml(text);

  // Code blocks
  html = html.replace(/```([\s\S]*?)```/g, '<pre><code>$1</code></pre>');
  // Inline code
  html = html.replace(/`([^`]+)`/g, '<code>$1</code>');
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
  // Links
  html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank">$1</a>');
  // Images (local paths)
  html = html.replace(/!\[([^\]]*)\]\(([^)]+)\)/g, '<img src="$2" alt="$1" style="max-width:100%;border-radius:6px;margin:4px 0">');
  // Line breaks
  html = html.replace(/\n/g, '<br>');

  return html;
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
