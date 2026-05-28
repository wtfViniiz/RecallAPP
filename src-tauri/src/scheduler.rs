use crate::cache::NoteCache;
use chrono::{DateTime, Datelike, Timelike, Utc};
use std::panic::catch_unwind;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

fn days_in_month(year: i32, month: u32) -> u32 {
    let next_year = if month == 12 { year + 1 } else { year };
    let next_month = if month == 12 { 1 } else { month + 1 };
    let first_of_next = chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1);
    let first_of_current = chrono::NaiveDate::from_ymd_opt(year, month, 1);
    match (first_of_current, first_of_next) {
        (Some(cur), Some(next)) => (next - cur).num_days() as u32,
        _ => 30,
    }
}

pub fn start_scheduler(app: AppHandle, cache: NoteCache) {
    let app = Arc::new(app);
    let cache = Arc::new(cache);

    // Check on startup for missed reminders
    {
        let app_clone = app.clone();
        let cache_clone = cache.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(2));
            let _ = catch_unwind(std::panic::AssertUnwindSafe(|| {
                check_and_fire(&app_clone, &cache_clone);
            }));
        });
    }

    // Periodic check every 30 seconds
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(30));
        let _ = catch_unwind(std::panic::AssertUnwindSafe(|| {
            check_and_fire(&app, &cache);
        }));
    });
}

fn check_and_fire(app: &AppHandle, cache: &NoteCache) {
    let reminders = cache.list_reminders(app, Some("pending".to_string()));
    let now = Utc::now();

    for mut reminder in reminders {
        // Re-read current state to avoid overwriting user actions (dismiss/snooze)
        if let Some(current) = cache.get_reminder(app, &reminder.id) {
            if current.status != "pending" {
                continue;
            }
            reminder = current;
        }

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
                    "daily" => {
                        // Preserve local time across DST transitions
                        let naive = trigger.naive_utc() + chrono::Duration::days(1);
                        chrono::DateTime::from_naive_utc_and_offset(naive, *trigger.offset())
                    },
                    "weekly" => {
                        let naive = trigger.naive_utc() + chrono::Duration::weeks(1);
                        chrono::DateTime::from_naive_utc_and_offset(naive, *trigger.offset())
                    },
                    "monthly" => {
                        let (new_year, new_month) = if trigger.month() == 12 {
                            (trigger.year() + 1, 1)
                        } else {
                            (trigger.year(), trigger.month() + 1)
                        };
                        let day = trigger.day().min(days_in_month(new_year, new_month));
                        chrono::NaiveDate::from_ymd_opt(new_year, new_month, day)
                            .and_then(|d| d.and_hms_opt(trigger.hour(), trigger.minute(), trigger.second()))
                            .map(|dt| chrono::DateTime::from_naive_utc_and_offset(dt, *trigger.offset()))
                            .unwrap_or(trigger + chrono::Duration::days(30))
                    },
                    _ => {
                        reminder.status = "fired".to_string();
                        let _ = cache.save_reminder(app, &reminder);
                        continue;
                    }
                };
                // Catch up: advance past now if multiple periods were missed
                let mut final_next = next;
                while final_next <= now {
                    final_next = match repeat.as_str() {
                        "daily" => {
                            let naive = final_next.naive_utc() + chrono::Duration::days(1);
                            chrono::DateTime::from_naive_utc_and_offset(naive, *final_next.offset())
                        },
                        "weekly" => {
                            let naive = final_next.naive_utc() + chrono::Duration::weeks(1);
                            chrono::DateTime::from_naive_utc_and_offset(naive, *final_next.offset())
                        },
                        "monthly" => {
                            let (ny, nm) = if final_next.month() == 12 {
                                (final_next.year() + 1, 1)
                            } else {
                                (final_next.year(), final_next.month() + 1)
                            };
                            let d = final_next.day().min(days_in_month(ny, nm));
                            chrono::NaiveDate::from_ymd_opt(ny, nm, d)
                                .and_then(|dt| dt.and_hms_opt(final_next.hour(), final_next.minute(), final_next.second()))
                                .map(|dt| chrono::DateTime::from_naive_utc_and_offset(dt, *final_next.offset()))
                                .unwrap_or(final_next + chrono::Duration::days(30))
                        },
                        _ => break,
                    };
                }
                reminder.trigger_at = final_next.to_rfc3339();
                let _ = cache.save_reminder(app, &reminder);
            } else {
                reminder.status = "fired".to_string();
                let _ = cache.save_reminder(app, &reminder);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_days_in_month_regular() {
        assert_eq!(days_in_month(2026, 1), 31);  // January
        assert_eq!(days_in_month(2026, 2), 28);  // February (non-leap)
        assert_eq!(days_in_month(2026, 3), 31);  // March
        assert_eq!(days_in_month(2026, 4), 30);  // April
        assert_eq!(days_in_month(2026, 12), 31); // December
    }

    #[test]
    fn test_days_in_month_leap_year() {
        assert_eq!(days_in_month(2024, 2), 29);  // 2024 is leap
        assert_eq!(days_in_month(2000, 2), 29);  // 2000 is leap
        assert_eq!(days_in_month(1900, 2), 28);  // 1900 is NOT leap
        assert_eq!(days_in_month(2025, 2), 28);  // 2025 is not leap
    }

    #[test]
    fn test_days_in_month_all_months() {
        let expected = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        for (i, &days) in expected.iter().enumerate() {
            assert_eq!(days_in_month(2026, (i + 1) as u32), days, "Month {} should have {} days", i + 1, days);
        }
    }
}
