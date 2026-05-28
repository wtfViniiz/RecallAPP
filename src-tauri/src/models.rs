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
}
