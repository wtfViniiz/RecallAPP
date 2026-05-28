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
        category: Some("Test".to_string()),
        tags: vec!["tag1".to_string()],
        pinned: false,
        trashed: false,
        trashed_at: None,
        position: None,
        schema_version: 1,
        created_at: "2026-05-28T10:00:00Z".to_string(),
        updated_at: "2026-05-28T10:00:00Z".to_string(),
    }
}

#[test]
fn test_save_and_list_versions() {
    let (_tmp, data_dir) = setup();
    let note = make_note("n1", "Original Title");
    storage::save_note_at(&data_dir, &note).unwrap();

    // Save a version
    storage::save_note_version_at(&data_dir, &note).unwrap();

    let versions = storage::list_note_versions_at(&data_dir, "n1");
    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0].note_id, "n1");
    assert_eq!(versions[0].title, "Original Title");
    assert_eq!(versions[0].content, "test content");
    assert_eq!(versions[0].category, Some("Test".to_string()));
    assert_eq!(versions[0].tags, vec!["tag1".to_string()]);
}

#[test]
fn test_list_versions_empty_for_nonexistent_note() {
    let (_tmp, data_dir) = setup();
    let versions = storage::list_note_versions_at(&data_dir, "nonexistent");
    assert_eq!(versions.len(), 0);
}

#[test]
fn test_list_versions_sorted_newest_first() {
    let (_tmp, data_dir) = setup();
    let note = make_note("n1", "Title");
    storage::save_note_at(&data_dir, &note).unwrap();

    // Save multiple versions with slight delay to get different timestamps
    storage::save_note_version_at(&data_dir, &note).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));

    let mut note2 = note.clone();
    note2.title = "Version 2".to_string();
    storage::save_note_version_at(&data_dir, &note2).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(50));

    let mut note3 = note.clone();
    note3.title = "Version 3".to_string();
    storage::save_note_version_at(&data_dir, &note3).unwrap();

    let versions = storage::list_note_versions_at(&data_dir, "n1");
    assert_eq!(versions.len(), 3);
    // Newest first: Version 3, Version 2, Title (original)
    assert_eq!(versions[0].title, "Version 3");
    assert_eq!(versions[1].title, "Version 2");
    assert_eq!(versions[2].title, "Title");
}

#[test]
fn test_save_version_prunes_beyond_20() {
    let (_tmp, data_dir) = setup();
    let note = make_note("n1", "Title");
    storage::save_note_at(&data_dir, &note).unwrap();

    // Save 25 versions
    for i in 0..25 {
        let mut v = note.clone();
        v.title = format!("Version {}", i);
        storage::save_note_version_at(&data_dir, &v).unwrap();
        // Small delay to ensure distinct timestamps
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    let versions = storage::list_note_versions_at(&data_dir, "n1");
    assert!(versions.len() <= 20, "Expected at most 20 versions, got {}", versions.len());
}

#[test]
fn test_restore_version_restores_fields() {
    let (_tmp, data_dir) = setup();
    let note = make_note("n1", "Original");
    storage::save_note_at(&data_dir, &note).unwrap();

    // Save version of original state
    storage::save_note_version_at(&data_dir, &note).unwrap();
    let versions = storage::list_note_versions_at(&data_dir, "n1");
    let version_id = versions[0].id.clone();

    // Modify the note
    let mut modified = note.clone();
    modified.title = "Modified Title".to_string();
    modified.content = "Modified content".to_string();
    modified.category = Some("NewCategory".to_string());
    modified.tags = vec!["newtag".to_string()];
    storage::save_note_at(&data_dir, &modified).unwrap();

    // Restore the version
    let restored = storage::restore_note_version_at(&data_dir, "n1", &version_id).unwrap();
    assert_eq!(restored.title, "Original");
    assert_eq!(restored.content, "test content");
    assert_eq!(restored.category, Some("Test".to_string()));
    assert_eq!(restored.tags, vec!["tag1".to_string()]);
}

#[test]
fn test_restore_version_saves_current_as_version_before_restoring() {
    let (_tmp, data_dir) = setup();
    let note = make_note("n1", "V1");
    storage::save_note_at(&data_dir, &note).unwrap();

    // Save V1 as version
    storage::save_note_version_at(&data_dir, &note).unwrap();
    let versions = storage::list_note_versions_at(&data_dir, "n1");
    let v1_id = versions[0].id.clone();

    // Modify to V2
    let mut v2 = note.clone();
    v2.title = "V2".to_string();
    storage::save_note_at(&data_dir, &v2).unwrap();

    // Restore V1 (this should save V2 as a new version first)
    let restored = storage::restore_note_version_at(&data_dir, "n1", &v1_id).unwrap();
    assert_eq!(restored.title, "V1");

    // There should be now 2 versions: the original V1 backup and the V2 backup made before restore
    let versions_after = storage::list_note_versions_at(&data_dir, "n1");
    assert!(versions_after.len() >= 2, "Expected at least 2 versions after restore, got {}", versions_after.len());
}

#[test]
fn test_restore_version_fails_nonexistent_version() {
    let (_tmp, data_dir) = setup();
    let note = make_note("n1", "Title");
    storage::save_note_at(&data_dir, &note).unwrap();

    let result = storage::restore_note_version_at(&data_dir, "n1", "nonexistent-version");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Versao nao encontrada"));
}

#[test]
fn test_restore_version_fails_nonexistent_note() {
    let (_tmp, data_dir) = setup();

    // Create a version file for a note that doesn't exist on disk
    // This way the version lookup succeeds but the note lookup fails
    let versions_dir = data_dir.join("versions").join("ghost-note");
    std::fs::create_dir_all(&versions_dir).unwrap();
    let version = NoteVersion {
        id: "v1".to_string(),
        note_id: "ghost-note".to_string(),
        title: "Ghost".to_string(),
        content: "".to_string(),
        category: None,
        tags: vec![],
        created_at: "2026-05-28T10:00:00Z".to_string(),
    };
    let json = serde_json::to_string_pretty(&version).unwrap();
    std::fs::write(versions_dir.join("v1.json"), json).unwrap();

    let result = storage::restore_note_version_at(&data_dir, "ghost-note", "v1");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Nota nao encontrada"));
}

#[test]
fn test_version_list_filters_tmp_files() {
    let (_tmp, data_dir) = setup();
    let note = make_note("n1", "Title");
    storage::save_note_at(&data_dir, &note).unwrap();

    // Save a valid version
    storage::save_note_version_at(&data_dir, &note).unwrap();

    // Manually create a .tmp file in the versions dir to simulate an orphaned temp file
    let versions_dir = data_dir.join("versions").join("n1");
    std::fs::create_dir_all(&versions_dir).unwrap();
    let tmp_path = versions_dir.join("orphan.json.tmp");
    std::fs::write(&tmp_path, r#"{"id":"tmp","note_id":"n1","title":"tmp","content":"","tags":[],"created_at":"2026-05-28T10:00:00Z"}"#).unwrap();

    let versions = storage::list_note_versions_at(&data_dir, "n1");
    assert_eq!(versions.len(), 1, "Should filter out .tmp files");
    assert_ne!(versions[0].id, "tmp");
}

#[test]
fn test_save_version_with_empty_content() {
    let (_tmp, data_dir) = setup();
    let mut note = make_note("n1", "Title");
    note.content = "".to_string();
    note.category = None;
    note.tags = vec![];
    storage::save_note_at(&data_dir, &note).unwrap();

    storage::save_note_version_at(&data_dir, &note).unwrap();
    let versions = storage::list_note_versions_at(&data_dir, "n1");
    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0].content, "");
    assert!(versions[0].category.is_none());
    assert!(versions[0].tags.is_empty());
}

#[test]
fn test_version_preserves_all_note_fields() {
    let (_tmp, data_dir) = setup();
    let note = Note {
        id: "n1".to_string(),
        title: "Title with special chars: <>&\"'".to_string(),
        content: "Content with **markdown** and unicode: acento".to_string(),
        category: Some("Categoria/Especial".to_string()),
        tags: vec!["tag-1".to_string(), "tag_2".to_string(), "tag 3".to_string()],
        pinned: true,
        trashed: false,
        trashed_at: None,
        position: Some(5),
        schema_version: 1,
        created_at: "2026-05-28T10:00:00Z".to_string(),
        updated_at: "2026-05-28T12:00:00Z".to_string(),
    };
    storage::save_note_at(&data_dir, &note).unwrap();

    storage::save_note_version_at(&data_dir, &note).unwrap();
    let versions = storage::list_note_versions_at(&data_dir, "n1");
    let v = &versions[0];

    assert_eq!(v.title, "Title with special chars: <>&\"'");
    assert_eq!(v.content, "Content with **markdown** and unicode: acento");
    assert_eq!(v.category, Some("Categoria/Especial".to_string()));
    assert_eq!(v.tags.len(), 3);
    assert!(v.tags.contains(&"tag-1".to_string()));
    assert!(v.tags.contains(&"tag_2".to_string()));
    assert!(v.tags.contains(&"tag 3".to_string()));
}

#[test]
fn test_multiple_notes_versions_isolated() {
    let (_tmp, data_dir) = setup();
    let note1 = make_note("n1", "Note 1");
    let note2 = make_note("n2", "Note 2");
    storage::save_note_at(&data_dir, &note1).unwrap();
    storage::save_note_at(&data_dir, &note2).unwrap();

    // Save versions only for note1
    storage::save_note_version_at(&data_dir, &note1).unwrap();
    storage::save_note_version_at(&data_dir, &note1).unwrap();

    let versions_n1 = storage::list_note_versions_at(&data_dir, "n1");
    let versions_n2 = storage::list_note_versions_at(&data_dir, "n2");

    assert_eq!(versions_n1.len(), 2);
    assert_eq!(versions_n2.len(), 0);
}
