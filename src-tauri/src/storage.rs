use crate::models::{Config, Note, NoteFilter, Reminder};
use std::fs;
use std::path::PathBuf;
use std::sync::Once;
use tauri::Manager;

static DIR_INIT: Once = Once::new();

fn data_dir(app_handle: &tauri::AppHandle) -> PathBuf {
    let dir = app_handle
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("data");
    DIR_INIT.call_once(|| {
        let _ = fs::create_dir_all(dir.join("notes"));
        let _ = fs::create_dir_all(dir.join("reminders"));
    });
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

    // Filter out trashed notes
    notes.retain(|n| !n.trashed);

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

pub fn list_all_notes(app_handle: &tauri::AppHandle) -> Vec<Note> {
    let dir = notes_dir(app_handle);
    fs::read_dir(&dir)
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
        .collect()
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

pub fn get_categories_and_tags(app_handle: &tauri::AppHandle) -> (Vec<String>, Vec<String>) {
    let notes = list_notes(app_handle, None);
    let mut categories = std::collections::HashSet::new();
    let mut tags = std::collections::HashSet::new();

    for note in &notes {
        if let Some(cat) = &note.category {
            categories.insert(cat.clone());
        }
        for tag in &note.tags {
            tags.insert(tag.clone());
        }
    }

    let mut categories: Vec<String> = categories.into_iter().collect();
    let mut tags: Vec<String> = tags.into_iter().collect();
    categories.sort();
    tags.sort();
    (categories, tags)
}

pub fn get_categories(app_handle: &tauri::AppHandle) -> Vec<String> {
    get_categories_and_tags(app_handle).0
}

pub fn get_tags(app_handle: &tauri::AppHandle) -> Vec<String> {
    get_categories_and_tags(app_handle).1
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

    #[test]
    fn test_note_migration_already_current() {
        let mut note = Note {
            id: "test".to_string(),
            title: "Test".to_string(),
            content: "".to_string(),
            category: None,
            tags: vec![],
            pinned: false,
            trashed: false,
            trashed_at: None,
            schema_version: 1,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        };
        migrate_note(&mut note);
        assert_eq!(note.schema_version, 1);
    }

    #[test]
    fn test_reminder_migration() {
        let mut reminder = Reminder {
            id: "test".to_string(),
            title: "Test".to_string(),
            description: None,
            note_id: None,
            trigger_at: "".to_string(),
            repeat: None,
            relative_minutes: None,
            status: "pending".to_string(),
            schema_version: 0,
            created_at: "".to_string(),
        };
        migrate_reminder(&mut reminder);
        assert_eq!(reminder.schema_version, 1);
    }

    #[test]
    fn test_reminder_migration_already_current() {
        let mut reminder = Reminder {
            id: "test".to_string(),
            title: "Test".to_string(),
            description: None,
            note_id: None,
            trigger_at: "".to_string(),
            repeat: None,
            relative_minutes: None,
            status: "pending".to_string(),
            schema_version: 1,
            created_at: "".to_string(),
        };
        migrate_reminder(&mut reminder);
        assert_eq!(reminder.schema_version, 1);
    }

    #[test]
    fn test_atomic_write_success() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.json");

        let content = r#"{"id":"1","title":"Test"}"#;
        let result = atomic_write(&file_path, content);
        assert!(result.is_ok());

        let written = fs::read_to_string(&file_path).unwrap();
        assert_eq!(written, content);

        // Verify temp file was cleaned up
        let tmp_path = file_path.with_extension("json.tmp");
        assert!(!tmp_path.exists());
    }

    #[test]
    fn test_atomic_write_overwrites_existing() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.json");

        // Write initial content
        fs::write(&file_path, "old content").unwrap();

        // Overwrite with atomic write
        let new_content = r#"{"id":"1","title":"Updated"}"#;
        let result = atomic_write(&file_path, new_content);
        assert!(result.is_ok());

        let written = fs::read_to_string(&file_path).unwrap();
        assert_eq!(written, new_content);
    }

    #[test]
    fn test_atomic_write_invalid_path() {
        let result = atomic_write(
            std::path::Path::new("/nonexistent/dir/test.json"),
            "content",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = Config {
            theme: "light".to_string(),
            shortcut: "Ctrl+Shift+N".to_string(),
            autostart: false,
            check_updates: true,
            window_width: 800,
            window_height: 600,
        };

        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.theme, "light");
        assert_eq!(parsed.shortcut, "Ctrl+Shift+N");
        assert!(!parsed.autostart);
        assert!(parsed.check_updates);
        assert_eq!(parsed.window_width, 800);
        assert_eq!(parsed.window_height, 600);
    }

    #[test]
    fn test_note_json_structure() {
        let note = Note {
            id: "abc".to_string(),
            title: "Title".to_string(),
            content: "Content".to_string(),
            category: Some("Cat".to_string()),
            tags: vec!["t1".to_string(), "t2".to_string()],
            pinned: true,
            trashed: false,
            trashed_at: None,
            schema_version: 1,
            created_at: "2026-05-28T10:00:00Z".to_string(),
            updated_at: "2026-05-28T10:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&note).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Verify JSON structure
        assert!(value.is_object());
        assert_eq!(value["id"].as_str().unwrap(), "abc");
        assert_eq!(value["title"].as_str().unwrap(), "Title");
        assert_eq!(value["category"].as_str().unwrap(), "Cat");
        assert!(value["tags"].is_array());
        assert_eq!(value["tags"].as_array().unwrap().len(), 2);
        assert!(value["pinned"].as_bool().unwrap());
    }

    #[test]
    fn test_note_filter_deserialization() {
        let json = r#"{"search":"test","category":"Work","tag":"urgent"}"#;
        let filter: NoteFilter = serde_json::from_str(json).unwrap();
        assert_eq!(filter.search, Some("test".to_string()));
        assert_eq!(filter.category, Some("Work".to_string()));
        assert_eq!(filter.tag, Some("urgent".to_string()));
    }

    #[test]
    fn test_note_filter_empty_json() {
        let json = r#"{}"#;
        let filter: NoteFilter = serde_json::from_str(json).unwrap();
        assert!(filter.search.is_none());
        assert!(filter.category.is_none());
        assert!(filter.tag.is_none());
    }
}
