#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod models;
mod storage;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_notes,
            commands::get_note,
            commands::create_note,
            commands::update_note,
            commands::delete_note,
            commands::get_reminders,
            commands::create_reminder,
            commands::update_reminder,
            commands::delete_reminder,
            commands::dismiss_reminder,
            commands::get_config,
            commands::update_config,
            commands::get_categories,
            commands::get_tags,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
