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
    if let Some(ref category) = input.category {
        validate_string(category, 100, "Categoria")?;
    }
    if let Some(ref tags) = input.tags {
        if tags.len() > 20 {
            return Err("Maximo de 20 tags".to_string());
        }
        for tag in tags {
            validate_string(tag, 100, "Tag")?;
        }
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
        position: None,
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

    let note = cache.get_note(&app, &input.id).ok_or("Nota nao encontrada")?;
    // Save version snapshot only when content changes (skip for position/pinned-only updates)
    let has_content_change = input.title.is_some() || input.content.is_some()
        || input.category.is_some() || input.tags.is_some();
    if has_content_change {
        if let Err(e) = storage::save_note_version(&app, &note) {
            eprintln!("[Recall] Aviso: falha ao salvar versao: {}", e);
        }
    }
    let mut note = note;
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
    if let Some(position) = input.position {
        note.position = Some(position);
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
        chrono::DateTime::parse_from_rfc3339(&trigger)
            .map_err(|_| format!("trigger_at invalido: '{}'. Use formato ISO 8601", trigger))?;
        trigger
    } else {
        return Err("Either trigger_at or relative_minutes is required".to_string());
    };

    if let Some(ref repeat) = input.repeat {
        let valid_repeats = ["daily", "weekly", "monthly"];
        if !valid_repeats.contains(&repeat.as_str()) {
            return Err(format!("Repeat invalido: '{}'. Use: {:?}", repeat, valid_repeats));
        }
    }

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
    if let Some(ref trigger_at) = input.trigger_at {
        chrono::DateTime::parse_from_rfc3339(trigger_at)
            .map_err(|_| format!("trigger_at invalido: '{}'. Use formato ISO 8601 (ex: 2026-01-01T12:00:00Z)", trigger_at))?;
        reminder.trigger_at = trigger_at.clone();
    }
    if let Some(repeat) = input.repeat {
        let valid_repeats = ["daily", "weekly", "monthly"];
        if !repeat.is_empty() && !valid_repeats.contains(&repeat.as_str()) {
            return Err(format!(
                "Repeat invalido: '{}'. Use: {:?}",
                repeat, valid_repeats
            ));
        }
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

    validate_id(&note_id)?;
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

    // Return absolute path so WebView can resolve the image
    let abs_path = path.to_string_lossy().to_string();
    Ok(abs_path)
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
pub fn list_note_versions(app: AppHandle, note_id: String) -> Result<Vec<NoteVersion>, String> {
    validate_id(&note_id)?;
    Ok(storage::list_note_versions(&app, &note_id))
}

#[tauri::command]
pub fn restore_note_version(app: AppHandle, cache: State<'_, NoteCache>, note_id: String, version_id: String) -> Result<Note, String> {
    validate_id(&note_id)?;
    validate_id(&version_id)?;
    let note = storage::restore_note_version(&app, &note_id, &version_id)?;
    cache.invalidate_notes();
    Ok(note)
}

#[tauri::command]
pub fn get_custom_templates(app: AppHandle) -> Vec<CustomTemplate> {
    storage::load_custom_templates(&app)
}

#[tauri::command]
pub fn save_custom_template(app: AppHandle, input: CustomTemplate) -> Result<CustomTemplate, String> {
    validate_string(&input.name, 100, "Nome do template")?;
    validate_string(&input.id, 100, "ID do template")?;
    let mut templates = storage::load_custom_templates(&app);
    if let Some(existing) = templates.iter_mut().find(|t| t.id == input.id) {
        *existing = input.clone();
    } else {
        templates.push(input.clone());
    }
    storage::save_custom_templates(&app, &templates)?;
    Ok(input)
}

#[tauri::command]
pub fn delete_custom_template(app: AppHandle, id: String) -> Result<(), String> {
    validate_id(&id)?;
    let mut templates = storage::load_custom_templates(&app);
    templates.retain(|t| t.id != id);
    storage::save_custom_templates(&app, &templates)
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
                    if note.content.chars().count() > 100_000 {
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
