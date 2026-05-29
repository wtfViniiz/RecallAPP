<p align="center">
  <img src="src-tauri/icons/icon.png" alt="Recall" width="128">
</p>

<h1 align="center">Recall</h1>

<p align="center">
  <strong>Notes, reminders, and quick screenshots<br>always at hand.</strong>
</p>

<p align="center">
  <a href="#features">Features</a> &bull;
  <a href="#stack">Stack</a> &bull;
  <a href="#getting-started">Getting Started</a> &bull;
  <a href="#shortcuts">Shortcuts</a> &bull;
  <a href="#license">License</a>
</p>

---

## Why Recall exists

You're in the middle of something. A thought crosses your mind — a task to do, a link to save, a reminder for later. You open a notes app, wait for it to load, navigate through folders, create a new note, pick a title, pick a category... and by then you've lost your train of thought.

Recall was built to solve this. It lives in your system tray and opens instantly with a keyboard shortcut. You type, you save, you're back to what you were doing. No friction, no context switching, no bloat.

It's not trying to be your second brain. It's a scratchpad that's always there when you need it and invisible when you don't.

## What it does

Recall started as a simple note-taking tool and grew into something more capable while staying true to its core: **speed and simplicity**.

The editor supports rich text formatting without getting in your way. Notes can be organized with categories and tags, but you don't have to — just write. Quick capture lets you jot something down in 2 seconds and it auto-expires after 24 hours, so your scratchpad doesn't become a graveyard.

Reminders work the way you'd expect: set a time, pick a recurrence, and get a native notification when it fires. Templates save you from writing the same structure over and over. Version history means you never have to worry about losing work.

Everything is local. Your notes live as JSON files on your machine. No accounts, no sync, no cloud — just your data.

## Goals

- **Zero friction** — from thought to captured in under 2 seconds
- **Stay out of the way** — system tray, global shortcuts, dismiss with Escape
- **Respect the user's time** — no loading screens, no account creation, no onboarding wizards
- **Keep it local** — no telemetry, no analytics, no phone-home. Your data stays on your machine
- **Build it right** — atomic writes, input validation, XSS prevention, comprehensive tests

## Roadmap

- Multi-window floating notes for side-by-side editing
- Encryption at rest
- Markdown export
- Plugin system for custom integrations

## Features

| Feature | Description |
|---------|-------------|
| **WYSIWYG editor** | Bold, italic, headings, lists, quotes, links, images |
| **System tray** | Launch with a global shortcut, dismiss with Escape, taskbar icon |
| **Quick capture** | `Ctrl+Shift+N` — temporary note that expires after 24h |
| **Reminders** | Daily/weekly/monthly recurrence, native notifications |
| **Templates** | Meeting, Task, Diary, Study built-in + custom templates |
| **Version history** | Up to 20 snapshots per note, restore any version |
| **Categories & tags** | Filter by category/tag, drag-and-drop reordering |
| **Image paste** | Paste screenshots directly from clipboard |
| **Import/export** | Full JSON backup with validation and conflict detection |
| **Themes** | Dark and light with customizable font size |
| **Configurable shortcuts** | Two global shortcuts (open app + quick new note) |
| **Always-on-top** | Pin mode to keep the window visible |

## Stack

| Layer | Technology |
|-------|------------|
| Runtime | [Tauri 2.x](https://v2.tauri.app/) |
| Backend | Rust (serde, chrono, uuid, base64) |
| Frontend | Vanilla JS (ES modules, zero bundler) |
| Storage | JSON files (one per note/reminder) |
| Icons | Inline SVG ([Lucide](https://lucide.dev/)) |
| Tests | 190+ Rust (unit + integration) + 28 Vitest |

**Zero runtime JS dependencies.**

## Requirements

| Component | Version |
|-----------|---------|
| Rust | 1.70+ |
| Node.js | 18+ (tests only) |
| OS | Windows 10/11 |

## Getting Started

```bash
# Install Tauri CLI
cargo install tauri-cli

# Development mode
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
| `Ctrl+Shift+N` | Quick capture (24h) |
| `Ctrl+P` | Focus search |
| `Ctrl+S` | Save note |
| `Ctrl+,` | Open settings |
| `Escape` | Hide window |

## Project Structure

```
src-tauri/src/
  main.rs          # Bootstrap, plugins, IPC
  commands.rs      # 30+ IPC commands (the entire API surface)
  models.rs        # Structs, DTOs, filters
  storage.rs       # JSON persistence (atomic writes)
  cache.rs         # In-memory cache with lazy invalidation
  scheduler.rs     # Background reminder polling
  shortcuts.rs     # Global shortcut parsing/registration
  tray.rs          # System tray icon and context menu
  window.rs        # Show/toggle/focus logic

src/
  index.html       # Single-page shell
  scripts/
    app.js         # Entry point, tabs, shortcuts
    notes.js       # List, search, filters, drag-drop
    editor.js      # WYSIWYG, toolbar, version history
    reminders.js   # List, calendar view
    settings.js    # Theme, shortcuts, font size, backup
    utils.js       # Markdown, toasts, sanitization
    icons.js       # Lucide SVG icon library
    api.js         # Tauri IPC bridge
  styles/
    base.css       # Reset, layout, scrollbar
    themes.css     # Dark/light CSS variables
    components.css # Cards, buttons, editor, calendar, modals
```

## Build

```bash
cargo tauri build
```

Generates the NSIS installer at `src-tauri/target/release/bundle/nsis/`.

## License

MIT
