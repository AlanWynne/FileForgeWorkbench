# Tasks -- Notification System

## Overview

Channel-based notification system in `ff-desktop`. No new crate.
New `notification/` module, `EventLogPanel`, and status bar bell icon.

---

## Task 1. Notification types and channel (Req 3)

- [x] 1.1 Create `ff-desktop/src/notification/mod.rs` with
        `NotificationLevel` enum (Info, Success, Warning, Error),
        `Notification` struct (id, timestamp, level, title, detail),
        `NotificationSender` newtype wrapping `mpsc::SyncSender<Notification>`
        - Satisfies: Req 3.1, 3.4
- [x] 1.2 Implement `NotificationSender::info/success/warning/error(title, detail)`
        convenience methods; use non-blocking `try_send`; log warning on full channel
        - Satisfies: Req 3.3, 3.4
- [x] 1.3 Derive `Clone` on `NotificationSender`; confirm it is `Send`
        - Satisfies: Req 3.2
- [x] 1.4 Write unit tests: `sender_is_clone_and_send`,
        `full_channel_drops_notification_without_panic`,
        `all_four_levels_construct_correctly`
        - Satisfies: Req 3.2, 3.3, 3.4

## Task 2. NotificationQueue (Req 2.2, 2.7)

- [x] 2.1 Create `ff-desktop/src/notification/queue.rs` with
        `NotificationQueue { entries: VecDeque<NotificationEntry>, unread: usize }`
        and `NotificationEntry { id, timestamp, level, title, detail, read, dismissed }`
        - Satisfies: Req 2.2, 2.3
- [x] 2.2 Implement `push(notification)`: prepend to front, cap at 1000,
        increment `unread` for Warning/Error
        - Satisfies: Req 2.7, 4.2
- [x] 2.3 Implement `mark_all_read()`, `clear()`, `filter_by_level()`,
        `filter_by_text()`
        - Satisfies: Req 2.4, 2.6, 4.4
- [x] 2.4 Write unit tests: `queue_caps_at_1000_entries`,
        `push_warning_increments_unread`, `mark_all_read_clears_unread`,
        `filter_by_level_returns_matching`
        - Satisfies: Req 2.7, 4.2, 4.4

## Task 3. Shell wiring (Req 3.1, 3.5)

- [x] 3.1 Add `notification_tx: mpsc::SyncSender<Notification>` and
        `notification_queue: Arc<Mutex<NotificationQueue>>` to `WorkbenchShell`
        - Satisfies: Req 3.1
- [x] 3.2 In `WorkbenchShell::new()`: create bounded channel (capacity 64),
        store sender and queue
        - Satisfies: Req 3.3
- [x] 3.3 In `shell/update.rs` `update()`: drain channel each frame, push
        to queue
        - Satisfies: Req 1.1
- [x] 3.4 Expose `notification_sender()` method on `WorkbenchShell` returning
        a cloned `NotificationSender`
        - Satisfies: Req 3.1
- [x] 3.5 Write unit test: `notifications_drained_from_channel_each_frame`
        - Satisfies: Req 1.1

## Task 4. Toast overlay rendering (Req 1)

- [ ] 4.1 Create `ff-desktop/src/notification/toast.rs` with
        `render_toasts(ctx, queue)` function
        - Satisfies: Req 1.1
- [ ] 4.2 Render up to 4 toasts as `egui::Area` with `Order::Foreground`
        anchored bottom-right; each shows level icon, title, detail, close button
        - Satisfies: Req 1.2, 1.5
- [ ] 4.3 Implement auto-dismiss for Info/Success: use
        `ctx.request_repaint_after(Duration::from_secs(timeout))`;
        mark dismissed when timeout expires
        - Satisfies: Req 1.3
- [ ] 4.4 Sticky behaviour for Warning/Error: no auto-dismiss; close button
        only; Escape key dismisses topmost sticky toast
        - Satisfies: Req 1.4
- [ ] 4.5 Render "N more..." button when queue has more than 4 undismissed
        toasts; clicking opens EventLog tab
        - Satisfies: Req 1.6
- [ ] 4.6 Call `render_toasts()` in `shell/render.rs` after all panels
        - Satisfies: Req 1.1
- [ ] 4.7 Write unit tests: `info_toast_auto_dismisses_after_timeout`,
        `error_toast_is_sticky`, `more_than_4_shows_overflow_indicator`
        - Satisfies: Req 1.3, 1.4, 1.6

## Task 5. Event Log panel (Req 2)

- [x] 5.1 Add `EventLog` variant to `TabKind` enum in `tab_state.rs`
        - Satisfies: Req 2.1
- [x] 5.2 Add routing in `shell/commands.rs`: `"LOG"` -> open EventLog tab
        - Satisfies: Req 2.1
- [x] 5.3 Create `ff-desktop/src/event_log_panel.rs` with
        `EventLogPanelState { level_filter, text_filter, selected_id }`
        and `render(ui, queue)`
        - Satisfies: Req 2.2
- [x] 5.4 Render reverse-chronological list: timestamp, level icon, title;
        selected entry shows full detail below list
        - Satisfies: Req 2.3, 2.5
- [x] 5.5 Implement level filter dropdown and text search field
        - Satisfies: Req 2.4
- [x] 5.6 Implement `Clear Log` button calling `queue.clear()`
        - Satisfies: Req 2.6
- [x] 5.7 Write unit tests: `event_log_shows_entries_newest_first`,
        `level_filter_hides_non_matching`, `clear_log_empties_queue`
        - Satisfies: Req 2.2, 2.4, 2.6

## Task 6. Status bar bell icon (Req 4)

- [ ] 6.1 Add bell icon to right side of status bar in `shell/render.rs`;
        show unread badge when `queue.unread > 0`
        - Satisfies: Req 4.1, 4.2
- [ ] 6.2 On bell click: open EventLog tab and call `queue.mark_all_read()`
        - Satisfies: Req 4.3
- [ ] 6.3 Hide badge when `queue.unread == 0`
        - Satisfies: Req 4.4
- [ ] 6.4 Write unit tests: `bell_badge_shows_unread_count`,
        `bell_click_marks_all_read`
        - Satisfies: Req 4.2, 4.3

## Task 7. Session persistence (Req 2.1)

- [x] 7.1 Add `EventLog` to session tab kind serialisation in
        `session_manager.rs` and `PersistedTabKind` in `ff-session`
        - Satisfies: Req 2.1
- [x] 7.2 Write unit test: `event_log_tab_round_trips_through_session`
        - Satisfies: Req 2.1

## Task 8. TCR and documentation update

- [x] 8.1 Update `docs/quality/TCR.md` -- add notification-system section
        with rows for Req 1.1-1.7, 2.1-2.7, 3.1-3.5, 4.1-4.4
        - Satisfies: project gate requirement
- [x] 8.2 Update `docs/specs/project-master/tasks.md` -- mark CO.6
        complete when all tasks above are [x]
        - Satisfies: project gate requirement
