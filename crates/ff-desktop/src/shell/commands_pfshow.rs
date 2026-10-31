//! # Shell Command Dispatch -- PFSHOW (key-label-bar) handler
//!
//! The key-label-bar `PFSHOW` command (CR-CH-046). Split out of
//! `commands_scrm.rs` to keep that file under the 400-line limit
//! (`rust-standards.md`); pure code movement, no behaviour change. The method is
//! on `WorkbenchShell` and keeps its `pub(super)` visibility.

use super::WorkbenchShell;

impl WorkbenchShell {
    /// Handle the PFSHOW command (CR-CH-046).
    ///
    /// `arg` is the uppercased, trimmed argument after `PFSHOW` (empty for the
    /// bare form). Bare PFSHOW cycles the five-state mode
    /// `Off -> Base -> Shift -> Ctrl -> Alt -> Off`; the scope keywords jump
    /// directly to a scope and show the bar; `ON`/`OFF` toggle visibility; an
    /// unrecognised argument leaves the mode unchanged and sets a non-fatal error.
    ///
    /// Validates: Requirement 12.1, 12.2, 12.3, 12.6, 12.7, 12.8, 12.9, 12.13
    pub(super) fn handle_pfshow(&mut self, arg: &str) {
        use super::KeyBarScope;
        match arg {
            // Bare PFSHOW: advance the five-state cycle.
            "" => {
                if !self.key_bar_visible {
                    // Off -> Base
                    self.key_bar_visible = true;
                    self.key_bar_scope = KeyBarScope::Base;
                } else {
                    match self.key_bar_scope {
                        KeyBarScope::Base => self.key_bar_scope = KeyBarScope::Shift,
                        KeyBarScope::Shift => self.key_bar_scope = KeyBarScope::Ctrl,
                        KeyBarScope::Ctrl => self.key_bar_scope = KeyBarScope::Alt,
                        // Alt -> Off (bar hidden; scope retained for PFSHOW ON).
                        KeyBarScope::Alt => self.key_bar_visible = false,
                    }
                }
                self.open_error = None;
            }
            // PFSHOW ON: show at the current-or-retained scope (12.1).
            "ON" => {
                self.key_bar_visible = true;
                self.open_error = None;
            }
            // PFSHOW OFF: hide (12.2). Scope retained.
            "OFF" => {
                self.key_bar_visible = false;
                self.open_error = None;
            }
            // Explicit scope jump (12.9).
            _ => {
                if let Some(scope) = KeyBarScope::parse(arg) {
                    self.key_bar_scope = scope;
                    self.key_bar_visible = true;
                    self.open_error = None;
                } else {
                    // Unknown argument (12.13): leave the mode unchanged, surface a
                    // clear non-fatal error.
                    self.open_error = Some(ff_i18n::t_args(
                        "ff-desktop-shell-msg-pfshow-bad-arg",
                        &ff_i18n::MessageArgs::new().set("arg", arg.to_string()),
                    ));
                }
            }
        }
    }
}
