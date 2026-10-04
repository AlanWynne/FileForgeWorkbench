//! # Shell key / history helper methods
//!
//! Command-line history persistence and function-key resolution/dispatch helpers
//! plus the split-region Tab handler. Extracted verbatim from `update.rs` as part
//! of the file-size split; behaviour is unchanged.

use eframe::egui;

use super::helpers::*;
use ff_keys::FunctionKey;
use ff_keys::{KeyModifier, ModifiedKey};

impl super::WorkbenchShell {
    /// Write the current command-line history to the History_Store
    /// (function-keys-and-history Requirement 6.3). Best-effort: a write failure
    /// is logged, never fatal. Called from `on_exit`; extracted so it is
    /// unit-testable without driving a full eframe shutdown.
    ///
    /// Validates: function-keys-and-history Requirement 6.3, 6.7
    pub(super) fn persist_command_history(&self) {
        if let Some(store) = &self.history_store {
            let ring = ff_command::CommandLineRing::from_command_strings(
                self.command_line_history.list(),
                self.command_line_history.max_entries(),
            );
            if let Err(e) = store.save(&ring) {
                ff_logging::log_warn!("[keys] command history save failed: {}", e);
            }
        }
    }

    /// Resolve a function-key press from `ctx.input` to its bound command string
    /// using the active key map, mirroring the Primary_Window's F-key detection.
    /// Returns `None` when no F-key is pressed or the key is unbound. Suppressed
    /// while a modal dialog is open.
    ///
    /// Validates: function-keys-and-history Requirement 3.1, 3.2; menu-and-statusbar 18.11
    pub(super) fn resolve_function_key_command(&self, ctx: &egui::Context) -> Option<String> {
        if self.modal_open {
            return None;
        }
        ctx.input(|i| {
            let modifier = if i.modifiers.shift {
                KeyModifier::Shift
            } else if i.modifiers.ctrl {
                KeyModifier::Ctrl
            } else if i.modifiers.alt {
                KeyModifier::Alt
            } else {
                KeyModifier::None
            };
            FunctionKey::ALL.iter().find_map(|&fk| {
                egui_fkey(fk).and_then(|ek| {
                    if i.key_pressed(ek) {
                        let mk = ModifiedKey { key: fk, modifier };
                        self.key_map_resolver
                            .active_key_map()
                            .get(mk)
                            .or_else(|| {
                                if modifier != KeyModifier::None {
                                    self.key_map_resolver.active_key_map().get_plain(fk)
                                } else {
                                    None
                                }
                            })
                            .map(|b| b.command().to_string())
                    } else {
                        None
                    }
                })
            })
        })
    }

    /// B068 (menu-and-statusbar Req 18.11): dispatch a function key pressed while
    /// a Detached_Workspace has OS focus, against THAT window's Context. Called
    /// from inside the detached viewport's `with_workspace_context` swap, so the
    /// resolved command acts on the detached tab (F3=END, F4=RETURN there).
    /// Resolution is identical to the Primary_Window (same key map + the same
    /// `dispatch_key_command` merge-with-command-field behaviour).
    pub(super) fn dispatch_detached_function_key(&mut self, vctx: &egui::Context) {
        if let Some(cmd) = self.resolve_function_key_command(vctx) {
            self.dispatch_key_command(&cmd);
        }
    }

    /// B073: while split, keep Tab/Shift+Tab focus WITHIN the focused region.
    ///
    /// Returns `true` when it handled (consumed) a Tab press. The focused region
    /// has two stable, shell-owned Tab stops: its command field
    /// (`("region_command_field_input", focused_leaf)`) and its menu-bar first
    /// button (captured in `focused_region_menu_first`). Tab toggles between them;
    /// Shift+Tab toggles the other way. The Tab event is CONSUMED so egui-native
    /// traversal never receives it and therefore cannot walk into another region.
    /// If focus is not currently on either of the focused region's stops (e.g. it
    /// is in the region body or elsewhere), Tab re-anchors it to the focused
    /// region's command field -- so Tab always lands back inside the focused
    /// region, never in a sibling region. Interior body controls remain reachable
    /// by mouse; region switching is via FOCUS / click (owner's expectation).
    ///
    /// Validates: layout-and-docking Requirement 16.8; menu-and-statusbar Req 16.15
    pub(super) fn handle_split_region_tab(&mut self, ctx: &egui::Context) -> bool {
        // Detect + consume Tab in one input pass (consuming during detection is
        // essential so egui's `give_to_next` focus machinery never latches it --
        // the same discipline the unsplit Boundary_Policy uses, B056).
        let (tab, shift) = ctx.input_mut(|i| {
            let pressed = i.key_pressed(egui::Key::Tab);
            let shift = i.modifiers.shift;
            if pressed {
                i.events.retain(|e| {
                    !matches!(
                        e,
                        egui::Event::Key {
                            key: egui::Key::Tab,
                            ..
                        }
                    )
                });
            }
            (pressed, shift)
        });
        if !tab {
            return false;
        }
        let focused_leaf = self.tabs.focused_leaf_id();
        let cmd_id = egui::Id::new(("region_command_field_input", focused_leaf.value()));
        let menu_first = self.detach_split.focused_region_menu_first;
        let current = ctx.memory(|m| m.focused());

        // Two-stop cycle within the focused region: command field <-> menu-first.
        // Forward: cmd -> menu -> cmd. Reverse: cmd -> menu -> cmd (symmetric with
        // only two stops). When focus is on neither stop, anchor to the command
        // field so Tab always returns INTO the focused region.
        let target = match (current, menu_first) {
            (Some(c), Some(mf)) if c == cmd_id => mf,
            (Some(c), Some(_)) if Some(c) == menu_first => cmd_id,
            _ => cmd_id,
        };
        let _ = shift; // symmetric two-stop cycle; Shift+Tab uses the same toggle
        ctx.memory_mut(|m| m.request_focus(target));
        true
    }
}
