# Analysis Record: notification-system (W4.7)

- **Wave**: 4 (UI, panels, layout)
- **Backing code**: NO dedicated crate -- implemented in `ff-desktop`:
  `notification/mod.rs` (189, `NotificationQueue` + `Notification` + notify API) +
  `event_log_panel.rs` (163, the Event Log Context). Toast overlay + status-bar
  bell are UNBUILT (see below).
- **Spec files**: requirements.md (128 lines, 4 requirements -- uses `## Requirement`
  double-hash headings), tasks.md (35 sub-tasks, **24 done / 11 OPEN**), design.md
- **Analysed**: Wave 4 pass

---

## 1. Split candidacy

N/A / NOT a split candidate. Small: 4 reqs, 128 lines. One cohesive concern
(non-modal notifications: toasts + event log + API + status-bar integration).
No file over cap (mod.rs 189, event_log_panel 163). No split.

---

## 2. Completeness -- PA-INCOMPLETE-011 (toast UI + status-bar bell UNBUILT)

24/35 tasks done, **11 OPEN**. The BUILT vs UNBUILT split maps cleanly to reqs:

BUILT:
- Req 2 (Event Log Context): `event_log_panel.rs` (163) -- the persistent scrollable
  event log tab.
- Req 3 (Notification API): `notification/mod.rs` (189) -- `Notification`,
  `NotificationQueue`, notify API, unread tracking. Any subsystem can emit events.

UNBUILT (the 11 open tasks -- two UI surfaces):
- Req 1 (Toast Notifications), Task 4.1-4.7: `notification/toast.rs` does NOT exist.
  No toast overlay (egui::Area foreground, auto-dismiss Info/Success, sticky
  Warning/Error, "N more..." overflow, render_toasts() in shell/render.rs, tests).
- Req 4 (Status Bar Integration), Task 6.1-6.4: no status-bar BELL icon + unread
  BADGE (bell click opens EventLog + mark_all_read; hide badge when unread==0; tests).

So the notification MODEL (queue + API + event log) is complete, but the two
user-facing UI surfaces (toast overlay + status-bar bell) are UNBUILT. Honest `[ ]`
tracking (NOT a false-positive -- toast.rs genuinely absent). Recorded
PA-INCOMPLETE-011 (MEDIUM): implement the toast rendering (Task 4) + the status-bar
bell/badge (Task 6). Self-contained egui UI work over the existing NotificationQueue;
the bell integrates with the ff-menu status bar (W3.6). This is a user-visible gap
(operations currently have no toast feedback + no notification indicator).

---

## 3. Cross-unit consistency

### Status-bar integration -- ties to ff-menu (W3.6)

Req 4 (status-bar bell/badge) integrates with the ff-menu status bar (W3.6). The
spec is explicit that notification-system COMPLEMENTS the status bar: the status
bar keeps single-line transient messages (cursor/encoding/mode); notifications
handle events needing prominence/persistence/structure. Clean division of the two
message surfaces. The bell (Task 6.1, unbuilt) is the integration point -- confirm
it wires into the ff-menu status-bar segment model (W3.6 Req 8 extensibility).

### Notification API as an emit target -- consistent

Req 3: any subsystem emits notification events into the `NotificationQueue`. This is
the structured-event sink that complements ff-logging (logs) and the status bar
(transient). Notable: notification-system is NOT ff-logging -- it is user-facing
event surfacing, distinct from the dev/debug logging (CR-NR-058). A subsystem that
both LOGS (ff-logging) and NOTIFIES (NotificationQueue) is expected for operational
events. No conflict; complementary channels.

### Public types and ownership

- `Notification`, `NotificationQueue`, `NotificationLevel`, Event Log Context,
  (pending) toast overlay + status-bar bell -- sole-owned by the ff-desktop
  notification module. No duplication. 0 fs (in-memory queue).

### Cross-reference integrity

Cross-refs (menu-and-statusbar for the bell, shell render for toasts) resolve. The
unbuilt pieces are the integration surfaces (PA-INCOMPLETE-011).

---

## 4. Logging audit

Scan of `ff-desktop/src/notification`:

- `ff_logging` / `log_*!`: 0
- `println!` / `eprintln!`: 0
- `std::fs`: 0
- non-ASCII: 0 (clean)

No dead ff-logging dep here (the module doesn't declare one). notification-system
is the USER-FACING event channel; ff-logging is the DEV/debug channel -- distinct
and complementary. Zero-log is correct for the notification module itself. Recorded
PA-LOG-033 (LOW): optionally MIRROR Warning/Error notifications to ff-logging at
WARN/ERROR (a subsystem that notifies the user of an error should typically also log
it for support) -- but that is the EMITTING subsystem's responsibility, not the
notification module's. Note only; low priority.

---

## 5. Task revision proposals

- **PA-INCOMPLETE-011 (MEDIUM)**: implement the two unbuilt UI surfaces over the
  existing NotificationQueue -- Req 1 toast overlay (`notification/toast.rs`, Task
  4.1-4.7: egui Area, auto-dismiss Info/Success, sticky Warning/Error, overflow,
  render in shell/render.rs, tests) + Req 4 status-bar bell/badge (Task 6.1-6.4,
  wire into the ff-menu status bar W3.6). Self-contained egui UI. User-visible gap.
- **PA-TCR (none)**: 5 TCR rows for 4 reqs -- adequate for the built portion; the
  toast/bell rows come with PA-INCOMPLETE-011.
- **PA-LOG-033 (LOW, note)**: emitting subsystems SHOULD mirror Warning/Error
  notifications to ff-logging (support diagnosability); the notification module
  itself needs no logging. Emitter responsibility.

No PA-STD (no file over cap, 0 non-ASCII, no ASCII issue). No requirement CHANGE
proposed; Reqs 1/4 are correctly-gated pending UI work.

---

## Summary

notification-system is a small non-modal notification subsystem (toasts + Event Log
Context + notify API + status-bar bell) implemented in `ff-desktop`. The MODEL is
complete and clean: `NotificationQueue` + notify API (Req 3) and the Event Log
Context (Req 2) are built (0 fs, 0 non-ASCII, no dead dep, no cap issue). The
finding is PA-INCOMPLETE-011 (MEDIUM): the two user-facing UI SURFACES are UNBUILT --
the toast overlay (Req 1, `toast.rs` absent) and the status-bar bell/badge (Req 4) --
11 honestly-tracked `[ ]` tasks; a user-visible gap (no toast feedback, no
notification indicator). It complements rather than conflicts with ff-menu status
bar (transient messages, W3.6) and ff-logging (dev channel); Warning/Error
notifications should be mirrored to ff-logging by the EMITTING subsystem (PA-LOG-033,
note). No conflict, no split, no size/ASCII issue -- the one item is completing the
two UI surfaces.
