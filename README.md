# Recall

A fast, keyboard-first note-taking app that lives in your system tray. Built with [Tauri 2](https://v2.tauri.app/), Rust, and vanilla JS.

## Features

- **WYSIWYG editor** with markdown toolbar (bold, italic, headings, lists, quotes, links)
- **System tray integration** — launch with a global shortcut, dismiss with Escape
- **Quick capture** (`Ctrl+Shift+N`) — jot down notes that auto-expire after 24h
- **Reminders** with recurring schedules (daily/weekly/monthly) and native notifications
- **Templates** — built-in (Meeting, Task, Diary, Study) + custom templates
- **Version history** — up to 20 snapshots per note, restore any version
- **Categories & tags** with drag-and-drop reordering
- **Image paste** — paste screenshots directly from clipboard
- **Import/export** — full JSON backup with validation and conflict detection
- **Dark & light themes** with customizable font size
- **Configurable shortcuts** — two global shortcuts (open app + quick new note)
- **Always-on-top** pin mode

## Stack

| Layer | Tech |
|-------|------|
| Runtime | Tauri 2.x |
| Backend | Rust (serde, chrono, uuid, base64) |
| Frontend | Vanilla JS (ES modules, no bundler) |
| Storage | JSON files (one per note/reminder) |
| Icons | Inline SVG (Lucide) |
| Tests | 190+ Rust unit/integration tests, 28 Vitest tests |

Zero runtime JS dependencies.

## Getting Started

```bash
# Prerequisites: Rust, Node.js, Tauri CLI
cargo install tauri-cli

# Run in dev mode
cargo tauri dev

# Build installer
cargo tauri build
```

The app starts minimized to the system tray. Click the tray icon or press `Ctrl+Alt+X` to open.

## Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Alt+X` | Toggle app window |
| `Ctrl+N` | New note |
| `Ctrl+Shift+N` | Quick capture (24h temporary) |
| `Ctrl+P` | Focus search |
| `Ctrl+S` | Save note (in editor) |
| `Ctrl+,` | Open settings |
| `Escape` | Hide window |

## Project Structure

```
src-tauri/src/
  main.rs          # App bootstrap, plugin setup
  commands.rs      # 30+ IPC commands (the entire API surface)
  models.rs        # Data structs, DTOs, filters
  storage.rs       # JSON file persistence (atomic writes)
  cache.rs         # In-memory cache with lazy invalidation
  scheduler.rs     # Background reminder polling
  shortcuts.rs     # Global shortcut parsing/registration
  tray.rs          # System tray icon and context menu
  window.rs        # Window show/toggle/focus logic

src/
  index.html       # Single-page shell
  scripts/
    app.js         # Entry point, tab switching, shortcuts
    notes.js       # Notes list, search, filters, drag-drop
    editor.js      # WYSIWYG editor, toolbar, version history
    reminders.js   # Reminders list, calendar view
    settings.js    # Theme, shortcuts, font size, backup
    utils.js       # Markdown, toasts, sanitization, formatters
    icons.js       # Lucide SVG icon library
    api.js         # Tauri IPC bridge
  styles/
    base.css       # Reset, layout, scrollbar
    themes.css     # Dark/light CSS variables
    components.css # Cards, buttons, editor, calendar, modals
```

## License

MIT
