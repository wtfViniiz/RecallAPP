use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, Manager,
};
use crate::window::{show_main_window, toggle_main_window};

pub fn setup_tray(app: &App) -> Result<tauri::tray::TrayIcon, Box<dyn std::error::Error>> {
    let open_i = MenuItem::with_id(app, "open", "Abrir", true, None::<&str>)?;
    let new_note_i = MenuItem::with_id(app, "new_note", "Nova Nota", true, None::<&str>)?;
    let new_reminder_i = MenuItem::with_id(app, "new_reminder", "Novo Lembrete", true, None::<&str>)?;
    let settings_i = MenuItem::with_id(app, "settings", "Configuracoes", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&open_i, &new_note_i, &new_reminder_i, &settings_i, &quit_i],
    )?;

    let icon = app
        .default_window_icon()
        .ok_or("Icon not found")?
        .clone();

    let tray = TrayIconBuilder::new()
        .icon(icon)
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
                    let _ = window.eval("window.dispatchEvent(new CustomEvent('tray-action', { detail: 'new-note' }));");
                }
            }
            "new_reminder" => {
                show_main_window(app);
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.eval("window.dispatchEvent(new CustomEvent('tray-action', { detail: 'new-reminder' }));");
                }
            }
            "settings" => {
                show_main_window(app);
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.eval("window.dispatchEvent(new CustomEvent('tray-action', { detail: 'settings' }));");
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

    Ok(tray)
}
