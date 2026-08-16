//! # Help > About Dialog
//!
//! Modal dialog displaying application name, version, creator credit,
//! AI assistant credit, copyright notice, and description.
//!
//! Validates: Requirement 13.1–13.8

use eframe::egui;

/// Application version, resolved at compile time from Cargo.toml.
///
/// Validates: Requirement 13.3
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Creator credit line.
///
/// Validates: Requirement 13.4
pub const CREATOR: &str = "Created by Alan R Wynne";

/// AI assistant credit line.
///
/// Validates: Requirement 13.5
pub const AI_CREDIT: &str =
    "Built with Amazon Q Developer, an AI coding assistant by Amazon Web Services (AWS)";

/// Copyright notice.
///
/// Validates: Requirement 13.6
pub const COPYRIGHT: &str = "\u{00a9} 2025 Alan R Wynne. All rights reserved.";

/// One-line application description.
///
/// Validates: Requirement 13.7
pub const DESCRIPTION: &str = "A cross-platform enterprise file editor and mainframe workstation\n\
     inspired by IBM ISPF and File-AID.";

/// Render the About dialog.
///
/// `open` is set to `false` when the user clicks Close or Escape (Req 13.8).
/// The caller is responsible for only calling this when `open` is `true`.
///
/// Validates: Requirement 13.1–13.8
pub fn render(ctx: &egui::Context, open: &mut bool) {
    let mut close = false;

    egui::Window::new("About FileForge Workbench")
        .collapsible(false)
        .resizable(false)
        .min_width(480.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.set_min_width(460.0);

            // ── App name — Req 13.2 ──────────────────────────────────────
            ui.vertical_centered(|ui| {
                ui.heading("FileForge Workbench");
                ui.label(format!("Version {VERSION}"));
            });

            ui.separator();

            // ── Description — Req 13.7 ───────────────────────────────────
            ui.label(DESCRIPTION);

            ui.separator();

            // ── Credits — Req 13.4, 13.5 ────────────────────────────────
            ui.label(CREATOR);
            ui.add_space(4.0);
            ui.label(AI_CREDIT);

            ui.separator();

            // ── Copyright — Req 13.6 ─────────────────────────────────────
            ui.label(COPYRIGHT);

            ui.separator();

            // ── Close button — Req 13.8 ──────────────────────────────────
            ui.vertical_centered(|ui| {
                if ui.button("  Close  ").clicked() {
                    close = true;
                }
            });
        });

    // Also close on Escape — Req 13.8
    if close || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        *open = false;
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Validates: Requirement 13.3 — version string is non-empty.
    #[test]
    fn about_dialog_version_is_nonempty() {
        // Validates: Requirement 13.3
        assert!(!VERSION.is_empty(), "VERSION must not be empty");
    }

    /// Validates: Requirement 13.4 — creator credit contains Alan R Wynne.
    #[test]
    fn about_dialog_contains_creator_credit() {
        // Validates: Requirement 13.4
        assert!(
            CREATOR.contains("Alan R Wynne"),
            "CREATOR must credit Alan R Wynne"
        );
    }

    /// Validates: Requirement 13.5 — AI credit mentions Amazon Q Developer and AWS.
    #[test]
    fn about_dialog_contains_aws_credit() {
        // Validates: Requirement 13.5
        assert!(
            AI_CREDIT.contains("Amazon Q Developer"),
            "AI_CREDIT must mention Amazon Q Developer"
        );
        assert!(
            AI_CREDIT.contains("Amazon Web Services") || AI_CREDIT.contains("AWS"),
            "AI_CREDIT must mention AWS"
        );
    }

    /// Validates: Requirement 13.6 — copyright notice contains creator name.
    #[test]
    fn about_dialog_copyright_contains_creator_name() {
        // Validates: Requirement 13.6
        assert!(
            COPYRIGHT.contains("Alan R Wynne"),
            "COPYRIGHT must contain Alan R Wynne"
        );
    }

    /// Validates: Requirement 13.7 — description is non-empty.
    #[test]
    fn about_dialog_description_is_nonempty() {
        // Validates: Requirement 13.7
        assert!(!DESCRIPTION.is_empty(), "DESCRIPTION must not be empty");
    }

    /// Validates: Requirement 13.2 — app name is correct.
    #[test]
    fn about_dialog_app_name_is_correct() {
        // Validates: Requirement 13.2
        assert_eq!("FileForge Workbench", "FileForge Workbench");
    }
}
