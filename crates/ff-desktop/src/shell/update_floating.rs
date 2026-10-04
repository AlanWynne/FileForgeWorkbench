//! # Detached_Workspace (floating tab) viewport rendering
//!
//! The floating-tab viewport loop of the eframe `update()` loop. Extracted
//! verbatim from `update.rs` as part of the file-size split; behaviour and
//! order are unchanged -- `update` calls this in the same place it ran inline.

use eframe::egui;

use super::WorkbenchShell;

impl WorkbenchShell {
    /// Floating tab viewports -- Validates: Requirement 18.1, 18.2, 18.5, 18.8
    ///
    /// CR-CH-035 (B045): render each Detached_Workspace with an IMMEDIATE
    /// viewport (synchronous, so the closure can borrow `&mut self`), drawing
    /// the tab's REAL Context via `render_active_tab_body` -- not a placeholder.
    /// Extracted verbatim from `update()`.
    pub(super) fn render_floating_tabs(&mut self, ctx: &egui::Context) {
        // CR-CH-035 (B045): render each Detached_Workspace with an IMMEDIATE
        // viewport (synchronous, so the closure can borrow `&mut self`), drawing
        // the tab's REAL Context via `render_active_tab_body` -- not a placeholder.
        // The detached tab is temporarily made the active tab for the duration of
        // its render, then the previous active index is restored, so the shared
        // render path (which operates on the active tab) draws the correct tab
        // without duplicating the whole `match tab.kind`. Detached tabs are
        // `is_floating`, so the primary tab bar never shows them as active; there
        // is no double-render of the same tab in one frame.
        for ft_idx in 0..self.detach_split.floating_tabs.len() {
            let vid = self.detach_split.floating_tabs[ft_idx].viewport_id;
            let tab_id = self.detach_split.floating_tabs[ft_idx].tab_id;
            // Resolve the live index from the stable id each frame.
            let Some(tab_index) = self.tabs.index_of_id(tab_id) else {
                continue;
            };
            let title = self
                .tabs
                .tabs()
                .get(tab_index)
                .map(|t| {
                    super::truncate_title(
                        &format!("{} -- FileForge Workbench", super::title_line_text(t)),
                        80,
                    )
                })
                .unwrap_or_else(|| "FileForge Workbench".to_string());
            // CR-CH-036 (Req 18.10): move THIS window's independent command
            // context out so the render closure can borrow it (and `&mut self`)
            // without aliasing `self.detach_split.floating_tabs`; put it back after the frame.
            let mut cmd_ctx = std::mem::take(&mut self.detach_split.floating_tabs[ft_idx].cmd_ctx);
            ctx.show_viewport_immediate(
                vid,
                egui::ViewportBuilder::default().with_title(&title),
                |vctx, class| {
                    // CR-CH-035: if the backend does not support multiple
                    // viewports, egui reports the child as an Embedded class and
                    // no separate OS window appears. Surface that instead of
                    // failing silently (the headless test harness reports
                    // Embedded; the real glow/wgpu backend reports Immediate).
                    if class != egui::ViewportClass::Immediate {
                        ff_logging::log_warn!(
                            "[shell] Detached_Workspace viewport is not Immediate (Embedded/\
                             Deferred) -- this backend may not support multiple OS windows"
                        );
                    }
                    if tab_index >= self.tabs.len() {
                        return;
                    }
                    // CR-CH-037: the OS-window Close button behaves as RETURN on
                    // THIS window's Context (not redock). A non-POM detached
                    // workspace returns to its POM (window stays); a detached POM
                    // closes the workspace (Option A). Detect the close request
                    // here and apply it via `nav_return` inside the swap below.
                    let close_requested = vctx.input(|i| i.viewport().close_requested());
                    // CR-CH-036 (Req 18.10): render + dispatch under this window's
                    // INDEPENDENT command context (its own active tab + command
                    // line). The whole existing pipeline runs against the detached
                    // tab. CR-CH-037/B068: the Close button and F-keys also apply
                    // here so RETURN/END act on the detached tab.
                    self.with_workspace_context(tab_index, &mut cmd_ctx, |shell| {
                        // CR-CH-037: window Close == RETURN on this Context.
                        if close_requested {
                            shell.nav_return();
                        }
                        // B068 (Req 18.11): dispatch F-keys pressed while THIS
                        // detached window has focus, against this window's Context
                        // (identical resolution to the Primary_Window path).
                        shell.dispatch_detached_function_key(vctx);
                        // CR-NR-089 (Req 18.12): this window's own Menu_Bar.
                        shell.render_detached_menu_bar(vctx, tab_id);
                        // Title_Line (read-only chrome, Req 18.1).
                        let detached_title = shell.kind_title(shell.tabs.active_tab());
                        egui::TopBottomPanel::top(egui::Id::new(("floating_title", tab_id.0)))
                            .show(vctx, |ui| {
                                ui.label(egui::RichText::new(detached_title).monospace().strong());
                            });
                        // This window's OWN Command ===> field (Req 18.2/18.10),
                        // ids salted per tab so they never collide with the
                        // Primary_Window's or another detached window's field.
                        shell.render_detached_command_field(vctx, tab_id);
                        // The tab's REAL Context body (Req 18.8).
                        egui::CentralPanel::default().show(vctx, |ui| {
                            shell.render_active_tab_body(vctx, ui);
                        });
                    });
                    // After RETURN: if the tab still exists (RETURN navigated it
                    // to its POM), keep the window open by cancelling the close;
                    // if it is gone (RETURN closed the workspace), let the OS
                    // window close (the FloatingTab is dropped below next frame).
                    if close_requested && self.tabs.index_of_id(tab_id).is_some() {
                        vctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                    }
                },
            );
            // Restore this window's context back onto the FloatingTab.
            if let Some(ft) = self.detach_split.floating_tabs.get_mut(ft_idx) {
                ft.cmd_ctx = cmd_ctx;
            }
        }
    }
}
