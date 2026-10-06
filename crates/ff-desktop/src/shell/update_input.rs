//! # Shell Per-Frame Input Handling
//!
//! The zoom / DPI phase (`handle_zoom_and_dpi`) and the unified Tab-order focus
//! cycle / Boundary_Policy (`handle_tab_order_focus`) of the eframe `update()`
//! loop. Extracted verbatim from `update.rs` as part of the Phase 2 task 2.2
//! file-size split; behaviour and order are unchanged -- `update` calls these in
//! the same place they ran inline.

use eframe::egui;

use super::WorkbenchShell;
use crate::files_panel;
use crate::tab_state::KindTag;

impl WorkbenchShell {
    /// Ctrl+Scroll global zoom, window-drag detection, and the deferred
    /// pixels_per_point apply. Extracted verbatim from `update()`.
    ///
    /// Validates: view-zoom Requirement 3.1, 3.2
    pub(super) fn handle_zoom_and_dpi(&mut self, ctx: &egui::Context) {
        // Validates: Requirement 3.1/3.2 (view-zoom) -- Ctrl+Scroll updates global zoom.
        // Single zoom level shared across all tab kinds and contexts.
        {
            let (scroll_delta, ctrl_held) = ctx.input_mut(|i| {
                let raw = i.raw_scroll_delta.y;
                let smooth = i.smooth_scroll_delta.y;
                let ctrl = i.modifiers.ctrl;
                if ctrl {
                    i.raw_scroll_delta = egui::Vec2::ZERO;
                    i.smooth_scroll_delta = egui::Vec2::ZERO;
                }
                let delta = if raw != 0.0 { raw } else { smooth };
                (delta, ctrl)
            });
            if ctrl_held && scroll_delta != 0.0 {
                if scroll_delta > 0.0 {
                    self.zoom.zoom_in();
                } else {
                    self.zoom.zoom_out();
                }
            }
        }
        // Track whether the primary mouse button is held -- window drag detection.
        // While the mouse is down we suppress any pixels_per_point change so that
        // WM_DPICHANGED messages fired as the window crosses a monitor boundary do
        // not trigger mid-move resize stuttering.  The change is applied on release.
        let mouse_down = ctx.input(|i| i.pointer.primary_down());
        if mouse_down {
            self.is_dragging = true;
        } else if self.is_dragging {
            // Mouse just released -- apply any deferred ppp now.
            self.is_dragging = false;
            if let Some(ppp) = self.pending_ppp.take() {
                self.last_ppp = ppp;
                ctx.set_pixels_per_point(ppp);
            }
        }

        // Apply global zoom only when it has changed -- do NOT call set_pixels_per_point
        // every frame, as that fights the OS DPI adjustment during cross-monitor moves
        // and causes the window to flash and stick at monitor boundaries.
        {
            let ppp = (1.0_f32 + self.zoom.offset().value() as f32 * 0.07).clamp(0.3, 4.0);
            if (ppp - self.last_ppp).abs() > f32::EPSILON {
                if self.is_dragging {
                    // Defer until mouse release.
                    self.pending_ppp = Some(ppp);
                } else {
                    self.last_ppp = ppp;
                    ctx.set_pixels_per_point(ppp);
                }
            }
        }
    }

    /// The unified Tab-order focus cycle: computes `modal_open`, runs the split
    /// region Tab handler, and otherwise applies the unsplit Boundary_Policy
    /// (command-field entry, menu-bar wrap, interior latches, File Explorer
    /// tree-transfer). Extracted verbatim from `update()`.
    ///
    /// Validates: menu-and-statusbar Requirement 16.2-16.22 (CR-CH-023)
    pub(super) fn handle_tab_order_focus(&mut self, ctx: &egui::Context) {
        // -- Tab-order focus cycle -- Validates: Requirement 16.2-16.22 -----------
        // Consume Tab / Shift+Tab before egui processes them so we control focus.
        // Suppressed when a modal dialog is open so Tab navigates inside the dialog.
        self.modal_open = self.show_about
            || self.palette_state.open
            || self.show_history_list.is_some()
            || self.show_swap_list.is_some()
            || self.show_unsaved_workspace_dialog
            // B077: the RESET BARE confirmation and the external-run confirmation
            // are modal too. They set `modal_open` again later in this same frame,
            // but the Tab/Boundary_Policy below reads the flag NOW -- so they MUST
            // be included here or background Tab keeps running while the popup is
            // open (accessibility Req 2.3 -- modal focus trap).
            || self.reset_bare_confirm.is_some()
            || self.pending_external.is_some()
            || !matches!(self.files_panel.dialog, files_panel::FilesDialogState::None);
        // B073: while the Workspace is split, the shell OWNS Tab entirely and
        // keeps focus WITHIN the focused region -- egui-native traversal would
        // otherwise walk across all regions (the top-level command field is
        // suppressed, so the unsplit Boundary_Policy below never engages). Cycle
        // the focused region's two stable stops: its command field <-> its
        // menu-bar first button. Consuming Tab here means egui never receives it,
        // so it cannot cross into another region. Switching regions is by
        // FOCUS / click, not Tab (matches the owner's expectation).
        let handled_split_tab = if self.tabs.is_split() && !self.modal_open {
            self.handle_split_region_tab(ctx)
        } else {
            false
        };
        if !self.tabs.is_split() && !handled_split_tab {
            let is_file_explorer = self.tabs.active_tab().kind.tag() == KindTag::FileExplorerPanel;
            let cmd_id = egui::Id::new("command_field_input");
            let cmd_has_focus = ctx.memory(|m| m.focused() == Some(cmd_id));

            // CR-CH-023 unified tab-order: the shell owns only the Boundary_Policy
            // (command-line entry, menu-bar-last, wrap). Interior order is
            // egui-native, so between boundaries the Tab event is LEFT in the
            // queue for egui to process. At a boundary the shell consumes Tab and
            // redirects focus. The File Explorer keeps its own tree-transfer.
            //
            // Snapshot the boundary anchors reported by last frame's render and
            // the currently-focused widget so we can decide, BEFORE egui gets the
            // Tab event, whether this press is a boundary (shell handles it) or an
            // interior/menu-internal move (egui-native handles it).
            let first_interior = self.focus.first_interior_id;
            let last_interior = self.focus.last_interior_id;
            let menu_first_id = self.focus.menu_first_id;
            let menu_last_id = self.focus.menu_last_id;
            let focused = ctx.memory(|m| m.focused());
            let on_menu_last = menu_last_id.is_some() && focused == menu_last_id;
            let on_menu_first = menu_first_id.is_some() && focused == menu_first_id;
            let on_last_interior = last_interior.is_some() && focused == last_interior;
            let on_first_interior = first_interior.is_some() && focused == first_interior;

            // Classify the boundary this frame (None = not a shell boundary; let
            // egui-native traversal handle it). Computed from focus position only.
            #[derive(Clone, Copy)]
            enum Boundary {
                None,
                /// Focus the given (reliable) id -- used for menu-bar buttons.
                Focus(egui::Id),
                /// Latch: interior render focuses its first control (fresh id).
                FirstInterior,
                /// Latch: interior render focuses its last control (fresh id).
                LastInterior,
                /// Re-arm command-field focus.
                CommandField,
            }
            let is_explorer_transfer = is_file_explorer && (cmd_has_focus || self.nav_focused);

            // Detect Tab/Shift+Tab AND, in the SAME input pass, decide the
            // boundary and consume the event ONLY when the shell will handle it.
            // Consuming during detection (not afterwards) is essential: egui
            // latches the Tab into its `give_to_next` focus machinery while
            // rendering, which would otherwise override our request_focus (B056).
            let boundary = ctx.input_mut(|i| {
                if self.modal_open {
                    return Boundary::None;
                }
                let shift = i.modifiers.shift;
                if !i.key_pressed(egui::Key::Tab) {
                    return Boundary::None;
                }
                // The File Explorer tree-transfer consumes Tab in its own branch.
                if is_explorer_transfer {
                    return Boundary::None;
                }
                // Boundary_Policy (B056). egui-native traversal handles the ORDER
                // WITHIN the interior and WITHIN the menu bar; the shell handles
                // the boundary JUMPS. The command -> first-interior jump uses a
                // one-shot latch (`FirstInterior`) honoured by the interior render
                // with the FRESH same-frame id, because interior option auto-ids
                // do NOT round-trip through egui focus when requested from a
                // previous frame. Menu-bar jumps use the reliable captured
                // menu-button ids directly.
                // egui-native traversal already produces the correct ORDER
                // through the interior and the menu bar and wraps back to the
                // command field on its own. The shell only needs to intercept the
                // command-field -> interior entry so egui does not first stop on
                // the SCROLL field / other command-panel widgets: use the
                // one-shot latch that focuses the FRESH first-interior id (B056).
                // Reverse Shift+Tab from the command field similarly latches to
                // the LAST interior. Everything else is egui-native.
                // egui-native traversal handles the interior order, interior ->
                // menu bar, and the menu-bar order. The shell handles the moves
                // egui gets wrong on its own (B056):
                //  - FORWARD from the command field: egui would stop on the
                //    SCROLL field (chrome) next, so latch to the FRESH
                //    first-interior id instead.
                //  - FORWARD from the last menu button: egui wraps to the first
                //    focusable of the frame (the first menu button, since the
                //    menu bar renders first), NOT the command field -- so wrap to
                //    the command field explicitly.
                //  - REVERSE from the command field: jump to the last menu button.
                //  - REVERSE from the first menu button: jump to the last interior
                //    (latch) or the command field when there is no interior.
                // Menu-bar button ids are reliable (they round-trip through
                // egui focus); the interior uses the fresh-id latch.
                let decision = if !shift {
                    if cmd_has_focus && first_interior.is_some() {
                        Boundary::FirstInterior
                    } else if on_menu_last {
                        Boundary::CommandField
                    } else {
                        Boundary::None
                    }
                } else if cmd_has_focus {
                    menu_last_id.map(Boundary::Focus).unwrap_or(Boundary::None)
                } else if on_menu_first {
                    if last_interior.is_some() {
                        Boundary::LastInterior
                    } else {
                        Boundary::CommandField
                    }
                } else {
                    Boundary::None
                };
                let _ = (menu_first_id, on_last_interior, on_first_interior);
                // Consume Tab only when the shell handles this boundary; otherwise
                // leave it for egui-native interior/menu-bar traversal
                // (Req 16.4, 16.6, 16.14).
                if !matches!(decision, Boundary::None) {
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
                decision
            });

            // Validates: Requirement 20.1 (file-tree-panel) -- Escape exits the
            // explorer tree back to the command field.
            if !self.modal_open
                && is_file_explorer
                && self.nav_focused
                && ctx.input(|i| i.key_pressed(egui::Key::Escape))
            {
                self.nav_focused = false;
                self.nav_selection.cursor = None;
                self.focus.command_field_focus_requested = true;
            } else if is_explorer_transfer
                && ctx.input_mut(|i| {
                    // Modern explorer Tab focus-transfer (Req 20.1 / 24.9):
                    // consume Tab in its own branch and move the tree cursor.
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
                        true
                    } else {
                        false
                    }
                })
            {
                use crate::explorer_view::{first_row_id, next_row_id};
                let next = if !self.nav_focused {
                    self.nav_focused = true;
                    first_row_id(&self.nav_model)
                } else {
                    self.nav_selection
                        .cursor
                        .and_then(|c| next_row_id(&self.nav_model, c))
                };
                match next {
                    Some(id) => self.nav_selection.move_cursor(id),
                    None => {
                        self.nav_focused = false;
                        self.nav_selection.cursor = None;
                        self.focus.command_field_focus_requested = true;
                    }
                }
            } else {
                match boundary {
                    Boundary::Focus(id) => ctx.memory_mut(|m| m.request_focus(id)),
                    Boundary::FirstInterior => {
                        self.focus.focus_first_interior_requested = true;
                        self.focus.focus_last_interior_requested = false;
                    }
                    Boundary::LastInterior => {
                        self.focus.focus_last_interior_requested = true;
                        self.focus.focus_first_interior_requested = false;
                    }
                    Boundary::CommandField => self.focus.command_field_focus_requested = true,
                    Boundary::None => {}
                }
            }
        }
    }
}
