use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub pinned: bool,
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
            window_width: 400,
            window_height: 600,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteFilter {
    pub search: Option<String>,
    pub category: Option<String>,
    pub tag: Option<String>,
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
        assert_eq!(config.window_width, 400);
        assert_eq!(config.window_height, 600);
        assert_eq!(config.shortcut, "Ctrl+Alt+x");
    }
}
