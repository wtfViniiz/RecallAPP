use crate::storage;
use crate::window::{show_main_window, toggle_main_window};
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

pub fn resolve_shortcut_with_fallback(config_shortcut: &str) -> Shortcut {
    parse_shortcut(config_shortcut)
        .or_else(|| parse_shortcut("Ctrl+Alt+X"))
        .unwrap_or_else(|| {
            eprintln!(
                "WARNING: fallback shortcut 'Ctrl+Alt+X' failed to parse, using hardcoded shortcut"
            );
            "Ctrl+Alt+X".parse().unwrap_or_else(|_| {
                eprintln!("ERROR: all shortcut parsing failed, using Ctrl+X as last resort");
                "Ctrl+X".parse().expect("Ctrl+X must parse")
            })
        })
}

pub fn register_shortcut(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let config = storage::load_config(app.handle());
    let shortcut = resolve_shortcut_with_fallback(&config.shortcut);

    app.global_shortcut()
        .on_shortcut(shortcut, |app, _shortcut, event| {
            if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                toggle_main_window(app);
            }
        })?;

    // Register new note shortcut if configured
    register_new_note_shortcut(app, &config.new_note_shortcut)?;

    Ok(())
}

fn register_new_note_shortcut(
    app: &tauri::App,
    shortcut_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if shortcut_str.is_empty() {
        return Ok(());
    }

    if let Some(shortcut) = parse_shortcut(shortcut_str) {
        app.global_shortcut().on_shortcut(shortcut, |app, _shortcut, event| {
            if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                show_main_window(app);
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.eval("window.dispatchEvent(new CustomEvent('tray-action', { detail: 'direct-new-note' }));");
                }
            }
        })?;
    }

    Ok(())
}

pub(crate) fn parse_shortcut(s: &str) -> Option<Shortcut> {
    normalize_shortcut(s)?.parse().ok()
}

fn normalize_shortcut(s: &str) -> Option<String> {
    let mut parts = Vec::new();
    let mut key_seen = false;

    for raw_part in s.split('+') {
        // Handle space key before trimming (trim would strip the space)
        let part = if raw_part == " " {
            "Space"
        } else {
            raw_part.trim()
        };
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
            " " | "space" => "Space".to_string(),
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
    use super::{normalize_shortcut, parse_shortcut};

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

    #[test]
    fn normalizes_modifiers() {
        assert_eq!(
            normalize_shortcut("ctrl+alt+x"),
            Some("Ctrl+Alt+x".to_string())
        );
        assert_eq!(
            normalize_shortcut("CONTROL+ALT+x"),
            Some("Ctrl+Alt+x".to_string())
        );
        assert_eq!(
            normalize_shortcut("shift+shift+x"),
            Some("Shift+Shift+x".to_string())
        );
    }

    #[test]
    fn normalizes_super_variants() {
        assert_eq!(normalize_shortcut("win+d"), Some("Super+d".to_string()));
        assert_eq!(normalize_shortcut("super+d"), Some("Super+d".to_string()));
        assert_eq!(normalize_shortcut("meta+d"), Some("Super+d".to_string()));
        assert_eq!(normalize_shortcut("cmd+d"), Some("Super+d".to_string()));
        assert_eq!(normalize_shortcut("command+d"), Some("Super+d".to_string()));
    }

    #[test]
    fn normalizes_special_keys() {
        assert_eq!(
            normalize_shortcut("ctrl+esc"),
            Some("Ctrl+Escape".to_string())
        );
        assert_eq!(
            normalize_shortcut("ctrl+return"),
            Some("Ctrl+Enter".to_string())
        );
        assert_eq!(
            normalize_shortcut("ctrl+del"),
            Some("Ctrl+Delete".to_string())
        );
        assert_eq!(
            normalize_shortcut("ctrl+pgup"),
            Some("Ctrl+PageUp".to_string())
        );
        assert_eq!(
            normalize_shortcut("ctrl+pgdn"),
            Some("Ctrl+PageDown".to_string())
        );
        assert_eq!(
            normalize_shortcut("ctrl+up"),
            Some("Ctrl+ArrowUp".to_string())
        );
        assert_eq!(
            normalize_shortcut("ctrl+down"),
            Some("Ctrl+ArrowDown".to_string())
        );
        assert_eq!(
            normalize_shortcut("ctrl+left"),
            Some("Ctrl+ArrowLeft".to_string())
        );
        assert_eq!(
            normalize_shortcut("ctrl+right"),
            Some("Ctrl+ArrowRight".to_string())
        );
    }

    #[test]
    fn rejects_empty_parts() {
        assert!(normalize_shortcut("").is_none());
        assert!(normalize_shortcut("ctrl++x").is_none());
        assert!(normalize_shortcut("+x").is_none());
    }

    #[test]
    fn rejects_double_keys() {
        // Two non-modifier keys
        assert!(normalize_shortcut("ctrl+x+y").is_none());
        assert!(normalize_shortcut("a+b").is_none());
    }

    #[test]
    fn rejects_modifier_after_key() {
        assert!(normalize_shortcut("x+ctrl").is_none());
        assert!(normalize_shortcut("x+alt").is_none());
    }

    #[test]
    fn accepts_single_modifier_plus_key() {
        assert!(parse_shortcut("Ctrl+X").is_some());
        assert!(parse_shortcut("Alt+F4").is_some());
        assert!(parse_shortcut("Shift+A").is_some());
    }

    #[test]
    fn handles_whitespace() {
        assert_eq!(
            normalize_shortcut(" Ctrl + Alt + X "),
            Some("Ctrl+Alt+X".to_string())
        );
    }

    #[test]
    fn case_insensitive_input() {
        assert!(parse_shortcut("ctrl+alt+x").is_some());
        assert!(parse_shortcut("CTRL+ALT+X").is_some());
        assert!(parse_shortcut("Ctrl+Alt+X").is_some());
    }
}
