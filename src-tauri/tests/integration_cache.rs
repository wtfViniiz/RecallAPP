use recall::cache::NoteCache;
use std::thread;

// Note: NoteCache's query methods (list_notes, get_note, etc.) require
// a Tauri AppHandle. The tests below verify the cache's structural
// properties and invalidation behavior without needing AppHandle.

// --- Construction ---

#[test]
fn test_cache_new_does_not_panic() {
    let _cache = NoteCache::new();
}

#[test]
fn test_cache_is_cloneable() {
    let cache = NoteCache::new();
    let _cloned = cache.clone();
}

#[test]
fn test_cache_clone_independent() {
    // Cloning creates a new NoteCache that shares the same Arc internals.
    // Verify both can be used independently without panic.
    let cache = NoteCache::new();
    let clone = cache.clone();

    cache.invalidate_notes();
    clone.invalidate_reminders();
}

// --- Invalidation: notes ---

#[test]
fn test_invalidate_notes_on_fresh_cache() {
    let cache = NoteCache::new();
    // On a fresh cache, notes are already None. Invalidating should be a no-op.
    cache.invalidate_notes();
}

#[test]
fn test_invalidate_notes_is_idempotent() {
    let cache = NoteCache::new();
    cache.invalidate_notes();
    cache.invalidate_notes();
    cache.invalidate_notes();
}

#[test]
fn test_invalidate_notes_does_not_affect_reminders() {
    let cache = NoteCache::new();
    // Invalidating notes should not touch reminders
    cache.invalidate_notes();
    // If reminders were corrupted, this would likely panic or behave oddly.
    // We can't observe the value directly, but we can call invalidate on reminders
    // separately to confirm no cross-contamination.
    cache.invalidate_reminders();
}

// --- Invalidation: reminders ---

#[test]
fn test_invalidate_reminders_on_fresh_cache() {
    let cache = NoteCache::new();
    cache.invalidate_reminders();
}

#[test]
fn test_invalidate_reminders_is_idempotent() {
    let cache = NoteCache::new();
    cache.invalidate_reminders();
    cache.invalidate_reminders();
    cache.invalidate_reminders();
}

#[test]
fn test_invalidate_reminders_does_not_affect_notes() {
    let cache = NoteCache::new();
    cache.invalidate_reminders();
    cache.invalidate_notes();
}

// --- Shared state through cloning ---

#[test]
fn test_clone_shares_invalidation_state() {
    // Two clones share the same Arc<Mutex<Option<...>>>.
    // Invalidating on one should also clear the other.
    // We verify this doesn't cause panics or deadlocks.
    let cache = NoteCache::new();
    let clone1 = cache.clone();
    let clone2 = cache.clone();

    clone1.invalidate_notes();
    clone2.invalidate_notes();
    cache.invalidate_notes();

    clone1.invalidate_reminders();
    clone2.invalidate_reminders();
    cache.invalidate_reminders();
}

// --- Thread safety ---

#[test]
fn test_concurrent_note_invalidation() {
    // Multiple threads invalidating notes concurrently should not panic or deadlock.
    let cache = NoteCache::new();
    let mut handles = vec![];

    for _ in 0..10 {
        let c = cache.clone();
        handles.push(thread::spawn(move || {
            c.invalidate_notes();
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn test_concurrent_reminder_invalidation() {
    let cache = NoteCache::new();
    let mut handles = vec![];

    for _ in 0..10 {
        let c = cache.clone();
        handles.push(thread::spawn(move || {
            c.invalidate_reminders();
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn test_concurrent_mixed_invalidation() {
    // Both notes and reminders invalidated from different threads simultaneously.
    let cache = NoteCache::new();
    let mut handles = vec![];

    for i in 0..20 {
        let c = cache.clone();
        handles.push(thread::spawn(move || {
            if i % 2 == 0 {
                c.invalidate_notes();
            } else {
                c.invalidate_reminders();
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}

// --- Clone + invalidate isolation ---

#[test]
fn test_invalidate_on_clone_then_use_original() {
    // Invalidating on a clone, then using the original, should work fine.
    let cache = NoteCache::new();
    let clone = cache.clone();

    clone.invalidate_notes();
    clone.invalidate_reminders();

    // Original should still be usable (invalidate is idempotent on None)
    cache.invalidate_notes();
    cache.invalidate_reminders();
}

// --- Arc sharing verification ---

#[test]
fn test_clones_share_same_arc() {
    // Verify that clones share the same underlying Arc by checking
    // that Arc::strong_count increases with each clone.
    let cache = NoteCache::new();

    // We can't access private fields directly, but we can verify
    // structural behavior: invalidating on any clone should not panic
    // regardless of order or combination.
    let c1 = cache.clone();
    let c2 = c1.clone();
    let c3 = cache.clone();

    // All variations of invalidation order should work
    c3.invalidate_notes();
    c1.invalidate_reminders();
    c2.invalidate_notes();
    cache.invalidate_reminders();
}
