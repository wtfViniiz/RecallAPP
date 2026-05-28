use crate::models::*;
use crate::storage;
use chrono::Utc;
use std::fs;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
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
pub fn save_image(app: AppHandle, base64_data: String, note_id: String) -> Result<String, String> {
    use base64::Engine;
    let data = base64::engine::general_purpose::STANDARD
        .decode(&base64_data)
        .map_err(|e| e.to_string())?;

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

#[tauri::command]
pub fn update_shortcut(app: AppHandle, shortcut_str: String) -> Result<(), String> {
    // Unregister all existing shortcuts
    app.global_shortcut().unregister_all().map_err(|e| e.to_string())?;

    // Parse and register new shortcut
    let shortcut = parse_shortcut(&shortcut_str).ok_or("Invalid shortcut format")?;

    app.global_shortcut().on_shortcut(shortcut, |app, _shortcut, event| {
        if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
            if let Some(window) = app.get_webview_window("main") {
                if window.is_visible().unwrap_or(false) {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        }
    }).map_err(|e| e.to_string())?;

    Ok(())
}

fn parse_shortcut(s: &str) -> Option<Shortcut> {
    let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
    let mut modifiers = Modifiers::empty();
    let mut code = None;

    for part in &parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            "super" | "win" => modifiers |= Modifiers::SUPER,
            key => {
                code = Some(match key {
                    "a" => Code::KeyA, "b" => Code::KeyB, "c" => Code::KeyC,
                    "d" => Code::KeyD, "e" => Code::KeyE, "f" => Code::KeyF,
                    "g" => Code::KeyG, "h" => Code::KeyH, "i" => Code::KeyI,
                    "j" => Code::KeyJ, "k" => Code::KeyK, "l" => Code::KeyL,
                    "m" => Code::KeyM, "n" => Code::KeyN, "o" => Code::KeyO,
                    "p" => Code::KeyP, "q" => Code::KeyQ, "r" => Code::KeyR,
                    "s" => Code::KeyS, "t" => Code::KeyT, "u" => Code::KeyU,
                    "v" => Code::KeyV, "w" => Code::KeyW, "x" => Code::KeyX,
                    "y" => Code::KeyY, "z" => Code::KeyZ,
                    "0" => Code::Digit0, "1" => Code::Digit1, "2" => Code::Digit2,
                    "3" => Code::Digit3, "4" => Code::Digit4, "5" => Code::Digit5,
                    "6" => Code::Digit6, "7" => Code::Digit7, "8" => Code::Digit8,
                    "9" => Code::Digit9,
                    "space" => Code::Space, "tab" => Code::Tab,
                    "enter" => Code::Enter, "escape" => Code::Escape,
                    "backspace" => Code::Backspace, "delete" => Code::Delete,
                    _ => return None,
                });
            }
        }
    }

    Some(Shortcut::new(Some(modifiers), code?))
}
