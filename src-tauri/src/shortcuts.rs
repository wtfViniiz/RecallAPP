use crate::storage;
use tauri::Manager;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

pub fn register_shortcut(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let config = storage::load_config(app.handle());
    let shortcut = parse_shortcut(&config.shortcut).unwrap_or_else(|| {
        // Fallback to Ctrl+Alt+X
        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyX)
    });

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
    })?;

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
