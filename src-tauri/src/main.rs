#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use recall::{cache, commands, scheduler, shortcuts, tray};
use tauri::Manager;

fn main() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            let note_cache = cache::NoteCache::new();
            app.manage(note_cache.clone());

            let tray = tray::setup_tray(app)?;
            app.manage(tray);
            scheduler::start_scheduler(app.handle().clone(), note_cache);
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
            commands::trash_note,
            commands::restore_note,
            commands::empty_trash,
            commands::get_trashed_notes,
            commands::get_reminders,
            commands::create_reminder,
            commands::update_reminder,
            commands::delete_reminder,
            commands::dismiss_reminder,
            commands::snooze_reminder,
            commands::get_config,
            commands::get_app_version,
            commands::update_config,
            commands::get_categories,
            commands::get_tags,
            commands::get_categories_and_tags,
            commands::set_always_on_top,
            commands::save_image,
            commands::update_shortcut,
            commands::update_new_note_shortcut,
            commands::export_data,
            commands::import_data,
            commands::list_note_versions,
            commands::restore_note_version,
            commands::get_custom_templates,
            commands::save_custom_template,
            commands::delete_custom_template,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, event| {
        if let tauri::RunEvent::ExitRequested { .. } = event {
            scheduler::request_shutdown();
        }
    });
}
