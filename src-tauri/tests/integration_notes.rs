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
        schema_version: 1,
        created_at: "2026-05-28T10:00:00Z".to_string(),
        updated_at: "2026-05-28T10:00:00Z".to_string(),
    }
}

#[test]
fn test_create_and_read_note() {
    let (_tmp, data_dir) = setup();
    let note = make_note("n1", "Test Note");
    storage::save_note_at(&data_dir, &note).unwrap();

    let notes = storage::list_notes_at(&data_dir, None);
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].title, "Test Note");
}

#[test]
fn test_note_full_lifecycle() {
    let (_tmp, data_dir) = setup();
    let note = make_note("n1", "Lifecycle Test");
    storage::save_note_at(&data_dir, &note).unwrap();

    // Read
    let fetched = storage::get_note_at(&data_dir, "n1");
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().title, "Lifecycle Test");

    // Update
    let mut updated = note.clone();
    updated.title = "Updated Title".to_string();
    storage::save_note_at(&data_dir, &updated).unwrap();
    let fetched = storage::get_note_at(&data_dir, "n1").unwrap();
    assert_eq!(fetched.title, "Updated Title");

    // Trash
    let mut trashed = fetched.clone();
    trashed.trashed = true;
    trashed.trashed_at = Some("2026-05-28T12:00:00Z".to_string());
    storage::save_note_at(&data_dir, &trashed).unwrap();

    // Verify filtered from normal list
    let notes = storage::list_notes_at(&data_dir, None);
    assert_eq!(notes.len(), 0);

    // Verify in trashed list
    let trashed_notes = storage::list_trashed_notes_at(&data_dir);
    assert_eq!(trashed_notes.len(), 1);
    assert!(trashed_notes[0].trashed);

    // Restore
    let mut restored = trashed.clone();
    restored.trashed = false;
    restored.trashed_at = None;
    storage::save_note_at(&data_dir, &restored).unwrap();

    let notes = storage::list_notes_at(&data_dir, None);
    assert_eq!(notes.len(), 1);
    assert!(!notes[0].trashed);

    // Delete
    storage::delete_note_at(&data_dir, "n1").unwrap();
    let notes = storage::list_notes_at(&data_dir, None);
    assert_eq!(notes.len(), 0);
    assert!(storage::get_note_at(&data_dir, "n1").is_none());
}

#[test]
fn test_search_filter() {
    let (_tmp, data_dir) = setup();
    storage::save_note_at(&data_dir, &make_note("n1", "Rust Programming")).unwrap();
    storage::save_note_at(&data_dir, &make_note("n2", "Python Basics")).unwrap();
    storage::save_note_at(&data_dir, &make_note("n3", "Rust Advanced Topics")).unwrap();

    let filter = NoteFilter {
        search: Some("rust".to_string()),
        category: None,
        tag: None,
        offset: None,
        limit: None,
    };
    let notes = storage::list_notes_at(&data_dir, Some(filter));
    assert_eq!(notes.len(), 2);
    assert!(notes.iter().all(|n| n.title.to_lowercase().contains("rust")));
}

#[test]
fn test_category_filter() {
    let (_tmp, data_dir) = setup();
    let mut n1 = make_note("n1", "Work Note");
    n1.category = Some("Trabalho".to_string());
    let mut n2 = make_note("n2", "Personal Note");
    n2.category = Some("Pessoal".to_string());
    storage::save_note_at(&data_dir, &n1).unwrap();
    storage::save_note_at(&data_dir, &n2).unwrap();

    let filter = NoteFilter {
        search: None,
        category: Some("Trabalho".to_string()),
        tag: None,
        offset: None,
        limit: None,
    };
    let notes = storage::list_notes_at(&data_dir, Some(filter));
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].title, "Work Note");
}

#[test]
fn test_tag_filter() {
    let (_tmp, data_dir) = setup();
    let mut n1 = make_note("n1", "Tagged Note");
    n1.tags = vec!["urgent".to_string(), "work".to_string()];
    let n2 = make_note("n2", "Untagged Note");
    storage::save_note_at(&data_dir, &n1).unwrap();
    storage::save_note_at(&data_dir, &n2).unwrap();

    let filter = NoteFilter {
        search: None,
        category: None,
        tag: Some("urgent".to_string()),
        offset: None,
        limit: None,
    };
    let notes = storage::list_notes_at(&data_dir, Some(filter));
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].title, "Tagged Note");
}

#[test]
fn test_combined_filter() {
    let (_tmp, data_dir) = setup();
    let mut n1 = make_note("n1", "Rust Work");
    n1.category = Some("Trabalho".to_string());
    n1.tags = vec!["urgent".to_string()];
    let mut n2 = make_note("n2", "Rust Personal");
    n2.category = Some("Pessoal".to_string());
    let n3 = make_note("n3", "Python Work");
    storage::save_note_at(&data_dir, &n1).unwrap();
    storage::save_note_at(&data_dir, &n2).unwrap();
    storage::save_note_at(&data_dir, &n3).unwrap();

    let filter = NoteFilter {
        search: Some("rust".to_string()),
        category: Some("Trabalho".to_string()),
        tag: None,
        offset: None,
        limit: None,
    };
    let notes = storage::list_notes_at(&data_dir, Some(filter));
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].title, "Rust Work");
}

#[test]
fn test_pinned_sort_order() {
    let (_tmp, data_dir) = setup();
    let mut n1 = make_note("n1", "Not Pinned");
    n1.updated_at = "2026-05-28T12:00:00Z".to_string();
    let mut n2 = make_note("n2", "Pinned");
    n2.pinned = true;
    n2.updated_at = "2026-05-28T10:00:00Z".to_string();
    storage::save_note_at(&data_dir, &n1).unwrap();
    storage::save_note_at(&data_dir, &n2).unwrap();

    let notes = storage::list_notes_at(&data_dir, None);
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[0].title, "Pinned");
    assert_eq!(notes[1].title, "Not Pinned");
}

#[test]
fn test_delete_nonexistent_note() {
    let (_tmp, data_dir) = setup();
    let result = storage::delete_note_at(&data_dir, "nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_list_all_notes_includes_trashed() {
    let (_tmp, data_dir) = setup();
    let n1 = make_note("n1", "Active");
    let mut n2 = make_note("n2", "Trashed");
    n2.trashed = true;
    n2.trashed_at = Some("2026-05-28T12:00:00Z".to_string());
    storage::save_note_at(&data_dir, &n1).unwrap();
    storage::save_note_at(&data_dir, &n2).unwrap();

    let all = storage::list_all_notes_at(&data_dir);
    assert_eq!(all.len(), 2);

    let active = storage::list_notes_at(&data_dir, None);
    assert_eq!(active.len(), 1);

    let trashed = storage::list_trashed_notes_at(&data_dir);
    assert_eq!(trashed.len(), 1);
}

#[test]
fn test_empty_directory() {
    let (_tmp, data_dir) = setup();
    let notes = storage::list_notes_at(&data_dir, None);
    assert_eq!(notes.len(), 0);

    let all = storage::list_all_notes_at(&data_dir);
    assert_eq!(all.len(), 0);

    let trashed = storage::list_trashed_notes_at(&data_dir);
    assert_eq!(trashed.len(), 0);
}

#[test]
fn test_pagination_limit() {
    let (_tmp, data_dir) = setup();
    for i in 0..10 {
        storage::save_note_at(&data_dir, &make_note(&format!("n{}", i), &format!("Note {}", i))).unwrap();
    }

    let filter = NoteFilter {
        search: None,
        category: None,
        tag: None,
        offset: None,
        limit: Some(3),
    };
    let notes = storage::list_notes_at(&data_dir, Some(filter));
    assert_eq!(notes.len(), 3);
}

#[test]
fn test_pagination_offset() {
    let (_tmp, data_dir) = setup();
    for i in 0..10 {
        storage::save_note_at(&data_dir, &make_note(&format!("n{}", i), &format!("Note {}", i))).unwrap();
    }

    let filter = NoteFilter {
        search: None,
        category: None,
        tag: None,
        offset: Some(5),
        limit: None,
    };
    let notes = storage::list_notes_at(&data_dir, Some(filter));
    assert_eq!(notes.len(), 5);
}

#[test]
fn test_pagination_offset_and_limit() {
    let (_tmp, data_dir) = setup();
    for i in 0..10 {
        storage::save_note_at(&data_dir, &make_note(&format!("n{}", i), &format!("Note {}", i))).unwrap();
    }

    // Page 1: offset=0, limit=3
    let filter1 = NoteFilter {
        search: None,
        category: None,
        tag: None,
        offset: Some(0),
        limit: Some(3),
    };
    let page1 = storage::list_notes_at(&data_dir, Some(filter1));
    assert_eq!(page1.len(), 3);

    // Page 2: offset=3, limit=3
    let filter2 = NoteFilter {
        search: None,
        category: None,
        tag: None,
        offset: Some(3),
        limit: Some(3),
    };
    let page2 = storage::list_notes_at(&data_dir, Some(filter2));
    assert_eq!(page2.len(), 3);

    // No overlap
    let page1_ids: Vec<_> = page1.iter().map(|n| &n.id).collect();
    let page2_ids: Vec<_> = page2.iter().map(|n| &n.id).collect();
    for id in &page1_ids {
        assert!(!page2_ids.contains(id));
    }

    // Page 3: offset=6, limit=3
    let filter3 = NoteFilter {
        search: None,
        category: None,
        tag: None,
        offset: Some(6),
        limit: Some(3),
    };
    let page3 = storage::list_notes_at(&data_dir, Some(filter3));
    assert_eq!(page3.len(), 3);

    // Page 4: offset=9, limit=3 (only 1 remaining)
    let filter4 = NoteFilter {
        search: None,
        category: None,
        tag: None,
        offset: Some(9),
        limit: Some(3),
    };
    let page4 = storage::list_notes_at(&data_dir, Some(filter4));
    assert_eq!(page4.len(), 1);
}

#[test]
fn test_pagination_offset_beyond_total() {
    let (_tmp, data_dir) = setup();
    for i in 0..5 {
        storage::save_note_at(&data_dir, &make_note(&format!("n{}", i), &format!("Note {}", i))).unwrap();
    }

    let filter = NoteFilter {
        search: None,
        category: None,
        tag: None,
        offset: Some(100),
        limit: Some(10),
    };
    let notes = storage::list_notes_at(&data_dir, Some(filter));
    assert_eq!(notes.len(), 0);
}

#[test]
fn test_pagination_with_filter() {
    let (_tmp, data_dir) = setup();
    let mut n1 = make_note("n1", "Rust A");
    n1.category = Some("Work".to_string());
    let mut n2 = make_note("n2", "Rust B");
    n2.category = Some("Work".to_string());
    let mut n3 = make_note("n3", "Rust C");
    n3.category = Some("Personal".to_string());
    let n4 = make_note("n4", "Python");
    storage::save_note_at(&data_dir, &n1).unwrap();
    storage::save_note_at(&data_dir, &n2).unwrap();
    storage::save_note_at(&data_dir, &n3).unwrap();
    storage::save_note_at(&data_dir, &n4).unwrap();

    // Filter by category + pagination
    let filter = NoteFilter {
        search: None,
        category: Some("Work".to_string()),
        tag: None,
        offset: Some(0),
        limit: Some(1),
    };
    let notes = storage::list_notes_at(&data_dir, Some(filter));
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].category, Some("Work".to_string()));
}
