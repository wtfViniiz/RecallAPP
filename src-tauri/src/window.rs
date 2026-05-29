use tauri::{AppHandle, Manager, UserAttentionType};

fn get_main_window(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    app.get_webview_window("main")
}

fn show_and_focus(window: &tauri::WebviewWindow) {
    if let Err(e) = window.set_always_on_top(true) {
        eprintln!("Warning: set_always_on_top(true) failed: {}", e);
    }
    if let Err(e) = window.show() {
        eprintln!("Warning: show() failed: {}", e);
    }
    if let Err(e) = window.unminimize() {
        eprintln!("Warning: unminimize() failed: {}", e);
    }
    if let Err(e) = window.request_user_attention(Some(UserAttentionType::Critical)) {
        eprintln!("Warning: request_user_attention() failed: {}", e);
    }
    if let Err(e) = window.set_focus() {
        eprintln!("Warning: set_focus() failed: {}", e);
    }
    if let Err(e) = window.set_always_on_top(false) {
        eprintln!("Warning: set_always_on_top(false) failed: {}", e);
    }
}

pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = get_main_window(app) {
        show_and_focus(&window);
    }
}

pub fn toggle_main_window(app: &AppHandle) {
    if let Some(window) = get_main_window(app) {
        let is_focused = window.is_focused().unwrap_or(false);
        let is_minimized = window.is_minimized().unwrap_or(false);

        if is_focused && !is_minimized {
            if let Err(e) = window.minimize() {
                eprintln!("Warning: minimize() failed: {}", e);
            }
        } else {
            show_and_focus(&window);
        }
    }
}
