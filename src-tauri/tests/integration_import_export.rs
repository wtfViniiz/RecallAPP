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
        category: Some("Test".to_string()),
        tags: vec!["tag1".to_string()],
        pinned: false,
        trashed: false,
        trashed_at: None,
        schema_version: 1,
        created_at: "2026-05-28T10:00:00Z".to_string(),
        updated_at: "2026-05-28T10:00:00Z".to_string(),
    }
}

fn make_reminder(id: &str, title: &str) -> Reminder {
    Reminder {
        id: id.to_string(),
        title: title.to_string(),
        description: Some("desc".to_string()),
        note_id: None,
        trigger_at: "2026-05-29T10:00:00Z".to_string(),
        repeat: Some("daily".to_string()),
        relative_minutes: None,
        status: "pending".to_string(),
        schema_version: 1,
        created_at: "2026-05-28T10:00:00Z".to_string(),
    }
}

#[test]
fn test_export_and_import_roundtrip() {
    // Create data in first instance
    let (_tmp1, data_dir1) = setup();
    let note = make_note("n1", "Exported Note");
    let reminder = make_reminder("r1", "Exported Reminder");
    storage::save_note_at(&data_dir1, &note).unwrap();
    storage::save_reminder_at(&data_dir1, &reminder).unwrap();

    // Export
    let notes = storage::list_all_notes_at(&data_dir1);
    let reminders = storage::list_reminders_at(&data_dir1, None);
    let export = serde_json::json!({
        "version": 1,
        "notes": notes,
        "reminders": reminders,
    });
    let json = serde_json::to_string(&export).unwrap();

    // Import into second instance
    let (_tmp2, data_dir2) = setup();
    let data: serde_json::Value = serde_json::from_str(&json).unwrap();

    for note_value in data["notes"].as_array().unwrap() {
        let note: Note = serde_json::from_value(note_value.clone()).unwrap();
        storage::save_note_at(&data_dir2, &note).unwrap();
    }
    for reminder_value in data["reminders"].as_array().unwrap() {
        let reminder: Reminder = serde_json::from_value(reminder_value.clone()).unwrap();
        storage::save_reminder_at(&data_dir2, &reminder).unwrap();
    }

    // Verify
    let imported_notes = storage::list_notes_at(&data_dir2, None);
    let imported_reminders = storage::list_reminders_at(&data_dir2, None);
    assert_eq!(imported_notes.len(), 1);
    assert_eq!(imported_reminders.len(), 1);
    assert_eq!(imported_notes[0].id, "n1");
    assert_eq!(imported_notes[0].title, "Exported Note");
    assert_eq!(imported_notes[0].category, Some("Test".to_string()));
    assert_eq!(imported_reminders[0].id, "r1");
    assert_eq!(imported_reminders[0].title, "Exported Reminder");
}

#[test]
fn test_export_includes_trashed_notes() {
    let (_tmp, data_dir) = setup();
    let n1 = make_note("n1", "Active");
    let mut n2 = make_note("n2", "Trashed");
    n2.trashed = true;
    n2.trashed_at = Some("2026-05-28T12:00:00Z".to_string());
    storage::save_note_at(&data_dir, &n1).unwrap();
    storage::save_note_at(&data_dir, &n2).unwrap();

    let all_notes = storage::list_all_notes_at(&data_dir);
    assert_eq!(all_notes.len(), 2);

    let active_notes = storage::list_notes_at(&data_dir, None);
    assert_eq!(active_notes.len(), 1);

    // Export should include trashed
    let export_notes = all_notes;
    assert_eq!(export_notes.len(), 2);
}

#[test]
fn test_import_preserves_categories_and_tags() {
    let (_tmp, data_dir) = setup();
    let mut note = make_note("n1", "Categorized");
    note.category = Some("Work".to_string());
    note.tags = vec!["urgent".to_string(), "project-x".to_string()];
    storage::save_note_at(&data_dir, &note).unwrap();

    // Export and re-import
    let notes = storage::list_all_notes_at(&data_dir);
    let export_json = serde_json::to_string(&notes).unwrap();
    let imported: Vec<Note> = serde_json::from_str(&export_json).unwrap();

    assert_eq!(imported[0].category, Some("Work".to_string()));
    assert_eq!(imported[0].tags.len(), 2);
    assert!(imported[0].tags.contains(&"urgent".to_string()));
}

#[test]
fn test_multiple_notes_export_import() {
    let (_tmp, data_dir) = setup();
    for i in 0..5 {
        storage::save_note_at(&data_dir, &make_note(&format!("n{}", i), &format!("Note {}", i))).unwrap();
    }

    let notes = storage::list_all_notes_at(&data_dir);
    assert_eq!(notes.len(), 5);

    let export_json = serde_json::to_string(&notes).unwrap();
    let imported: Vec<Note> = serde_json::from_str(&export_json).unwrap();
    assert_eq!(imported.len(), 5);
}
