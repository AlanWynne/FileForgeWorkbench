//! Shared helpers for the shell test submodules (split from the
//! former monolithic shell/tests.rs -- CR F3). Behaviour unchanged;
//! helpers are `pub(crate)` so sibling `tests_*` modules can call them.

#![allow(unused_imports)]

use ff_keys::{KeyMap, ModifiedKey};
use std::sync::{Arc, Mutex};

use ff_command::{
    CommandDispatch, CommandError, CommandHandler, CommandHistory, CommandId, CommandMetadata,
    CommandParams, CommandRegistry, CommandResult, ExecutionContext,
};

pub(crate) fn make_dispatch() -> (Arc<CommandRegistry>, CommandDispatch) {
    let registry = Arc::new(CommandRegistry::new());
    let history = Arc::new(CommandHistory::new(100));
    let dispatch = CommandDispatch::new(registry.clone(), history);
    (registry, dispatch)
}

pub(crate) fn meta(name: &str, cat: &str) -> CommandMetadata {
    CommandMetadata::builder(name, name).category(cat).build()
}

/// Returns true if the command is handled at the shell level (not routed to CommandEngine).
pub(crate) fn is_shell_command(cmd: &str) -> bool {
    let upper = cmd.trim().to_uppercase();
    upper == "EXIT"
        || upper == "QUIT"
        || upper == "=X"
        || upper == "X"
        || upper == "START"
        || upper == "POM"
        || upper == "CLOSE"
        || upper == "0"
        || upper == "SETTINGS"
        || upper == "=0"
        || upper == "1"
        || upper == "=1"
        || upper == "FILE CATALOGS"
        || upper == "2"
        || upper == "=2"
        || upper == "=FILES"
        || upper == "FILES"
        || upper == "3"
        || upper == "UTILITIES"
        || upper == "4"
        || upper == "COMPILERS"
        || upper == "7"
        || upper == "DATABASES"
        || upper == "8"
        || upper == "PLUGINS"
        || upper == "RETRIEVE"
        || upper == "RFIND"
        || upper == "RCHANGE"
        || upper == "EDIT"
        || upper.starts_with("EDIT ")
        || upper.starts_with("FIND ")
        || upper.starts_with("CHANGE ")
        || upper.starts_with("LOCATE ")
        || upper == "TOP"
        || upper == "BOTTOM"
        || upper == "UP"
        || upper.starts_with("UP ")
        || upper == "DOWN"
        || upper.starts_with("DOWN ")
        || upper == "LEFT"
        || upper.starts_with("LEFT ")
        || upper == "RIGHT"
        || upper.starts_with("RIGHT ")
        || upper == "SORT"
        || upper.starts_with("SORT ")
        || upper == "EXCLUDE ALL"
        || upper.starts_with("EXCLUDE ")
        || upper == "X ALL"
        || upper.starts_with("X ")
        || upper == "SHOW ALL"
        || upper.starts_with("SHOW ")
        || upper == "INCLUDE ALL"
        || upper.starts_with("INCLUDE ")
        || upper == "RESET"
        || upper == "RESET EXCLUDED"
        || upper == "RESET ALL"
        || upper == "PFSHOW"
        || upper.starts_with("PFSHOW ")
        || upper == "END"
        || upper == "RETURN"
        || upper == "KEYS"
        || upper == "LOGOFF"
        || upper == "TIME"
        || upper == "STATUS"
        || upper.starts_with("STATUS ")
}

// === Phase BW Group 2 -- Edit Profile Commands ===========================

/// Construct a minimal WorkbenchShell for command-dispatch unit tests.
pub(crate) fn make_shell() -> super::WorkbenchShell {
    use ff_config::init;
    use ff_config::ConfigInitOptions;
    use ff_core::WorkbenchApp;
    use ff_logging::LoggingStatus;
    use ff_theme::defaults::dark_palette;
    use tokio::runtime::Runtime;

    // Test isolation (B048): redirect user-config writes to a unique temp file so
    // `set_user_value`-invoking tests (theme, follow_os, workspace overrides)
    // never read or write the developer's real per-user config. Under nextest
    // (process-per-test) this fully isolates each test; the env var is read by
    // `ff_config::paths::user_config_path`. Set BEFORE `init()` so the config
    // system resolves the temp path from the start.
    let unique = format!(
        "ffwb_test_cfg_{}_{}.toml",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let cfg_path = std::env::temp_dir().join(unique);
    std::env::set_var("FFWB_USER_CONFIG_PATH", &cfg_path);
    // Test isolation (B048, function-keys Req 6): redirect the command-line
    // history file to a unique temp path so tests never read/write the real
    // command_history.toml. Distinct per test id so no cross-test bleed.
    let hist_unique = format!(
        "ffwb_test_hist_{}_{}.toml",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    std::env::set_var("FFWB_HISTORY_PATH", std::env::temp_dir().join(hist_unique));

    let config_handle = init(ConfigInitOptions::new().with_hot_reload(false)).expect("config init");
    let runtime = Runtime::new().expect("runtime");
    let app =
        WorkbenchApp::new(Box::new(config_handle.clone()), LoggingStatus::Fallback).expect("app");
    let palette = dark_palette();
    super::WorkbenchShell::new(app, runtime, palette, vec![], config_handle)
}

/// Build a shell whose command-line history file is the given path (function-keys
/// Req 6). Sets `FFWB_HISTORY_PATH` before `new` so the shell loads/saves there,
/// keeping the test isolated from the real user history file.
pub(crate) fn make_shell_with_history_path(
    history_path: &std::path::Path,
) -> super::WorkbenchShell {
    use ff_config::init;
    use ff_config::ConfigInitOptions;
    use ff_core::WorkbenchApp;
    use ff_logging::LoggingStatus;
    use ff_theme::defaults::dark_palette;
    use tokio::runtime::Runtime;

    let unique = format!(
        "ffwb_test_cfg_{}_{}.toml",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    std::env::set_var("FFWB_USER_CONFIG_PATH", std::env::temp_dir().join(unique));
    std::env::set_var("FFWB_HISTORY_PATH", history_path);

    let config_handle = init(ConfigInitOptions::new().with_hot_reload(false)).expect("config init");
    let runtime = Runtime::new().expect("runtime");
    let app =
        WorkbenchApp::new(Box::new(config_handle.clone()), LoggingStatus::Fallback).expect("app");
    let palette = dark_palette();
    super::WorkbenchShell::new(app, runtime, palette, vec![], config_handle)
}

// === DB.4 -- Command Target binding (menu options + shortcuts) ==============

/// Push a user Function definition into the shell's command store.
pub(crate) fn push_user_function_def(
    shell: &mut super::WorkbenchShell,
    id: &str,
    command_id: &str,
) {
    use ff_command::{CommandTarget, TargetParams};
    shell
        .command_store
        .definitions
        .push(crate::command_config::CommandDefinition {
            id: id.to_string(),
            label: format!("Run {command_id}"),
            description: None,
            category: "user".to_string(),
            target: CommandTarget::Function {
                command_id: command_id.to_string(),
                params: TargetParams::new(),
            },
        });
}

/// Open a temp-backed CommandStore on the shell so save() writes to a temp dir.
pub(crate) fn point_store_at_temp(shell: &mut super::WorkbenchShell) -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let path = dir.path().join("commands").join("commands.toml");
    shell.command_store = crate::command_config::store::CommandStore::new(path);
    dir
}

/// Build a Function EditForm on the configurator panel.
pub(crate) fn set_add_function_form(shell: &mut super::WorkbenchShell, id: &str, command_id: &str) {
    use crate::command_config::edit::{EditForm, TargetVariant};
    let mut form = EditForm::new_add();
    form.id = id.to_string();
    form.label = format!("Run {command_id}");
    form.variant = TargetVariant::Function;
    form.command_id = command_id.to_string();
    shell.command_configurator_panel.form = Some(form);
}

// === External execution adapter (command-configurator Requirement 3) ========

/// Set the shell engine's mode for deterministic external-execution tests.
pub(crate) fn set_shell_mode(shell: &super::WorkbenchShell, mode: ff_shell::ShellMode) {
    let cfg = ff_shell::ShellConfig {
        mode,
        ..Default::default()
    };
    shell.shell_engine.set_config(cfg);
}

/// A trivial no-op external target for the host platform.
pub(crate) fn noop_external_target() -> ff_command::CommandTarget {
    use ff_command::{CommandTarget, ExternalMode};
    let (program, args) = if cfg!(windows) {
        (
            "cmd".to_string(),
            vec!["/C".to_string(), "exit".to_string()],
        )
    } else {
        ("true".to_string(), Vec::new())
    };
    CommandTarget::External {
        program,
        args,
        working_dir: None,
        mode: ExternalMode::Detached,
    }
}

// === CR-CH-043: one Option-Selection path (menu is a dumb dispatcher) =======

// Small helpers describing the observable landing state of a menu selection, so
// two selection means can be compared for "identical result" (Req 19.2 / 14.2)
// without depending on internal dispatch shape.
#[cfg(test)]
pub(crate) fn active_menu_title(shell: &super::WorkbenchShell) -> Option<String> {
    shell
        .tabs
        .active_tab()
        .menu_workspace
        .as_ref()
        .and_then(|mw| mw.menu.as_ref())
        .map(|m| m.title.clone())
}

// === CR-NR-075: Menus Editor Context (Requirement 13) ======================

/// Seed a shell whose menus_dir is an isolated TempDir, with the Menus editor
/// open on the POM working copy.
pub(crate) fn make_shell_with_menus_editor() -> (super::WorkbenchShell, tempfile::TempDir) {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let mut shell = make_shell();
    shell.dir_overrides.menus = Some(dir.path().join("menus"));
    shell.dispatch_command_string("MENUS");
    (shell, dir)
}

/// Point the shell's themes directory at a fresh TempDir (isolates Theme editor
/// file operations) and materialise the built-in theme files there.
pub(crate) fn point_themes_at_temp(shell: &mut super::WorkbenchShell) -> tempfile::TempDir {
    let dir = tempfile::TempDir::new().expect("tempdir");
    crate::theme_defaults::ensure_default_theme_files(dir.path());
    shell.dir_overrides.themes = Some(dir.path().join("themes"));
    dir
}

// === CR-CH-023: full-shell tab-order (Boundary_Policy) via egui_kittest ======
//
// These drive the REAL WorkbenchShell headlessly through eframe::App::update
// (egui_kittest build_eframe), so the shared Boundary_Policy is exercised
// end-to-end: command-line entry, interior order, menu-bar-last, wrap, and the
// Shift+Tab reverse. This replaces the manual-only coverage that previously
// backed Req 16.1/16.3/16.5/16.7/16.8 and menu-workspace Req 15.9.

/// Build the shell in a headless harness and run enough frames for one-shot
/// startup (session/config load + first render that captures menu-bar ids).
pub(crate) fn harness_shell<'a>() -> egui_kittest::Harness<'a, super::WorkbenchShell> {
    use egui_kittest::Harness;
    let mut harness = Harness::builder()
        .with_size(egui::Vec2::new(1200.0, 900.0))
        .build_eframe(|_cc| make_shell());
    for _ in 0..4 {
        harness.run();
    }
    harness
}

pub(crate) fn cmd_field_id() -> egui::Id {
    egui::Id::new("command_field_input")
}

/// Press Tab, run a frame, return the focused id (if any).
pub(crate) fn tab_and_focus(
    harness: &mut egui_kittest::Harness<super::WorkbenchShell>,
) -> Option<egui::Id> {
    harness.key_press(egui::Key::Tab);
    harness.run();
    harness.ctx.memory(|m| m.focused())
}

// === CR-NR-096: Up/Down arrow command-history stepping ======================

/// Seed the shared Command_Line_History with the given commands in order, so
/// the LAST element is the most-recent (index 0) entry. Records directly on the
/// processor-owned history (bypassing dispatch side effects) exactly as a real
/// submission's `record` would.
pub(crate) fn seed_history(
    harness: &mut egui_kittest::Harness<super::WorkbenchShell>,
    cmds: &[&str],
) {
    for c in cmds {
        harness.state_mut().command_line_history.record(c);
    }
}

// === B059: every remaining workspace lands first Tab on its first interior ===

// Shared assertion (CR-CH-023 / B059, workspace-conformance steering rule): open
// the workspace via `command`, confirm entry focus is the command field, then the
// first Tab lands EXACTLY on the reported first interior control (no phantom stop).
pub(crate) fn assert_first_tab_lands_on_reported_interior(command: &str, workspace: &str) {
    let mut harness = harness_shell();
    harness.state_mut().handle_command(command);
    for _ in 0..4 {
        harness.run();
    }
    let expected = harness.state().focus.first_interior_id;
    assert!(
        expected.is_some(),
        "{workspace} must report a first interior control (B059)"
    );
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        Some(cmd_field_id()),
        "command field holds focus on entering {workspace}"
    );
    harness.key_press(egui::Key::Tab);
    harness.run();
    assert_eq!(
        harness.ctx.memory(|m| m.focused()),
        expected,
        "first Tab in {workspace} must focus the reported first interior, not a phantom stop (B059)"
    );
}

// === CR-NR-095: COMMAND command sets/toggles command-line position ==========

/// The active tab's Kind stable name (for asserting its effective position).
pub(crate) fn active_kind_name(shell: &super::WorkbenchShell) -> &'static str {
    let t = shell.tabs.active_tab();
    crate::workspace_kind::BuiltinKind::from_tab_kind(t.kind, t.is_home).stable_name()
}

// === CR-NR-094 Slice 2d: per-region command lines (full shell) ==============

/// The salted egui id of a split region's own `Command ===>` field. MUST match
/// the id built in `render_region_command_field` so focus assertions round-trip.
pub(crate) fn region_cmd_field_id(leaf: ff_layout::TabGroupId) -> egui::Id {
    egui::Id::new(("region_command_field_input", leaf.value()))
}

/// Drive a region's command line end-to-end: focus that region's field, put
/// `cmd` in its context, press Enter, and settle. Mirrors what a user typing in
/// the region field and pressing Enter does, without needing per-character
/// keystroke injection (the field body reads its bound `command_text`).
pub(crate) fn submit_region_command(
    harness: &mut egui_kittest::Harness<'_, super::WorkbenchShell>,
    leaf: ff_layout::TabGroupId,
    cmd: &str,
) {
    let field_id = region_cmd_field_id(leaf);
    harness.ctx.memory_mut(|m| m.request_focus(field_id));
    harness
        .state_mut()
        .detach_split
        .region_cmd_ctx
        .get_mut(&leaf)
        .expect("region has a command context")
        .command_text = cmd.to_string();
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
}

// === CR-CH-041 IRP-b: per-instance chrome renders in-region ================

/// The salted egui id of a split region's own menu-bar scope (CR-CH-041 IRP-b).
/// MUST match the `push_id` scope used in `render_region_menu_bar` so tests can
/// assert per-region chrome without colliding across regions.
pub(crate) fn region_menu_scope_id(leaf: ff_layout::TabGroupId) -> egui::Id {
    egui::Id::new(("region_menu_bar", leaf.value()))
}

// === CR-NR-098 Wave 3: CAPTURE EXPORT + SAVE/LOAD persistence ===============

/// Seed a shell with an isolated screen-collections dir and a one-capture POM
/// collection. Returns the shell and the TempDir (kept alive by the caller).
pub(crate) fn make_shell_with_scrm_dir() -> (super::WorkbenchShell, tempfile::TempDir) {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let mut shell = make_shell();
    shell.dir_overrides.scrm = Some(dir.path().to_path_buf());
    shell.tabs.insert_pom_tab(&shell.runtime);
    shell.ensure_pom_menu_loaded();
    shell.handle_command("CAPTURE START Repro");
    shell.handle_command("CAPTURE SCREEN");
    (shell, dir)
}
