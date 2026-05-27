const { invoke } = window.__TAURI__.core;

export const api = {
  // Notes
  getNotes: (filter) => invoke('get_notes', { filter }),
  getNote: (id) => invoke('get_note', { id }),
  createNote: (input) => invoke('create_note', { input }),
  updateNote: (input) => invoke('update_note', { input }),
  deleteNote: (id) => invoke('delete_note', { id }),

  // Reminders
  getReminders: (status) => invoke('get_reminders', { status }),
  createReminder: (input) => invoke('create_reminder', { input }),
  updateReminder: (input) => invoke('update_reminder', { input }),
  deleteReminder: (id) => invoke('delete_reminder', { id }),
  dismissReminder: (id) => invoke('dismiss_reminder', { id }),

  // Config
  getConfig: () => invoke('get_config'),
  updateConfig: (input) => invoke('update_config', { input }),

  // Helpers
  getCategories: () => invoke('get_categories'),
  getTags: () => invoke('get_tags'),
};
