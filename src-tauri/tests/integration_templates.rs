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

#[test]
fn test_load_templates_empty_when_no_file() {
    let (_tmp, data_dir) = setup();
    let templates = storage::load_custom_templates_at(&data_dir);
    assert_eq!(templates.len(), 0);
}

#[test]
fn test_save_and_load_templates_roundtrip() {
    let (_tmp, data_dir) = setup();
    let templates = vec![
        CustomTemplate {
            id: "t1".to_string(),
            name: "Meeting Notes".to_string(),
            title: "Meeting - {{date}}".to_string(),
            content: "# Attendees\n\n## Agenda\n\n## Notes\n".to_string(),
            icon: Some("clipboard".to_string()),
        },
        CustomTemplate {
            id: "t2".to_string(),
            name: "Bug Report".to_string(),
            title: "Bug: ".to_string(),
            content: "## Steps to Reproduce\n\n## Expected\n\n## Actual\n".to_string(),
            icon: None,
        },
    ];

    storage::save_custom_templates_at(&data_dir, &templates).unwrap();
    let loaded = storage::load_custom_templates_at(&data_dir);

    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].id, "t1");
    assert_eq!(loaded[0].name, "Meeting Notes");
    assert_eq!(loaded[0].title, "Meeting - {{date}}");
    assert_eq!(loaded[0].icon, Some("clipboard".to_string()));
    assert_eq!(loaded[1].id, "t2");
    assert_eq!(loaded[1].name, "Bug Report");
    assert!(loaded[1].icon.is_none());
}

#[test]
fn test_save_empty_list_load_empty() {
    let (_tmp, data_dir) = setup();
    let templates: Vec<CustomTemplate> = vec![CustomTemplate {
        id: "t1".to_string(),
        name: "Existing".to_string(),
        title: "Title".to_string(),
        content: "Content".to_string(),
        icon: None,
    }];
    storage::save_custom_templates_at(&data_dir, &templates).unwrap();
    assert_eq!(storage::load_custom_templates_at(&data_dir).len(), 1);

    // Overwrite with empty list
    let empty: Vec<CustomTemplate> = vec![];
    storage::save_custom_templates_at(&data_dir, &empty).unwrap();
    let loaded = storage::load_custom_templates_at(&data_dir);
    assert_eq!(loaded.len(), 0);
}

#[test]
fn test_templates_overwrite() {
    let (_tmp, data_dir) = setup();

    let templates1 = vec![CustomTemplate {
        id: "t1".to_string(),
        name: "First".to_string(),
        title: "First Title".to_string(),
        content: "First Content".to_string(),
        icon: None,
    }];
    storage::save_custom_templates_at(&data_dir, &templates1).unwrap();

    let templates2 = vec![
        CustomTemplate {
            id: "t1".to_string(),
            name: "Updated".to_string(),
            title: "Updated Title".to_string(),
            content: "Updated Content".to_string(),
            icon: Some("star".to_string()),
        },
        CustomTemplate {
            id: "t2".to_string(),
            name: "Second".to_string(),
            title: "Second Title".to_string(),
            content: "Second Content".to_string(),
            icon: None,
        },
    ];
    storage::save_custom_templates_at(&data_dir, &templates2).unwrap();

    let loaded = storage::load_custom_templates_at(&data_dir);
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].name, "Updated");
    assert_eq!(loaded[0].icon, Some("star".to_string()));
    assert_eq!(loaded[1].name, "Second");
}

#[test]
fn test_template_with_special_characters() {
    let (_tmp, data_dir) = setup();
    let templates = vec![CustomTemplate {
        id: "t1".to_string(),
        name: "Template com acentos: ação, não, são".to_string(),
        title: "Título com <html> & \"aspas\"".to_string(),
        content: "Conteúdo com\nnewlines\te\ttabs".to_string(),
        icon: Some("icon-with-dashes_and_underscores".to_string()),
    }];
    storage::save_custom_templates_at(&data_dir, &templates).unwrap();
    let loaded = storage::load_custom_templates_at(&data_dir);

    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].name, "Template com acentos: ação, não, são");
    assert_eq!(loaded[0].title, "Título com <html> & \"aspas\"");
    assert!(loaded[0].content.contains("newlines"));
    assert!(loaded[0].content.contains("\ttabs"));
}

#[test]
fn test_template_serialization_roundtrip() {
    let template = CustomTemplate {
        id: "test-id".to_string(),
        name: "My Template".to_string(),
        title: "Template Title".to_string(),
        content: "Template content here".to_string(),
        icon: Some("edit".to_string()),
    };

    let json = serde_json::to_string(&template).unwrap();
    let parsed: CustomTemplate = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.id, "test-id");
    assert_eq!(parsed.name, "My Template");
    assert_eq!(parsed.title, "Template Title");
    assert_eq!(parsed.content, "Template content here");
    assert_eq!(parsed.icon, Some("edit".to_string()));
}

#[test]
fn test_template_missing_optional_icon() {
    let json = r#"{
        "id": "t1",
        "name": "No Icon",
        "title": "Title",
        "content": "Content"
    }"#;

    let template: CustomTemplate = serde_json::from_str(json).unwrap();
    assert_eq!(template.id, "t1");
    assert!(template.icon.is_none());
}
