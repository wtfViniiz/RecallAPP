use crate::storage;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tauri::{AppHandle, Manager};

pub fn start_scheduler(app: AppHandle) {
    let app = Arc::new(app);

    // Check on startup for missed reminders
    {
        let app_clone = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(2));
            check_and_fire(&app_clone);
        });
    }

    // Periodic check every 30 seconds
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(30));
        check_and_fire(&app);
    });
}

fn check_and_fire(app: &AppHandle) {
    let reminders = storage::list_reminders(app, Some("pending".to_string()));
    let now = Utc::now();

    for mut reminder in reminders {
        let trigger: DateTime<Utc> = match reminder.trigger_at.parse() {
            Ok(t) => t,
            Err(_) => continue,
        };

        if trigger <= now {
            // Fire notification
            use tauri_plugin_notification::NotificationExt;
            let _ = app
                .notification()
                .builder()
                .title(&reminder.title)
                .body(reminder.description.as_deref().unwrap_or(""))
                .show();

            // Bring window to attention
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.request_user_attention(Some(tauri::UserAttentionType::Critical));
            }

            // Handle recurrence
            if let Some(ref repeat) = reminder.repeat {
                let next = match repeat.as_str() {
                    "daily" => trigger + chrono::Duration::days(1),
                    "weekly" => trigger + chrono::Duration::weeks(1),
                    "monthly" => trigger + chrono::Duration::days(30),
                    _ => {
                        reminder.status = "fired".to_string();
                        let _ = storage::save_reminder(app, &reminder);
                        continue;
                    }
                };
                reminder.trigger_at = next.to_rfc3339();
                let _ = storage::save_reminder(app, &reminder);
            } else {
                reminder.status = "fired".to_string();
                let _ = storage::save_reminder(app, &reminder);
            }
        }
    }
}
