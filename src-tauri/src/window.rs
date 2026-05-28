use tauri::{AppHandle, Manager, UserAttentionType};

fn get_main_window(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    app.get_webview_window("main")
}

pub fn show_main_window(app: &AppHandle) {
    if let Some(window) = get_main_window(app) {
        let _ = window.set_always_on_top(true);
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.request_user_attention(Some(UserAttentionType::Critical));
        let _ = window.set_focus();
        let _ = window.set_always_on_top(false);
    }
}

pub fn toggle_main_window(app: &AppHandle) {
    if let Some(window) = get_main_window(app) {
        let is_focused = window.is_focused().unwrap_or(false);
        let is_minimized = window.is_minimized().unwrap_or(false);

        if is_focused && !is_minimized {
            let _ = window.minimize();
        } else {
            let _ = window.set_always_on_top(true);
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.request_user_attention(Some(UserAttentionType::Critical));
            let _ = window.set_focus();
            let _ = window.set_always_on_top(false);
        }
    }
}
