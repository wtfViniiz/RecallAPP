use crate::models::{Config, Note, NoteFilter, Reminder};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

fn data_dir(app_handle: &tauri::AppHandle) -> PathBuf {
    let dir = app_handle
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("data");
    let _ = fs::create_dir_all(dir.join("notes"));
    let _ = fs::create_dir_all(dir.join("reminders"));
    dir
}

fn notes_dir(app_handle: &tauri::AppHandle) -> PathBuf {
    data_dir(app_handle).join("notes")
}

fn reminders_dir(app_handle: &tauri::AppHandle) -> PathBuf {
    data_dir(app_handle).join("reminders")
}

fn config_path(app_handle: &tauri::AppHandle) -> PathBuf {
    data_dir(app_handle).join("config.json")
}

/// Atomic write: writes to .tmp then renames to prevent corruption
fn atomic_write(path: &std::path::Path, content: &str) -> Result<(), String> {
    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, content).map_err(|e| e.to_string())?;
    fs::rename(&tmp_path, path).map_err(|e| e.to_string())?;
    Ok(())
}

// --- Notes ---

pub fn list_notes(app_handle: &tauri::AppHandle, filter: Option<NoteFilter>) -> Vec<Note> {
    let dir = notes_dir(app_handle);
    let mut notes: Vec<Note> = fs::read_dir(&dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.path().extension().map_or(false, |e| e == "tmp") {
                return None;
            }
            let content = fs::read_to_string(entry.path()).ok()?;
            let mut note: Note = serde_json::from_str(&content).ok()?;
            migrate_note(&mut note);
            Some(note)
        })
        .collect();

    if let Some(filter) = filter {
        if let Some(search) = &filter.search {
            let search_lower = search.to_lowercase();
            notes.retain(|n| {
                n.title.to_lowercase().contains(&search_lower)
                    || n.content.to_lowercase().contains(&search_lower)
            });
        }
        if let Some(category) = &filter.category {
            notes.retain(|n| n.category.as_deref() == Some(category));
        }
        if let Some(tag) = &filter.tag {
            notes.retain(|n| n.tags.contains(tag));
        }
    }

    notes.sort_by(|a, b| {
        a.pinned
            .cmp(&b.pinned)
            .reverse()
            .then_with(|| b.updated_at.cmp(&a.updated_at))
    });

    notes
}

pub fn list_trashed_notes(app_handle: &tauri::AppHandle) -> Vec<Note> {
    let dir = notes_dir(app_handle);
    let mut notes: Vec<Note> = fs::read_dir(&dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.path().extension().map_or(false, |e| e == "tmp") {
                return None;
            }
            let content = fs::read_to_string(entry.path()).ok()?;
            let mut note: Note = serde_json::from_str(&content).ok()?;
            migrate_note(&mut note);
            Some(note)
        })
        .filter(|n| n.trashed)
        .collect();

    notes.sort_by(|a, b| {
        b.trashed_at
            .as_deref()
            .unwrap_or("")
            .cmp(a.trashed_at.as_deref().unwrap_or(""))
    });

    notes
}

pub fn get_note(app_handle: &tauri::AppHandle, id: &str) -> Option<Note> {
    let path = notes_dir(app_handle).join(format!("{}.json", id));
    let content = fs::read_to_string(path).ok()?;
    let mut note: Note = serde_json::from_str(&content).ok()?;
    migrate_note(&mut note);
    Some(note)
}

pub fn save_note(app_handle: &tauri::AppHandle, note: &Note) -> Result<(), String> {
    let path = notes_dir(app_handle).join(format!("{}.json", note.id));
    let json = serde_json::to_string_pretty(note).map_err(|e| e.to_string())?;
    atomic_write(&path, &json)
}

pub fn delete_note(app_handle: &tauri::AppHandle, id: &str) -> Result<(), String> {
    let path = notes_dir(app_handle).join(format!("{}.json", id));
    fs::remove_file(path).map_err(|e| e.to_string())
}

// --- Reminders ---

pub fn list_reminders(app_handle: &tauri::AppHandle, status: Option<String>) -> Vec<Reminder> {
    let dir = reminders_dir(app_handle);
    let mut reminders: Vec<Reminder> = fs::read_dir(&dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.path().extension().map_or(false, |e| e == "tmp") {
                return None;
            }
            let content = fs::read_to_string(entry.path()).ok()?;
            let mut reminder: Reminder = serde_json::from_str(&content).ok()?;
            migrate_reminder(&mut reminder);
            Some(reminder)
        })
        .collect();

    if let Some(status) = status {
        reminders.retain(|r| r.status == status);
    }

    reminders.sort_by(|a, b| a.trigger_at.cmp(&b.trigger_at));
    reminders
}

pub fn get_reminder(app_handle: &tauri::AppHandle, id: &str) -> Option<Reminder> {
    let path = reminders_dir(app_handle).join(format!("{}.json", id));
    let content = fs::read_to_string(path).ok()?;
    let mut reminder: Reminder = serde_json::from_str(&content).ok()?;
    migrate_reminder(&mut reminder);
    Some(reminder)
}

pub fn save_reminder(app_handle: &tauri::AppHandle, reminder: &Reminder) -> Result<(), String> {
    let path = reminders_dir(app_handle).join(format!("{}.json", reminder.id));
    let json = serde_json::to_string_pretty(reminder).map_err(|e| e.to_string())?;
    atomic_write(&path, &json)
}

pub fn delete_reminder(app_handle: &tauri::AppHandle, id: &str) -> Result<(), String> {
    let path = reminders_dir(app_handle).join(format!("{}.json", id));
    fs::remove_file(path).map_err(|e| e.to_string())
}

// --- Config ---

pub fn load_config(app_handle: &tauri::AppHandle) -> Config {
    let path = config_path(app_handle);
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

pub fn save_config(app_handle: &tauri::AppHandle, config: &Config) -> Result<(), String> {
    let path = config_path(app_handle);
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    atomic_write(&path, &json)
}

// --- Helpers ---

pub fn get_categories(app_handle: &tauri::AppHandle) -> Vec<String> {
    let notes = list_notes(app_handle, None);
    let mut categories: Vec<String> = notes
        .iter()
        .filter_map(|n| n.category.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    categories.sort();
    categories
}

pub fn get_tags(app_handle: &tauri::AppHandle) -> Vec<String> {
    let notes = list_notes(app_handle, None);
    let mut tags: Vec<String> = notes
        .iter()
        .flat_map(|n| n.tags.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    tags.sort();
    tags
}

// --- Migration ---

fn migrate_note(note: &mut Note) {
    if note.schema_version < 1 {
        note.schema_version = 1;
    }
}

fn migrate_reminder(reminder: &mut Reminder) {
    if reminder.schema_version < 1 {
        reminder.schema_version = 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Config;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.theme, "dark");
        assert_eq!(config.shortcut, "Ctrl+Alt+x");
        assert!(config.autostart);
    }

    #[test]
    fn test_note_serialization() {
        let note = Note {
            id: "test-id".to_string(),
            title: "Test Note".to_string(),
            content: "Content".to_string(),
            category: Some("Work".to_string()),
            tags: vec!["tag1".to_string()],
            pinned: false,
            trashed: false,
            trashed_at: None,
            schema_version: 1,
            created_at: "2026-05-27T14:00:00Z".to_string(),
            updated_at: "2026-05-27T14:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&note).unwrap();
        let parsed: Note = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "test-id");
        assert_eq!(parsed.schema_version, 1);
    }

    #[test]
    fn test_reminder_serialization() {
        let reminder = Reminder {
            id: "test-id".to_string(),
            title: "Test Reminder".to_string(),
            description: Some("Desc".to_string()),
            note_id: None,
            trigger_at: "2026-05-28T10:00:00Z".to_string(),
            repeat: Some("daily".to_string()),
            relative_minutes: None,
            status: "pending".to_string(),
            schema_version: 1,
            created_at: "2026-05-27T14:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&reminder).unwrap();
        let parsed: Reminder = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.repeat, Some("daily".to_string()));
    }

    #[test]
    fn test_note_migration() {
        let mut note = Note {
            id: "test".to_string(),
            title: "Test".to_string(),
            content: "".to_string(),
            category: None,
            tags: vec![],
            pinned: false,
            trashed: false,
            trashed_at: None,
            schema_version: 0,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        };
        migrate_note(&mut note);
        assert_eq!(note.schema_version, 1);
    }
}
