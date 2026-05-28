use serde::{Deserialize, Serialize};

fn default_schema_version() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub pinned: bool,
    #[serde(default)]
    pub trashed: bool,
    #[serde(default)]
    pub trashed_at: Option<String>,
    #[serde(default)]
    pub position: Option<i32>,
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub note_id: Option<String>,
    pub trigger_at: String,
    pub repeat: Option<String>,
    pub relative_minutes: Option<i64>,
    pub status: String,
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme: String,
    pub shortcut: String,
    pub autostart: bool,
    pub check_updates: bool,
    pub window_width: u32,
    pub window_height: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            shortcut: "Ctrl+Alt+x".to_string(),
            autostart: true,
            check_updates: true,
            window_width: 500,
            window_height: 650,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteFilter {
    pub search: Option<String>,
    pub category: Option<String>,
    pub tag: Option<String>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderFilter {
    pub status: Option<String>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResult<T: Serialize> {
    pub items: Vec<T>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteVersion {
    pub id: String,
    pub note_id: String,
    pub title: String,
    pub content: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTemplate {
    pub id: String,
    pub name: String,
    pub title: String,
    pub content: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNote {
    pub title: String,
    pub content: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNote {
    pub id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub pinned: Option<bool>,
    pub position: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReminder {
    pub title: String,
    pub description: Option<String>,
    pub note_id: Option<String>,
    pub trigger_at: Option<String>,
    pub repeat: Option<String>,
    pub relative_minutes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReminder {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub trigger_at: Option<String>,
    pub repeat: Option<String>,
    pub relative_minutes: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_note_defaults() {
        let input = CreateNote {
            title: "Test".to_string(),
            content: None,
            category: None,
            tags: None,
        };
        assert_eq!(input.title, "Test");
        assert!(input.content.is_none());
        assert!(input.tags.is_none());
    }

    #[test]
    fn test_config_default_values() {
        let config = Config::default();
        assert_eq!(config.window_width, 500);
        assert_eq!(config.window_height, 650);
        assert_eq!(config.shortcut, "Ctrl+Alt+x");
        assert_eq!(config.theme, "dark");
        assert!(config.autostart);
        assert!(config.check_updates);
    }

    #[test]
    fn test_note_serialization_roundtrip() {
        let note = Note {
            id: "abc-123".to_string(),
            title: "Minha Nota".to_string(),
            content: "Conteudo com **markdown**".to_string(),
            category: Some("Trabalho".to_string()),
            tags: vec!["urgente".to_string(), "projeto-x".to_string()],
            pinned: true,
            trashed: false,
            trashed_at: None,
            position: None,
            schema_version: 1,
            created_at: "2026-05-28T10:00:00Z".to_string(),
            updated_at: "2026-05-28T12:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&note).unwrap();
        let parsed: Note = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, "abc-123");
        assert_eq!(parsed.title, "Minha Nota");
        assert_eq!(parsed.content, "Conteudo com **markdown**");
        assert_eq!(parsed.category, Some("Trabalho".to_string()));
        assert_eq!(parsed.tags.len(), 2);
        assert!(parsed.pinned);
        assert!(!parsed.trashed);
        assert!(parsed.trashed_at.is_none());
        assert_eq!(parsed.schema_version, 1);
    }

    #[test]
    fn test_note_trashed_serialization() {
        let note = Note {
            id: "trashed-1".to_string(),
            title: "Na Lixeira".to_string(),
            content: "".to_string(),
            category: None,
            tags: vec![],
            pinned: false,
            trashed: true,
            trashed_at: Some("2026-05-28T15:00:00Z".to_string()),
            position: None,
            schema_version: 1,
            created_at: "2026-05-28T10:00:00Z".to_string(),
            updated_at: "2026-05-28T15:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&note).unwrap();
        let parsed: Note = serde_json::from_str(&json).unwrap();

        assert!(parsed.trashed);
        assert_eq!(parsed.trashed_at, Some("2026-05-28T15:00:00Z".to_string()));
    }

    #[test]
    fn test_note_missing_optional_fields() {
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
        assert_eq!(note.id, "minimal");
        assert!(note.category.is_none());
        assert!(!note.trashed);
        assert!(note.trashed_at.is_none());
        assert_eq!(note.schema_version, 1); // default
    }

    #[test]
    fn test_reminder_serialization_roundtrip() {
        let reminder = Reminder {
            id: "rem-1".to_string(),
            title: "Reuniao".to_string(),
            description: Some("Com o time de dev".to_string()),
            note_id: Some("note-123".to_string()),
            trigger_at: "2026-05-29T14:00:00Z".to_string(),
            repeat: Some("weekly".to_string()),
            relative_minutes: None,
            status: "pending".to_string(),
            schema_version: 1,
            created_at: "2026-05-28T10:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&reminder).unwrap();
        let parsed: Reminder = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, "rem-1");
        assert_eq!(parsed.title, "Reuniao");
        assert_eq!(parsed.description, Some("Com o time de dev".to_string()));
        assert_eq!(parsed.note_id, Some("note-123".to_string()));
        assert_eq!(parsed.repeat, Some("weekly".to_string()));
        assert_eq!(parsed.status, "pending");
    }

    #[test]
    fn test_reminder_minimal() {
        let json = r#"{
            "id": "rem-min",
            "title": "Test",
            "trigger_at": "2026-05-28T10:00:00Z",
            "status": "pending",
            "created_at": "2026-05-28T10:00:00Z"
        }"#;

        let reminder: Reminder = serde_json::from_str(json).unwrap();
        assert!(reminder.description.is_none());
        assert!(reminder.note_id.is_none());
        assert!(reminder.repeat.is_none());
        assert!(reminder.relative_minutes.is_none());
        assert_eq!(reminder.schema_version, 1);
    }

    #[test]
    fn test_note_filter_all_fields() {
        let filter = NoteFilter {
            search: Some("test".to_string()),
            category: Some("Work".to_string()),
            tag: Some("urgent".to_string()),
            offset: Some(0),
            limit: Some(30),
        };

        assert_eq!(filter.search, Some("test".to_string()));
        assert_eq!(filter.category, Some("Work".to_string()));
        assert_eq!(filter.tag, Some("urgent".to_string()));
        assert_eq!(filter.offset, Some(0));
        assert_eq!(filter.limit, Some(30));
    }

    #[test]
    fn test_note_filter_empty() {
        let filter = NoteFilter {
            search: None,
            category: None,
            tag: None,
            offset: None,
            limit: None,
        };

        assert!(filter.search.is_none());
        assert!(filter.category.is_none());
        assert!(filter.tag.is_none());
        assert!(filter.offset.is_none());
        assert!(filter.limit.is_none());
    }

    #[test]
    fn test_create_reminder_with_relative_minutes() {
        let input = CreateReminder {
            title: "Lembrete rapido".to_string(),
            description: None,
            note_id: None,
            trigger_at: None,
            repeat: None,
            relative_minutes: Some(30),
        };

        assert_eq!(input.relative_minutes, Some(30));
        assert!(input.trigger_at.is_none());
    }

    #[test]
    fn test_update_note_partial() {
        let input = UpdateNote {
            id: "note-1".to_string(),
            title: Some("Novo titulo".to_string()),
            content: None,
            category: None,
            tags: None,
            pinned: None,
            position: None,
        };

        assert_eq!(input.id, "note-1");
        assert_eq!(input.title, Some("Novo titulo".to_string()));
        assert!(input.content.is_none());
        assert!(input.pinned.is_none());
    }

    #[test]
    fn test_update_reminder_partial() {
        let input = UpdateReminder {
            id: "rem-1".to_string(),
            title: Some("Novo titulo".to_string()),
            description: None,
            status: Some("dismissed".to_string()),
            trigger_at: None,
            repeat: None,
            relative_minutes: None,
        };

        assert_eq!(input.id, "rem-1");
        assert_eq!(input.status, Some("dismissed".to_string()));
        assert!(input.description.is_none());
    }

    #[test]
    fn test_update_note_position_only() {
        let input = UpdateNote {
            id: "note-1".to_string(),
            title: None,
            content: None,
            category: None,
            tags: None,
            pinned: None,
            position: Some(10),
        };

        assert_eq!(input.id, "note-1");
        assert_eq!(input.position, Some(10));
        assert!(input.title.is_none());
        assert!(input.content.is_none());
        assert!(input.category.is_none());
        assert!(input.tags.is_none());
        assert!(input.pinned.is_none());
    }

    #[test]
    fn test_update_note_all_none_except_id() {
        let input = UpdateNote {
            id: "note-1".to_string(),
            title: None,
            content: None,
            category: None,
            tags: None,
            pinned: None,
            position: None,
        };

        assert_eq!(input.id, "note-1");
        assert!(input.title.is_none());
        assert!(input.content.is_none());
        assert!(input.category.is_none());
        assert!(input.tags.is_none());
        assert!(input.pinned.is_none());
        assert!(input.position.is_none());
    }

    #[test]
    fn test_note_version_serialization_roundtrip() {
        let version = NoteVersion {
            id: "v-123".to_string(),
            note_id: "n-456".to_string(),
            title: "Version Title".to_string(),
            content: "Version content with **markdown**".to_string(),
            category: Some("Work".to_string()),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            created_at: "2026-05-28T10:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&version).unwrap();
        let parsed: NoteVersion = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, "v-123");
        assert_eq!(parsed.note_id, "n-456");
        assert_eq!(parsed.title, "Version Title");
        assert_eq!(parsed.content, "Version content with **markdown**");
        assert_eq!(parsed.category, Some("Work".to_string()));
        assert_eq!(parsed.tags.len(), 2);
        assert_eq!(parsed.created_at, "2026-05-28T10:00:00Z");
    }

    #[test]
    fn test_note_version_with_empty_fields() {
        let version = NoteVersion {
            id: "v-empty".to_string(),
            note_id: "n-1".to_string(),
            title: "".to_string(),
            content: "".to_string(),
            category: None,
            tags: vec![],
            created_at: "2026-05-28T10:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&version).unwrap();
        let parsed: NoteVersion = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.title, "");
        assert_eq!(parsed.content, "");
        assert!(parsed.category.is_none());
        assert!(parsed.tags.is_empty());
    }

    #[test]
    fn test_custom_template_serialization_roundtrip() {
        let template = CustomTemplate {
            id: "tpl-1".to_string(),
            name: "Daily Standup".to_string(),
            title: "Standup - {{date}}".to_string(),
            content: "## Yesterday\n\n## Today\n\n## Blockers\n".to_string(),
            icon: Some("calendar".to_string()),
        };

        let json = serde_json::to_string(&template).unwrap();
        let parsed: CustomTemplate = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, "tpl-1");
        assert_eq!(parsed.name, "Daily Standup");
        assert_eq!(parsed.title, "Standup - {{date}}");
        assert!(parsed.content.contains("Yesterday"));
        assert_eq!(parsed.icon, Some("calendar".to_string()));
    }

    #[test]
    fn test_custom_template_without_icon() {
        let template = CustomTemplate {
            id: "tpl-2".to_string(),
            name: "Simple".to_string(),
            title: "Simple Template".to_string(),
            content: "Content".to_string(),
            icon: None,
        };

        let json = serde_json::to_string(&template).unwrap();
        let parsed: CustomTemplate = serde_json::from_str(&json).unwrap();

        assert!(parsed.icon.is_none());
    }

    #[test]
    fn test_custom_template_json_structure() {
        let template = CustomTemplate {
            id: "t1".to_string(),
            name: "Test".to_string(),
            title: "Title".to_string(),
            content: "Body".to_string(),
            icon: None,
        };

        let json = serde_json::to_string(&template).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(value.is_object());
        assert_eq!(value["id"].as_str().unwrap(), "t1");
        assert_eq!(value["name"].as_str().unwrap(), "Test");
        assert_eq!(value["title"].as_str().unwrap(), "Title");
        assert_eq!(value["content"].as_str().unwrap(), "Body");
        // icon should be null in JSON when None
        assert!(value["icon"].is_null());
    }

    #[test]
    fn test_note_version_json_structure() {
        let version = NoteVersion {
            id: "v1".to_string(),
            note_id: "n1".to_string(),
            title: "T".to_string(),
            content: "C".to_string(),
            category: None,
            tags: vec![],
            created_at: "2026-05-28T10:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&version).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(value.is_object());
        assert_eq!(value["id"].as_str().unwrap(), "v1");
        assert_eq!(value["note_id"].as_str().unwrap(), "n1");
        assert!(value["tags"].is_array());
        assert_eq!(value["tags"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_update_note_with_position_negative() {
        let input = UpdateNote {
            id: "note-1".to_string(),
            title: None,
            content: None,
            category: None,
            tags: None,
            pinned: None,
            position: Some(-1),
        };
        assert_eq!(input.position, Some(-1));
    }

    #[test]
    fn test_note_with_position_zero() {
        let note = Note {
            id: "n1".to_string(),
            title: "Zero".to_string(),
            content: "".to_string(),
            category: None,
            tags: vec![],
            pinned: false,
            trashed: false,
            trashed_at: None,
            position: Some(0),
            schema_version: 1,
            created_at: "2026-05-28T10:00:00Z".to_string(),
            updated_at: "2026-05-28T10:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&note).unwrap();
        let parsed: Note = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.position, Some(0));
    }
}
