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

    // CR-NR-103 (Task 9): command-dispatch tests now assert on status/error
    // messages that resolve through the ff_i18n seam. Install the English
    // Identity_Base catalogue so t()/t_args() return real English text (not the
    // key fallback), exactly as harness_shell() does for full-shell tests. The
    // install is idempotent (swaps the process-wide catalogue).
    super::i18n_bootstrap::install_catalogue_for_tests();
    use ff_core::WorkbenchApp;
    use ff_keys::HistoryStore;
    use ff_logging::LoggingStatus;
    use ff_theme::defaults::dark_palette;
    use tokio::runtime::Runtime;

    // Test isolation (B048): redirect user-config and command-line-history writes
    // to UNIQUE temp files so `set_user_value`-invoking tests (theme, follow_os,
    // workspace overrides) and history-persistence tests never read or write the
    // developer's real files. Both paths are injected DIRECTLY through
    // constructor seams (`with_user_config_path` + `new_with_history_store`), not
    // the process-global `FFWB_USER_CONFIG_PATH` / `FFWB_HISTORY_PATH` env vars,
    // so parallel tests under plain multithreaded `cargo test` cannot race on a
    // shared env var.
    let cfg_path = std::env::temp_dir().join(unique_temp_name("ffwb_test_cfg"));
    let hist_path = std::env::temp_dir().join(unique_temp_name("ffwb_test_hist"));

    let config_handle = init(
        ConfigInitOptions::new()
            .with_hot_reload(false)
            .with_user_config_path(cfg_path),
    )
    .expect("config init");
    let runtime = Runtime::new().expect("runtime");
    let app =
        WorkbenchApp::new(Box::new(config_handle.clone()), LoggingStatus::Fallback).expect("app");
    let palette = dark_palette();
    super::WorkbenchShell::new_with_history_store(
        app,
        runtime,
        palette,
        vec![],
        config_handle,
        Some(HistoryStore::new(hist_path)),
    )
}

/// Build a unique temp file name (`<prefix>_<pid>_<nanos>.toml`) for per-call
/// isolation of injected config/history paths (B048).
fn unique_temp_name(prefix: &str) -> String {
    format!(
        "{}_{}_{}.toml",
        prefix,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    )
}

/// Build a shell whose command-line history file is the given path (function-keys
/// Req 6). Injects the history store DIRECTLY via `new_with_history_store` and a
/// unique temp user-config path via `with_user_config_path`, so the shell
/// loads/saves history at `history_path` without touching the process-global
/// `FFWB_HISTORY_PATH` / `FFWB_USER_CONFIG_PATH` env vars (B048 test isolation).
pub(crate) fn make_shell_with_history_path(
    history_path: &std::path::Path,
) -> super::WorkbenchShell {
    use ff_config::init;
    use ff_config::ConfigInitOptions;
    use ff_core::WorkbenchApp;
    use ff_keys::HistoryStore;
    use ff_logging::LoggingStatus;
    use ff_theme::defaults::dark_palette;
    use tokio::runtime::Runtime;

    let cfg_path = std::env::temp_dir().join(unique_temp_name("ffwb_test_cfg"));

    let config_handle = init(
        ConfigInitOptions::new()
            .with_hot_reload(false)
            .with_user_config_path(cfg_path),
    )
    .expect("config init");
    let runtime = Runtime::new().expect("runtime");
    let app =
        WorkbenchApp::new(Box::new(config_handle.clone()), LoggingStatus::Fallback).expect("app");
    let palette = dark_palette();
    super::WorkbenchShell::new_with_history_store(
        app,
        runtime,
        palette,
        vec![],
        config_handle,
        Some(HistoryStore::new(history_path.to_path_buf())),
    )
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
        .kind
        .menu_workspace()
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
    // Install the English Identity_Base on the process-wide i18n seam so the
    // full-shell render shows the same English labels production does (where
    // i18n_bootstrap::install_catalogue runs at startup). Without this, a fresh
    // nextest process leaves the seam uninstalled and every localised `t()`
    // label renders as its key, breaking `get_by_label("...")` queries
    // (CR-NR-103 Slice 2: ff-*-dialog/panel crates now resolve labels via t()).
    super::i18n_bootstrap::install_catalogue_for_tests();
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

// === CR-NR-109: locale-aware menu/resolver test setup =======================

/// Install a TEST catalogue defining `verb-find = SOEK` on the process-wide i18n
/// seam, so a localised display surface keyed `verb-find` resolves to SOEK
/// end-to-end through the real `ff_i18n::t` seam (the shipped `.ftl` tree has no
/// verb-* keys yet).
///
/// CRITICAL: this builds the catalogue from a COPY of the SHIPPED English base
/// PLUS the extra `verb-find` key, not a one-key catalogue. Because the i18n seam
/// is PROCESS-WIDE and the ff-desktop test binary runs many tests in one process,
/// installing a one-key catalogue would strip every other English label (e.g.
/// "Save") and break unrelated full-shell label-query tests that run later in the
/// same process. Copying the shipped `en/` tree keeps all real labels resolving
/// while adding verb-find. Call AFTER building the harness when a test needs the
/// localised surface to resolve.
///
/// Validates: localization Requirement 11.7, 11.8 (test-support)
pub(crate) fn install_verb_find_soek_catalogue() {
    // Resolve the shipped i18n root via the same search path production uses
    // (the dev-tree marker walk reaches crates/ff-i18n/i18n from the test binary).
    let shipped_root = super::i18n_bootstrap::resolve_i18n_root()
        .expect("shipped i18n/ root must resolve for the verb-find test catalogue");
    let shipped_en = shipped_root.join("en");

    // Build a temp i18n root with an en/ that mirrors the shipped English base
    // plus an extra verbs.ftl defining verb-find = SOEK.
    let dir = tempfile::TempDir::new().expect("tempdir");
    let en = dir.path().join("en");
    std::fs::create_dir_all(&en).expect("create en dir");
    if let Ok(entries) = std::fs::read_dir(&shipped_en) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("ftl") {
                if let Some(name) = path.file_name() {
                    // SKIP the shipped verb catalogue: it defines `verb-find = FIND`
                    // (CR-NR-109 Task 11.7 Identity_Base). Copying it alongside the
                    // test's own `verb-find = SOEK` below would make `verb-find`
                    // appear TWICE in the same en bundle -- a Fluent duplicate-message
                    // bundle error. The test's zz-test-verbs.ftl is the sole
                    // `verb-find` definition, overriding the root surface to SOEK.
                    if name.eq_ignore_ascii_case("ff-desktop-verbs.ftl") {
                        continue;
                    }
                    let _ = std::fs::copy(&path, en.join(name));
                }
            }
        }
    }
    std::fs::write(en.join("zz-test-verbs.ftl"), "verb-find = SOEK\n").expect("write ftl");
    let catalogue =
        ff_i18n::Catalogue::load_identity_base(dir.path()).expect("test catalogue loads");
    // Thread-local override (test isolation): per-thread, so concurrent tests do
    // not clobber each other's catalogue under threaded `cargo test`.
    ff_i18n::set_test_catalogue(catalogue);
}

/// Install a TEST catalogue (CR-NR-111) that, on top of a COPY of the shipped
/// English base, defines the localised MENU-OPTION surfaces for a `FIND` option
/// in a menu named `menu_name`: the mnemonic key `menu-<menu>-opt-find-key`, the
/// description key `menu-<menu>-opt-find-desc`, and the command-surface key
/// `verb-find`. `extra` lets a caller pass additional `key = value` lines (e.g.
/// to seed a localised mnemonic WITHOUT a description, proving per-part
/// fallback). Mirrors `install_verb_find_soek_catalogue` -- it copies the shipped
/// English tree (skipping the shipped verbs catalogue to avoid a duplicate
/// `verb-find`) so every unrelated label keeps resolving in the shared process.
///
/// This seeds the per-locale DATA IN-TEST (CR-NR-111 delivers the CODE contract
/// only; authoring built-in DATA is a separate deliverable), exactly as the
/// plan requires every proving test to do.
///
/// Validates: localization Requirement 12.1, 12.3, 12.6, 12.8 (test-support)
pub(crate) fn install_find_menu_localised_catalogue(menu_name: &str, extra: &str) {
    let shipped_root = super::i18n_bootstrap::resolve_i18n_root()
        .expect("shipped i18n/ root must resolve for the CR-NR-111 test catalogue");
    let shipped_en = shipped_root.join("en");

    let dir = tempfile::TempDir::new().expect("tempdir");
    let en = dir.path().join("en");
    std::fs::create_dir_all(&en).expect("create en dir");
    if let Ok(entries) = std::fs::read_dir(&shipped_en) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("ftl") {
                if let Some(name) = path.file_name() {
                    // Skip the shipped verbs catalogue (defines verb-find) to
                    // avoid a Fluent duplicate-message error with our verb-find.
                    if name.eq_ignore_ascii_case("ff-desktop-verbs.ftl") {
                        continue;
                    }
                    let _ = std::fs::copy(&path, en.join(name));
                }
            }
        }
    }
    // The localised surfaces for the FIND option in `menu_name`, plus any extra
    // lines the caller supplies. verb-find = SOEK is the command-column surface
    // (keyed by the alias row's display key); the mnemonic/description keys are
    // the CR-NR-111 render-time derivations.
    let ftl = format!(
        "verb-find = SOEK\n\
         menu-{menu_name}-opt-find-key = S\n\
         menu-{menu_name}-opt-find-desc = Soek teks\n\
         {extra}\n"
    );
    std::fs::write(en.join("zz-test-crnr111.ftl"), ftl).expect("write ftl");
    let catalogue =
        ff_i18n::Catalogue::load_identity_base(dir.path()).expect("test catalogue loads");
    // Thread-local override (test isolation): per-thread, so concurrent tests do
    // not clobber each other's catalogue under threaded `cargo test`.
    ff_i18n::set_test_catalogue(catalogue);
}

/// Install a TEST catalogue (CR-NR-111 per-part fallback) that seeds ONLY the
/// command surface `verb-find = SOEK` for a `FIND` option, with NO mnemonic or
/// description key for `menu_name`. Used to prove per-part fallback: the command
/// localises, the mnemonic falls back to the stored `option.key`, and the
/// description falls back to the parsed English prose -- never blank.
///
/// Validates: localization Requirement 12.8 (test-support)
pub(crate) fn install_find_command_only_catalogue() {
    install_verb_find_soek_catalogue();
}

/// Overlay a single `(surface -> canonical)` alias row (carrying the `verb-find`
/// localisation key) onto the given environment's table in the shell's alias
/// store, so a full-shell test can simulate an active-locale alias without a
/// TOML file on disk. Thin wrapper over the store's `overlay_row_for_test`.
pub(crate) fn overlay_alias_row(
    shell: &super::WorkbenchShell,
    env: &str,
    surface: &str,
    canonical: &str,
) {
    let mut store = shell.i18n.alias_store.lock().expect("alias store lock");
    store.overlay_row_for_test(env, surface, canonical);
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
    crate::workspace_kind::BuiltinKind::from_tab_kind(t.kind.tag(), t.is_home).stable_name()
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
