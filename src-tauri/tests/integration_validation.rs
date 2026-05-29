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

// --- Model serialization edge cases ---

#[test]
fn test_note_with_max_length_title() {
    let long_title = "A".repeat(500);
    let note = Note {
        id: "n1".to_string(),
        title: long_title.clone(),
        content: "c".to_string(),
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
    };
    let json = serde_json::to_string(&note).unwrap();
    let parsed: Note = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.title.len(), 500);
}

#[test]
fn test_note_with_unicode_content() {
    let note = Note {
        id: "n1".to_string(),
        title: "Acentos: ação, são, não".to_string(),
        content: "Emoji: 🎉🔥💡 | CJK: 你好 | Arabic: مرحبا".to_string(),
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
    };
    let json = serde_json::to_string(&note).unwrap();
    let parsed: Note = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.title, "Acentos: ação, são, não");
    assert!(parsed.content.contains("🎉"));
    assert!(parsed.content.contains("你好"));
}

#[test]
fn test_note_with_special_json_chars() {
    let note = Note {
        id: "n1".to_string(),
        title: r#"Quotes: "hello" 'world'"#.to_string(),
        content: "Backslash: \\ Newline: \n Tab: \t".to_string(),
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
    };
    let json = serde_json::to_string(&note).unwrap();
    let parsed: Note = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.title, r#"Quotes: "hello" 'world'"#);
}

#[test]
fn test_note_with_many_tags() {
    let tags: Vec<String> = (0..20).map(|i| format!("tag{}", i)).collect();
    let note = Note {
        id: "n1".to_string(),
        title: "Many tags".to_string(),
        content: "".to_string(),
        category: None,
        tags: tags.clone(),
        pinned: false,
        trashed: false,
        trashed_at: None,
        position: None,
            temporary: false,
            expires_at: None,
        schema_version: 1,
        created_at: "2026-05-28T10:00:00Z".to_string(),
        updated_at: "2026-05-28T10:00:00Z".to_string(),
    };
    let json = serde_json::to_string(&note).unwrap();
    let parsed: Note = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.tags.len(), 20);
    assert_eq!(parsed.tags, tags);
}

#[test]
fn test_update_note_all_fields_some() {
    let input = UpdateNote {
        id: "n1".to_string(),
        title: Some("New Title".to_string()),
        content: Some("New Content".to_string()),
        category: Some("NewCat".to_string()),
        tags: Some(vec!["t1".to_string()]),
        pinned: Some(true),
        position: Some(3),
    };
    assert!(input.title.is_some());
    assert!(input.content.is_some());
    assert!(input.category.is_some());
    assert!(input.tags.is_some());
    assert!(input.pinned.is_some());
    assert!(input.position.is_some());
}

#[test]
fn test_create_note_minimal() {
    let input = CreateNote {
        title: "".to_string(),
        content: None,
        category: None,
        tags: None,
        temporary: None,
    };
    assert_eq!(input.title, "");
    assert!(input.content.is_none());
}

#[test]
fn test_create_reminder_all_fields() {
    let input = CreateReminder {
        title: "Reminder".to_string(),
        description: Some("Desc".to_string()),
        note_id: Some("n1".to_string()),
        trigger_at: Some("2026-06-01T12:00:00Z".to_string()),
        repeat: Some("daily".to_string()),
        relative_minutes: None,
    };
    assert_eq!(input.title, "Reminder");
    assert_eq!(input.description, Some("Desc".to_string()));
    assert_eq!(input.repeat, Some("daily".to_string()));
}

#[test]
fn test_reminder_struct_defaults() {
    let reminder = Reminder {
        id: "r1".to_string(),
        note_id: None,
        title: "Test".to_string(),
        description: None,
        trigger_at: "2026-06-01T12:00:00Z".to_string(),
        relative_minutes: None,
        repeat: None,
        status: "pending".to_string(),
        schema_version: 1,
        created_at: "2026-05-28T10:00:00Z".to_string(),
    };
    assert_eq!(reminder.status, "pending");
    assert!(reminder.note_id.is_none());
    assert!(reminder.description.is_none());
    assert!(reminder.repeat.is_none());
}

#[test]
fn test_paginated_result_structure() {
    let result = PaginatedResult {
        items: vec!["a".to_string(), "b".to_string()],
        total: 10,
    };
    assert_eq!(result.items.len(), 2);
    assert_eq!(result.total, 10);
}

// --- Storage: save/load reminders ---

#[test]
fn test_save_and_load_reminder() {
    let (_tmp, data_dir) = setup();
    let reminder = Reminder {
        id: "r1".to_string(),
        note_id: Some("n1".to_string()),
        title: "Test reminder".to_string(),
        description: Some("A description".to_string()),
        trigger_at: "2026-06-01T12:00:00Z".to_string(),
        relative_minutes: Some(30),
        repeat: Some("daily".to_string()),
        status: "pending".to_string(),
        schema_version: 1,
        created_at: "2026-05-28T10:00:00Z".to_string(),
    };
    storage::save_reminder_at(&data_dir, &reminder).unwrap();
    let loaded = storage::get_reminder_at(&data_dir, "r1").unwrap();
    assert_eq!(loaded.title, "Test reminder");
    assert_eq!(loaded.status, "pending");
    assert_eq!(loaded.repeat, Some("daily".to_string()));
}

#[test]
fn test_delete_reminder() {
    let (_tmp, data_dir) = setup();
    let reminder = Reminder {
        id: "r1".to_string(),
        note_id: None,
        title: "Delete me".to_string(),
        description: None,
        trigger_at: "2026-06-01T12:00:00Z".to_string(),
        relative_minutes: None,
        repeat: None,
        status: "pending".to_string(),
        schema_version: 1,
        created_at: "2026-05-28T10:00:00Z".to_string(),
    };
    storage::save_reminder_at(&data_dir, &reminder).unwrap();
    assert!(storage::get_reminder_at(&data_dir, "r1").is_some());
    storage::delete_reminder_at(&data_dir, "r1").unwrap();
    assert!(storage::get_reminder_at(&data_dir, "r1").is_none());
}

// --- Storage: note with all fields ---

#[test]
fn test_save_note_with_all_fields() {
    let (_tmp, data_dir) = setup();
    let note = Note {
        id: "n-full".to_string(),
        title: "Full Note".to_string(),
        content: "Full **content** with markdown".to_string(),
        category: Some("Category".to_string()),
        tags: vec!["t1".to_string(), "t2".to_string()],
        pinned: true,
        trashed: false,
        trashed_at: None,
        position: Some(7),
        temporary: false,
        expires_at: None,
        schema_version: 1,
        created_at: "2026-05-28T10:00:00Z".to_string(),
        updated_at: "2026-05-28T12:00:00Z".to_string(),
    };
    storage::save_note_at(&data_dir, &note).unwrap();
    let loaded = storage::get_note_at(&data_dir, "n-full").unwrap();
    assert_eq!(loaded.title, "Full Note");
    assert_eq!(loaded.content, "Full **content** with markdown");
    assert_eq!(loaded.category, Some("Category".to_string()));
    assert_eq!(loaded.tags.len(), 2);
    assert!(loaded.pinned);
    assert_eq!(loaded.position, Some(7));
}

// --- Version: multiple notes isolation ---

#[test]
fn test_version_isolation_between_notes() {
    let (_tmp, data_dir) = setup();
    let n1 = make_note("n1", "Note 1");
    let n2 = make_note("n2", "Note 2");
    storage::save_note_at(&data_dir, &n1).unwrap();
    storage::save_note_at(&data_dir, &n2).unwrap();

    // Save 3 versions for n1, 1 for n2
    for i in 0..3 {
        let mut v = n1.clone();
        v.title = format!("N1 v{}", i);
        storage::save_note_version_at(&data_dir, &v).unwrap();
    }
    storage::save_note_version_at(&data_dir, &n2).unwrap();

    let v1 = storage::list_note_versions_at(&data_dir, "n1");
    let v2 = storage::list_note_versions_at(&data_dir, "n2");
    assert_eq!(v1.len(), 3);
    assert_eq!(v2.len(), 1);
    // All v1 entries should be for n1
    for v in &v1 {
        assert_eq!(v.note_id, "n1");
    }
}

// --- Template: CRUD ---

#[test]
fn test_template_crud_full_cycle() {
    let (_tmp, data_dir) = setup();

    // Empty
    let templates = storage::load_custom_templates_at(&data_dir);
    assert_eq!(templates.len(), 0);

    // Save 2 templates
    let t1 = CustomTemplate {
        id: "t1".to_string(),
        name: "Template 1".to_string(),
        title: "T1".to_string(),
        content: "C1".to_string(),
        icon: Some("📝".to_string()),
    };
    let t2 = CustomTemplate {
        id: "t2".to_string(),
        name: "Template 2".to_string(),
        title: "T2".to_string(),
        content: "C2".to_string(),
        icon: None,
    };
    storage::save_custom_templates_at(&data_dir, &[t1, t2]).unwrap();

    let loaded = storage::load_custom_templates_at(&data_dir);
    assert_eq!(loaded.len(), 2);

    // Update t1
    let mut t1_updated = loaded[0].clone();
    t1_updated.name = "Updated".to_string();
    let mut remaining: Vec<CustomTemplate> = loaded.into_iter().filter(|t| t.id != "t1").collect();
    remaining.push(t1_updated);
    storage::save_custom_templates_at(&data_dir, &remaining).unwrap();

    let after_update = storage::load_custom_templates_at(&data_dir);
    assert_eq!(after_update.len(), 2);
    let updated = after_update.iter().find(|t| t.id == "t1").unwrap();
    assert_eq!(updated.name, "Updated");

    // Delete t2
    let after_delete: Vec<CustomTemplate> = after_update.into_iter().filter(|t| t.id != "t2").collect();
    storage::save_custom_templates_at(&data_dir, &after_delete).unwrap();

    let final_templates = storage::load_custom_templates_at(&data_dir);
    assert_eq!(final_templates.len(), 1);
    assert_eq!(final_templates[0].id, "t1");
}
