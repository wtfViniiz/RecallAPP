use recall::models::Config;
use recall::storage;
use std::path::PathBuf;
use tempfile::TempDir;

fn setup() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let data_dir = dir.path().join("data");
    storage::ensure_dirs(&data_dir);
    (dir, data_dir)
}

#[test]
fn test_config_save_and_load() {
    let (_tmp, data_dir) = setup();
    let config = Config {
        theme: "light".to_string(),
        shortcut: "Ctrl+Shift+N".to_string(),
        autostart: false,
        check_updates: true,
        window_width: 800,
        window_height: 600,
    };
    storage::save_config_at(&data_dir, &config).unwrap();
    let loaded = storage::load_config_at(&data_dir);
    assert_eq!(loaded.theme, "light");
    assert_eq!(loaded.shortcut, "Ctrl+Shift+N");
    assert!(!loaded.autostart);
    assert!(loaded.check_updates);
    assert_eq!(loaded.window_width, 800);
    assert_eq!(loaded.window_height, 600);
}

#[test]
fn test_config_default_on_missing_file() {
    let (_tmp, data_dir) = setup();
    let loaded = storage::load_config_at(&data_dir);
    assert_eq!(loaded.theme, "dark");
    assert_eq!(loaded.shortcut, "Ctrl+Alt+x");
    assert!(loaded.autostart);
    assert!(loaded.check_updates);
    assert_eq!(loaded.window_width, 500);
    assert_eq!(loaded.window_height, 650);
}

#[test]
fn test_config_overwrite() {
    let (_tmp, data_dir) = setup();
    let config1 = Config {
        theme: "dark".to_string(),
        shortcut: "Ctrl+Alt+X".to_string(),
        autostart: true,
        check_updates: true,
        window_width: 500,
        window_height: 650,
    };
    storage::save_config_at(&data_dir, &config1).unwrap();

    let config2 = Config {
        theme: "light".to_string(),
        shortcut: "Ctrl+Shift+N".to_string(),
        autostart: false,
        check_updates: false,
        window_width: 1024,
        window_height: 768,
    };
    storage::save_config_at(&data_dir, &config2).unwrap();

    let loaded = storage::load_config_at(&data_dir);
    assert_eq!(loaded.theme, "light");
    assert_eq!(loaded.window_width, 1024);
}

#[test]
fn test_config_serialization_roundtrip() {
    let config = Config {
        theme: "custom".to_string(),
        shortcut: "Alt+F1".to_string(),
        autostart: false,
        check_updates: false,
        window_width: 1920,
        window_height: 1080,
    };
    let json = serde_json::to_string_pretty(&config).unwrap();
    let parsed: Config = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.theme, "custom");
    assert_eq!(parsed.shortcut, "Alt+F1");
    assert_eq!(parsed.window_width, 1920);
}
