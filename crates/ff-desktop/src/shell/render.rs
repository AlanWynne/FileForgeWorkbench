//! # Shell Panel Rendering
//!
//! Title line, command field, key label bar, status bar, and central panel rendering.

use eframe::egui;

use crate::primary_option_menu;
use ff_core::LifecyclePhase;
use ff_keys::FunctionKey;

use crate::catalog_manager_dialog::{self, NewCatalogForm};
use crate::dataset_alloc_dialog::{self};
use crate::editor_panel;
use crate::files_panel;
use crate::tab_state::TabKind;
use crate::toolchain_panel;

use super::helpers::*;
use super::WorkbenchShell;

/// Compute the logging-degradation reason for the status-bar indicator (B038,
/// CR-NR-086, logging-subsystem Req 8.7). Returns `Some(reason)` when logging
/// has degraded -- the subsystem is in fallback (no-op) mode, or records have
/// been dropped -- and `None` when logging is healthy (indicator hidden).
///
/// Pure so the decision is unit-testable without a running log subsystem; the
/// live values are supplied by the caller via `ff_logging::is_fallback()` /
/// `dropped_count()`.
pub(crate) fn logging_degradation_reason(is_fallback: bool, dropped: u64) -> Option<String> {
    if !is_fallback && dropped == 0 {
        return None;
    }
    let mut reason = String::new();
    if is_fallback {
        reason.push_str("log file unavailable (fallback mode)");
    }
    if dropped > 0 {
        if !reason.is_empty() {
            reason.push_str("; ");
        }
        reason.push_str(&format!("{dropped} log record(s) dropped"));
    }
    Some(reason)
}

/// The five stacked strip rects of a split region, top to bottom (CR-CH-041 +
/// B072). Extracted as a pure value so the chrome ORDER is unit-testable without
/// egui: tab bar, menu bar, Title_Line, command line, then the Context body.
pub(crate) struct SplitRegionRects {
    pub bar_rect: egui::Rect,
    pub menu_rect: egui::Rect,
    pub title_rect: egui::Rect,
    pub cmd_rect: egui::Rect,
    pub body_rect: egui::Rect,
}

/// Compute a split region's stacked chrome strips from its outer `rect`
/// (CR-CH-041 Req 16.2, B072). TOP TO BOTTOM: tab bar (24), menu bar (24),
/// Title_Line (20), command line (24), then the Context body fills the
/// remainder. This matches the unsplit / detached ISPF chrome order -- the
/// command line sits directly UNDER the Title_Line and ABOVE the body, NOT at
/// the region bottom (the B072 defect). Each strip is clamped to `rect.max.y`
/// so a very short region degrades gracefully (the body simply shrinks toward
/// empty rather than overflowing).
pub(crate) fn split_region_strip_rects(rect: egui::Rect) -> SplitRegionRects {
    let tab_bar_h = 24.0_f32;
    let menu_bar_h = 24.0_f32;
    let title_h = 20.0_f32;
    let cmd_field_h = 24.0_f32;
    let bar_rect = egui::Rect::from_min_max(
        rect.min,
        egui::pos2(rect.max.x, (rect.min.y + tab_bar_h).min(rect.max.y)),
    );
    let menu_rect = egui::Rect::from_min_max(
        egui::pos2(rect.min.x, bar_rect.max.y),
        egui::pos2(rect.max.x, (bar_rect.max.y + menu_bar_h).min(rect.max.y)),
    );
    let title_rect = egui::Rect::from_min_max(
        egui::pos2(rect.min.x, menu_rect.max.y),
        egui::pos2(rect.max.x, (menu_rect.max.y + title_h).min(rect.max.y)),
    );
    // Command line strip: directly UNDER the Title_Line (B072), not at the bottom.
    let cmd_rect = egui::Rect::from_min_max(
        egui::pos2(rect.min.x, title_rect.max.y),
        egui::pos2(rect.max.x, (title_rect.max.y + cmd_field_h).min(rect.max.y)),
    );
    // The Context body fills the remainder below the command line.
    let body_rect = egui::Rect::from_min_max(egui::pos2(rect.min.x, cmd_rect.max.y), rect.max);
    SplitRegionRects {
        bar_rect,
        menu_rect,
        title_rect,
        cmd_rect,
        body_rect,
    }
}

impl WorkbenchShell {
    /// The derived [`Placement`](crate::tab_manager::Placement) of the Workspace
    /// instance `tab_id` (CR-CH-041, Req 16.5/16.6). Detached takes precedence:
    /// a tab recorded in `floating_tabs` is in its own OS window; otherwise it is
    /// Docked in the layout-tree leaf that owns it (the root leaf when unsplit).
    /// Falls back to `Docked { root }` when the tab is neither floating nor found
    /// in any leaf (should not happen for a live tab; keeps the accessor total).
    ///
    /// Placement is DERIVED every call from the floating set + layout tree; it is
    /// never stored on the instance or persisted on its `Workspace_Descriptor`
    /// (the layout snapshot is the single source of truth for placement).
    // CR-CH-041: the public model accessor for an instance's derived Placement
    // (Req 16.5/16.6), validated by unit + full-shell tests. The in-window split
    // render resolves the leaf directly from the tree walk (so it does not call
    // this), but it is the canonical Placement query for the detached-fold-in
    // path and callers that hold only a TabId; `allow(dead_code)` because the
    // lib/bin clippy scope does not see its test consumers.
    #[allow(dead_code)]
    pub(super) fn placement_of(
        &self,
        tab_id: crate::tab_state::TabId,
    ) -> crate::tab_manager::Placement {
        use crate::tab_manager::Placement;
        if self.floating_tabs.iter().any(|ft| ft.tab_id == tab_id) {
            return Placement::Detached;
        }
        match self.tabs.docked_leaf_of(tab_id) {
            Some(leaf) => Placement::Docked { leaf },
            None => Placement::Docked {
                leaf: ff_layout::TabGroupId::new(0),
            },
        }
    }

    pub(super) fn render_title_line(&self, ctx: &egui::Context) {
        // CR-CH-041 IRP-b: the app-level Title_Line is the unsplit single
        // instance's chrome; it delegates to the shared Ui-level painter so the
        // split-region path (which draws the SAME Title_Line for the instance
        // placed in each region) is byte-identical. The active tab is the sole
        // docked instance when unsplit.
        let tab_index = self.tabs.active_index();
        egui::TopBottomPanel::top("title_line").show(ctx, |ui| {
            self.render_title_line_into_ui(ui, tab_index);
        });
    }

    /// Paint the Title_Line for the tab at `tab_index` INTO an existing `Ui`
    /// (CR-CH-041 IRP-b: the ctx-panel-free core of [`render_title_line`]).
    ///
    /// The app-level unsplit path wraps this in a `TopBottomPanel` (byte-identical
    /// to before the extraction, passing the active tab); the split-region path
    /// calls it directly for the instance placed in each region so every region
    /// shows its own instance's Title_Line. Home banner / editor path / menu
    /// label styling is preserved (delegated through `kind_title`).
    pub(super) fn render_title_line_into_ui(&self, ui: &mut egui::Ui, tab_index: usize) {
        use ff_theme::mode::VisualMode;
        let Some(tab) = self.tabs.tabs().get(tab_index) else {
            return;
        };
        // CR-NR-090 B.1: the Title_Line label comes from the Kind registry
        // (kind_title) so a reconfigured/user Kind shows its configured title;
        // Home banner / menu label / editor path are delegated inside kind_title.
        let text = self.kind_title(tab);
        let is_legacy = self.palette.mode == VisualMode::Legacy;
        let is_pom = tab.is_home;
        // CR-CH-042 (menu-workspace Req 20.2; menu-and-statusbar Req 17.11): EVERY
        // Menu Workspace Title_Line is CENTERED, uniform for the POM, Settings, and
        // user menus (the former POM-centered vs Settings-left inconsistency is
        // removed). Non-menu Contexts (editor path, panel Kind title) stay
        // left-aligned.
        let is_menu_workspace = tab.kind == crate::tab_state::TabKind::MenuWorkspace;
        {
            if is_pom {
                // POM title: black background, blue text, centered (theme concern).
                let bg = egui::Color32::BLACK;
                let fg = egui::Color32::from_rgb(0x00, 0x55, 0xFF);
                let rect = ui.max_rect();
                ui.painter().rect_filled(rect, 0.0, bg);
                ui.centered_and_justified(|ui| {
                    ui.colored_label(fg, egui::RichText::new(&text).monospace().strong());
                });
            } else if is_menu_workspace {
                // CR-CH-042: a non-Home Menu Workspace (Settings / user menu) is
                // CENTERED like the POM, using the theme's Title_Line colours
                // (Legacy blue-bg/white per Req 17.8, else accent-tinted per 21.5).
                let bg = to_egui_color(self.palette.ui.primary_menu_bg);
                let fg = to_egui_color(self.palette.ui.menu_bar_fg);
                let rect = ui.max_rect();
                ui.painter().rect_filled(rect, 0.0, bg);
                ui.centered_and_justified(|ui| {
                    ui.colored_label(fg, egui::RichText::new(&text).monospace());
                });
            } else if is_legacy {
                // Validates: Requirement 17.8 — Legacy: blue bg, white text
                let bg = to_egui_color(self.palette.ui.primary_menu_bg);
                let fg = to_egui_color(self.palette.ui.menu_bar_fg);
                let rect = ui.available_rect_before_wrap();
                ui.painter().rect_filled(rect, 0.0, bg);
                ui.colored_label(fg, egui::RichText::new(text).monospace());
            } else {
                // Validates: Requirement 21.5 -- non-Legacy title line paints the
                // accent-tinted primary_menu_bg background with menu_bar_fg text,
                // mirroring the Legacy branch (previously flat / no fill).
                let bg = to_egui_color(self.palette.ui.primary_menu_bg);
                let fg = to_egui_color(self.palette.ui.menu_bar_fg);
                let rect = ui.available_rect_before_wrap();
                ui.painter().rect_filled(rect, 0.0, bg);
                ui.colored_label(fg, egui::RichText::new(text).monospace());
            }
        }
    }

    // ── Command field ────────────────────────────────────────────────────

    pub(super) fn render_command_field(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("command_field").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Command ===>");
                let cmd_id = egui::Id::new("command_field_input");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.command_text)
                        .id(cmd_id)
                        .desired_width(f32::INFINITY)
                        .font(egui::TextStyle::Monospace),
                );
                // Validates: Requirement 16.1, 16.2 — request focus once after Tab cycle
                // lands on CommandField, or on startup. NOT every frame — that would steal
                // focus from POM buttons and other interactive elements.
                // Suppressed when a modal dialog is open so the dialog retains focus.
                if self.command_field_focus_requested && !self.modal_open {
                    self.command_field_focus_requested = false;
                    ctx.memory_mut(|m| m.request_focus(cmd_id));
                }
                // Validates: Requirement 8.1 — Enter while field has focus submits the command.
                // Use lost_focus() to catch the frame egui clears focus on Enter for
                // single-line TextEdit (egui 0.29 surrenders focus on Enter).
                // Also check has_focus() as a fallback for frames where focus is retained.
                let field_has_focus = response.has_focus() || response.lost_focus();
                // Validates: Requirement 2.5 -- register command field state for automation.
                self.automation.register_str(
                    crate::automation::ids::COMMAND_FIELD,
                    crate::automation::ControlState::with_value(&self.command_text),
                );
                if field_has_focus
                    && ctx.input(|i| i.key_pressed(egui::Key::Enter))
                    && !self.command_text.is_empty()
                {
                    let cmd = self.command_text.trim().to_string();
                    // CR-CH-033 (Req 13.1): run through the single decision point
                    // that applies the Command_Line_Outcome to the field (clear on
                    // success, restore on error, or the command's explicit outcome
                    // such as RETRIEVE's recall). The field is left intact during
                    // dispatch so field-reading commands (RETRIEVE) still work.
                    self.run_command_line(&cmd);
                    // Return focus to the command field after every command execution.
                    self.command_field_focus_requested = true;
                }

                // ── SCROLL ===> field — Validates: Requirement 19.1, 19.2, 19.3 ──
                ui.separator();
                ui.label("SCROLL ===>");
                let scroll_id = egui::Id::new("scroll_field_input");
                let scroll_resp = ui.add(
                    egui::TextEdit::singleline(&mut self.scroll_field_text)
                        .id(scroll_id)
                        .desired_width(60.0)
                        .font(egui::TextStyle::Monospace),
                );
                // On Enter in the SCROLL field, update the active scroll amount.
                // Validates: Requirement 19.2
                let scroll_has_focus = scroll_resp.has_focus() || scroll_resp.lost_focus();
                if scroll_has_focus && ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let text = self.scroll_field_text.trim().to_string();
                    if let Some(amount) = crate::scroll_amount::ScrollAmount::parse(&text) {
                        self.scroll_amount = amount;
                        self.scroll_field_text = self.scroll_amount.display_string();
                        self.open_error = None;
                    } else {
                        self.open_error = Some(format!(
                            "SCROLL: '{}' is not a valid scroll amount (PAGE/HALF/CSR/MAX/DATA/n)",
                            text
                        ));
                    }
                    // Return focus to command field.
                    self.command_field_focus_requested = true;
                }
            });
        });
    }

    /// Render a Detached_Workspace's own `Command ===>` field into its child
    /// viewport (CR-CH-036, menu-and-statusbar Req 18.10). Called from WITHIN
    /// `with_workspace_context`, so `self.command_text` currently holds THIS
    /// window's buffer and dispatch targets THIS window's tab. The panel and
    /// widget ids are salted by `tab_id` so they never collide with the
    /// Primary_Window's command field or another detached window's.
    ///
    /// Validates: menu-and-statusbar Requirement 18.10
    pub(super) fn render_detached_command_field(
        &mut self,
        ctx: &egui::Context,
        tab_id: crate::tab_state::TabId,
    ) {
        let panel_id = egui::Id::new(("detached_command_field", tab_id.0));
        let cmd_id = egui::Id::new(("detached_command_field_input", tab_id.0));
        let accent = to_egui_color(self.palette.editor.accent);
        let modal_open = self.modal_open;
        // The detached field's buffers are the shell's own fields right now
        // (installed by `with_workspace_context`). Render the shared body against
        // them via short-lived local bindings, then reflect focus/submit back.
        let mut command_text = std::mem::take(&mut self.command_text);
        let mut focus_requested = self.command_field_focus_requested;
        let open_error = self.open_error.clone();
        let submitted = egui::TopBottomPanel::top(panel_id)
            .show(ctx, |ui| {
                Self::render_command_field_body(
                    ctx,
                    ui,
                    cmd_id,
                    &mut command_text,
                    &mut focus_requested,
                    modal_open,
                    accent,
                    open_error.as_deref(),
                )
            })
            .inner;
        self.command_text = command_text;
        self.command_field_focus_requested = focus_requested;
        if let Some(cmd) = submitted {
            // Dispatches through the SAME pipeline; because we are inside
            // `with_workspace_context`, it acts on this window's tab and the
            // Command_Line_Outcome applies to this window's buffer.
            self.run_command_line(&cmd);
            self.command_field_focus_requested = true;
        }
    }

    /// Shared `Command ===>` field body used by BOTH the Detached_Workspace
    /// command line (Req 18.10) and each in-window split region's command line
    /// (CR-NR-094, Req 15.1-15.9). It borrows only the field's own state -- never
    /// `&mut self` -- so a caller can bind it to whichever command context owns
    /// the region (the shell's fields for a detached window, or an entry of
    /// `region_cmd_ctx` for a split leaf). Returns `Some(command)` when the user
    /// pressed Enter on a non-empty line this frame; the caller performs the
    /// dispatch against the correct tab. The `cmd_id` MUST be a stable, per-region
    /// salted id so focus round-trips and Tab-order stay deterministic (B056).
    ///
    /// Validates: layout-and-docking Requirement 15.1, 15.2, 15.3, 15.9
    #[allow(clippy::too_many_arguments)]
    fn render_command_field_body(
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        cmd_id: egui::Id,
        command_text: &mut String,
        focus_requested: &mut bool,
        modal_open: bool,
        accent: egui::Color32,
        open_error: Option<&str>,
    ) -> Option<String> {
        let mut submitted = None;
        ui.horizontal(|ui| {
            ui.label("Command ===>");
            let response = ui.add(
                egui::TextEdit::singleline(command_text)
                    .id(cmd_id)
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Monospace),
            );
            if *focus_requested && !modal_open {
                *focus_requested = false;
                ctx.memory_mut(|m| m.request_focus(cmd_id));
            }
            let field_has_focus = response.has_focus() || response.lost_focus();
            if field_has_focus
                && ctx.input(|i| i.key_pressed(egui::Key::Enter))
                && !command_text.is_empty()
            {
                submitted = Some(command_text.trim().to_string());
            }
            // Status/error line for THIS region (its own open_error).
            if let Some(err) = open_error {
                ui.separator();
                ui.colored_label(accent, egui::RichText::new(err).monospace().small());
            }
        });
        submitted
    }

    // ── Key label bar ─────────────────────────────────────────────────────

    /// Render the ISPF-style function key label bar in the footer.
    ///
    /// Shows only assigned slots as `Fn label` pairs.
    /// Validates: Requirement 4.1, 4.2, 4.3
    pub(super) fn render_key_label_bar(&mut self, ctx: &egui::Context) {
        if !self.key_bar_visible {
            return;
        }
        let key_color = to_egui_color(self.palette.editor.accent);
        let label_color = to_egui_color(self.palette.editor.foreground);
        let mut clicked_key: Option<FunctionKey> = None;
        egui::TopBottomPanel::bottom("key_label_bar").show(ctx, |ui| {
            for row in [self.key_label_bar.row0(), self.key_label_bar.row1()] {
                ui.horizontal(|ui| {
                    for slot in row {
                        let key = slot.key;
                        let btn_text = if let Some(lbl) = &slot.label {
                            format!("{} {}", key.display_name(), lbl)
                        } else {
                            key.display_name().to_string()
                        };
                        let enabled = slot.label.is_some();
                        let tooltip = self
                            .key_map_resolver
                            .active_key_map()
                            .get_plain(key)
                            .map(|b| b.command().to_string())
                            .unwrap_or_default();
                        // CR-CH-023 Req 16.9: the Key_Label_Bar slots are
                        // clickable but MUST NOT be keyboard Tab stops (they
                        // duplicate the physical function keys). A plain
                        // `Button` senses `CLICK | FOCUSABLE`; rendering a
                        // `Label` with a click-only Sense (no FOCUSABLE bit)
                        // keeps the mouse click while removing the widget from
                        // egui-native Tab traversal.
                        let text = egui::RichText::new(&btn_text)
                            .color(if enabled { label_color } else { key_color })
                            .monospace()
                            .small();
                        let resp = if enabled {
                            ui.add(egui::Label::new(text).sense(egui::Sense::CLICK))
                        } else {
                            // Disabled slots are pure display (never a Tab stop
                            // and not clickable).
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
            }
        });
        if let Some(key) = clicked_key {
            if let Some(cmd) = self
                .key_map_resolver
                .active_key_map()
                .get_plain(key)
                .map(|b| b.command().to_string())
            {
                // A clicked key-label bar slot is treated as if the function
                // key was pressed (function-keys Req 16.1): route it through the
                // shared key-dispatch so the Command ===> field content is
                // merged as the argument (Req 9.8, B066), then Target_Resolution
                // runs a user-defined command id's target (Req 4.4) or falls
                // through to the pipeline (Req 10.2).
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
                        ui.colored_label(to_egui_color(self.palette.editor.accent), text);
                        ui.separator();
                    }
                }
                // Requirement 7.4: real line count
                ui.label(format!("{} lines", tab.line_count));
                ui.separator();
                // Requirement 6.5: modified indicator
                if tab.is_modified {
                    ui.colored_label(to_egui_color(self.palette.editor.accent), "\u{25cf}");
                    ui.separator();
                }
                // Req 16.3: CAPS mode indicator
                if tab.edit_profile.caps.is_on() {
                    ui.colored_label(to_egui_color(self.palette.editor.accent), "CAPS");
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
                *pending.lock().expect("pending lock") = Some(path);
            }
        });
    }

    // ── Central panel ────────────────────────────────────────────────────

    /// Honour the one-shot interior-focus latches set by the Boundary_Policy
    /// (B056). Called by each Workspace render arm AFTER it has produced its
    /// interior controls this frame, so the id passed is the FRESH same-frame id
    /// that egui will actually recognise (a stale previous-frame id does not
    /// round-trip through `request_focus`).
    ///
    /// Validates: Requirement 16.3, 16.8 (CR-CH-023)
    fn honour_interior_focus_latch(
        &mut self,
        ctx: &egui::Context,
        first_interior: Option<egui::Id>,
        last_interior: Option<egui::Id>,
    ) {
        if self.focus_first_interior_requested {
            self.focus_first_interior_requested = false;
            if let Some(id) = first_interior {
                ctx.memory_mut(|m| m.request_focus(id));
            }
        }
        if self.focus_last_interior_requested {
            self.focus_last_interior_requested = false;
            if let Some(id) = last_interior {
                ctx.memory_mut(|m| m.request_focus(id));
            }
        }
    }

    /// Apply the [`ShellRequest`]s a `WorkspaceContext` enqueued during its
    /// `render`, routing each through the existing shell pipelines so the
    /// observable result is identical to the pre-framework per-arm handling
    /// (CR-NR-078, Requirement 2.2). Drained AFTER the panel is put back, so no
    /// borrow conflicts arise.
    pub(super) fn apply_shell_requests(
        &mut self,
        requests: Vec<super::workspace_context::ShellRequest>,
    ) {
        use super::workspace_context::ShellRequest;
        for req in requests {
            match req {
                ShellRequest::Command(cmd) => self.handle_command(&cmd),
                ShellRequest::Target(target) => self.dispatch_command_target(&target),
                ShellRequest::OpenFile(path) => {
                    let mut p = ff_command::CommandParams::new();
                    p.insert("path", path.as_str());
                    if let ff_command::CommandResult::Err(e) =
                        self.dispatch.execute_command("file.open", p)
                    {
                        self.open_error = Some(e.to_string());
                    }
                }
                ShellRequest::Status(message) => self.open_error = Some(message),
            }
        }
    }

    /// Dispatch a [`WorkspaceContext`] through the single framework code path
    /// (CR-NR-078, Requirement 1.2), using the owned-panel-swap borrow strategy
    /// (Option A): the caller moves the panel OUT of the shell, hands it here
    /// with a `ShellServices` builder that borrows the shell's OTHER fields, we
    /// render, drain the panel's `ShellRequest`s, and honour the focus latch with
    /// the returned `InteriorFocus`. The panel is put back by the caller. This is
    /// the ONE place the focus contract is applied, so a migrated arm cannot omit
    /// it.
    ///
    /// Returns the `InteriorFocus` the Context reported (so the caller records it
    /// on `first_interior_id`/`last_interior_id`).
    pub(super) fn render_workspace_context<C>(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        context: &mut C,
    ) -> super::workspace_context::InteriorFocus
    where
        C: super::workspace_context::WorkspaceContext,
    {
        use super::workspace_context::ShellServices;
        let mut requests = Vec::new();
        let focus = {
            let mut services = ShellServices {
                config: &self.config_handle,
                runtime: &self.runtime,
                notifications: &self.notification_queue,
                themes_dir: self.themes_dir(),
                menus_dir: self.menus_dir(),
                requests: &mut requests,
            };
            context.render(ui, &mut services)
        };
        self.apply_shell_requests(requests);
        self.apply_interior_focus(ctx, focus);
        focus
    }

    /// Record a Context's reported [`InteriorFocus`] on the shell and honour the
    /// focus latch -- the ONE place the CR-CH-023 Boundary_Policy interior anchors
    /// are applied (CR-NR-078, Requirement 1.2). Both the generic trait dispatch
    /// (`render_workspace_context`) and the specialized MenuWorkspace arm (whose
    /// render signature carries menu-specific calendar inputs/outputs and so does
    /// not fit the `ShellServices`-only trait) route through here, so no arm can
    /// "forget" to apply the focus contract.
    pub(super) fn apply_interior_focus(
        &mut self,
        ctx: &egui::Context,
        focus: super::workspace_context::InteriorFocus,
    ) {
        self.first_interior_id = focus.first;
        self.last_interior_id = focus.last;
        self.honour_interior_focus_latch(ctx, focus.first, focus.last);
    }

    pub(super) fn render_central_panel(&mut self, ctx: &egui::Context) {
        // ── File Explorer side panel (ctx-level, resizable) ────────────
        // Must be shown before CentralPanel so egui allocates space correctly.
        // Validates: Requirement 1.3 file-tree-panel, fix B019
        // Validates: Requirement 20.1 -- Tab from CommandField in FilesPanel transfers focus
        // to the first catalog node in the catalog tree.
        // ── Tab from FilesPanel command field → first catalog node ──────
        // Validates: Requirement 20.1
        // The Files Panel has its own internal "Command ===>" TextEdit.
        // When that field has egui focus and the user presses plain Tab,
        // we consume the Tab event and request focus on the first catalog node.
        // We cannot use focus_stop here because focus_stop tracks the shell's
        // top-level command field, not the panel-internal one.
        let is_files_panel = self.tabs.active_tab().kind == TabKind::FilesPanel;
        if is_files_panel && !self.modal_open {
            let files_cmd_id = egui::Id::new("files_panel_cmd");
            let files_cmd_focused = ctx.memory(|m| m.focused() == Some(files_cmd_id));
            if files_cmd_focused {
                let tab_pressed = ctx.input_mut(|i| {
                    if i.key_pressed(egui::Key::Tab) && !i.modifiers.shift {
                        i.events.retain(|e| {
                            !matches!(
                                e,
                                egui::Event::Key {
                                    key: egui::Key::Tab,
                                    ..
                                }
                            )
                        });
                        return true;
                    }
                    false
                });
                if tab_pressed {
                    self.files_panel.tree_focus_requested = true;
                }
            }
        }

        // CR-NR-093 B071: the ctx-level full-window File Explorer only applies
        // when the Workspace is NOT split. While split, the File Explorer renders
        // INSIDE its region (via render_active_tab_body's FileExplorerPanel arm),
        // so the split must stay visible -- do not take the full-window branch.
        let is_file_explorer =
            self.tabs.active_tab().kind == TabKind::FileExplorerPanel && !self.tabs.is_split();
        // CR-NR-060 Slice A: the NavModel-backed modern explorer is the sole File
        // Explorer content (legacy inline tree retired).
        // ── Toolchain Panel (bottom dock) ────────────────────────────────
        if self.show_toolchain_panel {
            egui::TopBottomPanel::bottom("toolchain_panel")
                .resizable(true)
                .min_height(160.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Toolchain Panel").monospace().strong());
                        if ui.small_button("✕").clicked() {
                            self.show_toolchain_panel = false;
                        }
                    });
                    ui.separator();
                    if let Some((file, line, col)) =
                        toolchain_panel::render(ui, &mut self.toolchain_panel)
                    {
                        // Navigate editor to the clicked diagnostic location.
                        // Req 16.7, 18.6 — open the file if not already open,
                        // then scroll to the target line.
                        let _ = self.shell_open_file(&file);
                        self.nav_manager.locate(&line.to_string(), &mut self.tabs);
                        let _ = col; // column navigation deferred to Phase W follow-up
                    }
                });
        }

        // CR-NR-060 Slice A: the modern NavModel explorer is the File Explorer
        // Context content. Rendered here -- after all bottom/side panels -- so
        // the CentralPanel is added last (egui panel-ordering requirement).
        if is_file_explorer {
            self.render_nav_explorer(ctx);
        }

        // Validates: Requirement 14.8 — central panel dispatches on tab kind
        if !is_file_explorer {
            egui::CentralPanel::default().show(ctx, |ui| {
                // CR-NR-092 (Slice 2b): when split, the central panel is divided
                // into two regions, each with its own tab bar + Context body and
                // a draggable Splitter between them (Req 13.1, 13.4, 13.5, 13.6).
                if self.tabs.is_split() {
                    self.render_split_central(ctx, ui);
                } else {
                    self.render_active_tab_body(ctx, ui);
                }
            });
        } // end !is_file_explorer
    }

    /// Render the recursive split inside the CentralPanel (CR-NR-093, Slice 2c.1).
    ///
    /// Walks the authoritative `TabGroupTree` to ARBITRARY depth: each internal
    /// `Split` node divides its rect by direction/proportion with a draggable
    /// Splitter; each `Leaf` is a region drawn by
    /// [`render_split_region`](Self::render_split_region) (its own tab bar + the
    /// active Context body via [`render_active_tab_body`], focused-leaf highlight).
    /// A snapshot of the tree is cloned up front so the immutable walk can compute
    /// rects while the per-leaf render borrows `&mut self`.
    ///
    /// Validates: layout-and-docking Requirement 14.1, 14.2, 14.4
    fn render_split_central(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        // Defensive: caller checked is_split(); a single leaf renders normally.
        if !self.tabs.is_split() {
            self.render_active_tab_body(ctx, ui);
            return;
        }
        // CR-NR-094 Slice 2d: keep the per-region command-line contexts in
        // lockstep with the current leaves before rendering the regions.
        self.reconcile_region_cmd_ctx();
        // B073: reset the focused-region menu-first anchor; the focused region's
        // menu render (below) re-captures it this frame.
        self.focused_region_menu_first = None;
        let tree = self.tabs.layout_tree().clone();
        let focused = self.tabs.focused_leaf_id();
        let full = ui.available_rect_before_wrap();
        // CR-NR-093 Slice 2c.2: rebuild the per-frame leaf-rect map so a
        // tab-header drag can be resolved to a drop target on release.
        self.split_leaf_rects.clear();
        self.render_tree_node(ctx, ui, &tree, full, focused);
        // Resolve a completed tab-header drag (drop) now that every leaf rect is
        // known this frame (Req 14.6, 14.7, 14.8).
        self.resolve_split_tab_drop(ctx);
    }

    /// Reconcile the per-region command-line context map with the current split
    /// leaves (CR-NR-094, Slice 2d, Req 15.5). Inserts a fresh default
    /// `WorkspaceCommandContext` for any leaf id that lacks one, and drops any
    /// entry whose leaf no longer exists (collapsed/merged). Because leaf ids are
    /// allocated monotonically and never reused within a split session, a moved
    /// tab (Req 14.6) changes leaf MEMBERSHIP but not leaf IDENTITY, so command
    /// text stays with the leaf and never travels with a moved tab.
    ///
    /// Validates: layout-and-docking Requirement 15.5
    pub(super) fn reconcile_region_cmd_ctx(&mut self) {
        if !self.tabs.is_split() {
            // Unsplit: no per-region contexts (the single top-level field is used).
            if !self.region_cmd_ctx.is_empty() {
                self.region_cmd_ctx.clear();
            }
            return;
        }
        let leaves: std::collections::HashSet<ff_layout::TabGroupId> =
            self.tabs.leaf_ids().into_iter().collect();
        // Drop contexts for leaves that no longer exist.
        self.region_cmd_ctx.retain(|id, _| leaves.contains(id));
        // Insert a fresh context for any new leaf.
        for id in leaves {
            self.region_cmd_ctx.entry(id).or_default();
        }
    }

    /// Resolve an in-progress tab-header drag on release (CR-NR-093, Slice 2c.2).
    ///
    /// While `split_tab_drag` is set, on pointer release: if the pointer is over
    /// a DIFFERENT leaf's recorded rect, move the dragged tab into that leaf
    /// (Req 14.6); a drop over the tab's own leaf is a no-op (Req 14.8). Dropping
    /// OUTSIDE every leaf rect (e.g. beyond the workbench) falls through to the
    /// existing detach gesture handled by the tab bar, so it is left alone here.
    /// The drag state is cleared on release regardless.
    ///
    /// Validates: layout-and-docking Requirement 14.6, 14.7, 14.8
    fn resolve_split_tab_drop(&mut self, ctx: &egui::Context) {
        let Some((tab_id, source_leaf)) = self.split_tab_drag else {
            return;
        };
        // Still dragging? Keep the state and wait for release.
        let pointer_released = ctx.input(|i| i.pointer.any_released());
        if !pointer_released {
            return;
        }
        // Release: clear the drag and, if over another leaf, perform the move.
        self.split_tab_drag = None;
        let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) else {
            return;
        };
        let target = self
            .split_leaf_rects
            .iter()
            .find(|(_, r)| r.contains(pos))
            .map(|(id, _)| *id);
        if let Some(target) = target {
            if target != source_leaf {
                self.tabs.move_tab_to_group(tab_id, target);
            }
        }
    }

    /// Recursively render one `TabGroupTree` node into `rect` (CR-NR-093).
    fn render_tree_node(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        node: &ff_layout::TabGroupTree,
        rect: egui::Rect,
        focused: ff_layout::TabGroupId,
    ) {
        use ff_layout::TabGroupTree;
        match node {
            TabGroupTree::Leaf(group) => {
                self.render_split_region(ctx, ui, group.id, rect, group.id == focused);
            }
            TabGroupTree::Split {
                direction,
                proportion,
                first,
                second,
            } => {
                let horizontal = matches!(direction, ff_layout::SplitDirection::Horizontal);
                let splitter_thickness = 6.0_f32;
                let min = ff_layout::MIN_TAB_GROUP_SIZE;
                let (first_rect, splitter_rect, second_rect) = if horizontal {
                    let avail = (rect.width() - splitter_thickness).max(0.0);
                    let mut first_w = (avail * *proportion).clamp(0.0, avail);
                    if avail >= 2.0 * min {
                        first_w = first_w.clamp(min, avail - min);
                    }
                    let x_split = rect.min.x + first_w;
                    (
                        egui::Rect::from_min_max(rect.min, egui::pos2(x_split, rect.max.y)),
                        egui::Rect::from_min_max(
                            egui::pos2(x_split, rect.min.y),
                            egui::pos2(x_split + splitter_thickness, rect.max.y),
                        ),
                        egui::Rect::from_min_max(
                            egui::pos2(x_split + splitter_thickness, rect.min.y),
                            rect.max,
                        ),
                    )
                } else {
                    let avail = (rect.height() - splitter_thickness).max(0.0);
                    let mut first_h = (avail * *proportion).clamp(0.0, avail);
                    if avail >= 2.0 * min {
                        first_h = first_h.clamp(min, avail - min);
                    }
                    let y_split = rect.min.y + first_h;
                    (
                        egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, y_split)),
                        egui::Rect::from_min_max(
                            egui::pos2(rect.min.x, y_split),
                            egui::pos2(rect.max.x, y_split + splitter_thickness),
                        ),
                        egui::Rect::from_min_max(
                            egui::pos2(rect.min.x, y_split + splitter_thickness),
                            rect.max,
                        ),
                    )
                };

                // Recurse into children first (they draw regions + nested nodes).
                self.render_tree_node(ctx, ui, first, first_rect, focused);
                self.render_tree_node(ctx, ui, second, second_rect, focused);

                // Draggable Splitter for THIS node (Req 14.4). Identity: the
                // first leaf id of the first child (stable per node).
                let node_key = first.all_group_ids().first().copied();
                let splitter_id = ui
                    .id()
                    .with(("workspace_splitter", node_key.map(|k| k.value())));
                let resp = ui.interact(splitter_rect, splitter_id, egui::Sense::click_and_drag());
                let hovered = resp.hovered() || resp.dragged();
                let visual = if hovered {
                    ui.visuals().widgets.active.bg_fill
                } else {
                    ui.visuals().widgets.noninteractive.bg_stroke.color
                };
                ui.painter().rect_filled(splitter_rect, 0.0, visual);
                if hovered {
                    ctx.set_cursor_icon(if horizontal {
                        egui::CursorIcon::ResizeHorizontal
                    } else {
                        egui::CursorIcon::ResizeVertical
                    });
                }
                if resp.dragged() {
                    if let (Some(pos), Some(key)) =
                        (ctx.input(|i| i.pointer.interact_pos()), node_key)
                    {
                        let new_prop = if horizontal {
                            (pos.x - rect.min.x) / rect.width().max(1.0)
                        } else {
                            (pos.y - rect.min.y) / rect.height().max(1.0)
                        };
                        self.tabs.set_node_proportion(key, new_prop);
                    }
                }
            }
        }
    }

    /// Render one split region (CR-NR-093): a per-leaf tab bar in a strip at the
    /// top of `rect`, then that leaf's active Context body below it, then a
    /// focus-highlight border if `is_focused`. Clicking a tab header or the body
    /// focuses this region and activates the clicked tab (Req 14.5).
    fn render_split_region(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        leaf_id: ff_layout::TabGroupId,
        rect: egui::Rect,
        is_focused: bool,
    ) {
        // CR-CH-041 (Req 16.2) + B072: a split region draws the FULL per-instance
        // chrome of the instance placed in it, TOP TO BOTTOM in the SAME order as
        // the unsplit / detached placements (ISPF order): tab bar, then the
        // instance's Kind menu bar, then its Title_Line, then this region's own
        // `Command ===>` line, then the Context body. The rect math is a PURE
        // helper (`split_region_strip_rects`) so the ordering is unit-testable
        // without egui (B072 regression guard).
        let SplitRegionRects {
            bar_rect,
            menu_rect,
            title_rect,
            cmd_rect,
            body_rect,
        } = split_region_strip_rects(rect);

        // CR-NR-093 Slice 2c.2: record this leaf's rect so a tab-header drag can
        // be resolved to a drop target on release (Req 14.6).
        self.split_leaf_rects.push((leaf_id, rect));

        // === Per-leaf tab bar ===
        let indices = self.tabs.leaf_tab_store_indices(leaf_id);
        let active_store = self.tabs.leaf_active_store_index(leaf_id);
        let active_bg = to_egui_color(self.palette.tab_bar.active_bg);
        let inactive_bg = to_egui_color(self.palette.tab_bar.inactive_bg);
        let active_text = to_egui_color(self.palette.tab_bar.active_text);
        let inactive_text = to_egui_color(self.palette.tab_bar.inactive_text);

        let mut clicked_store: Option<usize> = None;
        // A tab-header drag that STARTED this frame in this region: (tab_id, leaf).
        let mut drag_started: Option<(crate::tab_state::TabId, ff_layout::TabGroupId)> = None;
        let mut bar_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(bar_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        bar_ui.set_clip_rect(bar_rect);
        for store_idx in &indices {
            let store_idx = *store_idx;
            let Some(tab) = self.tabs.tabs().get(store_idx) else {
                continue;
            };
            let tab_id = tab.id;
            let is_active = Some(store_idx) == active_store;
            let base_title = self.kind_title(tab);
            let label = if tab.is_modified {
                format!("\u{25cf} {}", base_title)
            } else {
                base_title
            };
            let bg = if is_active { active_bg } else { inactive_bg };
            let fg = if is_active {
                active_text
            } else {
                inactive_text
            };
            // CR-NR-093 Slice 2c.2: headers sense click AND drag so a tab can be
            // dragged from one region and dropped onto another (Req 14.6).
            let btn = egui::Button::new(egui::RichText::new(&label).color(fg).monospace())
                .fill(bg)
                .stroke(if is_active {
                    egui::Stroke::new(1.0_f32, fg)
                } else {
                    egui::Stroke::NONE
                })
                .min_size(egui::vec2(0.0, bar_rect.height()))
                .sense(egui::Sense::click_and_drag());
            let resp = bar_ui.add(btn);
            if resp.clicked() {
                clicked_store = Some(store_idx);
            }
            if resp.drag_started() {
                drag_started = Some((tab_id, leaf_id));
            }
        }
        // Record a newly-started drag on the shell (resolved on release in
        // resolve_split_tab_drop). A fresh drag_started supersedes any stale one.
        if let Some(started) = drag_started {
            self.split_tab_drag = Some(started);
        }

        // === Per-region menu bar (CR-CH-041, Req 16.2) ===
        // The instance placed in this region owns its menu bar; resolve it from
        // that instance's Kind and draw it into the region's menu strip via the
        // shared Ui-level renderer (the same one the app-level and detached bars
        // use). The bar is scoped by leaf id so its widget ids are per-region and
        // never collide with another region's or the (suppressed-while-split)
        // app-level bar.
        self.render_region_menu_bar(ui, leaf_id, menu_rect);

        // === Per-region Title_Line (CR-CH-041, Req 16.2) ===
        // The instance's Title_Line, drawn into the region's title strip via the
        // shared Ui-level painter (byte-identical styling to the app-level one).
        if let Some(store_idx) = active_store {
            let mut title_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(title_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            title_ui.set_clip_rect(title_rect);
            self.render_title_line_into_ui(&mut title_ui, store_idx);
        }

        // === Region body: render this leaf's active Context ===
        let body_response = ui.interact(
            body_rect,
            ui.id().with(("split_region_body", leaf_id.value())),
            egui::Sense::click(),
        );
        let token = self.tabs.set_render_focus_leaf(leaf_id);
        let mut body_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(body_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        body_ui.set_clip_rect(body_rect);
        self.render_active_tab_body(ctx, &mut body_ui);
        self.tabs.restore_render_focus(token);

        // === Per-region command line (CR-NR-094 Slice 2d, Req 15.1-15.9) ===
        self.render_region_command_field(ctx, ui, leaf_id, cmd_rect);

        // === Focus highlight (Req 14.4) ===
        if is_focused {
            let accent = to_egui_color(self.palette.editor.accent);
            ui.painter().rect_stroke(
                rect,
                0.0,
                egui::Stroke::new(2.0_f32, accent),
                egui::StrokeKind::Inside,
            );
        }

        // === Drop_Zone highlight (Req 14.9) ===
        // While a tab-header drag is in progress, mark the region under the
        // pointer as the drop target (unless it is the tab's own region).
        if let Some((_, source_leaf)) = self.split_tab_drag {
            if source_leaf != leaf_id {
                let over = ctx
                    .input(|i| i.pointer.interact_pos())
                    .map(|p| rect.contains(p))
                    .unwrap_or(false);
                if over {
                    let accent = to_egui_color(self.palette.editor.accent);
                    // A translucent fill + a thicker border so the drop target is
                    // unambiguous before release.
                    let fill = egui::Color32::from_rgba_unmultiplied(
                        accent.r(),
                        accent.g(),
                        accent.b(),
                        40,
                    );
                    ui.painter().rect_filled(rect, 0.0, fill);
                    ui.painter().rect_stroke(
                        rect,
                        0.0,
                        egui::Stroke::new(3.0_f32, accent),
                        egui::StrokeKind::Inside,
                    );
                }
            }
        }

        // === Focus routing (Req 14.5) ===
        if let Some(store_idx) = clicked_store {
            self.tabs.focus_leaf_and_activate(leaf_id, store_idx);
        } else if body_response.clicked() {
            // Clicking anywhere in the region focuses it without changing its
            // active tab.
            self.tabs.focus_leaf(leaf_id);
        }
    }

    /// Render the per-region menu bar into `menu_rect` for the instance placed in
    /// leaf `leaf_id` (CR-CH-041, Req 16.2). The bar is the instance's Kind menu
    /// bar (resolved via `resolve_menu_bar_menu_for` -- workspace-kinds Req 4),
    /// drawn through the shared Ui-level renderer inside a per-leaf-salted id
    /// scope so its widget ids never collide with another region's bar or the
    /// (suppressed-while-split) app-level bar. No-op when the leaf has no active
    /// tab (an empty region degrades gracefully).
    ///
    /// Validates: layout-and-docking Requirement 16.2, 16.4, 16.8
    fn render_region_menu_bar(
        &mut self,
        ui: &mut egui::Ui,
        leaf_id: ff_layout::TabGroupId,
        menu_rect: egui::Rect,
    ) {
        let Some(store_idx) = self.tabs.leaf_active_store_index(leaf_id) else {
            return;
        };
        let Some(tab) = self.tabs.tabs().get(store_idx) else {
            return;
        };
        let menu = self.resolve_menu_bar_menu_for(tab);
        let mut menu_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(menu_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        menu_ui.set_clip_rect(menu_rect);
        // Per-region id scope so menu button ids are salted by leaf (Req 16.8:
        // stable, non-colliding ids; the workspace-conformance no-phantom-stop
        // contract). The shared renderer also updates menu_first_id/menu_last_id;
        // while split the app-level bar is suppressed so the focused region's bar
        // is the live one the Boundary_Policy uses.
        menu_ui.push_id(("region_menu_bar", leaf_id.value()), |ui| {
            self.render_menu_bar_into_ui(ui, &menu);
        });
        // B073: if THIS is the focused region, record its menu-bar first-button id
        // so the shell can include it in the focused-region Tab cycle (keeping Tab
        // inside the region). `render_menu_bar_into_ui` just set `menu_first_id`
        // for the bar it drew; capture it for the focused leaf only.
        if self.tabs.focused_leaf_id() == leaf_id {
            self.focused_region_menu_first = self.menu_first_id;
        }
    }

    /// Render one split region's own `Command ===>` line into `cmd_rect` and, on
    /// Enter, dispatch the command against THAT region's active tab (CR-NR-094,
    /// Slice 2d, Req 15.1-15.9).
    ///
    /// The region's command context is taken out of `region_cmd_ctx` for the
    /// duration of the render/dispatch so the shared field body (which borrows
    /// only the field state, never `&mut self`) can be bound to it, and dispatch
    /// can route through `with_workspace_context` -- the SAME Focus_Context seam
    /// the Detached_Workspace uses -- installing the region's active tab and its
    /// command buffers, running the UNCHANGED pipeline, then swapping the
    /// (possibly command-modified) buffers back. The context is reinstated into
    /// the map afterwards so its text/status/scroll persist across frames while
    /// the split lives (Req 15.4, 15.6). Submitting a non-focused region's line
    /// also focuses that region (Req 15.7).
    ///
    /// The widget id is salted by `leaf_id` so it is a stable, per-region Tab
    /// stop that never collides with another region or the top-level field
    /// (B056, Req 15.9).
    ///
    /// Validates: layout-and-docking Requirement 15.1, 15.2, 15.3, 15.4, 15.6, 15.7, 15.9
    fn render_region_command_field(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        leaf_id: ff_layout::TabGroupId,
        cmd_rect: egui::Rect,
    ) {
        // Take this region's context out of the map so the shared body can borrow
        // its fields without also borrowing `self`. Reconciliation guarantees an
        // entry exists for every current leaf, but be defensive.
        let mut region_ctx = self.region_cmd_ctx.remove(&leaf_id).unwrap_or_default();
        let cmd_id = egui::Id::new(("region_command_field_input", leaf_id.value()));
        let accent = to_egui_color(self.palette.editor.accent);
        let modal_open = self.modal_open;

        let mut command_text = std::mem::take(&mut region_ctx.command_text);
        let mut focus_requested = region_ctx.command_field_focus_requested;
        let open_error = region_ctx.open_error.clone();

        let mut field_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(cmd_rect)
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        field_ui.set_clip_rect(cmd_rect);
        let submitted = Self::render_command_field_body(
            ctx,
            &mut field_ui,
            cmd_id,
            &mut command_text,
            &mut focus_requested,
            modal_open,
            accent,
            open_error.as_deref(),
        );

        region_ctx.command_text = command_text;
        region_ctx.command_field_focus_requested = focus_requested;

        if let Some(cmd) = submitted {
            // Dispatch against this region's active tab through the shared
            // Focus_Context seam. Focus the region first so a submit from a
            // non-focused region acts on and focuses that region (Req 15.7).
            self.tabs.focus_leaf(leaf_id);
            if let Some(store_index) = self.tabs.leaf_active_store_index(leaf_id) {
                self.with_workspace_context(store_index, &mut region_ctx, |shell| {
                    shell.run_command_line(&cmd);
                });
            }
            region_ctx.command_field_focus_requested = true;
        }

        // Reinstate the (possibly modified) context so it persists across frames.
        self.region_cmd_ctx.insert(leaf_id, region_ctx);
    }

    /// Render the ACTIVE tab's Context body (the `match tab.kind` dispatch) into
    /// `ui`. Extracted from `render_central_panel` (CR-CH-035, B045) so the SAME
    /// code renders a docked tab in the primary CentralPanel AND a detached tab
    /// inside its floating viewport: the floating loop temporarily sets the
    /// active index to the detached tab, calls this, and restores it. Behaviour
    /// for the docked path is unchanged.
    ///
    /// Validates: menu-and-statusbar Requirement 18.8 (real detached content)
    pub(super) fn render_active_tab_body(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        {
            // CR-CH-023: reset the Boundary_Policy interior anchors each
            // frame; the active Workspace arm re-populates them if it has
            // interior focus stops. Workspaces that leave them None cause
            // Tab from the command field to go straight to the menu bar.
            self.first_interior_id = None;
            self.last_interior_id = None;
            match self.tabs.active_tab().kind {
                TabKind::FilesPanel => {
                    // Validates: Requirement 1.1, 1.7
                    let action = files_panel::render(ui, &mut self.files_panel);
                    match action {
                        files_panel::FilesPanelAction::ReturnToPom => {
                            self.pending_return_to_pom = true;
                        }
                        files_panel::FilesPanelAction::NewCatalog => {
                            if matches!(
                                self.files_panel.dialog,
                                files_panel::FilesDialogState::None
                            ) {
                                // Req 12.1, 12.2 — pre-populate with configured defaults
                                let mf_root = self
                                    .config_handle
                                    .get_string(ff_config::keys::catalogs::DEFAULT_MAINFRAME_ROOT)
                                    .unwrap_or_default();
                                let posix_root = self
                                    .config_handle
                                    .get_string(ff_config::keys::catalogs::DEFAULT_POSIX_ROOT)
                                    .unwrap_or_default();
                                self.files_panel.dialog = files_panel::FilesDialogState::NewCatalog(
                                    NewCatalogForm::with_defaults(mf_root, posix_root),
                                );
                            }
                        }
                        files_panel::FilesPanelAction::EditCatalog(name) => {
                            // Req 4.1 - open Edit Catalog dialog pre-populated
                            if matches!(
                                self.files_panel.dialog,
                                files_panel::FilesDialogState::None
                            ) {
                                if let Some(cat) = self.files_panel.registry.get_by_name(&name) {
                                    let form =
                                        catalog_manager_dialog::EditCatalogForm::from_catalog(cat);
                                    self.files_panel.dialog =
                                        files_panel::FilesDialogState::EditCatalog(form);
                                }
                            }
                        }
                        files_panel::FilesPanelAction::DeleteCatalog(name) => {
                            // Req 4.3 - open Delete Catalog confirmation dialog
                            if matches!(
                                self.files_panel.dialog,
                                files_panel::FilesDialogState::None
                            ) {
                                if let Some(cat) = self.files_panel.registry.get_by_name(&name) {
                                    let confirm =
                                        catalog_manager_dialog::DeleteCatalogConfirm::from_catalog(
                                            cat,
                                        );
                                    self.files_panel.dialog =
                                        files_panel::FilesDialogState::DeleteCatalog(confirm);
                                }
                            }
                        }

                        files_panel::FilesPanelAction::AllocateDataset(catalog_name) => {
                            // Req 5.1 - open Allocate Dataset dialog
                            // Req 13.2 - record which catalog opened the dialog
                            if matches!(
                                self.files_panel.dialog,
                                files_panel::FilesDialogState::None
                            ) {
                                self.files_panel.pending_alloc_catalog = Some(catalog_name.clone());
                                // Req 5.7 — pre-populate Dataset Name with catalog HLQ if set
                                let form = self
                                    .files_panel
                                    .registry
                                    .get_by_name(&catalog_name)
                                    .and_then(|c| c.default_hlq.as_deref())
                                    .map(dataset_alloc_dialog::AllocDatasetForm::with_hlq)
                                    .unwrap_or_default();
                                self.files_panel.dialog =
                                    files_panel::FilesDialogState::AllocateDataset(form);
                            }
                        }
                        files_panel::FilesPanelAction::OpenFile(dsn) => {
                            // Req 16 — resolve physical path from catalog repository + DSN
                            let is_mainframe = self
                                .files_panel
                                .content
                                .selected_catalog
                                .as_deref()
                                .and_then(|n| self.files_panel.registry.get_by_name(n))
                                .map(|c| {
                                    c.catalog_type
                                        == crate::catalog_registry::CatalogType::Mainframe
                                })
                                .unwrap_or(false);
                            if is_mainframe {
                                let catalog_name = self
                                    .files_panel
                                    .content
                                    .selected_catalog
                                    .clone()
                                    .unwrap_or_default();
                                match open_mainframe_dsn(
                                    &self.files_panel.registry,
                                    &catalog_name,
                                    &dsn,
                                ) {
                                    Err(e) => self.open_error = Some(e),
                                    Ok(path_str) => {
                                        let mut p = ff_command::CommandParams::new();
                                        p.insert("path", path_str.as_str());
                                        let _ = self.dispatch.execute_command("file.open", p);
                                    }
                                }
                            } else {
                                let mut p = ff_command::CommandParams::new();
                                p.insert("path", dsn.as_str());
                                let _ = self.dispatch.execute_command("file.open", p);
                            }
                        }
                        files_panel::FilesPanelAction::NavigateInto(_) => {}
                        files_panel::FilesPanelAction::None => {}
                    }
                }
                TabKind::FileEditor | TabKind::Untitled => {
                    let tab_id = self.tabs.active_tab().id;
                    let scroll_amount = self.scroll_amount.clone();
                    let tab = self.tabs.active_tab_mut();
                    if let Some(err) = editor_panel::render(
                        ui,
                        tab,
                        &self.runtime,
                        &mut self.cmd_engine,
                        &mut self.exclude_manager,
                        tab_id,
                        &scroll_amount,
                    ) {
                        self.open_error = Some(err);
                    }
                }
                TabKind::ConfigPanel => {
                    // Validates: Requirement 15.1-15.3; CR-NR-078 (framework).
                    // Migrated to the WorkspaceContext trait: owned-panel swap
                    // (Option A) so the single dispatch path reports the focus
                    // contract and honours the latch -- no inline ritual.
                    let mut panel = std::mem::take(&mut self.config_panel);
                    self.render_workspace_context(ctx, ui, &mut panel);
                    self.config_panel = panel;
                }
                TabKind::PluginManager => {
                    // Validates: plugin-manager-ui Requirement 1.1-1.6
                    crate::plugin_manager_panel::render(ui, &mut self.plugin_manager_panel);
                    // CR-CH-023 (B059): Filter field is the first/last interior.
                    let id = crate::plugin_manager_panel::filter_field_id();
                    self.first_interior_id = Some(id);
                    self.last_interior_id = Some(id);
                    self.honour_interior_focus_latch(ctx, Some(id), Some(id));
                }
                TabKind::EventLog => {
                    // Validates: notification-system Requirement 2.1-2.6
                    let first_interior_ev = crate::event_log_panel::render(
                        ui,
                        &mut self.event_log_panel,
                        &self.notification_queue,
                    );
                    if self.event_log_panel.clear_requested {
                        self.event_log_panel.clear_requested = false;
                        self.notification_queue.lock().expect("queue").clear();
                    }
                    // CR-CH-023 (B059): the level-filter combo is the first
                    // interior; its fresh id is returned by the render.
                    self.first_interior_id = first_interior_ev;
                    self.last_interior_id = first_interior_ev;
                    self.honour_interior_focus_latch(ctx, first_interior_ev, first_interior_ev);
                }
                TabKind::SearchResults => {
                    // Validates: global-search Requirement 1.1, 4.1
                    let roots = collect_search_roots(
                        &self.files_panel.registry,
                        self.active_workspace.as_ref(),
                    );
                    let outcome = crate::search_results_panel::render(
                        ui,
                        &mut self.search_results_panel,
                        &roots,
                        &self.runtime,
                    );
                    match outcome {
                        crate::search_results_panel::SearchPanelOutcome::OpenMatch {
                            path,
                            line,
                        } => {
                            if let Err(e) = self.shell_open_file(&path) {
                                self.open_error = Some(e);
                            } else {
                                // Scroll to the matching line.
                                let idx = self.tabs.active_index();
                                if let Some(tab) = self.tabs.tabs_mut().get_mut(idx) {
                                    tab.viewport.scroll_to_line(
                                        line.saturating_sub(1).max(1),
                                        &tab.cursor.clone(),
                                    );
                                }
                            }
                        }
                        crate::search_results_panel::SearchPanelOutcome::ReplaceAll => {
                            let unsaved: Vec<String> = self
                                .tabs
                                .tabs()
                                .iter()
                                .filter(|t| t.is_modified)
                                .filter_map(|t| t.path.clone())
                                .collect();
                            let req = self.search_results_panel.build_request(roots).ok();
                            if let Some(r) = req {
                                let results = self.search_results_panel.results.clone();
                                match ff_global_search::GlobalReplaceEngine::replace_all(
                                    &results,
                                    &r,
                                    &self.search_results_panel.replace_text.clone(),
                                    &unsaved,
                                ) {
                                    Ok((summary, _conflicts)) => {
                                        self.open_error = Some(format!(
                                            "Replaced {} occurrence(s) in {} file(s)",
                                            summary.replacements, summary.files_modified
                                        ));
                                    }
                                    Err(e) => {
                                        self.open_error = Some(format!("Replace failed: {e}"));
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    // CR-CH-023 (B059): the Search query field is the first/last interior.
                    let id = crate::search_results_panel::query_field_id();
                    self.first_interior_id = Some(id);
                    self.last_interior_id = Some(id);
                    self.honour_interior_focus_latch(ctx, Some(id), Some(id));
                }
                TabKind::FileExplorerPanel => {
                    // Unsplit: rendered as a full-window ctx-level panel by
                    // render_central_panel (the is_file_explorer branch), so this
                    // arm is skipped. Split (CR-NR-093 B071): the ctx-level branch
                    // is suppressed, so render the File Explorer HERE, inside this
                    // region's `ui`, keeping the split visible (Req 13.4/14.4).
                    if self.tabs.is_split() {
                        self.nav_explorer_seed();
                        let effects = self.render_nav_explorer_body(ui);
                        self.apply_nav_explorer_effects(ctx, effects);
                    }
                }
                TabKind::MacroLibrary => {
                    // Validates: lua-macro-engine Requirement 12.1-12.8
                    let action =
                        crate::macro_library_panel::render(ui, &mut self.macro_library_panel);
                    match action {
                        crate::macro_library_panel::MacroLibraryAction::Edit(path) => {
                            let mut p = ff_command::CommandParams::new();
                            p.insert("path", path.as_str());
                            let _ = self.dispatch.execute_command("file.open", p);
                        }
                        crate::macro_library_panel::MacroLibraryAction::Run(_path) => {
                            self.open_error = Some("Lua execution not yet available".to_string());
                        }
                        crate::macro_library_panel::MacroLibraryAction::Delete(path) => {
                            if let Err(e) = std::fs::remove_file(&path) {
                                self.open_error = Some(format!("Delete failed: {e}"));
                            } else {
                                let dirs = self.macro_dirs();
                                self.macro_library_panel.refresh(&dirs);
                                self.open_error = None;
                            }
                        }
                        crate::macro_library_panel::MacroLibraryAction::None => {}
                    }
                    // CR-CH-023 (B059): the Filter field is the first/last interior.
                    let id = crate::macro_library_panel::filter_field_id();
                    self.first_interior_id = Some(id);
                    self.last_interior_id = Some(id);
                    self.honour_interior_focus_latch(ctx, Some(id), Some(id));
                }
                TabKind::MenuWorkspace => {
                    // Validates: menu-workspace Requirement 2.1-2.6, 2.1a-2.1c,
                    // 18.2, 18.4. After CR-NR-082 Slice 1 the Home Context (POM)
                    // is a MenuWorkspace tab, so this single arm renders both the
                    // POM and every named menu via the SHARED menu renderer.
                    // ensure_pom_menu_loaded is a no-op unless the tab is Home
                    // (it seeds pom.toml / the barebones Recovery_Baseline).
                    self.ensure_pom_menu_loaded();
                    // Resolve calendar colours and month offset before borrowing
                    // the active tab mutably (menu_calendar_colours borrows &self).
                    let menu_cal = self.menu_colours();
                    let calendar_offset = self.pom_calendar_offset;
                    let active_idx = self.tabs.active_index();
                    let mut calendar_nav = None;
                    let mut first_interior = None;
                    let mut last_interior = None;
                    if let Some(mw) = self
                        .tabs
                        .tabs_mut()
                        .get_mut(active_idx)
                        .and_then(|t| t.menu_workspace.as_mut())
                    {
                        mw.poll_reload();
                        let result = crate::menu_workspace::render::render_menu_workspace(
                            mw,
                            ui,
                            calendar_offset,
                            menu_cal,
                        );
                        if let Some(option) = result.selected {
                            self.pending_menu_option = Some(option);
                        }
                        calendar_nav = result.calendar_nav;
                        first_interior = result.first_interior_id;
                        last_interior = result.last_interior_id;
                        // CR-CH-028: record the focused option for the
                        // Cursor_Context (Req 12.2).
                        self.focused_menu_option = result.focused_option;
                    }
                    // CR-NR-078: MenuWorkspace routes its focus contract
                    // through the shared framework helper (single latch path).
                    self.apply_interior_focus(
                        ctx,
                        crate::shell::workspace_context::InteriorFocus {
                            first: first_interior,
                            last: last_interior,
                        },
                    );
                    if let Some(nav) = calendar_nav {
                        match nav {
                            primary_option_menu::CalendarNav::Prev => self.pom_calendar_offset -= 1,
                            primary_option_menu::CalendarNav::Next => self.pom_calendar_offset += 1,
                        }
                    }
                }
                TabKind::ThemeEditor => {
                    // Validates: theme-and-appearance Req 20.1, 20.3-20.9;
                    // CR-NR-078 (framework). Owned-panel swap: render through
                    // the trait (reports first/last interior + honours the
                    // latch), then apply the stashed action.
                    let mut panel = std::mem::take(&mut self.theme_editor_panel);
                    self.render_workspace_context(ctx, ui, &mut panel);
                    let action = std::mem::take(&mut panel.pending_action);
                    self.theme_editor_panel = panel;
                    self.apply_theme_editor_action(action);
                }
                TabKind::MenusEditor => {
                    // Validates: menu-workspace Req 13.1-13.12 (CR-NR-075);
                    // CR-NR-078 (framework). Owned-panel swap: render through
                    // the trait, then apply the stashed action.
                    let mut panel = std::mem::take(&mut self.menus_editor_panel);
                    self.render_workspace_context(ctx, ui, &mut panel);
                    let action = std::mem::take(&mut panel.pending_action);
                    self.menus_editor_panel = panel;
                    self.apply_menus_editor_action(action);
                }
                TabKind::KeysEditor => {
                    // Validates: function-keys Req 22 (CR-CH-029); CR-NR-078
                    // (framework). Owned-panel swap: render through the trait,
                    // then apply the stashed action.
                    let mut panel = std::mem::take(&mut self.keys_editor_panel);
                    self.render_workspace_context(ctx, ui, &mut panel);
                    let action = std::mem::take(&mut panel.pending_action);
                    self.keys_editor_panel = panel;
                    self.apply_keys_editor_action(action);
                }
                TabKind::KindsEditor => {
                    // Validates: workspace-kinds Req 6 (CR-NR-090 B.4); CR-NR-078
                    // (framework). Owned-panel swap: render through the trait,
                    // then apply the stashed action (save/select/new).
                    let mut panel = std::mem::take(&mut self.kinds_editor_panel);
                    self.render_workspace_context(ctx, ui, &mut panel);
                    let action = std::mem::take(&mut panel.pending_action);
                    self.kinds_editor_panel = panel;
                    self.apply_kinds_editor_action(action);
                }
                TabKind::CommandConfigurator => {
                    // Validates: command-configurator Requirement 2.2-2.6
                    self.command_store.poll_reload();
                    let action = crate::command_config::render::render(
                        ui,
                        &mut self.command_configurator_panel,
                        &self.command_store,
                    );
                    self.apply_configurator_action(action);
                    // CR-CH-023 (B059): the "Add" button is the first interior
                    // (its fresh id captured by the render onto panel state).
                    let id = self.command_configurator_panel.first_interior_id;
                    self.first_interior_id = id;
                    self.last_interior_id = id;
                    self.honour_interior_focus_latch(ctx, id, id);
                }
            }
        }
    }

    /// Render the NavModel-backed File Explorer as the primary File Explorer
    /// Context content (CR-NR-060 Slice A). Seeds the Local Files root from the
    /// local/POSIX provider on first display, renders the modern tree in the
    /// CentralPanel, and applies the returned effects (expand -> async VFS list;
    /// collapse; open -> file.open dispatch, OS default app, or dataset resolve;
    /// copy path; reveal). This is the default explorer; the legacy inline tree
    /// remains available as a fallback when `use_legacy_explorer` is set.
    ///
    /// Validates: Requirement 24.1, 24.3, 24.5, 24.7, 24.9
    fn render_nav_explorer(&mut self, ctx: &egui::Context) {
        // CR-NR-093 B071: the File Explorer is now rendered in THREE parts --
        // seed (state), body (into a Ui), apply (effects + dialogs) -- so it can
        // render either as this full-window CentralPanel (unsplit) OR inside a
        // split region's Ui (via render_active_tab_body). This ctx-level path is
        // the unsplit case; behaviour is unchanged.
        self.nav_explorer_seed();
        let mut effects = Vec::new();
        egui::CentralPanel::default().show(ctx, |ui| {
            effects = self.render_nav_explorer_body(ui);
        });
        self.apply_nav_explorer_effects(ctx, effects);
    }

    /// Seed the modern File Explorer's NavModel on first display (CR-NR-093 B071
    /// split of `render_nav_explorer`): associate the Local Files root URI and
    /// load its children via the provider, and populate the Catalogs root from
    /// the registry. Idempotent -- a no-op once `children_loaded` is set.
    fn nav_explorer_seed(&mut self) {
        use crate::nav_model::list_via_provider;

        // Seed the Local Files root on first display: associate its URI and load
        // its immediate children via the provider (never std::fs).
        let local_root = self.nav_model.tree.root_categories[0];
        let needs_seed = self
            .nav_model
            .tree
            .get_node(local_root)
            .map(|n| !n.children_loaded)
            .unwrap_or(false);
        if needs_seed {
            let root_dir = dirs::home_dir()
                .or_else(|| std::env::current_dir().ok())
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            // Enter the Tokio runtime context so the provider's file-watcher
            // (which calls `tokio::spawn`) can start; constructing the provider
            // outside a runtime context panics ("no reactor running") -- B040.
            let provider = {
                let _rt_guard = self.runtime.enter();
                crate::posix_provider::PosixProvider::new(root_dir, true)
            };
            if let Ok(provider) = provider {
                let root_uri = ff_vfs::ResourceUri::new("posix", "/");
                self.nav_model.set_uri(local_root, root_uri.clone());
                match list_via_provider(&self.runtime, &provider, root_uri.path()) {
                    Ok(entries) => self.nav_model.apply_listing(local_root, "posix", &entries),
                    Err(e) => self.nav_model.apply_load_error(local_root, e),
                }
            }

            // Seed the Catalogs root generically: one CatalogRoot node per
            // registered catalog (Slice A -- no mainframe qualifier/dataset
            // duality; that is Slice B). Requirement 24.8.
            let catalogs_root = self.nav_model.tree.root_categories[1];
            for cat in self.files_panel.registry.list() {
                let uri = ff_vfs::ResourceUri::new("catalog", format!("/{}", cat.name));
                self.nav_model.add_child(
                    catalogs_root,
                    cat.name.clone(),
                    ff_file_tree::NodeType::CatalogRoot,
                    uri,
                );
            }
            if let Some(n) = self.nav_model.tree.get_node_mut(catalogs_root) {
                n.children_loaded = true;
            }
        }
    }

    /// Render the modern File Explorer tree into `ui` and return the interaction
    /// effects to apply (CR-NR-093 B071 split of `render_nav_explorer`). Used by
    /// both the full-window (unsplit) path and a split region's Ui.
    fn render_nav_explorer_body(
        &mut self,
        ui: &mut egui::Ui,
    ) -> Vec<crate::explorer_view::ExplorerEffect> {
        use crate::explorer_view::{keyboard_effects, render_tree};
        let mut effects = Vec::new();
        ui.label(egui::RichText::new("File Explorer").monospace().strong());
        ui.separator();
        // Keyboard navigation (Req 8/20) when the explorer is hovered/
        // focused, then mouse interactions from the tree.
        if ui.rect_contains_pointer(ui.max_rect()) {
            effects.extend(keyboard_effects(
                ui,
                &self.nav_model,
                &mut self.nav_selection,
            ));
        }
        effects.extend(render_tree(
            ui,
            &self.nav_model,
            &mut self.nav_selection,
            &self.palette,
        ));
        effects
    }

    /// Apply the File Explorer interaction effects and render its modal dialogs
    /// (CR-NR-093 B071 split of `render_nav_explorer`).
    fn apply_nav_explorer_effects(
        &mut self,
        ctx: &egui::Context,
        effects: Vec<crate::explorer_view::ExplorerEffect>,
    ) {
        use crate::explorer_view::{resolve_open, ExplorerEffect, OpenTarget};
        // Apply interaction effects outside the render borrow.
        for eff in effects {
            match eff {
                ExplorerEffect::Expand(id) => {
                    if let Some(uri) = self.nav_model.uri_of(id).cloned() {
                        match uri.scheme() {
                            // Catalogs root child: resolve the catalog by name and
                            // list its backing directory generically (Req 24.8 --
                            // no mainframe qualifier/dataset duality; that is
                            // Slice B). URI is vfs://catalog/{name}.
                            "catalog" => self.expand_catalog_node(id, &uri),
                            // Local Files (posix/local): root-jailed provider at the
                            // local root; URI path is provider-relative (Req 24.3).
                            _ => self.expand_local_node(id, &uri),
                        }
                    } else {
                        self.nav_model.tree.toggle_expand(id);
                    }
                }
                ExplorerEffect::Collapse(id) => self.nav_model.tree.toggle_expand(id),
                ExplorerEffect::Open(id) => match resolve_open(&self.nav_model, id) {
                    OpenTarget::Editor(uri) => {
                        // Resolve the provider-relative navigator URI to a real
                        // absolute host path before dispatching file.open, which
                        // reads through a default-rooted provider (B047).
                        match self.nav_open_path(&uri) {
                            Ok(host_path) => {
                                let mut p = ff_command::CommandParams::new();
                                p.insert("path", host_path.as_str());
                                let _ = self.dispatch.execute_command("file.open", p);
                            }
                            Err(e) => self.open_error = Some(e),
                        }
                    }
                    OpenTarget::External(uri) => {
                        // Launch the OS default app with the real host path.
                        match self.nav_open_path(&uri) {
                            Ok(host_path) => crate::context_menu::launch_default_app(&host_path),
                            Err(e) => self.open_error = Some(e),
                        }
                    }
                    OpenTarget::Dataset { catalog, dsn } => {
                        // Resolve the DSN to its physical file (create if
                        // missing) and open it in the editor -- same path the
                        // legacy panel uses. Req 16.1/16.3.
                        match open_mainframe_dsn(&self.files_panel.registry, &catalog, &dsn) {
                            Ok(path_str) => {
                                let mut p = ff_command::CommandParams::new();
                                p.insert("path", path_str.as_str());
                                let _ = self.dispatch.execute_command("file.open", p);
                            }
                            Err(e) => self.open_error = Some(e),
                        }
                    }
                    OpenTarget::None => {}
                },
                ExplorerEffect::CopyPath(id) => {
                    // Req 16 Copy Full Path: copy the node's resource path.
                    if let Some(uri) = self.nav_model.uri_of(id) {
                        if let Ok(mut cb) = arboard::Clipboard::new() {
                            let _ = cb.set_text(uri.path());
                        }
                    }
                }
                ExplorerEffect::CopySelection(text) => {
                    // Req 19.5/19.6: copy the indented text tree of the current
                    // multi-selection to the OS clipboard.
                    if let Ok(mut cb) = arboard::Clipboard::new() {
                        let _ = cb.set_text(&text);
                    }
                }
                ExplorerEffect::MarkCopy(id) => {
                    // Req 21.1: mark the selection (or this node) for a file copy.
                    // Record the source URIs of the selected local/POSIX files.
                    let ids: Vec<ff_file_tree::NodeId> = if self.nav_selection.selected.is_empty() {
                        vec![id]
                    } else {
                        self.nav_selection.selected.iter().copied().collect()
                    };
                    self.nav_file_clipboard = ids
                        .into_iter()
                        .filter_map(|n| self.nav_model.uri_of(n).cloned())
                        .filter(|u| u.scheme() == "posix" || u.scheme() == "local")
                        .collect();
                }
                ExplorerEffect::Paste(anchor) => {
                    self.apply_nav_paste(anchor);
                }
                ExplorerEffect::Rename(id) => {
                    // Open the rename dialog seeded with the node's current label
                    // (Req 16 Rename). The rename is applied on dialog confirm.
                    if let Some(label) = self.nav_model.tree.get_node(id).map(|n| n.label.clone()) {
                        self.nav_rename = Some((id, label));
                    }
                }
                ExplorerEffect::Delete(id) => {
                    // Open the delete-confirmation dialog (Req 16 Delete). The
                    // delete is applied only on explicit confirm.
                    if let Some(label) = self.nav_model.tree.get_node(id).map(|n| n.label.clone()) {
                        self.nav_delete = Some((id, label));
                    }
                }
                ExplorerEffect::NewChild { anchor, is_dir } => {
                    // Resolve the containing directory: the anchor itself if it
                    // is a directory, else the anchor's parent (Req 16 New).
                    if let Some(node) = self.nav_model.tree.get_node(anchor) {
                        let parent_dir = if node.node_type.is_expandable() {
                            anchor
                        } else {
                            node.parent
                        };
                        self.nav_new = Some((parent_dir, is_dir, String::new()));
                    }
                }
                ExplorerEffect::Reveal(id) => {
                    // Req 16 Reveal in Explorer: open the OS file manager at the
                    // node. Only local/POSIX nodes map to a real host path.
                    if let Some(uri) = self.nav_model.uri_of(id).cloned() {
                        if uri.scheme() == "posix" || uri.scheme() == "local" {
                            let root_dir = dirs::home_dir()
                                .or_else(|| std::env::current_dir().ok())
                                .unwrap_or_else(|| std::path::PathBuf::from("."));
                            let rel = uri.path().trim_start_matches('/');
                            let full = root_dir.join(rel);
                            crate::context_menu::reveal_in_explorer(&full.to_string_lossy());
                        }
                    }
                }
                ExplorerEffect::None => {}
            }
        }

        // Rename + Delete + New dialogs (Req 16) -- modal, applied on confirm.
        self.render_nav_rename_dialog(ctx);
        self.render_nav_delete_dialog(ctx);
        self.render_nav_new_dialog(ctx);
    }

    /// Render the modern-explorer new-file / new-folder dialog and create the
    /// child on confirm (local/POSIX only), then refresh the parent listing.
    ///
    /// Validates: Requirement 24.2 (file-tree-panel Req 16 New File / New Folder)
    fn render_nav_new_dialog(&mut self, ctx: &egui::Context) {
        let Some((parent, is_dir, mut buffer)) = self.nav_new.take() else {
            return;
        };
        let title = if is_dir { "New Folder" } else { "New File" };
        let mut still_open = true;
        let mut confirm = false;
        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label("Name:");
                let resp = ui.text_edit_singleline(&mut buffer);
                resp.request_focus();
                let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() || enter {
                        confirm = true;
                    }
                    if ui.button("Cancel").clicked() {
                        still_open = false;
                    }
                });
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    still_open = false;
                }
            });

        if confirm {
            self.apply_nav_new(parent, is_dir, buffer.trim());
            return;
        }
        if still_open {
            self.nav_new = Some((parent, is_dir, buffer));
        }
    }

    /// Create a new child (`is_dir` chooses folder vs file) named `name` under
    /// the `parent` directory node via a writable provider, then refresh the
    /// parent listing. Local/POSIX only.
    ///
    /// Validates: Requirement 24.2, 24.3 (Req 16 New File / New Folder)
    fn apply_nav_new(&mut self, parent: ff_file_tree::NodeId, is_dir: bool, name: &str) {
        use crate::nav_model::child_uri;
        use ff_vfs::{CreateOptions, VfsProvider};
        if name.is_empty() {
            return;
        }
        let Some(parent_uri) = self.nav_model.uri_of(parent).cloned() else {
            return;
        };
        let child = child_uri(&parent_uri, name);
        let (provider, child_path) = match self.nav_edit_provider(&child) {
            Ok(v) => v,
            Err(e) => {
                self.open_error = Some(format!("New file/folder: {e}"));
                return;
            }
        };
        let opts = CreateOptions {
            create_parents: false,
            is_directory: is_dir,
        };
        let result = self.runtime.block_on(provider.create(&child_path, opts));
        match result {
            Ok(()) => {
                // Ensure the parent is expanded, then refresh its listing.
                if let Some(n) = self.nav_model.tree.get_node_mut(parent) {
                    n.expanded = true;
                }
                self.refresh_nav_parent(parent);
            }
            Err(e) => self.open_error = Some(format!("Create failed: {e}")),
        }
    }

    /// Paste the file clipboard into the directory resolved from `anchor` (the
    /// anchor if a directory, else its parent). Each source file is read and
    /// written into the target directory via a writable provider, then the
    /// target listing is refreshed. Local/POSIX only (Req 21.2/21.3).
    ///
    /// Validates: Requirement 24.2 (file-tree-panel Req 21)
    fn apply_nav_paste(&mut self, anchor: ff_file_tree::NodeId) {
        use crate::explorer_view::paste_target;
        use crate::nav_model::child_uri;
        use ff_vfs::{CreateOptions, VfsProvider};
        if self.nav_file_clipboard.is_empty() {
            return;
        }
        let Some(target) = paste_target(&self.nav_model, anchor) else {
            return;
        };
        let Some(target_uri) = self.nav_model.uri_of(target).cloned() else {
            return;
        };
        // Resolve the writable target provider (Local Files or a POSIX/Native
        // catalog rooted at its backing path); rejects Mainframe/read-only.
        let (target_provider, _target_path) = match self.nav_edit_provider(&target_uri) {
            Ok(v) => v,
            Err(e) => {
                self.open_error = Some(format!("Paste: {e}"));
                return;
            }
        };
        let sources = self.nav_file_clipboard.clone();
        let mut errors = 0;
        for src in &sources {
            // Derive the destination file name from the source path's last segment.
            let name = src.path().rsplit('/').next().unwrap_or("");
            if name.is_empty() {
                continue;
            }
            // Destination URI inherits the target scheme; resolve both source and
            // destination to their (possibly different) providers + relative paths.
            let dest = child_uri(&target_uri, name);
            if dest.path() == src.path() && dest.scheme() == src.scheme() {
                continue; // no-op copy onto itself
            }
            let (src_provider, src_path) = match self.nav_edit_provider(src) {
                Ok(v) => v,
                Err(_) => {
                    errors += 1;
                    continue;
                }
            };
            let (_dp, dest_path) = match self.nav_edit_provider(&dest) {
                Ok(v) => v,
                Err(_) => {
                    errors += 1;
                    continue;
                }
            };
            let copy = self.runtime.block_on(async {
                let bytes = src_provider.read(&src_path).await?;
                target_provider
                    .create(
                        &dest_path,
                        CreateOptions {
                            create_parents: false,
                            is_directory: false,
                        },
                    )
                    .await?;
                target_provider.write(&dest_path, &bytes).await
            });
            if copy.is_err() {
                errors += 1;
            }
        }
        if errors > 0 {
            self.open_error = Some(format!("Paste: {errors} file(s) could not be copied"));
        }
        // Refresh the target directory listing so the pasted files appear.
        if let Some(n) = self.nav_model.tree.get_node_mut(target) {
            n.expanded = true;
        }
        self.refresh_nav_parent(target);
    }

    /// Render the modern-explorer delete-confirmation dialog and apply the delete
    /// on confirm. Local/POSIX nodes only; directories delete recursively.
    ///
    /// Validates: Requirement 24.2 (file-tree-panel Req 16 Delete)
    fn render_nav_delete_dialog(&mut self, ctx: &egui::Context) {
        let Some((id, label)) = self.nav_delete.take() else {
            return;
        };
        let mut decision: Option<bool> = None; // Some(true)=delete, Some(false)=cancel
        egui::Window::new("Delete")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label(format!("Delete '{label}'? This cannot be undone."));
                ui.horizontal(|ui| {
                    if ui.button("Delete").clicked() {
                        decision = Some(true);
                    }
                    if ui.button("Cancel").clicked() {
                        decision = Some(false);
                    }
                });
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    decision = Some(false);
                }
            });

        match decision {
            Some(true) => self.apply_nav_delete(id),
            Some(false) => {} // cancelled -- dialog already taken (closed)
            None => self.nav_delete = Some((id, label)), // keep open
        }
    }

    /// Apply a delete of node `id` (local/POSIX only) via a writable provider,
    /// then refresh the parent listing. Directories are deleted recursively.
    ///
    /// Validates: Requirement 24.2, 24.3 (Req 16 Delete)
    fn apply_nav_delete(&mut self, id: ff_file_tree::NodeId) {
        use ff_vfs::{DeleteOptions, VfsProvider};
        let Some(uri) = self.nav_model.uri_of(id).cloned() else {
            return;
        };
        let recursive = self
            .nav_model
            .tree
            .get_node(id)
            .map(|n| n.node_type.is_expandable())
            .unwrap_or(false);
        let parent = self.nav_model.tree.get_node(id).map(|n| n.parent);
        let (provider, path) = match self.nav_edit_provider(&uri) {
            Ok(v) => v,
            Err(e) => {
                self.open_error = Some(format!("Delete: {e}"));
                return;
            }
        };
        let result = self
            .runtime
            .block_on(provider.delete(&path, DeleteOptions { recursive }));
        match result {
            Ok(()) => {
                if let Some(parent) = parent {
                    self.refresh_nav_parent(parent);
                }
            }
            Err(e) => self.open_error = Some(format!("Delete failed: {e}")),
        }
    }

    /// Render the modern-explorer rename dialog when a rename is in progress and
    /// apply the rename on confirm. Local/POSIX nodes are renamed via a writable
    /// provider then the parent listing is refreshed. Non-local schemes (catalog
    /// roots, datasets) are not renamable here in Slice A.
    ///
    /// Validates: Requirement 24.2 (file-tree-panel Req 16 Rename)
    fn render_nav_rename_dialog(&mut self, ctx: &egui::Context) {
        let Some((id, mut buffer)) = self.nav_rename.take() else {
            return;
        };
        let mut still_open = true;
        let mut confirm = false;
        egui::Window::new("Rename")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.label("New name:");
                let resp = ui.text_edit_singleline(&mut buffer);
                resp.request_focus();
                let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                ui.horizontal(|ui| {
                    if ui.button("Rename").clicked() || enter {
                        confirm = true;
                    }
                    if ui.button("Cancel").clicked() {
                        still_open = false;
                    }
                });
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    still_open = false;
                }
            });

        if confirm {
            self.apply_nav_rename(id, buffer.trim());
            return; // dialog closed
        }
        if still_open {
            self.nav_rename = Some((id, buffer));
        }
    }

    /// Apply a rename of node `id` to `new_name` (local/POSIX only), then refresh
    /// the parent listing so the model reflects the new name.
    ///
    /// Validates: Requirement 24.2, 24.3 (Req 16 Rename)
    fn apply_nav_rename(&mut self, id: ff_file_tree::NodeId, new_name: &str) {
        use crate::nav_model::rename_uri;
        use ff_vfs::VfsProvider;
        if new_name.is_empty() {
            return;
        }
        let Some(uri) = self.nav_model.uri_of(id).cloned() else {
            return;
        };
        // `rename_uri` preserves scheme + parent, so the new URI resolves to the
        // same provider; take its provider-relative path for the destination.
        let new_uri = rename_uri(&uri, new_name);
        let (provider, old_path) = match self.nav_edit_provider(&uri) {
            Ok(v) => v,
            Err(e) => {
                self.open_error = Some(format!("Rename: {e}"));
                return;
            }
        };
        let new_path = match self.nav_edit_provider(&new_uri) {
            Ok((_, p)) => p,
            Err(e) => {
                self.open_error = Some(format!("Rename: {e}"));
                return;
            }
        };
        let result = self.runtime.block_on(provider.rename(&old_path, &new_path));
        match result {
            Ok(()) => {
                let parent = self.nav_model.tree.get_node(id).map(|n| n.parent);
                if let Some(parent) = parent {
                    self.refresh_nav_parent(parent);
                }
            }
            Err(e) => self.open_error = Some(format!("Rename failed: {e}")),
        }
    }

    /// Resolve a node URI to a writable provider plus the provider-relative path
    /// for an edit operation (delete/rename/new/paste). Supports the Local Files
    /// subtree (`posix`/`local`, rooted at home) and POSIX/Native catalog
    /// subtrees (`catalog`, rooted at the catalog's backing path). Mainframe
    /// dataset nodes (`dataset`) and read-only catalogs are rejected with a
    /// message. (B042.)
    ///
    /// Validates: Requirement 24.6 (file-tree-panel Req 16.10-16.13)
    fn nav_edit_provider(
        &self,
        uri: &ff_vfs::ResourceUri,
    ) -> Result<(crate::posix_provider::PosixProvider, String), String> {
        use crate::catalog_registry::CatalogType;
        use crate::nav_model::split_catalog_uri_path;
        let (root_dir, rel_path) = match uri.scheme() {
            "posix" | "local" => {
                let home = dirs::home_dir()
                    .or_else(|| std::env::current_dir().ok())
                    .unwrap_or_else(|| std::path::PathBuf::from("."));
                (home, uri.path().to_string())
            }
            "catalog" => {
                let (name, sub_path) = split_catalog_uri_path(uri.path());
                let cat = self
                    .files_panel
                    .registry
                    .get_by_name(name)
                    .ok_or_else(|| format!("Catalog '{name}' not found"))?;
                match cat.catalog_type {
                    CatalogType::Posix | CatalogType::Native => {}
                    CatalogType::Mainframe => {
                        return Err("Editing Mainframe datasets is available in a later update."
                            .to_string());
                    }
                }
                if cat.read_only {
                    return Err(format!("Catalog '{name}' is read-only."));
                }
                (std::path::PathBuf::from(&cat.path), sub_path)
            }
            "dataset" => {
                return Err("Editing Mainframe datasets is available in a later update.".to_string())
            }
            other => return Err(format!("Editing is not supported for '{other}' resources.")),
        };
        // Writable provider; enter the runtime so the watcher can spawn (B040).
        let provider = {
            let _rt_guard = self.runtime.enter();
            crate::posix_provider::PosixProvider::new(root_dir, false)
        };
        provider
            .map(|p| (p, rel_path))
            .map_err(|_| "cannot open provider".to_string())
    }

    /// Resolve a navigator node URI to a real absolute host filesystem path so
    /// it can be opened via `file.open` (which reads through a default-rooted
    /// `LocalFsProvider`, i.e. it expects a real path, not a provider-relative
    /// one).
    ///
    /// The modern explorer seeds Local Files through a `posix` provider jailed
    /// at the home directory, so a node's `uri.path()` (e.g.
    /// `/OneDrive - Standard Bank/Clipbook.md`) is relative to that jail root,
    /// NOT an absolute path. Passing it straight to `file.open` produced
    /// "resource not found" (B047). This joins the correct root (home for
    /// posix/local; the catalog `path` for catalog subtrees) with the
    /// provider-relative path, using the host path separator.
    ///
    /// Validates: file-tree-panel Requirement 24.9 (open resolves to the real
    /// file); B047.
    pub(super) fn nav_open_path(&self, uri: &ff_vfs::ResourceUri) -> Result<String, String> {
        use crate::nav_model::split_catalog_uri_path;
        let (root_dir, rel_path) = match uri.scheme() {
            "posix" | "local" => {
                let home = dirs::home_dir()
                    .or_else(|| std::env::current_dir().ok())
                    .unwrap_or_else(|| std::path::PathBuf::from("."));
                (home, uri.path().to_string())
            }
            "catalog" => {
                let (name, sub_path) = split_catalog_uri_path(uri.path());
                let cat = self
                    .files_panel
                    .registry
                    .get_by_name(name)
                    .ok_or_else(|| format!("Catalog '{name}' not found"))?;
                (std::path::PathBuf::from(&cat.path), sub_path)
            }
            other => {
                return Err(format!(
                    "Cannot resolve a host path for '{other}' resources."
                ))
            }
        };
        // Join the root with each forward-slash segment of the provider-relative
        // path, using the host separator. Reject `..` traversal out of the jail.
        let mut resolved = root_dir;
        for segment in rel_path.trim_start_matches('/').split('/') {
            match segment {
                "" | "." => {}
                ".." => return Err("Invalid path (parent traversal)".to_string()),
                s => resolved.push(s),
            }
        }
        Ok(resolved.to_string_lossy().into_owned())
    }

    /// Refresh a parent node's listing after an edit, dispatching on its URI
    /// scheme (catalog subtrees re-list via the catalog path). (B042.)
    fn refresh_nav_parent(&mut self, parent: ff_file_tree::NodeId) {
        if let Some(puri) = self.nav_model.uri_of(parent).cloned() {
            if puri.scheme() == "catalog" {
                self.expand_catalog_node(parent, &puri);
            } else {
                self.expand_local_node(parent, &puri);
            }
        }
        self.nav_model.prune_uris();
    }

    /// Expand a Local Files (posix/local) node: list its provider-relative path
    /// through a root-jailed provider rooted at the local root.
    ///
    /// Validates: Requirement 24.3, 24.5
    fn expand_local_node(&mut self, id: ff_file_tree::NodeId, uri: &ff_vfs::ResourceUri) {
        use crate::nav_model::list_via_provider;
        let root_dir = dirs::home_dir()
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        // Enter the runtime context so the provider's watcher can spawn
        // (constructing outside a runtime panics -- B040).
        let provider = {
            let _rt_guard = self.runtime.enter();
            crate::posix_provider::PosixProvider::new(root_dir, true)
        };
        if let Ok(provider) = provider {
            match list_via_provider(&self.runtime, &provider, uri.path()) {
                Ok(entries) => self.nav_model.apply_listing(id, "posix", &entries),
                Err(e) => self.nav_model.apply_load_error(id, e),
            }
        }
    }

    /// Expand a Catalogs root child generically (Req 24.8): resolve the catalog
    /// by name (URI `vfs://catalog/{name}`) and list its backing directory
    /// through a provider rooted at the catalog `path`. No mainframe qualifier/
    /// dataset duality is applied here -- that is Slice B. If the catalog is
    /// unknown or its directory cannot be listed, an error node is shown.
    ///
    /// Validates: Requirement 24.8, 24.5
    fn expand_catalog_node(&mut self, id: ff_file_tree::NodeId, uri: &ff_vfs::ResourceUri) {
        use crate::catalog_registry::CatalogType;
        use crate::nav_model::split_catalog_uri_path;
        // URI path is "/{name}[/{subpath...}]": the first segment is the catalog
        // name, the remainder is a directory path relative to the catalog root.
        let (name, sub_path) = split_catalog_uri_path(uri.path());
        let resolved = self.files_panel.registry.get_by_name(name).map(|c| {
            (
                c.catalog_type,
                std::path::PathBuf::from(&c.path),
                c.read_only,
            )
        });
        let (catalog_type, root_dir, read_only) = match resolved {
            Some(v) => v,
            None => {
                self.nav_model
                    .apply_load_error(id, format!("Catalog '{name}' not found"));
                return;
            }
        };
        match catalog_type {
            // Mainframe catalog root: list the datasets registered in SQLite,
            // not the repository's internal directory layout. Members/qualifier
            // duality is Slice B; a dataset node is a leaf (or shows a Slice B
            // placeholder if expanded).
            CatalogType::Mainframe if sub_path == "/" => self.list_catalog_datasets(id, name),
            CatalogType::Mainframe => {
                // A dataset node under a Mainframe catalog was expanded: member
                // navigation is deferred to Slice B.
                self.nav_model.apply_load_error(
                    id,
                    "Dataset member browsing is available in a later update.".to_string(),
                );
            }
            // POSIX / Native catalog: a real host directory -- list it generically.
            CatalogType::Posix | CatalogType::Native => {
                self.list_catalog_directory(id, root_dir, read_only, &sub_path)
            }
        }
    }

    /// List the datasets of a Mainframe catalog (from SQLite) as tree nodes.
    /// PS -> sequential (leaf), PO -> partitioned, GDG -> GDG base. Dataset node
    /// URIs use the `dataset` scheme (`vfs://dataset/{catalog}/{DSN}`) so open/
    /// expand routing can distinguish them from directories.
    ///
    /// Validates: Requirement 24.8
    fn list_catalog_datasets(&mut self, id: ff_file_tree::NodeId, catalog: &str) {
        use crate::catalog_registry::dataset_node;
        match self.files_panel.registry.list_datasets(catalog) {
            Ok(records) => {
                let children = records
                    .iter()
                    .map(|r| dataset_node(catalog, r))
                    .collect::<Vec<_>>();
                self.nav_model.apply_child_data(id, children);
            }
            Err(e) => self.nav_model.apply_load_error(id, e.to_string()),
        }
    }

    /// List a POSIX/Native catalog's backing directory generically (Req 24.8).
    ///
    /// Validates: Requirement 24.8, 24.5
    fn list_catalog_directory(
        &mut self,
        id: ff_file_tree::NodeId,
        root_dir: std::path::PathBuf,
        read_only: bool,
        sub_path: &str,
    ) {
        use crate::nav_model::list_via_provider;
        // Enter the runtime context so the provider's watcher can spawn (B040).
        let provider = {
            let _rt_guard = self.runtime.enter();
            crate::posix_provider::PosixProvider::new(root_dir, read_only)
        };
        match provider {
            Ok(provider) => match list_via_provider(&self.runtime, &provider, sub_path) {
                Ok(entries) => self.nav_model.apply_listing(id, "posix", &entries),
                Err(e) => self.nav_model.apply_load_error(id, e),
            },
            Err(e) => self.nav_model.apply_load_error(id, e.to_string()),
        }
    }
}

// === Helpers ================================================================

/// Draw a 2 px focus ring around `rect` using the theme's `focus_ring` colour.
///
/// Retained for accessibility Requirement 3.1/3.3/3.4 (focus indicator helper).
/// Under CR-CH-023 the shell no longer drives a per-widget focus ring
/// (egui draws its own focus highlight for the focused Interior_Control and
/// Menu_Bar item), so this helper currently has no production caller; kept for
/// the accessibility API contract and potential future custom indicators.
#[allow(dead_code)]
pub(crate) fn render_focus_indicator(
    ui: &egui::Ui,
    rect: egui::Rect,
    palette: &ff_theme::ThemePalette,
) {
    use ff_theme::ColourToken;
    let colour = to_egui_color(palette.colour(ColourToken::UiFocusRing));
    ui.painter().rect_stroke(
        rect.expand(2.0),
        2.0,
        egui::Stroke::new(2.0_f32, colour),
        egui::StrokeKind::Outside,
    );
}

/// Collect search root paths from the active workspace or mounted Native catalogs.
///
/// Validates: global-search Requirement 2.4
fn collect_search_roots(
    registry: &crate::catalog_registry::CatalogRegistry,
    workspace: Option<&ff_session::WorkspaceState>,
) -> Vec<String> {
    if let Some(ws) = workspace {
        if !ws.roots.is_empty() {
            return ws
                .roots
                .iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect();
        }
    }
    registry
        .list_by_type(crate::catalog_registry::CatalogType::Native)
        .iter()
        .map(|c| c.path.clone())
        .collect()
}

/// Resolve a Mainframe DSN to a physical path via the ff-desktop CatalogRegistry,
/// creating the file on disk if it does not yet exist.
///
/// Returns the path as a `String` on success, or a human-readable error.
///
/// Validates: Requirement 16.1, 16.3, 16.4
fn open_mainframe_dsn(
    registry: &crate::catalog_registry::CatalogRegistry,
    catalog_name: &str,
    dsn: &str,
) -> Result<String, String> {
    let parsed = ff_dscatalog::dsn::Dsn::parse(dsn)
        .map_err(|_| format!("'{}': invalid dataset name", dsn))?;
    let path = registry
        .resolve_dsn(catalog_name, &parsed)
        .map_err(|_| format!("'{}': dataset not found in catalog '{}'", dsn, catalog_name))?;
    if !path.exists() {
        crate::files_panel::FilesPanelState::create_dataset_file(&path).map_err(|e| {
            format!(
                "'{}': cannot create dataset file at {}: {}",
                dsn,
                path.display(),
                e
            )
        })?;
    }
    Ok(path.to_string_lossy().into_owned())
}
