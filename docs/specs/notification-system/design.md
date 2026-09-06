# Notification System Design

## Overview

The notification system is implemented entirely within `ff-desktop`. It
consists of a `NotificationSender` (a `tokio::sync::mpsc::Sender` wrapper),
a `NotificationQueue` held in `WorkbenchShell`, and two rendering paths:
toast overlay and Event Log panel.

No new crate is required. The system is self-contained in `ff-desktop`.

---

## Design Decisions

### 1. Channel-Based Decoupling

Background tasks (file copy, global replace, build) run on Tokio threads
and cannot directly mutate egui state. The `NotificationSender` wraps a
bounded `mpsc::Sender<Notification>`. The shell drains the channel each
frame in `update()` and appends to the in-memory queue.

Channel capacity: 64 notifications. If full, the sender logs a warning
and drops the notification (non-blocking guarantee, Req 3.3).

### 2. NotificationQueue

```rust
pub struct NotificationQueue {
    entries: VecDeque<NotificationEntry>,  // all entries, newest first
    unread_warnings_errors: usize,
}

pub struct NotificationEntry {
    id: u64,
    timestamp: chrono::DateTime<chrono::Local>,
    level: NotificationLevel,
    title: String,
    detail: Option<String>,
    read: bool,
    dismissed: bool,
}
```

The queue is capped at 1000 entries (oldest dropped when full, Req 2.7).

### 3. Toast Rendering

Toasts are rendered in `shell/render.rs` after all panels, using
`egui::Area` with `Order::Foreground` anchored to the bottom-right of
the central panel. Up to 4 toasts are shown; a "N more..." button opens
the Event Log.

Auto-dismiss uses `egui::Context::request_repaint_after(Duration)` to
schedule the dismiss without a busy loop.

### 4. EventLogPanel

A new `TabKind::EventLog` tab. Routing: `LOG` command or bell icon click.
The panel holds a reference to the shared `NotificationQueue` (via
`Arc<Mutex<NotificationQueue>>`).

### 5. Status Bar Bell Icon

A new right-aligned element in the status bar render. Uses a Unicode bell
character with a coloured badge overlay when unread count > 0.

---

## Module Layout

```
ff-desktop/src/
  notification/
    mod.rs          -- NotificationLevel, Notification, NotificationSender
    queue.rs        -- NotificationQueue, NotificationEntry
    toast.rs        -- render_toasts() -- overlay rendering
  event_log_panel.rs  -- EventLogPanel state and render()
  shell/state.rs      -- NotificationQueue field, NotificationSender field
  shell/update.rs     -- drain channel each frame
  shell/render.rs     -- call render_toasts(); bell icon in status bar
  shell/commands.rs   -- route "LOG" command
  tab_state.rs        -- add EventLog variant to TabKind
  session_manager.rs  -- persist EventLog tab
```
