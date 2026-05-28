use crate::cache::NoteCache;
use crate::models::*;
use crate::shortcuts::parse_shortcut;
use crate::storage;
use crate::window::toggle_main_window;
use chrono::Utc;
use std::fs;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use uuid::Uuid;

fn validate_string(s: &str, max_len: usize, field_name: &str) -> Result<(), String> {
    let char_count = s.chars().count();
    if char_count > max_len {
        return Err(format!("{} excede {} caracteres (tem {})", field_name, max_len, char_count));
    }
    Ok(())
}

fn sanitize_path_component(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

fn validate_id(id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err("ID nao pode ser vazio".to_string());
    }
    if id.len() > 100 {
        return Err("ID excede 100 caracteres".to_string());
    }
    if id.contains("..") || id.contains('/') || id.contains('\\') || id.contains('\0') {
        return Err("ID contem caracteres invalidos".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn get_notes(
    app: AppHandle,
    cache: State<'_, NoteCache>,
    filter: Option<NoteFilter>,
) -> PaginatedResult<Note> {
    cache.list_notes_paginated(&app, filter)
}

#[tauri::command]
pub fn get_note(app: AppHandle, cache: State<'_, NoteCache>, id: String) -> Result<Option<Note>, String> {
    validate_id(&id)?;
    Ok(cache.get_note(&app, &id))
}

#[tauri::command]
pub fn create_note(app: AppHandle, cache: State<'_, NoteCache>, input: CreateNote) -> Result<Note, String> {
    validate_string(&input.title, 500, "Titulo")?;
    if let Some(ref content) = input.content {
        validate_string(content, 100_000, "Conteudo")?;
    }

    let now = Utc::now().to_rfc3339();
    let note = Note {
        id: Uuid::new_v4().to_string(),
        title: input.title,
        content: input.content.unwrap_or_default(),
        category: input.category,
        tags: input.tags.unwrap_or_default(),
        pinned: false,
        trashed: false,
        trashed_at: None,
        schema_version: 1,
        created_at: now.clone(),
        updated_at: now,
    };
    cache.save_note(&app, &note)?;
    Ok(note)
}

#[tauri::command]
pub fn update_note(app: AppHandle, cache: State<'_, NoteCache>, input: UpdateNote) -> Result<Note, String> {
    validate_id(&input.id)?;
    if let Some(ref title) = input.title {
        validate_string(title, 500, "Titulo")?;
    }
    if let Some(ref content) = input.content {
        validate_string(content, 100_000, "Conteudo")?;
    }

    let mut note = cache.get_note(&app, &input.id).ok_or("Nota nao encontrada")?;
    if let Some(title) = input.title {
        note.title = title;
    }
    if let Some(content) = input.content {
        note.content = content;
    }
    note.category = input
        .category
        .and_then(|c| if c.is_empty() { None } else { Some(c) });
    if let Some(tags) = input.tags {
        note.tags = tags;
    }
    if let Some(pinned) = input.pinned {
        note.pinned = pinned;
    }
    note.updated_at = Utc::now().to_rfc3339();
    cache.save_note(&app, &note)?;
    Ok(note)
}

#[tauri::command]
pub fn delete_note(app: AppHandle, cache: State<'_, NoteCache>, id: String) -> Result<(), String> {
    validate_id(&id)?;
    cache.delete_note(&app, &id)
}

#[tauri::command]
pub fn trash_note(app: AppHandle, cache: State<'_, NoteCache>, id: String) -> Result<(), String> {
    validate_id(&id)?;
    let mut note = cache.get_note(&app, &id).ok_or("Nota nao encontrada")?;
    note.trashed = true;
    note.trashed_at = Some(Utc::now().to_rfc3339());
    note.updated_at = Utc::now().to_rfc3339();
    cache.save_note(&app, &note)
}

#[tauri::command]
pub fn restore_note(app: AppHandle, cache: State<'_, NoteCache>, id: String) -> Result<(), String> {
    validate_id(&id)?;
    let mut note = cache.get_note(&app, &id).ok_or("Nota nao encontrada")?;
    note.trashed = false;
    note.trashed_at = None;
    note.updated_at = Utc::now().to_rfc3339();
    cache.save_note(&app, &note)
}

#[tauri::command]
pub fn empty_trash(app: AppHandle, cache: State<'_, NoteCache>) -> Result<u32, String> {
    let notes = cache.list_trashed_notes(&app);
    let count = notes.len() as u32;
    let mut errors = Vec::new();
    for note in notes {
        if let Err(e) = cache.delete_note(&app, &note.id) {
            errors.push(format!("{}: {}", note.id, e));
        }
    }
    if errors.is_empty() {
        Ok(count)
    } else {
        Err(format!("{} notas deletadas, {} erros: {}", count - errors.len() as u32, errors.len(), errors.join("; ")))
    }
}

#[tauri::command]
pub fn get_trashed_notes(app: AppHandle, cache: State<'_, NoteCache>) -> Vec<Note> {
    cache.list_trashed_notes(&app)
}

#[tauri::command]
pub fn get_reminders(
    app: AppHandle,
    cache: State<'_, NoteCache>,
    filter: Option<ReminderFilter>,
) -> PaginatedResult<Reminder> {
    let filter = filter.unwrap_or(ReminderFilter {
        status: None,
        offset: None,
        limit: None,
    });
    cache.list_reminders_paginated(&app, &filter)
}

#[tauri::command]
pub fn create_reminder(app: AppHandle, cache: State<'_, NoteCache>, input: CreateReminder) -> Result<Reminder, String> {
    validate_string(&input.title, 500, "Titulo")?;
    if let Some(ref desc) = input.description {
        validate_string(desc, 2000, "Descricao")?;
    }

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
        schema_version: 1,
        created_at: now.to_rfc3339(),
    };
    cache.save_reminder(&app, &reminder)?;
    Ok(reminder)
}

#[tauri::command]
pub fn update_reminder(app: AppHandle, cache: State<'_, NoteCache>, input: UpdateReminder) -> Result<Reminder, String> {
    validate_id(&input.id)?;
    if let Some(ref title) = input.title {
        validate_string(title, 500, "Titulo")?;
    }
    if let Some(ref desc) = input.description {
        validate_string(desc, 2000, "Descricao")?;
    }

    let mut reminder = cache
        .get_reminder(&app, &input.id)
        .ok_or("Lembrete nao encontrado")?;
    if let Some(title) = input.title {
        reminder.title = title;
    }
    if let Some(description) = input.description {
        reminder.description = Some(description);
    }
    if let Some(status) = input.status {
        let valid_statuses = ["pending", "fired", "dismissed"];
        if !valid_statuses.contains(&status.as_str()) {
            return Err(format!(
                "Status invalido: {}. Use: {:?}",
                status, valid_statuses
            ));
        }
        reminder.status = status;
    }
    if let Some(trigger_at) = input.trigger_at {
        reminder.trigger_at = trigger_at;
    }
    if let Some(repeat) = input.repeat {
        reminder.repeat = if repeat.is_empty() {
            None
        } else {
            Some(repeat)
        };
    }
    if let Some(relative_minutes) = input.relative_minutes {
        reminder.relative_minutes = Some(relative_minutes);
    }
    cache.save_reminder(&app, &reminder)?;
    Ok(reminder)
}

#[tauri::command]
pub fn delete_reminder(app: AppHandle, cache: State<'_, NoteCache>, id: String) -> Result<(), String> {
    validate_id(&id)?;
    cache.delete_reminder(&app, &id)
}

#[tauri::command]
pub fn dismiss_reminder(app: AppHandle, cache: State<'_, NoteCache>, id: String) -> Result<(), String> {
    validate_id(&id)?;
    let mut reminder = cache
        .get_reminder(&app, &id)
        .ok_or("Lembrete nao encontrado")?;
    reminder.status = "dismissed".to_string();
    cache.save_reminder(&app, &reminder)
}

#[tauri::command]
pub fn snooze_reminder(app: AppHandle, cache: State<'_, NoteCache>, id: String, minutes: i64) -> Result<(), String> {
    validate_id(&id)?;
    if minutes < 1 || minutes > 10080 {
        return Err("Snooze deve ser entre 1 minuto e 7 dias".to_string());
    }
    let mut reminder = cache
        .get_reminder(&app, &id)
        .ok_or("Lembrete nao encontrado")?;
    let trigger: chrono::DateTime<chrono::Utc> = reminder
        .trigger_at
        .parse()
        .map_err(|e: chrono::ParseError| e.to_string())?;
    reminder.trigger_at = (trigger + chrono::Duration::minutes(minutes)).to_rfc3339();
    reminder.status = "pending".to_string();
    cache.save_reminder(&app, &reminder)
}

#[tauri::command]
pub fn get_config(app: AppHandle) -> Config {
    storage::load_config(&app)
}

#[tauri::command]
pub fn get_app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command]
pub fn update_config(app: AppHandle, input: Config) -> Result<Config, String> {
    let valid_themes = ["dark", "light"];
    if !valid_themes.contains(&input.theme.as_str()) {
        return Err(format!("Tema invalido: {}. Use: {:?}", input.theme, valid_themes));
    }
    if input.window_width < 300 || input.window_width > 3840 {
        return Err("Largura da janela deve estar entre 300 e 3840".to_string());
    }
    if input.window_height < 300 || input.window_height > 2160 {
        return Err("Altura da janela deve estar entre 300 e 2160".to_string());
    }
    if input.shortcut.is_empty() || input.shortcut.len() > 50 {
        return Err("Atalho invalido".to_string());
    }
    storage::save_config(&app, &input)?;
    Ok(input)
}

#[tauri::command]
pub fn get_categories(app: AppHandle, cache: State<'_, NoteCache>) -> Vec<String> {
    let notes = cache.list_notes(&app, None);
    let mut cats: Vec<String> = notes
        .iter()
        .filter_map(|n| n.category.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    cats.sort();
    cats
}

#[tauri::command]
pub fn get_tags(app: AppHandle, cache: State<'_, NoteCache>) -> Vec<String> {
    let notes = cache.list_notes(&app, None);
    let mut tags: Vec<String> = notes
        .iter()
        .flat_map(|n| n.tags.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    tags.sort();
    tags
}

#[derive(serde::Serialize)]
pub struct CategoriesAndTags {
    pub categories: Vec<String>,
    pub tags: Vec<String>,
}

#[tauri::command]
pub fn get_categories_and_tags(app: AppHandle, cache: State<'_, NoteCache>) -> CategoriesAndTags {
    let notes = cache.list_notes(&app, None);
    let mut cats: Vec<String> = notes
        .iter()
        .filter_map(|n| n.category.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    cats.sort();
    let mut tags: Vec<String> = notes
        .iter()
        .flat_map(|n| n.tags.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    tags.sort();
    CategoriesAndTags {
        categories: cats,
        tags,
    }
}

#[tauri::command]
pub fn set_always_on_top(app: AppHandle, pinned: bool) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_always_on_top(pinned)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn save_image(
    app: AppHandle,
    base64_data: String,
    note_id: String,
) -> Result<String, String> {
    use base64::Engine;

    const MAX_IMAGE_SIZE: usize = 20 * 1024 * 1024; // 20MB encoded
    if base64_data.len() > MAX_IMAGE_SIZE {
        return Err("Imagem excede 20MB".to_string());
    }

    let data = base64::engine::general_purpose::STANDARD
        .decode(&base64_data)
        .map_err(|e| e.to_string())?;

    if data.len() > 15 * 1024 * 1024 {
        return Err("Imagem decodificada excede 15MB".to_string());
    }

    let safe_id = sanitize_path_component(&note_id);
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("data")
        .join("images");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    // Detect format from magic bytes
    let ext = if data.len() >= 4 && data[0..4] == [0x89, 0x50, 0x4E, 0x47] {
        "png"
    } else if data.len() >= 2 && data[0..2] == [0xFF, 0xD8] {
        "jpg"
    } else if data.len() >= 4 && data[0..4] == [0x47, 0x49, 0x46, 0x38] {
        "gif"
    } else if data.len() >= 4 && data[0..4] == [0x52, 0x49, 0x46, 0x46] {
        "webp"
    } else if data.len() >= 4 && data[0..4] == [0x3C, 0x73, 0x76, 0x67] {
        "svg"
    } else {
        "png"
    };

    let filename = format!("{}_{}.{}", safe_id, Uuid::new_v4(), ext);
    let path = dir.join(&filename);
    fs::write(&path, &data).map_err(|e| e.to_string())?;

    Ok(filename)
}

#[tauri::command]
pub fn update_shortcut(app: AppHandle, shortcut_str: String) -> Result<(), String> {
    let shortcut = parse_shortcut(&shortcut_str).ok_or("Formato de atalho invalido")?;

    app.global_shortcut()
        .unregister_all()
        .map_err(|e| e.to_string())?;

    app.global_shortcut()
        .on_shortcut(shortcut, |app, _shortcut, event| {
            if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                toggle_main_window(app);
            }
        })
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn export_data(app: AppHandle) -> Result<String, String> {
    let notes = storage::list_all_notes(&app);
    let reminders = storage::list_reminders(&app, None);
    let config = storage::load_config(&app);

    let export = serde_json::json!({
        "version": 1,
        "exported_at": Utc::now().to_rfc3339(),
        "notes": notes,
        "reminders": reminders,
        "config": config,
    });

    serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_data(app: AppHandle, cache: State<'_, NoteCache>, json_data: String) -> Result<String, String> {
    const MAX_IMPORT_SIZE: usize = 50 * 1024 * 1024; // 50MB
    if json_data.len() > MAX_IMPORT_SIZE {
        return Err("Arquivo de importacao excede 50MB".to_string());
    }

    let data: serde_json::Value =
        serde_json::from_str(&json_data).map_err(|e| format!("JSON invalido: {}", e))?;

    let mut imported_notes = 0;
    let mut imported_reminders = 0;
    let mut skipped = 0;

    if let Some(notes) = data["notes"].as_array() {
        for note_value in notes {
            match serde_json::from_value::<Note>(note_value.clone()) {
                Ok(note) => {
                    if validate_id(&note.id).is_err() {
                        skipped += 1;
                        continue;
                    }
                    validate_string(&note.title, 500, "Titulo (import)")
                        .map_err(|e| format!("Nota '{}': {}", note.id, e))?;
                    if note.content.len() > 100_000 {
                        return Err(format!(
                            "Nota '{}' conteudo excede 100k caracteres",
                            note.id
                        ));
                    }
                    cache.save_note(&app, &note)?;
                    imported_notes += 1;
                }
                Err(_) => skipped += 1,
            }
        }
    }

    if let Some(reminders) = data["reminders"].as_array() {
        for reminder_value in reminders {
            match serde_json::from_value::<Reminder>(reminder_value.clone()) {
                Ok(reminder) => {
                    if validate_id(&reminder.id).is_err() {
                        skipped += 1;
                        continue;
                    }
                    validate_string(&reminder.title, 500, "Titulo (import)")
                        .map_err(|e| format!("Lembrete '{}': {}", reminder.id, e))?;
                    cache.save_reminder(&app, &reminder)?;
                    imported_reminders += 1;
                }
                Err(_) => skipped += 1,
            }
        }
    }

    let msg = format!(
        "Importadas {} notas e {} lembretes",
        imported_notes, imported_reminders
    );
    if skipped > 0 {
        Ok(format!("{} ({} itens ignorados)", msg, skipped))
    } else {
        Ok(msg)
    }
}
