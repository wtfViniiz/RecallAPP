use crate::cache::NoteCache;
use chrono::{DateTime, Datelike, Timelike, Utc};
use std::panic::catch_unwind;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Manager};

static SHUTDOWN: AtomicBool = AtomicBool::new(false);

pub fn request_shutdown() {
    SHUTDOWN.store(true, Ordering::Release);
}

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

/// Advance a datetime by one recurrence period.
fn advance_recurrence(dt: &DateTime<Utc>, repeat: &str) -> Option<DateTime<Utc>> {
    match repeat {
        "daily" => {
            let naive = dt.naive_utc() + chrono::Duration::days(1);
            Some(chrono::DateTime::from_naive_utc_and_offset(naive, *dt.offset()))
        }
        "weekly" => {
            let naive = dt.naive_utc() + chrono::Duration::weeks(1);
            Some(chrono::DateTime::from_naive_utc_and_offset(naive, *dt.offset()))
        }
        "monthly" => {
            let (new_year, new_month) = if dt.month() == 12 {
                (dt.year() + 1, 1)
            } else {
                (dt.year(), dt.month() + 1)
            };
            let day = dt.day().min(days_in_month(new_year, new_month));
            chrono::NaiveDate::from_ymd_opt(new_year, new_month, day)
                .and_then(|d| d.and_hms_opt(dt.hour(), dt.minute(), dt.second()))
                .map(|ndt| chrono::DateTime::from_naive_utc_and_offset(ndt, *dt.offset()))
        }
        _ => None,
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
        // Sleep in 1-second intervals so shutdown is responsive (< 1s latency)
        for _ in 0..30 {
            if SHUTDOWN.load(Ordering::Acquire) {
                return;
            }
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
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
            // Snapshot trigger_at for TOCTOU check — if a concurrent dismiss/snooze
            // modifies the reminder, trigger_at will differ and we skip the save.
            let original_trigger_at = reminder.trigger_at.clone();

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
                let next = match advance_recurrence(&trigger, repeat) {
                    Some(dt) => dt,
                    None => {
                        reminder.status = "fired".to_string();
                        if let Some(current) = cache.get_reminder(app, &reminder.id) {
                            if current.trigger_at == original_trigger_at {
                                let _ = cache.save_reminder(app, &reminder);
                            }
                        }
                        continue;
                    }
                };

                // Catch up: advance past now if multiple periods were missed
                const MAX_CATCHUP_ITERATIONS: u32 = 1000;
                let mut final_next = next;
                let mut iterations = 0u32;
                while final_next <= now && iterations < MAX_CATCHUP_ITERATIONS {
                    iterations += 1;
                    if let Some(advanced) = advance_recurrence(&final_next, repeat) {
                        final_next = advanced;
                    } else {
                        break;
                    }
                }
                if iterations >= MAX_CATCHUP_ITERATIONS {
                    eprintln!("[Recall] Warning: reminder {} catch-up exceeded {} iterations", reminder.id, MAX_CATCHUP_ITERATIONS);
                }
                reminder.trigger_at = final_next.to_rfc3339();

                // TOCTOU guard: only save if no concurrent modification occurred
                if let Some(current) = cache.get_reminder(app, &reminder.id) {
                    if current.trigger_at == original_trigger_at {
                        let _ = cache.save_reminder(app, &reminder);
                    }
                }
            } else {
                reminder.status = "fired".to_string();
                // TOCTOU guard: only save if no concurrent modification occurred
                if let Some(current) = cache.get_reminder(app, &reminder.id) {
                    if current.trigger_at == original_trigger_at {
                        let _ = cache.save_reminder(app, &reminder);
                    }
                }
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

    #[test]
    fn test_advance_recurrence_daily() {
        let dt = chrono::DateTime::parse_from_rfc3339("2026-05-28T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let next = advance_recurrence(&dt, "daily").unwrap();
        assert_eq!(next.to_rfc3339(), "2026-05-29T10:00:00+00:00");
    }

    #[test]
    fn test_advance_recurrence_weekly() {
        let dt = chrono::DateTime::parse_from_rfc3339("2026-05-28T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let next = advance_recurrence(&dt, "weekly").unwrap();
        assert_eq!(next.to_rfc3339(), "2026-06-04T10:00:00+00:00");
    }

    #[test]
    fn test_advance_recurrence_monthly() {
        let dt = chrono::DateTime::parse_from_rfc3339("2026-01-31T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let next = advance_recurrence(&dt, "monthly").unwrap();
        // Jan 31 -> Feb 28 (2026 is not leap)
        assert_eq!(next.to_rfc3339(), "2026-02-28T10:00:00+00:00");
    }

    #[test]
    fn test_advance_recurrence_unknown_returns_none() {
        let dt = chrono::DateTime::parse_from_rfc3339("2026-05-28T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert!(advance_recurrence(&dt, "yearly").is_none());
    }

    #[test]
    fn test_advance_recurrence_monthly_preserves_time() {
        let dt = chrono::DateTime::parse_from_rfc3339("2026-03-15T14:30:45Z")
            .unwrap()
            .with_timezone(&Utc);
        let next = advance_recurrence(&dt, "monthly").unwrap();
        assert_eq!(next.to_rfc3339(), "2026-04-15T14:30:45+00:00");
    }
}
