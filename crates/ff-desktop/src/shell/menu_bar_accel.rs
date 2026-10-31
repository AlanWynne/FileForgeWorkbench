//! Mnemonic-letter menu-bar accelerators (screen-snapshot-scrm Requirement 26,
//! CR-NR-114).
//!
//! Split out of `render_chrome.rs` (file-size rule). Each bar-visible option of
//! the active POM-echo bar has an Alt+<letter> accelerator derived by
//! `render_menu_bar::menu_bar_mnemonics`. When the accelerator fires, the bar
//! item's command is dispatched through the single front door
//! (`dispatch_command_string`) -- the SAME seam a click uses (Req 26.5) -- so a
//! submenu option (SCRM, Settings) opens its menu and a direct action (Help)
//! runs. No new accelerator mechanism is introduced (Req 26.2).

use eframe::egui;

use super::WorkbenchShell;

impl WorkbenchShell {
    /// Handle the mnemonic-letter menu-bar accelerators (screen-snapshot-scrm
    /// Requirement 26, CR-NR-114).
    ///
    /// Resolves the active POM-echo bar, derives its Alt+<letter> mnemonics
    /// (`menu_bar_mnemonics`), and when Alt+<letter> is pressed this frame,
    /// dispatches that bar item's command through the single front door
    /// (`dispatch_command_string`). Suppressed while a modal dialog is open; the
    /// Alt event is CONSUMED so it neither double-fires nor leaks to another Alt
    /// consumer.
    ///
    /// Validates: screen-snapshot-scrm Requirement 26.1, 26.2, 26.3, 26.4, 26.5
    /// (CR-NR-114)
    pub(super) fn handle_menu_bar_accelerators(&mut self, ctx: &egui::Context) {
        if self.modal_open {
            return;
        }
        let bar = self.resolve_menu_bar_menu();
        let mnemonics = super::render_menu_bar::menu_bar_mnemonics(&bar.options);
        // Find the first mnemonic whose Alt+<letter> is pressed this frame, then
        // CONSUME that key so it does not double-fire or leak to other consumers.
        let mut fire: Option<String> = None;
        for (letter, command) in &mnemonics {
            let Some(key) = letter_to_key(*letter) else {
                continue;
            };
            let pressed = ctx.input_mut(|i| {
                i.consume_key(egui::Modifiers::ALT, key)
                    || i.consume_key(egui::Modifiers::ALT | egui::Modifiers::SHIFT, key)
            });
            if pressed {
                fire = Some(command.clone());
                break;
            }
        }
        if let Some(command) = fire {
            self.dispatch_command_string(&command);
        }
    }
}

/// Map an ASCII letter `A`-`Z` to its `egui::Key` (for the Alt+<letter> menu-bar
/// accelerators, screen-snapshot-scrm Req 26). Returns `None` for any
/// non-letter. An explicit match avoids relying on egui key-name parsing.
fn letter_to_key(letter: char) -> Option<egui::Key> {
    use egui::Key;
    Some(match letter.to_ascii_uppercase() {
        'A' => Key::A,
        'B' => Key::B,
        'C' => Key::C,
        'D' => Key::D,
        'E' => Key::E,
        'F' => Key::F,
        'G' => Key::G,
        'H' => Key::H,
        'I' => Key::I,
        'J' => Key::J,
        'K' => Key::K,
        'L' => Key::L,
        'M' => Key::M,
        'N' => Key::N,
        'O' => Key::O,
        'P' => Key::P,
        'Q' => Key::Q,
        'R' => Key::R,
        'S' => Key::S,
        'T' => Key::T,
        'U' => Key::U,
        'V' => Key::V,
        'W' => Key::W,
        'X' => Key::X,
        'Y' => Key::Y,
        'Z' => Key::Z,
        _ => return None,
    })
}
