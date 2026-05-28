use crate::storage;
use crate::window::toggle_main_window;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

pub fn register_shortcut(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let config = storage::load_config(app.handle());
    let shortcut = parse_shortcut(&config.shortcut).unwrap_or_else(|| {
        // Fallback to Ctrl+Alt+X
        "Ctrl+Alt+X"
            .parse()
            .expect("fallback shortcut should always parse")
    });

    app.global_shortcut().on_shortcut(shortcut, |app, _shortcut, event| {
        if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
            toggle_main_window(app);
        }
    })?;

    Ok(())
}

pub(crate) fn parse_shortcut(s: &str) -> Option<Shortcut> {
    normalize_shortcut(s)?.parse().ok()
}

fn normalize_shortcut(s: &str) -> Option<String> {
    let mut parts = Vec::new();
    let mut key_seen = false;

    for raw_part in s.split('+') {
        let part = raw_part.trim();
        if part.is_empty() {
            return None;
        }

        let normalized = match part.to_lowercase().as_str() {
            "ctrl" | "control" => "Ctrl".to_string(),
            "alt" | "option" => "Alt".to_string(),
            "shift" => "Shift".to_string(),
            "super" | "win" | "windows" | "meta" | "cmd" | "command" => "Super".to_string(),
            "esc" => "Escape".to_string(),
            "return" => "Enter".to_string(),
            "del" => "Delete".to_string(),
            "pgup" => "PageUp".to_string(),
            "pgdn" => "PageDown".to_string(),
            "up" => "ArrowUp".to_string(),
            "down" => "ArrowDown".to_string(),
            "left" => "ArrowLeft".to_string(),
            "right" => "ArrowRight".to_string(),
            " " => "Space".to_string(),
            _ => part.to_string(),
        };

        let is_modifier = matches!(normalized.as_str(), "Ctrl" | "Alt" | "Shift" | "Super");
        if is_modifier && key_seen {
            return None;
        }
        if !is_modifier {
            if key_seen {
                return None;
            }
            key_seen = true;
        }

        parts.push(normalized);
    }

    key_seen.then(|| parts.join("+"))
}

#[cfg(test)]
mod tests {
    use super::parse_shortcut;

    #[test]
    fn parses_legacy_shortcuts() {
        assert!(parse_shortcut("Ctrl+Alt+x").is_some());
        assert!(parse_shortcut("Win+D").is_some());
    }

    #[test]
    fn parses_keyboard_event_codes() {
        assert!(parse_shortcut("Ctrl+Alt+KeyX").is_some());
        assert!(parse_shortcut("Alt+Digit1").is_some());
        assert!(parse_shortcut("Shift+F2").is_some());
        assert!(parse_shortcut("Ctrl+ArrowUp").is_some());
    }
}
