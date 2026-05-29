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
        content: "test content".to_string(),
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

#[test]
fn test_note_position_none_serde_roundtrip() {
    let note = make_note("n1", "No Position");
    let json = serde_json::to_string(&note).unwrap();
    let parsed: Note = serde_json::from_str(&json).unwrap();
    assert!(parsed.position.is_none());
}

#[test]
fn test_note_position_some_roundtrip() {
    let mut note = make_note("n1", "With Position");
    note.position = Some(5);
    let json = serde_json::to_string(&note).unwrap();
    let parsed: Note = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.position, Some(5));
}

#[test]
fn test_note_position_none_from_json_missing_field() {
    let json = r#"{
        "id": "minimal",
        "title": "Min",
        "content": "",
        "tags": [],
        "pinned": false,
        "created_at": "2026-05-28T10:00:00Z",
        "updated_at": "2026-05-28T10:00:00Z"
    }"#;
    let note: Note = serde_json::from_str(json).unwrap();
    assert!(note.position.is_none());
}

#[test]
fn test_note_position_from_json_with_value() {
    let json = r#"{
        "id": "pos",
        "title": "Pos",
        "content": "",
        "tags": [],
        "pinned": false,
        "position": 42,
        "created_at": "2026-05-28T10:00:00Z",
        "updated_at": "2026-05-28T10:00:00Z"
    }"#;
    let note: Note = serde_json::from_str(json).unwrap();
    assert_eq!(note.position, Some(42));
}

#[test]
fn test_note_position_negative_value() {
    let mut note = make_note("n1", "Negative");
    note.position = Some(-1);
    let json = serde_json::to_string(&note).unwrap();
    let parsed: Note = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.position, Some(-1));
}

#[test]
fn test_sorting_pinned_first_then_position_then_updated() {
    let (_tmp, data_dir) = setup();

    // Unpinned, no position, newer
    let mut n1 = make_note("n1", "Unpinned NoPos Newer");
    n1.updated_at = "2026-05-28T15:00:00Z".to_string();

    // Unpinned, position 2, older
    let mut n2 = make_note("n2", "Unpinned Pos2");
    n2.position = Some(2);
    n2.updated_at = "2026-05-28T10:00:00Z".to_string();

    // Unpinned, position 1, older
    let mut n3 = make_note("n3", "Unpinned Pos1");
    n3.position = Some(1);
    n3.updated_at = "2026-05-28T10:00:00Z".to_string();

    // Pinned, no position
    let mut n4 = make_note("n4", "Pinned");
    n4.pinned = true;
    n4.updated_at = "2026-05-28T08:00:00Z".to_string();

    // Unpinned, no position, older
    let mut n5 = make_note("n5", "Unpinned NoPos Older");
    n5.updated_at = "2026-05-28T12:00:00Z".to_string();

    storage::save_note_at(&data_dir, &n1).unwrap();
    storage::save_note_at(&data_dir, &n2).unwrap();
    storage::save_note_at(&data_dir, &n3).unwrap();
    storage::save_note_at(&data_dir, &n4).unwrap();
    storage::save_note_at(&data_dir, &n5).unwrap();

    let notes = storage::list_notes_at(&data_dir, None);
    assert_eq!(notes.len(), 5);

    // 1st: Pinned
    assert_eq!(notes[0].title, "Pinned");
    // 2nd: Unpinned Pos1 (position ascending)
    assert_eq!(notes[1].title, "Unpinned Pos1");
    // 3rd: Unpinned Pos2 (position ascending)
    assert_eq!(notes[2].title, "Unpinned Pos2");
    // 4th: NoPos Newer (updated_at descending among no-position notes)
    assert_eq!(notes[3].title, "Unpinned NoPos Newer");
    // 5th: NoPos Older
    assert_eq!(notes[4].title, "Unpinned NoPos Older");
}

#[test]
fn test_sorting_multiple_pinned_with_positions() {
    let (_tmp, data_dir) = setup();

    let mut n1 = make_note("n1", "Pinned Pos10");
    n1.pinned = true;
    n1.position = Some(10);
    n1.updated_at = "2026-05-28T10:00:00Z".to_string();

    let mut n2 = make_note("n2", "Pinned Pos2");
    n2.pinned = true;
    n2.position = Some(2);
    n2.updated_at = "2026-05-28T10:00:00Z".to_string();

    let mut n3 = make_note("n3", "Unpinned Pos1");
    n3.pinned = false;
    n3.position = Some(1);
    n3.updated_at = "2026-05-28T10:00:00Z".to_string();

    storage::save_note_at(&data_dir, &n1).unwrap();
    storage::save_note_at(&data_dir, &n2).unwrap();
    storage::save_note_at(&data_dir, &n3).unwrap();

    let notes = storage::list_notes_at(&data_dir, None);
    assert_eq!(notes.len(), 3);
    assert_eq!(notes[0].title, "Pinned Pos2");
    assert_eq!(notes[1].title, "Pinned Pos10");
    assert_eq!(notes[2].title, "Unpinned Pos1");
}

#[test]
fn test_note_with_empty_content() {
    let (_tmp, data_dir) = setup();
    let mut note = make_note("n1", "Empty Content");
    note.content = "".to_string();
    storage::save_note_at(&data_dir, &note).unwrap();

    let fetched = storage::get_note_at(&data_dir, "n1").unwrap();
    assert_eq!(fetched.content, "");
    assert_eq!(fetched.title, "Empty Content");
}

#[test]
fn test_note_with_long_title() {
    let (_tmp, data_dir) = setup();
    let long_title: String = "A".repeat(500);
    let mut note = make_note("n1", &long_title);
    note.title = long_title.clone();
    storage::save_note_at(&data_dir, &note).unwrap();

    let fetched = storage::get_note_at(&data_dir, "n1").unwrap();
    assert_eq!(fetched.title.len(), 500);
    assert_eq!(fetched.title, long_title);
}

#[test]
fn test_note_with_special_characters_in_all_fields() {
    let (_tmp, data_dir) = setup();
    let note = Note {
        id: "special-id_123".to_string(),
        title: "Title with <html> & \"quotes\" and 'single'".to_string(),
        content: "Content with\nnewlines\ttabs and unicode: ação são não".to_string(),
        category: Some("Cat/with/slashes".to_string()),
        tags: vec![
            "tag with spaces".to_string(),
            "tag-with-dashes".to_string(),
            "tag_with_underscores".to_string(),
            "tag<with>brackets".to_string(),
        ],
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
    storage::save_note_at(&data_dir, &note).unwrap();

    let fetched = storage::get_note_at(&data_dir, "special-id_123").unwrap();
    assert_eq!(fetched.title, "Title with <html> & \"quotes\" and 'single'");
    assert!(fetched.content.contains("ação"));
    assert!(fetched.content.contains("\ttabs"));
    assert_eq!(fetched.category, Some("Cat/with/slashes".to_string()));
    assert_eq!(fetched.tags.len(), 4);
    assert!(fetched.tags.contains(&"tag<with>brackets".to_string()));
}

#[test]
fn test_note_with_maximal_position() {
    let (_tmp, data_dir) = setup();
    let mut note = make_note("n1", "Max Position");
    note.position = Some(i32::MAX);
    storage::save_note_at(&data_dir, &note).unwrap();

    let fetched = storage::get_note_at(&data_dir, "n1").unwrap();
    assert_eq!(fetched.position, Some(i32::MAX));
}

#[test]
fn test_note_with_minimal_position() {
    let (_tmp, data_dir) = setup();
    let mut note = make_note("n1", "Min Position");
    note.position = Some(i32::MIN);
    storage::save_note_at(&data_dir, &note).unwrap();

    let fetched = storage::get_note_at(&data_dir, "n1").unwrap();
    assert_eq!(fetched.position, Some(i32::MIN));
}

#[test]
fn test_note_with_zero_position() {
    let (_tmp, data_dir) = setup();
    let mut note = make_note("n1", "Zero Position");
    note.position = Some(0);
    storage::save_note_at(&data_dir, &note).unwrap();

    let fetched = storage::get_note_at(&data_dir, "n1").unwrap();
    assert_eq!(fetched.position, Some(0));
}

#[test]
fn test_empty_content_note_searchable() {
    let (_tmp, data_dir) = setup();
    let mut n1 = make_note("n1", "Searchable Title");
    n1.content = "".to_string();
    let n2 = make_note("n2", "Other Note");
    storage::save_note_at(&data_dir, &n1).unwrap();
    storage::save_note_at(&data_dir, &n2).unwrap();

    let filter = NoteFilter {
        search: Some("Searchable".to_string()),
        category: None,
        tag: None,
        offset: None,
        limit: None,
    };
    let notes = storage::list_notes_at(&data_dir, Some(filter));
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].title, "Searchable Title");
}
