use crate::models::*;
use crate::storage;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

#[derive(Clone)]
pub struct NoteCache {
    notes: Arc<Mutex<Option<Vec<Note>>>>,
    reminders: Arc<Mutex<Option<Vec<Reminder>>>>,
}

impl NoteCache {
    pub fn new() -> Self {
        Self {
            notes: Arc::new(Mutex::new(None)),
            reminders: Arc::new(Mutex::new(None)),
        }
    }

    // --- Internal: single-lock helpers to prevent TOCTOU races ---

    fn with_notes<F, R>(&self, app: &AppHandle, f: F) -> R
    where
        F: FnOnce(&[Note]) -> R,
    {
        let mut guard = self.notes.lock().unwrap();
        if guard.is_none() {
            *guard = Some(storage::list_all_notes(app));
        }
        f(guard.as_ref().unwrap())
    }

    fn with_reminders<F, R>(&self, app: &AppHandle, f: F) -> R
    where
        F: FnOnce(&[Reminder]) -> R,
    {
        let mut guard = self.reminders.lock().unwrap();
        if guard.is_none() {
            *guard = Some(storage::list_reminders(app, None));
        }
        f(guard.as_ref().unwrap())
    }

    // --- Notes ---

    pub fn list_notes(&self, app: &AppHandle, filter: Option<NoteFilter>) -> Vec<Note> {
        self.with_notes(app, |notes| {
            let mut filtered: Vec<Note> = notes
                .iter()
                .filter(|n| !n.trashed)
                .filter(|n| {
                    if let Some(ref f) = filter {
                        if let Some(ref search) = f.search {
                            let search_lower = search.to_lowercase();
                            if !n.title.to_lowercase().contains(&search_lower)
                                && !n.content.to_lowercase().contains(&search_lower)
                            {
                                return false;
                            }
                        }
                        if let Some(ref cat) = f.category {
                            if n.category.as_ref() != Some(cat) {
                                return false;
                            }
                        }
                        if let Some(ref tag) = f.tag {
                            if !n.tags.contains(tag) {
                                return false;
                            }
                        }
                    }
                    true
                })
                .cloned()
                .collect();

            filtered.sort_by(|a, b| {
                b.pinned
                    .cmp(&a.pinned)
                    .then_with(|| b.updated_at.cmp(&a.updated_at))
            });

            filtered
        })
    }

    pub fn list_notes_paginated(
        &self,
        app: &AppHandle,
        filter: Option<NoteFilter>,
    ) -> PaginatedResult<Note> {
        let (offset, limit) = match &filter {
            Some(f) => (f.offset.unwrap_or(0), f.limit),
            None => (0, None),
        };
        let mut notes = self.list_notes(app, filter);
        let total = notes.len();
        if offset > 0 {
            notes = notes.into_iter().skip(offset).collect();
        }
        if let Some(limit) = limit {
            notes.truncate(limit);
        }
        PaginatedResult { items: notes, total }
    }

    pub fn list_all_notes(&self, app: &AppHandle) -> Vec<Note> {
        self.with_notes(app, |notes| notes.to_vec())
    }

    pub fn list_trashed_notes(&self, app: &AppHandle) -> Vec<Note> {
        self.with_notes(app, |notes| {
            notes.iter().filter(|n| n.trashed).cloned().collect()
        })
    }

    pub fn get_note(&self, app: &AppHandle, id: &str) -> Option<Note> {
        self.with_notes(app, |notes| {
            notes.iter().find(|n| n.id == id).cloned()
        })
    }

    pub fn save_note(&self, app: &AppHandle, note: &Note) -> Result<(), String> {
        storage::save_note(app, note)?;
        self.invalidate_notes();
        Ok(())
    }

    pub fn delete_note(&self, app: &AppHandle, id: &str) -> Result<(), String> {
        storage::delete_note(app, id)?;
        self.invalidate_notes();
        Ok(())
    }

    pub fn invalidate_notes(&self) {
        let mut guard = self.notes.lock().unwrap();
        *guard = None;
    }

    // --- Reminders ---

    pub fn list_reminders(&self, app: &AppHandle, status: Option<String>) -> Vec<Reminder> {
        self.with_reminders(app, |reminders| {
            let mut filtered: Vec<Reminder> = reminders
                .iter()
                .filter(|r| {
                    if let Some(ref s) = status {
                        r.status == *s
                    } else {
                        true
                    }
                })
                .cloned()
                .collect();

            filtered.sort_by(|a, b| a.trigger_at.cmp(&b.trigger_at));
            filtered
        })
    }

    pub fn list_reminders_paginated(
        &self,
        app: &AppHandle,
        filter: &ReminderFilter,
    ) -> PaginatedResult<Reminder> {
        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit;
        let mut reminders = self.list_reminders(app, filter.status.clone());
        let total = reminders.len();
        if offset > 0 {
            reminders = reminders.into_iter().skip(offset).collect();
        }
        if let Some(limit) = limit {
            reminders.truncate(limit);
        }
        PaginatedResult {
            items: reminders,
            total,
        }
    }

    pub fn get_reminder(&self, app: &AppHandle, id: &str) -> Option<Reminder> {
        self.with_reminders(app, |reminders| {
            reminders.iter().find(|r| r.id == id).cloned()
        })
    }

    pub fn save_reminder(&self, app: &AppHandle, reminder: &Reminder) -> Result<(), String> {
        storage::save_reminder(app, reminder)?;
        self.invalidate_reminders();
        Ok(())
    }

    pub fn delete_reminder(&self, app: &AppHandle, id: &str) -> Result<(), String> {
        storage::delete_reminder(app, id)?;
        self.invalidate_reminders();
        Ok(())
    }

    pub fn invalidate_reminders(&self) {
        let mut guard = self.reminders.lock().unwrap();
        *guard = None;
    }
}
