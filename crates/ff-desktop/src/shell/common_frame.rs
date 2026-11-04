//! # Shared common-frame chrome helper (CR-NR-115 Phase 2H, Req 30)
//!
//! The SINGLE place the SNAPSHOT and CAPTURE seam composes the eight common
//! workspace-frame elements onto a per-Context [`ScreenModel`] from LIVE logical
//! state (never egui). It is called ONCE from
//! [`super::WorkbenchShell::active_screen_capture`] after each per-Context model
//! is built, so SNAPSHOT and CAPTURE reproduce the SAME frame.
//!
//! ORDER: this helper only fills the model's chrome SLOTS (`header_lines`,
//! `command_line`, `messages`, `status_bar`, `footer_lines`). The top-to-bottom
//! placement INSIDE one box (title bar, menu bar, tab bar, centered title,
//! Command line, content, message, status, key line) is owned by the
//! `ff-screen-model` framed renderer (`render/framed.rs`), selected whenever
//! `header_lines` is non-empty. The legacy slot order emitted the chrome after
//! the content and left the Command line and status bar outside the box.
//!
//! The eight elements (Req 30.1): the window title bar, the menu bar, the FULL
//! tab bar (every open tab's `[XXX]` Kind tag + an ASCII close marker), the
//! centered title (already carried in [`ScreenModel::title`]), the `Command ===>`
//! line (set HERE for every Context, empty field), the Message_Line when present
//! (omitted when empty), the common + per-Kind Status_Bar (NO RUNNING), and the
//! pfshow key bar (only when visible, the LAST band).
//!
//! The Menu Kinds (POM, Settings, `menus/<name>.toml`) go through this SAME path:
//! the menu builder returns the CONTENT AREA only, so there is no menu-specific
//! branch and no folded command row or PF footer (Req 30.27).
//!
//! Validates: screen-snapshot-scrm Requirement 30.1-30.11, 30.15, 30.16, 30.17,
//! 30.27, 30.28.

use ff_screen_model::ScreenModel;

use super::WorkbenchShell;

/// The ASCII window-control glyphs shown at the right of the window title bar
/// (documentation-charset: ASCII only -- a minimise underscore, a maximise `[]`,
/// and a close `X`).
const WINDOW_CONTROLS: &str = "_ [] X";

/// The ASCII close marker appended to each tab's `[XXX]` tag in the full tab bar.
const TAB_CLOSE_MARKER: &str = "x";

/// The leading title-bar cell (a dedicated `#` cell before the app title).
const TITLE_BAR_LEAD_CELL: &str = "#";

/// OWNER DEFAULT 1 (CR-NR-115 Req 30.24): the markers wrapped around the ACTIVE
/// tab's cell (`>[TAG] x<`). Set both to `""` to drop the active-tab marker.
const ACTIVE_TAB_MARKERS: (&str, &str) = (">", "<");

/// The cell separator the framed renderer splits bands on (menu bar, tab bar,
/// status bar). Matches `ff-screen-model`'s band-cell convention.
const CELL_SEPARATOR: &str = " | ";

impl WorkbenchShell {
    /// Compose the eight common workspace-frame elements onto `model` from the
    /// shell's LIVE logical state (CR-NR-115 Phase 2H, Req 30). Called once by
    /// `active_screen_capture`, so SNAPSHOT and CAPTURE share the one frame.
    ///
    /// `header_lines` becomes `[window title bar, menu-bar echo, FULL tab bar]`,
    /// prepended before any header chrome the Context already carried. The shared
    /// and per-Kind Status_Bar text is set (no RUNNING). The Message_Line entries
    /// are added above the status bar (omitted when empty). The pfshow key-bar
    /// line is appended to `footer_lines` when the Key_Label_Bar is visible, for
    /// every Context (Req 30.11, 30.27).
    ///
    /// The centered title is left in `model.title`; the `Command ===>` field is
    /// set here for every Context.
    ///
    /// Validates: screen-snapshot-scrm Requirement 30.1-30.11, 30.15, 30.16, 30.27.
    pub(super) fn apply_common_frame(&self, model: &mut ScreenModel) {
        // Element 1-3: window title bar, menu-bar echo, FULL tab bar. Prepend so
        // the common chrome sits ABOVE any Context-specific header lines.
        let mut header = vec![
            self.common_frame_window_title_line(),
            self.common_frame_menu_bar_line(),
            self.common_frame_tab_bar_line(),
        ];
        header.append(&mut model.header_lines);
        model.header_lines = header;

        // Element 7: common + per-Kind Status_Bar (no RUNNING).
        model.status_bar.text = self.common_frame_status_bar_text();

        // Element 6: the Message_Line, above the status bar, omitted when empty.
        // Read through the pre-action seam so a capture shows the band as it was
        // when the triggering command was issued (Req 7.29, 30.28).
        for message in self.frame_message_lines() {
            model.messages.push(message.clone());
        }

        // Element 5: the Command line, for EVERY Context including the Menu
        // Kinds (Req 30.6, 30.27): the empty field, underscores to the border.
        model.command_line =
            crate::screen_snapshot::command_field_value(&self.common_frame_command_field_text());

        // Element 8: the pfshow key bar, the LAST band, only when visible, for
        // EVERY Context including the Menu Kinds (Req 30.11, 30.27).
        if self.key_bar_visible {
            model
                .footer_lines
                .push(crate::screen_snapshot::key_bar_line(
                    self.key_map_resolver.active_key_map(),
                    self.key_bar_scope.segment_label(),
                    self.key_bar_scope.to_modifier(),
                ));
        }
    }

    /// OWNER DEFAULT 2 (CR-NR-115 Req 30.24(b)): the text the snapshot's Command
    /// line shows -- the field as it was BEFORE the triggering command, read
    /// through the pre-action stash. A typed command line clears it (the field
    /// held the triggering command, e.g. `ss`); a key-forward action keeps the
    /// text the user typed first. An empty field renders as underscores only.
    fn common_frame_command_field_text(&self) -> String {
        self.frame_command_text()
    }

    /// The window title bar band: a dedicated `#` cell, the ONE shared
    /// `FileForge Workbench` title constant, and the ASCII window controls
    /// RIGHT-ALIGNED against the border via the renderer's marker (Req 30.2,
    /// 30.19).
    fn common_frame_window_title_line(&self) -> String {
        format!(
            "{TITLE_BAR_LEAD_CELL}{CELL_SEPARATOR}{}{}{}",
            crate::screen_snapshot::WINDOW_TITLE,
            ff_screen_model::RIGHT_ALIGN,
            WINDOW_CONTROLS
        )
    }

    /// The menu-bar echo line: the bar-visible options of the active POM menu,
    /// labelled by their command verb and joined by ` | ` -- the SAME options the
    /// live menu bar echoes (Req 30.3).
    fn common_frame_menu_bar_line(&self) -> String {
        let menu = self.resolve_menu_bar_menu();
        menu.options
            .iter()
            .filter(|o| o.show_in_menu_bar)
            .map(|o| o.command.clone())
            .collect::<Vec<_>>()
            .join(" | ")
    }

    /// The FULL tab bar line: EVERY open tab's `[XXX]` Kind tag (not just the
    /// active one) followed by an ASCII close marker, joined by a space (Req
    /// 30.4). The active tab's tag is wrapped in a `>...<` focus marker so the
    /// capture shows which tab is current without dropping the others.
    fn common_frame_tab_bar_line(&self) -> String {
        let active = self.tabs.active_index();
        self.tabs
            .tabs()
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                let tag =
                    crate::workspace_kind::BuiltinKind::from_tab_kind(tab.kind.tag(), tab.is_home)
                        .default_title();
                if i == active {
                    let (open, close) = ACTIVE_TAB_MARKERS;
                    format!("{open}{tag} {TAB_CLOSE_MARKER}{close}")
                } else {
                    format!("{tag} {TAB_CLOSE_MARKER}")
                }
            })
            .collect::<Vec<_>>()
            .join(CELL_SEPARATOR)
    }

    /// The Status_Bar text (Req 30.10, revised 2026-11-04): `Last: HH:MM |
    /// Prof: <name> | Zoom: <n>` plus the active Kind's Workspace_Status_Segment
    /// cells, each its own cell. Last, Prof and Zoom are the ONLY shell-common
    /// segments. Ln/Col, encoding and line count are workspace-owned (the editor
    /// Context returns them), so Config and Alias show only the three common
    /// segments until their per-Kind gates. There is NO RUNNING segment. Values
    /// come from the SAME state the live bar reads (`format_zoom_label`,
    /// `active_profile_label`, `workspace_status_cells`), so the captured band
    /// equals the live bar. `Last:` reads the pre-action stamp while an action is
    /// in flight (Req 7.29).
    fn common_frame_status_bar_text(&self) -> String {
        let mut segments = vec![
            Self::last_action_label(self.frame_last_action_at()),
            self.active_profile_label(),
            self.format_zoom_label(),
        ];
        segments.extend(self.workspace_status_cells());
        segments.join(CELL_SEPARATOR)
    }
}
