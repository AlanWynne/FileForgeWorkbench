//! # Shell Panel Rendering -- key label bar, status bar, open-file dialog
//!
//! The ISPF function-key label bar, the footer status bar, and the open-file
//! dialog launcher. Split out of `render.rs` / `render_command_line.rs`
//! (TASK 2.2, pure code movement, no behaviour change).

use eframe::egui;

use ff_keys::{FunctionKey, KeyLabelBarModel};

use super::render::logging_degradation_reason;
use super::WorkbenchShell;

impl WorkbenchShell {
    // ── Key label bar ─────────────────────────────────────────────────────

    /// Render the ISPF-style function key label bar in the footer.
    ///
    /// CR-CH-046: renders a SINGLE line showing ONE modifier layer's F1-F12
    /// labels, prefixed with a Scope_Segment naming the current scope
    /// (`Base` / `Shift` / `Ctrl` / `Alt`), e.g. `Base | F1 Help | F2 Detach | ...`.
    /// The scope is chosen by the PFSHOW command (`key_bar_scope`).
    ///
    /// Validates: Requirement 12.10, 12.11, 12.12; Requirement 13.1, 13.3-13.5
    pub(super) fn render_key_label_bar(&mut self, ctx: &egui::Context) {
        if !self.key_bar_visible {
            return;
        }
        // CR-CH-056 Req 23.3: chrome accent + body text come from the chrome layer.
        let key_color = self.palette.chrome_style.accent_color();
        let label_color = self.palette.chrome_style.foreground_color();
        let modifier = self.key_bar_scope.to_modifier();
        // Build the single-row model for the active scope from the active map.
        let row =
            KeyLabelBarModel::row_for_modifier(self.key_map_resolver.active_key_map(), modifier);
        let mut clicked_key: Option<FunctionKey> = None;
        egui::TopBottomPanel::bottom("key_label_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Leading Scope_Segment (non-interactive) naming the layer.
                ui.add(egui::Label::new(
                    egui::RichText::new(self.key_bar_scope.segment_label())
                        .color(label_color)
                        .monospace()
                        .small()
                        .strong(),
                ));
                for slot in row.slots() {
                    let key = slot.key;
                    let btn_text = if let Some(lbl) = &slot.label {
                        format!("| {} {}", key.display_name(), lbl)
                    } else {
                        format!("| {}", key.display_name())
                    };
                    let enabled = slot.label.is_some();
                    let tooltip = self
                        .key_map_resolver
                        .active_key_map()
                        .get(ff_keys::ModifiedKey { key, modifier })
                        .map(|b| b.command().to_string())
                        .unwrap_or_default();
                    // CR-CH-023 Req 16.9: the Key_Label_Bar slots are clickable
                    // but MUST NOT be keyboard Tab stops (they duplicate the
                    // physical function keys). Rendering a `Label` with a
                    // click-only Sense (no FOCUSABLE bit) keeps the mouse click
                    // while removing the widget from egui-native Tab traversal.
                    let text = egui::RichText::new(&btn_text)
                        .color(if enabled { label_color } else { key_color })
                        .monospace()
                        .small();
                    let resp = if enabled {
                        ui.add(egui::Label::new(text).sense(egui::Sense::CLICK))
                    } else {
                        // Blank slots are pure display (never a Tab stop, not
                        // clickable), preserving the fixed grid (Req 13.2).
                        ui.add(egui::Label::new(text))
                    };
                    if enabled && !tooltip.is_empty() {
                        resp.clone().on_hover_text(&tooltip);
                    }
                    if resp.clicked() && enabled {
                        clicked_key = Some(key);
                    }
                }
            });
        });
        if let Some(key) = clicked_key {
            if let Some(cmd) = self
                .key_map_resolver
                .active_key_map()
                .get(ff_keys::ModifiedKey { key, modifier })
                .map(|b| b.command().to_string())
            {
                // A clicked slot is treated as if that modifier+function key was
                // pressed (function-keys Req 16.1): route through the shared
                // key-dispatch so the Command ===> field content is merged as the
                // argument (Req 9.8, B066), then Target_Resolution runs the
                // command's target (Req 4.4) or falls through (Req 10.2).
                self.dispatch_key_command(&cmd);
            }
        }
    }

    // ── Status bar ───────────────────────────────────────────────────────

    pub(super) fn render_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // CR-NR-115 Phase CF (Req 7.15, 7.20): the redundant RUNNING
                // lifecycle-phase segment is removed from the common frame -- the
                // information message area (the Message_Line panel above) now
                // carries transient status, and the phase was always "Running"
                // in practice. The lifecycle phase is still observable via
                // logging; it is simply no longer a Status_Bar segment.
                //
                // Status_Bar segments are non-interactive labels, NOT
                // SelectableLabel. SelectableLabel uses Sense::click() and is a
                // focusable egui widget; that made each segment an egui-native
                // Tab focus stop, so at launch Tab walked the segments before
                // reaching the menu/POM (B055). Requirement 16 enumerates the
                // exclusive shell tab stops and the Status_Bar is not among them,
                // so the segments must not be focusable.
                // Validates: Requirement 7.15, 7.20 (CR-NR-115); B055 (not a tab stop).

                // Validates: Requirement 7.16, 7.18 (CR-NR-115) -- `Last: HH:MM`
                // from last_action_at (NOT a ticking clock), replacing the former
                // `Started:` session-start segment.
                ui.label(self.format_last_action());
                ui.separator();

                // Validates: Requirement 7.19 (CR-NR-115); startup-and-session
                // Req 22.8 (CR-NR-081) -- the Active_Profile is shown in the
                // Status_Bar as `Prof: <name>` (explicitly "default" when no
                // --profile was given).
                ui.label(self.active_profile_label());
                ui.separator();

                // Validates: Requirement 7.16, 7.16b (CR-NR-115, revised
                // 2026-11-04) -- Zoom is a COMMON segment, ALWAYS shown directly
                // after Prof (`Zoom: 0`, `Zoom: +14`, `Zoom: -3`), from the ONE
                // formatter the snapshot band also uses. Context help is a HOVER
                // TOOLTIP only: a plain `ui.label` (never a SelectableLabel or a
                // click Sense) so it is NOT focusable and NOT a Tab stop (B055).
                ui.label(self.format_zoom_label())
                    .on_hover_text(ff_i18n::t("ff-desktop-shell-status-zoom-help"));
                ui.separator();

                // CR-NR-115 Phase CF (Req 7.16-7.21, revised 2026-11-04): the
                // per-Kind Workspace_Status_Segment composition point. The ONLY
                // common segments are Last, Prof and Zoom above; Ln/Col, encoding and
                // line count are now WORKSPACE-owned (they describe a document
                // cursor/content, which only a document-backed Kind has -- a
                // settings-type Workspace like Config/Alias has neither). The
                // editor Context keeps emitting its own Ln/Col + encoding + line
                // count here (not as a shared default) so the live bar is
                // unchanged for it; every other Kind defaults to empty until its
                // own per-Kind gate. A non-interactive label (never a Tab stop,
                // B055).
                for cell in self.workspace_status_cells() {
                    ui.label(cell);
                    ui.separator();
                }
                let tab = self.tabs.active_tab();
                // Requirement 6.5: modified indicator
                if tab.is_modified {
                    ui.colored_label(self.palette.chrome_style.accent_color(), "\u{25cf}");
                    ui.separator();
                }
                // Req 16.3: CAPS mode indicator
                if tab.edit_profile.caps.is_on() {
                    ui.colored_label(
                        self.palette.chrome_style.accent_color(),
                        ff_i18n::t("ff-desktop-shell-status-caps"),
                    );
                    ui.separator();
                }

                if let Some(err) = &self.open_error {
                    ui.colored_label(egui::Color32::RED, err);
                    ui.separator();
                    // Validates: Requirement 2.5 -- register error message for automation.
                    self.automation.register_str(
                        crate::automation::ids::STATUSBAR_MESSAGE,
                        crate::automation::ControlState::with_value(err.as_str()),
                    );
                } else {
                    self.automation.register_str(
                        crate::automation::ids::STATUSBAR_MESSAGE,
                        crate::automation::ControlState::with_value(""),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Non-interactive version label (B055: not a Tab focus stop).
                    ui.label(ff_i18n::t("ff-desktop-shell-status-banner"));

                    // B038 (CR-NR-086, logging-subsystem Req 8.7): surface logging
                    // DEGRADATION to the user. Shown only when the subsystem is in
                    // fallback (no-op) mode -- the log file could not be created --
                    // OR records have been dropped due to buffer overflow. Hidden
                    // when logging is healthy. A non-interactive label (B055: not a
                    // Tab focus stop), with a tooltip stating the reason.
                    let reason = logging_degradation_reason(
                        ff_logging::is_fallback(),
                        ff_logging::dropped_count(),
                    );
                    if let Some(reason) = reason {
                        ui.separator();
                        let resp = ui
                            .add(egui::Label::new(
                                egui::RichText::new(ff_i18n::t(
                                    "ff-desktop-shell-status-log-indicator",
                                ))
                                .strong()
                                .color(egui::Color32::from_rgb(0xD2, 0x0F, 0x39)),
                            ))
                            .on_hover_text(ff_i18n::t_args(
                                "ff-desktop-shell-status-log-degraded-hover",
                                &ff_i18n::MessageArgs::new().set("reason", reason.clone()),
                            ));
                        // Register for automation / headless assertion (B038 test).
                        self.automation.register_str(
                            crate::automation::ids::LOGGING_DEGRADED,
                            crate::automation::ControlState::with_value(&reason),
                        );
                        let _ = resp;
                    } else {
                        self.automation.register_str(
                            crate::automation::ids::LOGGING_DEGRADED,
                            crate::automation::ControlState::with_value(""),
                        );
                    }
                });
            });
        });
    }

    /// The per-Kind Workspace_Status_Segment CELLS for the active Workspace
    /// (CR-NR-115, workspace-framework Req 7.16-7.21/7.27, revised 2026-11-04).
    /// The only COMMON Status_Bar segments are `Last: HH:MM`, `Prof: <name>` and `Zoom: <n>`;
    /// EVERYTHING ELSE is workspace-owned and returned here, one string per cell
    /// (the live bar and the snapshot status band both render exactly these).
    ///
    /// Encoding, Ln/Col and line count describe the DATA a workspace shows (the
    /// shell UI itself is always Unicode), so they belong to a document-backed
    /// Kind: the editor Context returns its document encoding, Ln/Col and line
    /// count (same values as before the split). Every other Kind returns an
    /// empty Vec for now - its content is decided in its own per-Kind gate.
    ///
    /// Validates: Requirement 7.17, 7.21, 7.27
    pub(super) fn workspace_status_cells(&self) -> Vec<String> {
        use crate::tab_state::KindTag;
        let tab = self.tabs.active_tab();
        if !matches!(tab.kind.tag(), KindTag::FileEditor | KindTag::Untitled) {
            return Vec::new();
        }
        // Requirement 7.1: "Ln {line}, Col {col}" (1-based) as TEXT placeables
        // (byte-identical, no Fluent NUMBER() grouping).
        let line_col = ff_i18n::t_args(
            "ff-desktop-shell-status-line-col",
            &ff_i18n::MessageArgs::new()
                .set("line", tab.cursor.cursor_line().to_string())
                .set("col", tab.cursor.cursor_column().to_string()),
        );
        let line_count = ff_i18n::t_args(
            "ff-desktop-shell-status-line-count",
            &ff_i18n::MessageArgs::new().set("count", tab.line_count.to_string()),
        );
        // Requirement 7.3: real encoding from the document.
        vec![line_col, tab.encoding_label().to_string(), line_count]
    }

    // === Message line =======================================================

    /// Render the transient Message_Line as a bottom panel directly ABOVE the
    /// Status_Bar (CR-NR-115, workspace-framework Phase CF, Req 7.8/7.13/7.14).
    ///
    /// When the Message_Line store is empty the panel is ABSENT (early return, no
    /// space reserved). Otherwise it renders one non-interactive label row per
    /// message, expanding for multiple messages. The rows are display-only: like
    /// the Status_Bar segments they are plain `ui.label` calls and so are NOT
    /// egui-native Tab focus stops (B055). This panel is added to the frame AFTER
    /// the Status_Bar in `update.rs`; egui bottom panels stack upward in call
    /// order, so it sits directly above the Status_Bar.
    ///
    /// Validates: Requirement 7.8, 7.13, 7.14
    pub(super) fn render_message_line(&mut self, ctx: &egui::Context) {
        if self.message_line.is_empty() {
            return;
        }
        egui::TopBottomPanel::bottom("message_line").show(ctx, |ui| {
            for message in &self.message_line {
                // Non-interactive display row (not a Tab stop, mirroring the
                // Status_Bar label pattern).
                ui.label(message);
            }
        });
    }

    // ── File-open dialog ──────────────────────────────────────────────────

    /// Spawn a native file-open dialog on a blocking thread.
    ///
    /// When the user picks a file the path is written into `pending_open`;
    /// the next egui frame will pick it up and open the tab.
    ///
    /// CR-NR-080 Slice A: the data-driven menu bar no longer hardwires a
    /// "Files > Open..." item, so this has no current caller. It is retained
    /// (not deleted) because a menu-file option or command will invoke it once
    /// the file-operations menu content is authored (Slice B onward).
    #[allow(dead_code)]
    pub(super) fn open_file_dialog(&self) {
        let pending = self.pending_open.clone();
        self.runtime.spawn_blocking(move || {
            if let Some(handle) = rfd::FileDialog::new().pick_file() {
                let path = handle.to_string_lossy().into_owned();
                // The native file dialog opens a host-filesystem file -> host FS
                // Owning_Environment (None -> default, CR-CH-053 Task 19 Req 15.3);
                // no dataset identity / record format (RC.B.8 (c)).
                *pending.lock().expect("pending lock") =
                    Some(super::pending_open::PendingOpenReq {
                        path,
                        owning_env: None,
                    });
            }
        });
    }
}
