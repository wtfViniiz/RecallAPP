use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Manager,
};
use crate::window::{show_main_window, toggle_main_window};

pub fn setup_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let open_i = MenuItem::with_id(app, "open", "Abrir", true, None::<&str>)?;
    let new_note_i = MenuItem::with_id(app, "new_note", "Nova Nota", true, None::<&str>)?;
    let new_reminder_i = MenuItem::with_id(app, "new_reminder", "Novo Lembrete", true, None::<&str>)?;
    let settings_i = MenuItem::with_id(app, "settings", "Configuracoes", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&open_i, &new_note_i, &new_reminder_i, &settings_i, &quit_i],
    )?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Recall")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "open" => {
                show_main_window(app);
            }
            "new_note" => {
                show_main_window(app);
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.eval(
                        "document.querySelector('[data-tab=\"notes\"]').click(); setTimeout(() => document.getElementById('btn-new-note')?.click(), 100);"
                    );
                }
            }
            "new_reminder" => {
                show_main_window(app);
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.eval(
                        "document.querySelector('[data-tab=\"reminders\"]').click(); setTimeout(() => document.getElementById('btn-new-reminder')?.click(), 100);"
                    );
                }
            }
            "settings" => {
                show_main_window(app);
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.eval("document.querySelector('[data-tab=\"settings\"]').click();");
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                toggle_main_window(app);
            }
        })
        .build(app)?;

    Ok(())
}
