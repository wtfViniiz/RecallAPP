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
        schema_version: 1,
        created_at: "2026-05-28T10:00:00Z".to_string(),
        updated_at: "2026-05-28T10:00:00Z".to_string(),
    }
}

// --- Storage edge cases ---

#[test]
fn test_list_notes_empty_dir() {
    let (_tmp, data_dir) = setup();
    let notes = storage::list_notes_at(&data_dir, None);
    assert_eq!(notes.len(), 0);
}

#[test]
fn test_list_all_notes_includes_trashed() {
    let (_tmp, data_dir) = setup();
    let mut n1 = make_note("n1", "Active");
    n1.trashed = false;
    let mut n2 = make_note("n2", "Trashed");
    n2.trashed = true;
    storage::save_note_at(&data_dir, &n1).unwrap();
    storage::save_note_at(&data_dir, &n2).unwrap();

    let all = storage::list_all_notes_at(&data_dir);
    assert_eq!(all.len(), 2);
}

#[test]
fn test_save_note_overwrites_existing() {
    let (_tmp, data_dir) = setup();
    let mut note = make_note("n1", "Original");
    storage::save_note_at(&data_dir, &note).unwrap();

    note.title = "Updated".to_string();
    storage::save_note_at(&data_dir, &note).unwrap();

    let loaded = storage::get_note_at(&data_dir, "n1").unwrap();
    assert_eq!(loaded.title, "Updated");
}

#[test]
fn test_note_with_empty_tags() {
    let (_tmp, data_dir) = setup();
    let note = make_note("n1", "No tags");
    storage::save_note_at(&data_dir, &note).unwrap();
    let loaded = storage::get_note_at(&data_dir, "n1").unwrap();
    assert!(loaded.tags.is_empty());
}

#[test]
fn test_note_with_empty_category() {
    let (_tmp, data_dir) = setup();
    let note = make_note("n1", "No category");
    storage::save_note_at(&data_dir, &note).unwrap();
    let loaded = storage::get_note_at(&data_dir, "n1").unwrap();
    assert!(loaded.category.is_none());
}

#[test]
fn test_trashed_at_preserved() {
    let (_tmp, data_dir) = setup();
    let mut note = make_note("n1", "Trashed");
    note.trashed = true;
    note.trashed_at = Some("2026-05-28T15:00:00Z".to_string());
    storage::save_note_at(&data_dir, &note).unwrap();
    let loaded = storage::get_note_at(&data_dir, "n1").unwrap();
    assert_eq!(loaded.trashed_at, Some("2026-05-28T15:00:00Z".to_string()));
}

// --- Version: concurrent save simulation ---

#[test]
fn test_rapid_version_saves() {
    let (_tmp, data_dir) = setup();
    let note = make_note("n1", "Title");
    storage::save_note_at(&data_dir, &note).unwrap();

    // Save 5 versions rapidly (no sleep)
    for i in 0..5 {
        let mut v = note.clone();
        v.title = format!("V{}", i);
        storage::save_note_version_at(&data_dir, &v).unwrap();
    }

    let versions = storage::list_note_versions_at(&data_dir, "n1");
    assert_eq!(versions.len(), 5);
}

// --- Version: restore preserves updated_at ---

#[test]
fn test_restore_updates_timestamp() {
    let (_tmp, data_dir) = setup();
    let mut note = make_note("n1", "Original");
    note.updated_at = "2026-05-28T10:00:00Z".to_string();
    storage::save_note_at(&data_dir, &note).unwrap();
    storage::save_note_version_at(&data_dir, &note).unwrap();

    let versions = storage::list_note_versions_at(&data_dir, "n1");
    let version_id = versions[0].id.clone();

    // Modify
    note.title = "Modified".to_string();
    note.updated_at = "2026-05-28T12:00:00Z".to_string();
    storage::save_note_at(&data_dir, &note).unwrap();

    // Restore
    let restored = storage::restore_note_version_at(&data_dir, "n1", &version_id).unwrap();
    assert_eq!(restored.title, "Original");
    // updated_at should be updated (not the old 10:00)
    assert!(restored.updated_at.as_str() >= "2026-05-28T12:00:00Z");
}

// --- Template: special characters ---

#[test]
fn test_template_with_special_chars() {
    let (_tmp, data_dir) = setup();
    let t = CustomTemplate {
        id: "t1".to_string(),
        name: "Template <with> \"special\" &chars".to_string(),
        title: "Title with 'quotes'".to_string(),
        content: "Content\nwith\nnewlines\nand\ttabs".to_string(),
        icon: Some("🎯".to_string()),
    };
    storage::save_custom_templates_at(&data_dir, &[t]).unwrap();
    let loaded = storage::load_custom_templates_at(&data_dir);
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].name, "Template <with> \"special\" &chars");
    assert!(loaded[0].content.contains('\n'));
    assert!(loaded[0].content.contains('\t'));
}

// --- Config: overwrite ---

#[test]
fn test_config_overwrite() {
    let (_tmp, data_dir) = setup();
    let mut config = Config {
        theme: "dark".to_string(),
        window_width: 700,
        window_height: 560,
        shortcut: "Ctrl+Alt+N".to_string(),
        new_note_shortcut: String::new(),
        autostart: false,
        check_updates: false,
        font_size: 14,
    };
    storage::save_config_at(&data_dir, &config).unwrap();

    config.theme = "light".to_string();
    config.window_width = 1200;
    storage::save_config_at(&data_dir, &config).unwrap();

    let loaded = storage::load_config_at(&data_dir);
    assert_eq!(loaded.theme, "light");
    assert_eq!(loaded.window_width, 1200);
}

// --- List notes with multiple filters combined ---

#[test]
fn test_list_notes_combined_filters() {
    let (_tmp, data_dir) = setup();
    let mut n1 = make_note("n1", "Rust Work");
    n1.category = Some("Work".to_string());
    n1.tags = vec!["rust".to_string()];
    let mut n2 = make_note("n2", "Rust Personal");
    n2.category = Some("Personal".to_string());
    n2.tags = vec!["rust".to_string()];
    let mut n3 = make_note("n3", "Python Work");
    n3.category = Some("Work".to_string());
    n3.tags = vec!["python".to_string()];
    storage::save_note_at(&data_dir, &n1).unwrap();
    storage::save_note_at(&data_dir, &n2).unwrap();
    storage::save_note_at(&data_dir, &n3).unwrap();

    // Filter: Work + rust
    let filter = NoteFilter {
        search: None,
        category: Some("Work".to_string()),
        tag: Some("rust".to_string()),
        offset: None,
        limit: None,
    };
    let notes = storage::list_notes_at(&data_dir, Some(filter));
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].id, "n1");
}

#[test]
fn test_list_notes_search_case_insensitive() {
    let (_tmp, data_dir) = setup();
    let mut n1 = make_note("n1", "RUST Programming");
    n1.content = "learning RUST".to_string();
    storage::save_note_at(&data_dir, &n1).unwrap();

    let filter = NoteFilter {
        search: Some("rust".to_string()),
        category: None,
        tag: None,
        offset: None,
        limit: None,
    };
    let notes = storage::list_notes_at(&data_dir, Some(filter));
    assert_eq!(notes.len(), 1);
}
