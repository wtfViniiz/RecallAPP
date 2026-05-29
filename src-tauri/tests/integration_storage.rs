use recall::models::*;
use recall::storage;
use std::path::PathBuf;
use tempfile::TempDir;

fn setup() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let data_dir = dir.path().join("data");
    storage::ensure_dirs(&data_dir);
    (dir, data_dir)
}

fn make_note(id: &str, title: &str) -> Note {
    Note {
        id: id.to_string(),
        title: title.to_string(),
        content: "content".to_string(),
        category: None,
        tags: vec![],
        pinned: false,
        trashed: false,
        trashed_at: None,
        position: None,
            temporary: false,
            expires_at: None,
        schema_version: 1,
        created_at: "2026-05-28T10:00:00Z".to_string(),
        updated_at: "2026-05-28T10:00:00Z".to_string(),
    }
}

// --- Config roundtrip ---

#[test]
fn test_config_save_and_load_roundtrip() {
    let (_tmp, data_dir) = setup();
    let config = Config {
        theme: "dark".to_string(),
        window_width: 900,
        window_height: 700,
        shortcut: "Ctrl+Alt+N".to_string(),
        new_note_shortcut: String::new(),
        autostart: true,
        check_updates: true,
        font_size: 14,
    };
    storage::save_config_at(&data_dir, &config).unwrap();
    let loaded = storage::load_config_at(&data_dir);
    assert_eq!(loaded.theme, "dark");
    assert_eq!(loaded.window_width, 900);
    assert_eq!(loaded.window_height, 700);
    assert_eq!(loaded.shortcut, "Ctrl+Alt+N");
    assert_eq!(loaded.autostart, true);
    assert_eq!(loaded.check_updates, true);
}

#[test]
fn test_config_missing_file_returns_defaults() {
    let (_tmp, data_dir) = setup();
    let config = storage::load_config_at(&data_dir);
    assert_eq!(config.theme, "dark");
    assert_eq!(config.window_width, 500);
    assert_eq!(config.window_height, 650);
}

// --- Atomic write tested indirectly via save_note ---

#[test]
fn test_save_note_atomic_no_tmp_left() {
    let (_tmp, data_dir) = setup();
    let note = make_note("n1", "Title");
    storage::save_note_at(&data_dir, &note).unwrap();
    let tmp_path = data_dir.join("notes").join("n1.json.tmp");
    assert!(!tmp_path.exists(), ".tmp file should not exist after save");
}

#[test]
fn test_save_note_json_is_valid() {
    let (_tmp, data_dir) = setup();
    let note = make_note("n1", "Title");
    storage::save_note_at(&data_dir, &note).unwrap();
    let path = data_dir.join("notes").join("n1.json");
    let content = std::fs::read_to_string(&path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(parsed.is_object());
    assert_eq!(parsed["id"].as_str().unwrap(), "n1");
}

// --- Note migration (tested indirectly via load) ---

#[test]
fn test_note_schema_version_zero_gets_migrated_on_load() {
    let (_tmp, data_dir) = setup();
    // Write a note JSON with schema_version 0 directly
    let note_json = r#"{
        "id": "n-migrate",
        "title": "Old Note",
        "content": "content",
        "tags": [],
        "pinned": false,
        "trashed": false,
        "schema_version": 0,
        "created_at": "2026-05-28T10:00:00Z",
        "updated_at": "2026-05-28T10:00:00Z"
    }"#;
    let notes_dir = data_dir.join("notes");
    std::fs::create_dir_all(&notes_dir).unwrap();
    std::fs::write(notes_dir.join("n-migrate.json"), note_json).unwrap();

    let loaded = storage::get_note_at(&data_dir, "n-migrate").unwrap();
    // After migration, schema_version should be 1
    assert_eq!(loaded.schema_version, 1);
}

// --- Note sorting ---

#[test]
fn test_note_sort_pinned_first() {
    let (_tmp, data_dir) = setup();
    let mut n1 = make_note("n1", "Not pinned");
    n1.pinned = false;
    n1.updated_at = "2026-05-28T12:00:00Z".to_string();
    let mut n2 = make_note("n2", "Pinned");
    n2.pinned = true;
    n2.updated_at = "2026-05-28T10:00:00Z".to_string();
    storage::save_note_at(&data_dir, &n1).unwrap();
    storage::save_note_at(&data_dir, &n2).unwrap();

    let notes = storage::list_notes_at(&data_dir, None);
    assert_eq!(notes[0].id, "n2");
    assert_eq!(notes[1].id, "n1");
}

#[test]
fn test_note_sort_by_position_when_set() {
    let (_tmp, data_dir) = setup();
    let mut n1 = make_note("n1", "Pos 10");
    n1.position = Some(10);
    n1.updated_at = "2026-05-28T12:00:00Z".to_string();
    let mut n2 = make_note("n2", "Pos 1");
    n2.position = Some(1);
    n2.updated_at = "2026-05-28T10:00:00Z".to_string();
    storage::save_note_at(&data_dir, &n1).unwrap();
    storage::save_note_at(&data_dir, &n2).unwrap();

    let notes = storage::list_notes_at(&data_dir, None);
    assert_eq!(notes[0].id, "n2");
    assert_eq!(notes[1].id, "n1");
}

#[test]
fn test_note_sort_position_none_goes_after_positioned() {
    let (_tmp, data_dir) = setup();
    let mut n1 = make_note("n1", "No position");
    n1.position = None;
    n1.updated_at = "2026-05-28T12:00:00Z".to_string();
    let mut n2 = make_note("n2", "Has position");
    n2.position = Some(5);
    n2.updated_at = "2026-05-28T10:00:00Z".to_string();
    storage::save_note_at(&data_dir, &n1).unwrap();
    storage::save_note_at(&data_dir, &n2).unwrap();

    let notes = storage::list_notes_at(&data_dir, None);
    assert_eq!(notes[0].id, "n2");
    assert_eq!(notes[1].id, "n1");
}

// --- Note filtering ---

#[test]
fn test_list_notes_excludes_trashed() {
    let (_tmp, data_dir) = setup();
    let mut n1 = make_note("n1", "Active");
    n1.trashed = false;
    let mut n2 = make_note("n2", "Trashed");
    n2.trashed = true;
    storage::save_note_at(&data_dir, &n1).unwrap();
    storage::save_note_at(&data_dir, &n2).unwrap();

    let notes = storage::list_notes_at(&data_dir, None);
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].id, "n1");
}

#[test]
fn test_list_trashed_notes_only() {
    let (_tmp, data_dir) = setup();
    let mut n1 = make_note("n1", "Active");
    n1.trashed = false;
    let mut n2 = make_note("n2", "Trashed");
    n2.trashed = true;
    storage::save_note_at(&data_dir, &n1).unwrap();
    storage::save_note_at(&data_dir, &n2).unwrap();

    let trashed = storage::list_trashed_notes_at(&data_dir);
    assert_eq!(trashed.len(), 1);
    assert_eq!(trashed[0].id, "n2");
}

#[test]
fn test_list_notes_filter_by_category() {
    let (_tmp, data_dir) = setup();
    let mut n1 = make_note("n1", "Work note");
    n1.category = Some("Work".to_string());
    let mut n2 = make_note("n2", "Personal note");
    n2.category = Some("Personal".to_string());
    storage::save_note_at(&data_dir, &n1).unwrap();
    storage::save_note_at(&data_dir, &n2).unwrap();

    let filter = NoteFilter {
        search: None,
        category: Some("Work".to_string()),
        tag: None,
        offset: None,
        limit: None,
    };
    let notes = storage::list_notes_at(&data_dir, Some(filter));
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].id, "n1");
}

#[test]
fn test_list_notes_filter_by_tag() {
    let (_tmp, data_dir) = setup();
    let mut n1 = make_note("n1", "Tagged");
    n1.tags = vec!["rust".to_string(), "code".to_string()];
    let mut n2 = make_note("n2", "Other");
    n2.tags = vec!["design".to_string()];
    storage::save_note_at(&data_dir, &n1).unwrap();
    storage::save_note_at(&data_dir, &n2).unwrap();

    let filter = NoteFilter {
        search: None,
        category: None,
        tag: Some("rust".to_string()),
        offset: None,
        limit: None,
    };
    let notes = storage::list_notes_at(&data_dir, Some(filter));
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].id, "n1");
}

#[test]
fn test_list_notes_filter_by_search() {
    let (_tmp, data_dir) = setup();
    let mut n1 = make_note("n1", "Rust programming");
    n1.content = "Learning Rust".to_string();
    let mut n2 = make_note("n2", "Python guide");
    n2.content = "Python basics".to_string();
    storage::save_note_at(&data_dir, &n1).unwrap();
    storage::save_note_at(&data_dir, &n2).unwrap();

    let filter = NoteFilter {
        search: Some("rust".to_string()),
        category: None,
        tag: None,
        offset: None,
        limit: None,
    };
    let notes = storage::list_notes_at(&data_dir, Some(filter));
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].id, "n1");
}

// --- get_note ---

#[test]
fn test_get_note_returns_none_for_missing() {
    let (_tmp, data_dir) = setup();
    let result = storage::get_note_at(&data_dir, "nonexistent");
    assert!(result.is_none());
}

#[test]
fn test_get_note_returns_note() {
    let (_tmp, data_dir) = setup();
    let note = make_note("n1", "Title");
    storage::save_note_at(&data_dir, &note).unwrap();
    let loaded = storage::get_note_at(&data_dir, "n1").unwrap();
    assert_eq!(loaded.title, "Title");
}

// --- delete_note ---

#[test]
fn test_delete_note_removes_file() {
    let (_tmp, data_dir) = setup();
    let note = make_note("n1", "Title");
    storage::save_note_at(&data_dir, &note).unwrap();
    assert!(storage::get_note_at(&data_dir, "n1").is_some());
    storage::delete_note_at(&data_dir, "n1").unwrap();
    assert!(storage::get_note_at(&data_dir, "n1").is_none());
}

// --- Categories and tags extraction ---

#[test]
fn test_categories_and_tags_from_notes() {
    let (_tmp, data_dir) = setup();
    let mut n1 = make_note("n1", "A");
    n1.category = Some("Work".to_string());
    n1.tags = vec!["rust".to_string(), "code".to_string()];
    let mut n2 = make_note("n2", "B");
    n2.category = Some("Personal".to_string());
    n2.tags = vec!["design".to_string(), "rust".to_string()];
    storage::save_note_at(&data_dir, &n1).unwrap();
    storage::save_note_at(&data_dir, &n2).unwrap();

    let notes = storage::list_notes_at(&data_dir, None);
    let mut categories: Vec<String> = notes.iter().filter_map(|n| n.category.clone()).collect();
    categories.sort();
    categories.dedup();
    assert_eq!(categories, vec!["Personal", "Work"]);

    let mut tags: Vec<String> = notes.iter().flat_map(|n| n.tags.clone()).collect();
    tags.sort();
    tags.dedup();
    assert_eq!(tags, vec!["code", "design", "rust"]);
}
