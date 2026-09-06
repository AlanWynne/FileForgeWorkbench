# Notification System Requirements

## Introduction

This sub-project defines a non-modal notification system for FileForge
Workbench. It replaces ad-hoc status bar messages for multi-step
operations with structured toast notifications and a persistent event log.

The status bar remains for single-line transient messages (cursor position,
encoding, mode). The notification system handles events that need more
prominence, persistence, or structured detail.

## Glossary

| Term | Definition |
|------|-----------|
| Toast notification | A brief, non-modal overlay that appears and auto-dismisses |
| Event log | A persistent, scrollable panel showing all past notifications |
| Notification level | One of: Info, Success, Warning, Error |
| Auto-dismiss | A toast that disappears after a configurable timeout without user action |
| Sticky notification | A toast that persists until the user explicitly dismisses it |

---

## Requirement 1: Toast Notifications

**User Story:** As a workbench user, I want brief non-modal notifications
for completed operations, so that I am informed of outcomes without my
workflow being interrupted by a modal dialog.

**Source:** Gap analysis section 8.2 -- notification-system (MISSING).
Executive assessment -- replaces ad-hoc status bar messages for multi-step
operations.

### Acceptance Criteria

1. WHEN any subsystem emits a notification event, THE workbench SHALL
   display a toast overlay in the bottom-right corner of the main window.
2. THE toast SHALL display: a level icon (Info/Success/Warning/Error),
   a title (one line), and an optional detail message (up to three lines).
3. WHEN a toast is at Info or Success level, IT SHALL auto-dismiss after
   a configurable timeout (default 4 seconds).
4. WHEN a toast is at Warning or Error level, IT SHALL be sticky -- it
   SHALL NOT auto-dismiss and SHALL require explicit user dismissal via
   a close button or pressing Escape.
5. WHEN multiple notifications arrive within the auto-dismiss window,
   THE workbench SHALL stack them vertically (newest on top), up to a
   maximum of 4 visible toasts.
6. WHEN more than 4 toasts are pending, THE workbench SHALL show a
   "N more..." indicator below the stack; clicking it SHALL open the
   Event Log panel.
7. THE toast auto-dismiss timeout SHALL be configurable via
   `notifications.auto_dismiss_seconds` (range 1-30, default 4).

---

## Requirement 2: Event Log Panel

**User Story:** As a workbench user, I want a persistent log of all
notifications, so that I can review what happened during a long-running
operation even after the toasts have dismissed.

**Source:** Gap analysis section 8.2 -- notification-system (MISSING).

### Acceptance Criteria

1. WHEN the user types `LOG` in the Command Field or clicks the
   notification bell icon in the status bar, THE workbench SHALL open
   an `EventLogPanel` tab.
2. THE Event Log panel SHALL display all notifications emitted since
   the workbench started, in reverse-chronological order (newest first).
3. FOR EACH log entry, THE panel SHALL display: timestamp (HH:MM:SS),
   level icon, title, and detail message.
4. THE panel SHALL include a filter by level (All / Info / Success /
   Warning / Error) and a text search field.
5. WHEN the user selects a log entry, THE panel SHALL display the full
   detail text in an expandable area below the list.
6. THE panel SHALL include a `Clear Log` button that removes all entries
   from the in-memory log (does not affect the persistent log file).
7. THE event log SHALL be written to a rolling log file at
   `{session_dir}/notifications.log` with a maximum of 1000 entries.

---

## Requirement 3: Notification API

**User Story:** As a workbench developer, I want a simple API for
emitting notifications from any subsystem, so that all parts of the
workbench can surface events to the user consistently.

**Source:** Architectural requirement -- replaces scattered `status_message`
calls with a unified notification channel.

### Acceptance Criteria

1. THE notification system SHALL expose a `NotificationSender` handle
   that any subsystem can obtain from the `WorkbenchShell` and use to
   emit notifications without knowing about the UI.
2. THE `NotificationSender` SHALL be `Clone + Send` so it can be passed
   to background Tokio tasks.
3. WHEN a notification is emitted, THE sender SHALL be non-blocking --
   it SHALL use a bounded channel and drop the notification (with a
   logged warning) if the channel is full.
4. THE notification API SHALL support four levels: `info`, `success`,
   `warning`, `error`, each accepting a title string and an optional
   detail string.
5. WHEN a background operation (file copy, global replace, build) emits
   a completion notification, THE notification SHALL include the
   operation name and outcome summary in the title.

---

## Requirement 4: Status Bar Integration

**User Story:** As a workbench user, I want a visual indicator in the
status bar showing unread notifications, so that I know when something
needs my attention even if I missed the toast.

**Source:** Consistent with status bar requirements in `menu-and-statusbar`.

### Acceptance Criteria

1. THE status bar SHALL display a notification bell icon on the right side.
2. WHEN there are unread Warning or Error notifications, THE bell icon
   SHALL show a badge with the count of unread items.
3. WHEN the user clicks the bell icon, THE workbench SHALL open the
   Event Log panel and mark all notifications as read.
4. WHEN all notifications have been read, THE badge SHALL be hidden.
