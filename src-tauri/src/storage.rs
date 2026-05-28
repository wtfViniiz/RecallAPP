use crate::models::{Config, CustomTemplate, Note, NoteFilter, NoteVersion, Reminder};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Once;
use tauri::Manager;
use uuid::Uuid;

static DIR_INIT: Once = Once::new();
static DIR_INIT_OK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

fn data_dir(app_handle: &tauri::AppHandle) -> PathBuf {
    let dir = match app_handle.path().app_data_dir() {
        Ok(path) => path.join("data"),
        Err(e) => {
            let fallback = std::env::var("LOCALAPPDATA")
                .or_else(|_| std::env::var("XDG_DATA_HOME"))
                .map(PathBuf::from)
                .unwrap_or_else(|_| app_handle.path().home_dir().unwrap_or_else(|_| PathBuf::from(".")))
                .join("Recall").join("data");
            eprintln!("[Recall] Erro ao obter diretorio de dados: {}. Usando fallback: {:?}", e, fallback);
            fallback
        }
    };
    DIR_INIT.call_once(|| {
        let mut ok = true;
        if let Err(e) = fs::create_dir_all(dir.join("notes")) {
            eprintln!("[Recall] Erro ao criar diretorio notes: {}", e);
            ok = false;
        }
        if let Err(e) = fs::create_dir_all(dir.join("reminders")) {
            eprintln!("[Recall] Erro ao criar diretorio reminders: {}", e);
            ok = false;
        }
        if let Err(e) = fs::create_dir_all(dir.join("versions")) {
            eprintln!("[Recall] Erro ao criar diretorio versions: {}", e);
            ok = false;
        }
        DIR_INIT_OK.store(ok, std::sync::atomic::Ordering::Relaxed);
    });
    if !DIR_INIT_OK.load(std::sync::atomic::Ordering::Relaxed) {
        eprintln!("[Recall] Aviso: diretorios de dados nao foram criados corretamente");
    }
    dir
}


/// Ensure data directories exist (for test and non-AppHandle usage)
pub fn ensure_dirs(data_dir: &Path) {
    let _ = fs::create_dir_all(data_dir.join("notes"));
    let _ = fs::create_dir_all(data_dir.join("reminders"));
    let _ = fs::create_dir_all(data_dir.join("versions"));
}

/// Atomic write: writes to .tmp then renames to prevent corruption
fn atomic_write(path: &std::path::Path, content: &str) -> Result<(), String> {
    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, content).map_err(|e| e.to_string())?;
    if let Err(e) = fs::rename(&tmp_path, path) {
        // Clean up orphaned .tmp file on rename failure
        let _ = fs::remove_file(&tmp_path);
        return Err(e.to_string());
    }
    Ok(())
}

// --- Notes (AppHandle variants - delegate to _at) ---

pub fn list_notes(app_handle: &tauri::AppHandle, filter: Option<NoteFilter>) -> Vec<Note> {
    list_notes_at(&data_dir(app_handle), filter)
}

pub fn list_all_notes(app_handle: &tauri::AppHandle) -> Vec<Note> {
    list_all_notes_at(&data_dir(app_handle))
}

pub fn list_trashed_notes(app_handle: &tauri::AppHandle) -> Vec<Note> {
    list_trashed_notes_at(&data_dir(app_handle))
}

pub fn get_note(app_handle: &tauri::AppHandle, id: &str) -> Option<Note> {
    get_note_at(&data_dir(app_handle), id)
}

pub fn save_note(app_handle: &tauri::AppHandle, note: &Note) -> Result<(), String> {
    save_note_at(&data_dir(app_handle), note)
}

pub fn delete_note(app_handle: &tauri::AppHandle, id: &str) -> Result<(), String> {
    delete_note_at(&data_dir(app_handle), id)
}

// --- Reminders (AppHandle variants - delegate to _at) ---

pub fn list_reminders(app_handle: &tauri::AppHandle, status: Option<String>) -> Vec<Reminder> {
    list_reminders_at(&data_dir(app_handle), status)
}

pub fn get_reminder(app_handle: &tauri::AppHandle, id: &str) -> Option<Reminder> {
    get_reminder_at(&data_dir(app_handle), id)
}

pub fn save_reminder(app_handle: &tauri::AppHandle, reminder: &Reminder) -> Result<(), String> {
    save_reminder_at(&data_dir(app_handle), reminder)
}

pub fn delete_reminder(app_handle: &tauri::AppHandle, id: &str) -> Result<(), String> {
    delete_reminder_at(&data_dir(app_handle), id)
}

// --- Notes (_at variants - take Path directly, testable without AppHandle) ---

pub fn list_notes_at(data_dir: &Path, filter: Option<NoteFilter>) -> Vec<Note> {
    let dir = data_dir.join("notes");
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
            if migrate_note(&mut note) {
                let _ = save_note_at(data_dir, &note);
            }
            Some(note)
        })
        .collect();

    notes.retain(|n| !n.trashed);

    if let Some(ref filter) = filter {
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
            .then_with(|| match (a.position, b.position) {
                (Some(pa), Some(pb)) => pa.cmp(&pb),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => b.updated_at.cmp(&a.updated_at),
            })
    });

    // Apply pagination if requested
    let offset = filter.as_ref().and_then(|f| f.offset).unwrap_or(0);
    let limit = filter.as_ref().and_then(|f| f.limit);
    if offset > 0 && offset < notes.len() {
        notes = notes.into_iter().skip(offset).collect();
    } else if offset >= notes.len() {
        notes.clear();
    }
    if let Some(limit) = limit {
        notes.truncate(limit);
    }

    notes
}

pub fn list_all_notes_at(data_dir: &Path) -> Vec<Note> {
    let dir = data_dir.join("notes");
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
            if migrate_note(&mut note) {
                let _ = save_note_at(data_dir, &note);
            }
            Some(note)
        })
        .collect()
}

pub fn list_trashed_notes_at(data_dir: &Path) -> Vec<Note> {
    let dir = data_dir.join("notes");
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
            if migrate_note(&mut note) {
                let _ = save_note_at(data_dir, &note);
            }
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

pub fn get_note_at(data_dir: &Path, id: &str) -> Option<Note> {
    let path = data_dir.join("notes").join(format!("{}.json", id));
    let content = fs::read_to_string(&path).ok()?;
    let mut note: Note = serde_json::from_str(&content).ok()?;
    if migrate_note(&mut note) {
        let _ = save_note_at(data_dir, &note);
    }
    Some(note)
}

pub fn save_note_at(data_dir: &Path, note: &Note) -> Result<(), String> {
    let path = data_dir.join("notes").join(format!("{}.json", note.id));
    let json = serde_json::to_string_pretty(note).map_err(|e| e.to_string())?;
    atomic_write(&path, &json)
}

pub fn delete_note_at(data_dir: &Path, id: &str) -> Result<(), String> {
    let path = data_dir.join("notes").join(format!("{}.json", id));
    fs::remove_file(path).map_err(|e| e.to_string())
}

// --- Note Versions ---

pub fn save_note_version(app_handle: &tauri::AppHandle, note: &Note) -> Result<(), String> {
    save_note_version_at(&data_dir(app_handle), note)
}

pub fn save_note_version_at(data_dir: &Path, note: &Note) -> Result<(), String> {
    let versions_dir = data_dir.join("versions").join(&note.id);
    fs::create_dir_all(&versions_dir).map_err(|e| e.to_string())?;

    let version = NoteVersion {
        id: Uuid::new_v4().to_string(),
        note_id: note.id.clone(),
        title: note.title.clone(),
        content: note.content.clone(),
        category: note.category.clone(),
        tags: note.tags.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    let path = versions_dir.join(format!("{}.json", version.id));
    let json = serde_json::to_string_pretty(&version).map_err(|e| e.to_string())?;
    atomic_write(&path, &json)?;

    // Keep only last 20 versions
    let mut versions = list_note_versions_at(data_dir, &note.id);
    if versions.len() > 20 {
        versions.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        for v in &versions[..versions.len() - 20] {
            let _ = fs::remove_file(versions_dir.join(format!("{}.json", v.id)));
        }
    }

    Ok(())
}

pub fn list_note_versions(app_handle: &tauri::AppHandle, note_id: &str) -> Vec<NoteVersion> {
    list_note_versions_at(&data_dir(app_handle), note_id)
}

pub fn list_note_versions_at(data_dir: &Path, note_id: &str) -> Vec<NoteVersion> {
    let versions_dir = data_dir.join("versions").join(note_id);
    let mut versions: Vec<NoteVersion> = fs::read_dir(&versions_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.path().extension().map_or(false, |e| e == "tmp") {
                return None;
            }
            let content = fs::read_to_string(entry.path()).ok()?;
            serde_json::from_str(&content).ok()
        })
        .collect();

    versions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    versions
}

pub fn restore_note_version(app_handle: &tauri::AppHandle, note_id: &str, version_id: &str) -> Result<Note, String> {
    restore_note_version_at(&data_dir(app_handle), note_id, version_id)
}

pub fn restore_note_version_at(data_dir: &Path, note_id: &str, version_id: &str) -> Result<Note, String> {
    let version_path = data_dir.join("versions").join(note_id).join(format!("{}.json", version_id));
    let content = fs::read_to_string(&version_path).map_err(|e| format!("Versao nao encontrada: {}", e))?;
    let version: NoteVersion = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    let mut note = get_note_at(data_dir, note_id).ok_or("Nota nao encontrada")?;
    // Save current state as a version before restoring
    let _ = save_note_version_at(data_dir, &note);

    note.title = version.title;
    note.content = version.content;
    note.category = version.category;
    note.tags = version.tags;
    note.updated_at = chrono::Utc::now().to_rfc3339();
    save_note_at(data_dir, &note)?;

    Ok(note)
}

// --- Custom Templates ---

pub fn load_custom_templates(app_handle: &tauri::AppHandle) -> Vec<CustomTemplate> {
    load_custom_templates_at(&data_dir(app_handle))
}

pub fn load_custom_templates_at(data_dir: &Path) -> Vec<CustomTemplate> {
    let path = data_dir.join("templates.json");
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn save_custom_templates(app_handle: &tauri::AppHandle, templates: &[CustomTemplate]) -> Result<(), String> {
    save_custom_templates_at(&data_dir(app_handle), templates)
}

pub fn save_custom_templates_at(data_dir: &Path, templates: &[CustomTemplate]) -> Result<(), String> {
    let path = data_dir.join("templates.json");
    let json = serde_json::to_string_pretty(templates).map_err(|e| e.to_string())?;
    atomic_write(&path, &json)
}

// --- Reminders (_at variants) ---

pub fn list_reminders_at(data_dir: &Path, status: Option<String>) -> Vec<Reminder> {
    let dir = data_dir.join("reminders");
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
            if migrate_reminder(&mut reminder) {
                let _ = save_reminder_at(data_dir, &reminder);
            }
            Some(reminder)
        })
        .collect();

    if let Some(status) = status {
        reminders.retain(|r| r.status == status);
    }

    reminders.sort_by(|a, b| a.trigger_at.cmp(&b.trigger_at));
    reminders
}

pub fn get_reminder_at(data_dir: &Path, id: &str) -> Option<Reminder> {
    let path = data_dir.join("reminders").join(format!("{}.json", id));
    let content = fs::read_to_string(&path).ok()?;
    let mut reminder: Reminder = serde_json::from_str(&content).ok()?;
    if migrate_reminder(&mut reminder) {
        let _ = save_reminder_at(data_dir, &reminder);
    }
    Some(reminder)
}

pub fn save_reminder_at(data_dir: &Path, reminder: &Reminder) -> Result<(), String> {
    let path = data_dir.join("reminders").join(format!("{}.json", reminder.id));
    let json = serde_json::to_string_pretty(reminder).map_err(|e| e.to_string())?;
    atomic_write(&path, &json)
}

pub fn delete_reminder_at(data_dir: &Path, id: &str) -> Result<(), String> {
    let path = data_dir.join("reminders").join(format!("{}.json", id));
    fs::remove_file(path).map_err(|e| e.to_string())
}

// --- Config (AppHandle variants) ---

pub fn load_config(app_handle: &tauri::AppHandle) -> Config {
    load_config_at(&data_dir(app_handle))
}

pub fn save_config(app_handle: &tauri::AppHandle, config: &Config) -> Result<(), String> {
    save_config_at(&data_dir(app_handle), config)
}

// --- Config (_at variants) ---

pub fn load_config_at(data_dir: &Path) -> Config {
    let path = data_dir.join("config.json");
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

pub fn save_config_at(data_dir: &Path, config: &Config) -> Result<(), String> {
    let path = data_dir.join("config.json");
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

fn migrate_note(note: &mut Note) -> bool {
    let original = note.schema_version;
    if note.schema_version < 1 {
        note.schema_version = 1;
    }
    note.schema_version != original
}

fn migrate_reminder(reminder: &mut Reminder) -> bool {
    let original = reminder.schema_version;
    if reminder.schema_version < 1 {
        reminder.schema_version = 1;
    }
    reminder.schema_version != original
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
            position: None,
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
            position: None,
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
            position: None,
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
            position: None,
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
