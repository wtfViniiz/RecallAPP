use recall::models::*;
use recall::storage;
use std::path::PathBuf;
use tempfile::TempDir;

fn setup() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let data_dir = dir.path().join("data");
    storage::ensure_dirs(&data_dir);
    (dir, data_dir)
}

fn make_reminder(id: &str, title: &str, status: &str) -> Reminder {
    Reminder {
        id: id.to_string(),
        title: title.to_string(),
        description: None,
        note_id: None,
        trigger_at: "2026-05-29T10:00:00Z".to_string(),
        repeat: None,
        relative_minutes: None,
        status: status.to_string(),
        schema_version: 1,
        created_at: "2026-05-28T10:00:00Z".to_string(),
    }
}

#[test]
fn test_reminder_full_lifecycle() {
    let (_tmp, data_dir) = setup();
    let reminder = make_reminder("r1", "Test Reminder", "pending");
    storage::save_reminder_at(&data_dir, &reminder).unwrap();

    // Read
    let fetched = storage::get_reminder_at(&data_dir, "r1").unwrap();
    assert_eq!(fetched.status, "pending");
    assert_eq!(fetched.title, "Test Reminder");

    // Update
    let mut updated = fetched.clone();
    updated.title = "Updated Reminder".to_string();
    storage::save_reminder_at(&data_dir, &updated).unwrap();
    let fetched = storage::get_reminder_at(&data_dir, "r1").unwrap();
    assert_eq!(fetched.title, "Updated Reminder");

    // Dismiss
    let mut dismissed = fetched.clone();
    dismissed.status = "dismissed".to_string();
    storage::save_reminder_at(&data_dir, &dismissed).unwrap();

    let all = storage::list_reminders_at(&data_dir, None);
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].status, "dismissed");

    // Filter by status
    let pending = storage::list_reminders_at(&data_dir, Some("pending".to_string()));
    assert_eq!(pending.len(), 0);

    let dismissed_list = storage::list_reminders_at(&data_dir, Some("dismissed".to_string()));
    assert_eq!(dismissed_list.len(), 1);

    // Delete
    storage::delete_reminder_at(&data_dir, "r1").unwrap();
    let all = storage::list_reminders_at(&data_dir, None);
    assert_eq!(all.len(), 0);
}

#[test]
fn test_reminder_status_filter() {
    let (_tmp, data_dir) = setup();
    storage::save_reminder_at(&data_dir, &make_reminder("r1", "Pending", "pending")).unwrap();
    storage::save_reminder_at(&data_dir, &make_reminder("r2", "Fired", "fired")).unwrap();
    storage::save_reminder_at(&data_dir, &make_reminder("r3", "Dismissed", "dismissed")).unwrap();

    let pending = storage::list_reminders_at(&data_dir, Some("pending".to_string()));
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].title, "Pending");

    let fired = storage::list_reminders_at(&data_dir, Some("fired".to_string()));
    assert_eq!(fired.len(), 1);
    assert_eq!(fired[0].title, "Fired");

    let dismissed = storage::list_reminders_at(&data_dir, Some("dismissed".to_string()));
    assert_eq!(dismissed.len(), 1);
    assert_eq!(dismissed[0].title, "Dismissed");
}

#[test]
fn test_reminder_sort_by_trigger_at() {
    let (_tmp, data_dir) = setup();
    let mut r1 = make_reminder("r1", "Later", "pending");
    r1.trigger_at = "2026-05-30T10:00:00Z".to_string();
    let mut r2 = make_reminder("r2", "Earlier", "pending");
    r2.trigger_at = "2026-05-29T10:00:00Z".to_string();
    storage::save_reminder_at(&data_dir, &r1).unwrap();
    storage::save_reminder_at(&data_dir, &r2).unwrap();

    let reminders = storage::list_reminders_at(&data_dir, None);
    assert_eq!(reminders.len(), 2);
    assert_eq!(reminders[0].title, "Earlier");
    assert_eq!(reminders[1].title, "Later");
}

#[test]
fn test_reminder_with_description() {
    let (_tmp, data_dir) = setup();
    let mut reminder = make_reminder("r1", "With Desc", "pending");
    reminder.description = Some("This is a description".to_string());
    storage::save_reminder_at(&data_dir, &reminder).unwrap();

    let fetched = storage::get_reminder_at(&data_dir, "r1").unwrap();
    assert_eq!(fetched.description, Some("This is a description".to_string()));
}

#[test]
fn test_reminder_with_repeat() {
    let (_tmp, data_dir) = setup();
    let mut reminder = make_reminder("r1", "Daily", "pending");
    reminder.repeat = Some("daily".to_string());
    storage::save_reminder_at(&data_dir, &reminder).unwrap();

    let fetched = storage::get_reminder_at(&data_dir, "r1").unwrap();
    assert_eq!(fetched.repeat, Some("daily".to_string()));
}

#[test]
fn test_delete_nonexistent_reminder() {
    let (_tmp, data_dir) = setup();
    let result = storage::delete_reminder_at(&data_dir, "nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_empty_reminders() {
    let (_tmp, data_dir) = setup();
    let reminders = storage::list_reminders_at(&data_dir, None);
    assert_eq!(reminders.len(), 0);

    let pending = storage::list_reminders_at(&data_dir, Some("pending".to_string()));
    assert_eq!(pending.len(), 0);
}
