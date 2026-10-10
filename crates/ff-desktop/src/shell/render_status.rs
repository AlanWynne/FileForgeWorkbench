//! # Shell Panel Rendering -- key label bar, status bar, open-file dialog
//!
//! The ISPF function-key label bar, the footer status bar, and the open-file
//! dialog launcher. Split out of `render.rs` / `render_command_line.rs`
//! (TASK 2.2, pure code movement, no behaviour change).

use eframe::egui;

use ff_core::LifecyclePhase;
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
                let phase_label = match self.app.phase() {
                    LifecyclePhase::Running => "RUNNING",
                    LifecyclePhase::Initializing => "STARTING",
                    LifecyclePhase::ShuttingDown => "SHUTTING DOWN",
                    LifecyclePhase::Terminated => "TERMINATED",
                };
                // Status_Bar segments are non-interactive labels, NOT
                // SelectableLabel. SelectableLabel uses Sense::click() and is a
                // focusable egui widget; that made each segment an egui-native
                // Tab focus stop, so at launch Tab walked all six segments
                // before reaching the menu/POM (B055). Requirement 16 enumerates
                // the exclusive shell tab stops and the Status_Bar is not among
                // them, so the segments must not be focusable.
                // Validates: Requirement 5 (status bar display); B055 (not a tab stop).
                ui.label(phase_label);
                ui.separator();

                // Validates: Requirement 20.1 -- session start timestamp
                ui.label(self.format_session_start());
                ui.separator();

                // Validates: startup-and-session Req 22.8 (CR-NR-081) -- the
                // Active_Profile is shown in the Status_Bar (explicitly "default"
                // when no --profile was given).
                ui.label(self.active_profile_label());
                ui.separator();

                let tab = self.tabs.active_tab();
                let line = tab.cursor.cursor_line();
                let col = tab.cursor.cursor_column();
                // Requirement 7.1: format "Ln {line}, Col {col}" (1-based)
                ui.label(format!("Ln {line}, Col {col}"));
                ui.separator();
                // Requirement 7.3: real encoding from document
                ui.label(tab.encoding_label());
                ui.separator();
                // Requirement 7.1 (view-zoom) -- zoom indicator when non-zero
                {
                    use ff_zoom::ZoomIndicatorState;
                    if let ZoomIndicatorState::Visible { text, .. } =
                        ZoomIndicatorState::from_offset(self.zoom.offset())
                    {
                        ui.colored_label(self.palette.chrome_style.accent_color(), text);
                        ui.separator();
                    }
                }
                // Requirement 7.4: real line count
                ui.label(format!("{} lines", tab.line_count));
                ui.separator();
                // Requirement 6.5: modified indicator
                if tab.is_modified {
                    ui.colored_label(self.palette.chrome_style.accent_color(), "\u{25cf}");
                    ui.separator();
                }
                // Req 16.3: CAPS mode indicator
                if tab.edit_profile.caps.is_on() {
                    ui.colored_label(self.palette.chrome_style.accent_color(), "CAPS");
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
                    ui.label("FileForge Workbench v0.1.0");

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
                                egui::RichText::new("LOG!")
                                    .strong()
                                    .color(egui::Color32::from_rgb(0xD2, 0x0F, 0x39)),
                            ))
                            .on_hover_text(format!("Logging degraded: {reason}"));
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
                *pending.lock().expect("pending lock") = Some(super::state::PendingOpenReq {
                    path,
                    owning_env: None,
                    identity: None,
                    recfm_lrecl: None,
                });
            }
        });
    }
}
