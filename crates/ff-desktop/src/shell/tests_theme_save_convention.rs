//! Theme Editor ISPF-style explicit-save convention tests (B081,
//! project-master Requirement 11).
//!
//! These drive the SHARED SAVE / END / RETURN command seam
//! (`try_commands_a` -> `editable_context_kind` -> `editable_persist` /
//! `editable_discard` / `editable_reload`) against the Theme Editor Context,
//! proving the net behaviour the convention requires:
//!   - typed SAVE persists the working theme and STAYS open (11.3);
//!   - END persists then EXITS, and STAYS open reporting an error when persist
//!     fails (11.4);
//!   - RETURN discards edits, reverts the live-preview palette to the persisted
//!     theme, and EXITS (11.5, 11.6);
//!   - a bare colour edit does NOT persist (11.2);
//!   - the in-panel Save button and the typed SAVE command share ONE persist
//!     path (11.8).

#![allow(unused_imports)]
use super::tests_common::*;
use crate::tab_state::KindTag;
use crate::theme_editor_panel::{EditableToken, ThemeEditorAction};
use ff_theme::{ColourRGBA, ThemePalette};

/// Resolve a derived editable token by its (English) label (mirrors the helper
/// in `tests_menu_workspace.rs`; the derived surface replaced the fixed enum in
/// CR-CH-056).
fn token_by_label(label: &str) -> EditableToken {
    EditableToken::all()
        .iter()
        .copied()
        .find(|t| t.label() == label)
        .unwrap_or_else(|| panic!("no editable token with label '{label}'"))
}

/// Open the Theme Editor on a fresh USER theme (so Save writes to its own file,
/// not a built-in Save As redirect). Returns the shell, the temp dir guard, and
/// the user theme name.
fn open_on_user_theme() -> (super::WorkbenchShell, tempfile::TempDir, String) {
    let mut shell = make_shell();
    let dir = point_themes_at_temp(&mut shell);
    shell.dispatch_command_string("THEME");
    // Copy the active (built-in) theme to a user theme and make it the target.
    shell.apply_theme_editor_action(ThemeEditorAction::Copy("user-theme".to_string()));
    assert_eq!(
        shell.theme_editor_panel.selected.as_deref(),
        Some("user-theme"),
        "setup: Copy must select the new user theme"
    );
    (shell, dir, "user-theme".to_string())
}

/// Read the persisted panel_bg colour of a user theme file from the themes dir.
fn persisted_panel_bg(shell: &super::WorkbenchShell, name: &str) -> ColourRGBA {
    let themes = shell.dir_overrides.themes.clone().unwrap();
    let path = themes.join(format!("{}.toml", crate::theme_defaults::theme_slug(name)));
    let toml = std::fs::read_to_string(&path).expect("read persisted theme file");
    let reloaded: ThemePalette =
        ff_theme::loader::load_from_toml(&toml, ff_theme::mode::VisualMode::Dark).expect("parse");
    reloaded.ui.panel_bg
}

// Validates: project-master Requirement 11.3 -- the typed SAVE command persists
// the working theme to its backing file AND leaves the Context open.
#[test]
fn theme_editor_save_command_persists_and_stays_open() {
    let (mut shell, _dir, name) = open_on_user_theme();
    let red = ColourRGBA::rgb(255, 0, 0);
    shell.apply_theme_editor_action(ThemeEditorAction::EditToken(
        token_by_label("Chrome: window fill"),
        red,
    ));
    shell.dispatch_command_string("SAVE");
    assert_eq!(
        persisted_panel_bg(&shell, &name),
        red,
        "typed SAVE must persist the edited colour to the backing file"
    );
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::ThemeEditor,
        "SAVE must leave the Theme Editor Context OPEN"
    );
    assert!(
        shell.theme_editor_panel.error.is_none(),
        "a successful SAVE clears any inline error"
    );
}

// Validates: project-master Requirement 11.4 -- END persists the working theme
// AND THEN exits the Context (END implies SAVE).
#[test]
fn theme_editor_end_persists_then_exits() {
    let (mut shell, _dir, name) = open_on_user_theme();
    let green = ColourRGBA::rgb(0, 255, 0);
    shell.apply_theme_editor_action(ThemeEditorAction::EditToken(
        token_by_label("Chrome: window fill"),
        green,
    ));
    shell.dispatch_command_string("END");
    assert_eq!(
        persisted_panel_bg(&shell, &name),
        green,
        "END must persist the edited colour (END implies SAVE)"
    );
    assert_ne!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::ThemeEditor,
        "END must EXIT the Theme Editor Context after persisting"
    );
}

// Validates: project-master Requirement 11.4 -- when persistence FAILS, END
// keeps the Context OPEN and reports the error rather than exiting.
#[test]
fn theme_editor_end_on_persist_error_stays_open() {
    let mut shell = make_shell();
    let _dir = point_themes_at_temp(&mut shell);
    shell.dispatch_command_string("THEME");
    // A built-in is selected on open; clearing the pre-filled New-name buffer
    // forces the built-in Save redirect to FAIL (no name to Save As under),
    // which is the persist-failure path.
    assert!(
        ff_theme::is_builtin_theme(&shell.theme_editor_panel.selected.clone().unwrap()),
        "setup: the editor opens on a built-in theme"
    );
    shell.theme_editor_panel.name_buffer.clear();
    shell.dispatch_command_string("END");
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::ThemeEditor,
        "END must STAY in the Context when persistence fails"
    );
    assert!(
        shell.theme_editor_panel.error.is_some(),
        "a failed END persist must surface the error inline"
    );
}

// Validates: project-master Requirement 11.5, 11.6 -- RETURN discards the
// working edits WITHOUT persisting, reverts the live-preview palette to the
// persisted theme, and exits; the backing file is left as it was on entry.
#[test]
fn theme_editor_return_discards_reverts_preview_and_exits() {
    let (mut shell, _dir, name) = open_on_user_theme();
    let persisted = persisted_panel_bg(&shell, &name);
    let magenta = ColourRGBA::rgb(255, 0, 255);
    assert_ne!(persisted, magenta, "setup: edit must change the colour");
    // Edit a token: the live preview mutates the RUNNING palette.
    shell.apply_theme_editor_action(ThemeEditorAction::EditToken(
        token_by_label("Chrome: window fill"),
        magenta,
    ));
    assert_eq!(
        shell.palette.ui.panel_bg, magenta,
        "setup: the live preview applied the edit to the running palette"
    );
    shell.dispatch_command_string("RETURN");
    // The preview is reverted to the persisted theme (11.6).
    assert_eq!(
        shell.palette.ui.panel_bg, persisted,
        "RETURN must revert the live-preview palette to the persisted theme"
    );
    // The backing file is unchanged (11.5 -- no persist on RETURN).
    assert_eq!(
        persisted_panel_bg(&shell, &name),
        persisted,
        "RETURN must leave the backing file exactly as it was on entry"
    );
    // The Context has exited.
    assert_ne!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::ThemeEditor,
        "RETURN must exit the Theme Editor Context"
    );
}

// Validates: project-master Requirement 11.2 -- a bare colour edit does NOT
// persist: the backing file is unchanged until an explicit commit.
#[test]
fn theme_editor_bare_edit_does_not_autosave() {
    let (mut shell, _dir, name) = open_on_user_theme();
    let original = persisted_panel_bg(&shell, &name);
    let blue = ColourRGBA::rgb(0, 0, 255);
    assert_ne!(original, blue, "setup: edit must change the colour");
    shell.apply_theme_editor_action(ThemeEditorAction::EditToken(
        token_by_label("Chrome: window fill"),
        blue,
    ));
    // No SAVE / END issued: the file must still hold the original colour.
    assert_eq!(
        persisted_panel_bg(&shell, &name),
        original,
        "a bare edit must NOT be persisted (no autosave-on-edit)"
    );
}

// Validates: project-master Requirement 11.8 -- the in-panel Save affordance
// (ThemeEditorAction::Save) gates persistence through the SAME path as the typed
// SAVE command; both persist the working copy identically (no divergent paths).
#[test]
fn theme_editor_save_button_matches_save_command_path() {
    let (mut shell, _dir, name) = open_on_user_theme();
    let cyan = ColourRGBA::rgb(0, 255, 255);
    shell.apply_theme_editor_action(ThemeEditorAction::EditToken(
        token_by_label("Chrome: window fill"),
        cyan,
    ));
    // The in-panel Save button action.
    shell.apply_theme_editor_action(ThemeEditorAction::Save);
    assert_eq!(
        persisted_panel_bg(&shell, &name),
        cyan,
        "the in-panel Save button must persist via the shared persist path"
    );
    assert_eq!(
        shell.tabs.active_tab().kind.tag(),
        KindTag::ThemeEditor,
        "the Save button leaves the Context open, like the SAVE command"
    );
}
