//! # WorkbenchShell — egui/eframe Rendering Shell
//!
//! Sole point of contact between egui/eframe and the platform-core layer.
//! Implements `eframe::App` and owns the `TabManager` (all open tabs),
//! the Tokio runtime, and the active theme palette.
//!
//! This module is a thin coordinator after the Phase 2 task 2.2 file-size split.
//! The `WorkbenchShell` struct definition lives in `state.rs`; its constructors
//! in `construct.rs`; the built-in command handlers in `handlers.rs`; the
//! KeyBarScope / Detached_Workspace value types in `types.rs`; and the method
//! groups in `actions.rs`, `workspace_io.rs`, `titles.rs`, plus the pre-existing
//! `render*`/`commands*`/`update`/etc. submodules. mod.rs itself holds only the
//! submodule declarations and the re-exports that keep every existing
//! `super::...` / `crate::shell::...` path resolving unchanged.

// Where the former inline items now live (Phase 2 task 2.2 file-size split):
// - The WorkbenchShell struct definition -> `state.rs` (re-exported below).
// - The KeyBarScope enum and the Detached_Workspace bookkeeping types
//   (WorkspaceCommandContext, FloatingTab) -> `types.rs` (re-exported below).
// - The built-in command handlers (FileOpenHandler, FileExitHandler,
//   MenuOpenHandler, ShellContextProvider, ConfigOpenHandler) -> `handlers.rs`;
//   constructed in `new_with_history_store` (construct.rs).
//
// Menu bar (data-driven) -- Validates: menu-workspace Req 17.1. CR-NR-080: the
// menu bar is DATA-DRIVEN, rendered from the compiled default Menu_Bar
// (`menu_workspace::defaults::default_menubar_menu`) rather than a hardcoded
// label list. The former `MENU_BAR_TOP_LEVEL_LABELS` const (and its
// `debug_assert_eq!` in `render_menu_bar`) were removed; the bar's top-level
// entries are the default Menu_Bar's option `description`s, and tests assert
// against `default_menubar_menu()` instead.

mod actions;
mod alias_editor;
mod alias_editor_line_commands;
mod alias_overlay;
pub(crate) mod alias_reader;
mod alias_template;
mod command_line_outcome;
mod commands;
mod commands_config_save;
mod commands_configurator_save;
mod commands_environment;
mod commands_fastpath;
mod commands_keys_save;
mod commands_kinds_save;
mod commands_ladder_a;
mod commands_ladder_b;
mod commands_ladder_c;
mod commands_line;
mod commands_menu;
mod commands_menus_save;
mod commands_scrm;
mod commands_session;
mod commands_snapshot;
mod commands_theme;
mod commands_theme_save;
mod configurator;
mod construct;
mod construct_commands;
mod dispatch;
mod editable_context;
mod environment;
mod environment_contract;
pub(crate) mod environment_registry;
mod external_adapter;
mod handlers;
pub(crate) mod help;
/// Convert a `ff_config::ConfigValue` to a `toml::Value` for key-map parsing.
mod helpers;
pub(crate) mod i18n_bootstrap;
mod keys_editor;
mod kinds_editor;
mod locale_resolver;
mod menus_editor;
mod mod_helpers;
mod nav_reconstruct;
mod nav_stack;
mod render;
mod render_body;
mod render_body_arms;
mod render_chrome;
mod render_command_line;
mod render_menu_bar;
mod render_split;
mod render_split_region;
mod render_status;
mod render_tab_bar;
mod render_theme;
mod reset_bare;
mod screen_provider;
mod screen_providers;
mod state;
mod state_groups;
mod target_dispatch;
mod titles;
mod types;
mod update;
mod update_dialogs;
mod update_floating;
mod update_input;
mod update_keys;
mod update_startup;
pub(crate) mod user_aliases;
pub(crate) mod workspace_context;
mod workspace_io;

// Re-exported so `super::WorkbenchShell` / `crate::shell::WorkbenchShell` resolve
// unchanged after the Phase 2 task 2.2 move of the struct definition into
// `state.rs`.
pub use state::WorkbenchShell;
// Re-exported so siblings/tests keep resolving `super::title_line_text`,
// `super::truncate_title`, `super::line_end_from_name`, etc. unchanged after the
// Phase 2 task 2.2 move of these free functions into `mod_helpers.rs`.
pub(crate) use mod_helpers::*;
// Re-exported so siblings/tests keep resolving `super::KeyBarScope`,
// `super::WorkspaceCommandContext`, and `super::FloatingTab` unchanged after the
// Phase 2 task 2.2 move of these value types into `types.rs`.
pub(crate) use types::*;
// Re-exported so `tests_menu_workspace` keeps resolving
// `super::config_value_to_toml_value` -- the binding the former `use helpers::*;`
// provided before mod.rs stopped using helpers directly (Phase 2 task 2.2).
// Only the test module consumes it via `super::` (production callers import it
// straight from `helpers`), so gate the re-export behind cfg(test) to avoid an
// unused-import warning in the bin build.
#[cfg(test)]
pub(crate) use helpers::config_value_to_toml_value;

// Shell tests, split by feature area from the former monolithic tests.rs (CR F3).
// Each module is a direct child of `shell` so every `super::` reference inside a
// moved test still resolves to `shell`, unchanged. Shared helpers live in
// `tests_common` and are glob-imported by each area module.
#[cfg(test)]
mod tests_alias_editor;
#[cfg(test)]
mod tests_alias_template;
#[cfg(test)]
mod tests_command;
#[cfg(test)]
mod tests_common;
#[cfg(test)]
mod tests_config_save_convention;
#[cfg(test)]
mod tests_configurator_save_convention;
// core-001 strip: tests_crch061 (mainframe catalog-backed SAVE routing) and
// tests_crch071 (FFEDIT/FFCMD editor verb seam) validated now-parked domains
// (catalog, mainframe CE, FFEDIT) and were removed with those features.
#[cfg(test)]
mod tests_focus;
#[cfg(test)]
mod tests_keys_save_convention;
#[cfg(test)]
mod tests_kinds_save_convention;
#[cfg(test)]
mod tests_locale_menu;
#[cfg(test)]
mod tests_locale_menu_full;
#[cfg(test)]
mod tests_menu_bar_keynav;
#[cfg(test)]
mod tests_menu_bar_keynav_bar;
#[cfg(test)]
mod tests_menu_bar_keynav_tab;
#[cfg(test)]
mod tests_menu_bar_keynav_tabmove;
#[cfg(test)]
mod tests_menu_workspace;
#[cfg(test)]
mod tests_menus_save_convention;
#[cfg(test)]
mod tests_misc;
#[cfg(test)]
mod tests_nav;
#[cfg(test)]
mod tests_nav_ladder;
#[cfg(test)]
mod tests_screen_directive;
#[cfg(test)]
mod tests_scrm;
#[cfg(test)]
mod tests_session;
#[cfg(test)]
mod tests_snapshot_picture;
#[cfg(test)]
mod tests_split_detach;
#[cfg(test)]
mod tests_theme_save_convention;
