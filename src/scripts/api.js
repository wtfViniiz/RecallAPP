async function invoke(cmd, args = {}) {
  return window.__TAURI_INTERNALS__.invoke(cmd, args);
}

export const api = {
  // Notes
  getNotes: (filter) => invoke('get_notes', { filter }),
  getNote: (id) => invoke('get_note', { id }),
  createNote: (input) => invoke('create_note', { input }),
  updateNote: (input) => invoke('update_note', { input }),
  deleteNote: (id) => invoke('delete_note', { id }),
  trashNote: (id) => invoke('trash_note', { id }),
  restoreNote: (id) => invoke('restore_note', { id }),
  emptyTrash: () => invoke('empty_trash'),
  getTrashedNotes: () => invoke('get_trashed_notes'),

  // Reminders
  getReminders: (status) => invoke('get_reminders', { status }),
  createReminder: (input) => invoke('create_reminder', { input }),
  updateReminder: (input) => invoke('update_reminder', { input }),
  deleteReminder: (id) => invoke('delete_reminder', { id }),
  dismissReminder: (id) => invoke('dismiss_reminder', { id }),
  snoozeReminder: (id, minutes) => invoke('snooze_reminder', { id, minutes }),

  // Config
  getConfig: () => invoke('get_config'),
  updateConfig: (input) => invoke('update_config', { input }),
  updateShortcut: (shortcutStr) => invoke('update_shortcut', { shortcutStr }),

  // Helpers
  getCategories: () => invoke('get_categories'),
  getTags: () => invoke('get_tags'),

  // Window
  setAlwaysOnTop: (pinned) => invoke('set_always_on_top', { pinned }),

  // Images
  saveImage: (base64Data, noteId) => invoke('save_image', { base64_data: base64Data, note_id: noteId }),

  // Export/Import
  exportData: () => invoke('export_data'),
  importData: (jsonData) => invoke('import_data', { jsonData }),
};
