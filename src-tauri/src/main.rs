#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

mod commands;
mod models;
mod scheduler;
mod shortcuts;
mod storage;
mod tray;
mod window;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            tray::setup_tray(app)?;
            scheduler::start_scheduler(app.handle().clone());
            shortcuts::register_shortcut(app)?;

            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_clone.hide();
                    }
                });
            }

            Ok(())
        })
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
            commands::set_always_on_top,
            commands::save_image,
            commands::update_shortcut,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
