use crate::models::*;
use crate::storage;
use chrono::Utc;
use std::fs;
use tauri::{AppHandle, Manager};
use uuid::Uuid;

#[tauri::command]
pub fn get_notes(app: AppHandle, filter: Option<NoteFilter>) -> Vec<Note> {
    storage::list_notes(&app, filter)
}

#[tauri::command]
pub fn get_note(app: AppHandle, id: String) -> Option<Note> {
    storage::get_note(&app, &id)
}

#[tauri::command]
pub fn create_note(app: AppHandle, input: CreateNote) -> Result<Note, String> {
    let now = Utc::now().to_rfc3339();
    let note = Note {
        id: Uuid::new_v4().to_string(),
        title: input.title,
        content: input.content.unwrap_or_default(),
        category: input.category,
        tags: input.tags.unwrap_or_default(),
        pinned: false,
        created_at: now.clone(),
        updated_at: now,
    };
    storage::save_note(&app, &note)?;
    Ok(note)
}

#[tauri::command]
pub fn update_note(app: AppHandle, input: UpdateNote) -> Result<Note, String> {
    let mut note = storage::get_note(&app, &input.id).ok_or("Note not found")?;
    if let Some(title) = input.title {
        note.title = title;
    }
    if let Some(content) = input.content {
        note.content = content;
    }
    if let Some(category) = input.category {
        note.category = Some(category);
    }
    if let Some(tags) = input.tags {
        note.tags = tags;
    }
    if let Some(pinned) = input.pinned {
        note.pinned = pinned;
    }
    note.updated_at = Utc::now().to_rfc3339();
    storage::save_note(&app, &note)?;
    Ok(note)
}

#[tauri::command]
pub fn delete_note(app: AppHandle, id: String) -> Result<(), String> {
    storage::delete_note(&app, &id)
}

#[tauri::command]
pub fn get_reminders(app: AppHandle, status: Option<String>) -> Vec<Reminder> {
    storage::list_reminders(&app, status)
}

#[tauri::command]
pub fn create_reminder(app: AppHandle, input: CreateReminder) -> Result<Reminder, String> {
    let now = Utc::now();
    let trigger_at = if let Some(minutes) = input.relative_minutes {
        (now + chrono::Duration::minutes(minutes)).to_rfc3339()
    } else if let Some(trigger) = input.trigger_at {
        trigger
    } else {
        return Err("Either trigger_at or relative_minutes is required".to_string());
    };

    let reminder = Reminder {
        id: Uuid::new_v4().to_string(),
        title: input.title,
        description: input.description,
        note_id: input.note_id,
        trigger_at,
        repeat: input.repeat,
        relative_minutes: input.relative_minutes,
        status: "pending".to_string(),
        created_at: now.to_rfc3339(),
    };
    storage::save_reminder(&app, &reminder)?;
    Ok(reminder)
}

#[tauri::command]
pub fn update_reminder(app: AppHandle, input: UpdateReminder) -> Result<Reminder, String> {
    let mut reminder =
        storage::get_reminder(&app, &input.id).ok_or("Reminder not found")?;
    if let Some(title) = input.title {
        reminder.title = title;
    }
    if let Some(description) = input.description {
        reminder.description = Some(description);
    }
    if let Some(status) = input.status {
        reminder.status = status;
    }
    storage::save_reminder(&app, &reminder)?;
    Ok(reminder)
}

#[tauri::command]
pub fn delete_reminder(app: AppHandle, id: String) -> Result<(), String> {
    storage::delete_reminder(&app, &id)
}

#[tauri::command]
pub fn dismiss_reminder(app: AppHandle, id: String) -> Result<(), String> {
    let mut reminder =
        storage::get_reminder(&app, &id).ok_or("Reminder not found")?;
    reminder.status = "dismissed".to_string();
    storage::save_reminder(&app, &reminder)
}

#[tauri::command]
pub fn get_config(app: AppHandle) -> Config {
    storage::load_config(&app)
}

#[tauri::command]
pub fn update_config(app: AppHandle, input: Config) -> Result<Config, String> {
    storage::save_config(&app, &input)?;
    Ok(input)
}

#[tauri::command]
pub fn get_categories(app: AppHandle) -> Vec<String> {
    storage::get_categories(&app)
}

#[tauri::command]
pub fn get_tags(app: AppHandle) -> Vec<String> {
    storage::get_tags(&app)
}

#[tauri::command]
pub fn set_always_on_top(app: AppHandle, pinned: bool) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.set_always_on_top(pinned).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn save_image(app: AppHandle, data: Vec<u8>, note_id: String) -> Result<String, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("data")
        .join("images");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let filename = format!("{}_{}.png", note_id, Uuid::new_v4());
    let path = dir.join(&filename);
    fs::write(&path, &data).map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}
