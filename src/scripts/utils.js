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
