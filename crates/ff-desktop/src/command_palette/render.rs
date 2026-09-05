//! Command Palette rendering.
//!
//! Renders the centered modal overlay, search input, entry list with
//! match highlighting, detail area, recently-used section, and empty state.
//!
//! Validates: Requirement 1.1, 2.6, 3.1-3.4, 4.5, 5.1 (command-palette)

use std::sync::Arc;

use eframe::egui;

use super::fuzzy::{fuzzy_match, fuzzy_score};
use super::state::{CommandPaletteState, PaletteEntry};

const MAX_VISIBLE: usize = 20;

/// Outcome returned by `render_command_palette` each frame.
#[derive(Debug, PartialEq)]
pub enum PaletteOutcome {
    /// No action this frame.
    None,
    /// User executed the given command ID.
    Execute(String),
    /// User dismissed the palette without executing.
    Dismissed,
}

/// Render the Command Palette overlay.
///
/// `all_entries` is the full list of available commands (pre-built each frame
/// from the CommandRegistry). `recent` is the ordered list of recently-used
/// command IDs (most recent first).
///
/// Returns a `PaletteOutcome` indicating what happened this frame.
///
/// Validates: Requirement 1.1, 1.2, 1.3, 2.1, 2.6, 3.1-3.4, 4.1-4.5, 5.1
pub fn render_command_palette(
    ctx: &egui::Context,
    state: &mut CommandPaletteState,
    all_entries: &[PaletteEntry],
    recent: &[String],
) -> PaletteOutcome {
    if !state.open {
        return PaletteOutcome::None;
    }

    rebuild_filtered(state, all_entries, recent);

    let mut outcome = PaletteOutcome::None;
    let mut close_requested = false;

    // Escape closes -- Validates: Requirement 1.2
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        close_requested = true;
        outcome = PaletteOutcome::Dismissed;
    }

    // Up/Down navigation -- Validates: Requirement 4.3
    if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
        state.select_next();
    }
    if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
        state.select_prev();
    }

    // Enter executes -- Validates: Requirement 4.1
    if ctx.input(|i| i.key_pressed(egui::Key::Enter)) {
        if let Some(entry) = state.selected_entry() {
            if entry.enabled {
                let id = entry.command_id.clone();
                close_requested = true;
                outcome = PaletteOutcome::Execute(id);
            }
            // Disabled: Req 4.5 -- caller sets status message
        }
    }

    // Centered window -- Validates: Requirement 1.1
    let screen = ctx.screen_rect();
    let win_width = (screen.width() * 0.5).clamp(400.0, 700.0);
    let win_x = screen.center().x - win_width / 2.0;
    let win_y = screen.top() + screen.height() * 0.15;

    let mut clicked_outside = false;
    let window_resp = egui::Window::new("Command Palette")
        .id(egui::Id::new("command_palette_window"))
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .fixed_pos(egui::pos2(win_x, win_y))
        .fixed_size(egui::vec2(win_width, 0.0))
        .show(ctx, |ui| {
            render_palette_contents(ui, state, recent, &mut outcome, &mut close_requested);
        });

    // Click-outside detection -- Validates: Requirement 1.3
    if let Some(resp) = window_resp {
        if ctx.input(|i| i.pointer.any_click()) {
            if let Some(p) = ctx.input(|i| i.pointer.interact_pos()) {
                if !resp.response.rect.contains(p) {
                    clicked_outside = true;
                }
            }
        }
    }

    if clicked_outside {
        close_requested = true;
        if outcome == PaletteOutcome::None {
            outcome = PaletteOutcome::Dismissed;
        }
    }

    if close_requested {
        state.close();
    }

    outcome
}

fn render_palette_contents(
    ui: &mut egui::Ui,
    state: &mut CommandPaletteState,
    recent: &[String],
    outcome: &mut PaletteOutcome,
    close_requested: &mut bool,
) {
    // Search input -- Validates: Requirement 1.1
    let search_id = egui::Id::new("palette_search_input");
    let _resp = ui.add(
        egui::TextEdit::singleline(&mut state.query)
            .id(search_id)
            .hint_text("Type to search commands...")
            .desired_width(f32::INFINITY)
            .font(egui::TextStyle::Monospace),
    );
    if state.focus_search {
        state.focus_search = false;
        ui.memory_mut(|m| m.request_focus(search_id));
    }

    ui.separator();

    let query_empty = state.query.trim().is_empty();

    // Recently Used header -- Validates: Requirement 5.1
    if query_empty && !recent.is_empty() {
        ui.label(
            egui::RichText::new("Recently Used")
                .small()
                .color(egui::Color32::GRAY),
        );
    }

    let visible_count = state.filtered.len().min(MAX_VISIBLE);
    if visible_count == 0 && !query_empty {
        // Empty state -- Validates: Requirement 2.6
        ui.label(
            egui::RichText::new(format!("No commands match '{}'", state.query.trim()))
                .color(egui::Color32::GRAY),
        );
    } else {
        egui::ScrollArea::vertical()
            .max_height(320.0)
            .show(ui, |ui| {
                let entries: Vec<_> = state.filtered.iter().take(MAX_VISIBLE).cloned().collect();
                for (i, entry) in entries.iter().enumerate() {
                    let is_selected = i == state.selected_index;
                    if render_entry(ui, entry, is_selected, &state.query) && entry.enabled {
                        *outcome = PaletteOutcome::Execute(entry.command_id.clone());
                        *close_requested = true;
                    }
                    if is_selected {
                        ui.scroll_to_cursor(Some(egui::Align::Center));
                    }
                }
            });
    }

    // Detail area -- Validates: Requirement 3.2
    if let Some(entry) = state.selected_entry() {
        if !entry.description.is_empty() {
            ui.separator();
            ui.label(
                egui::RichText::new(&entry.description)
                    .small()
                    .color(egui::Color32::GRAY),
            );
        }
    }
}

/// Render one palette entry row. Returns true if clicked.
///
/// Validates: Requirement 3.1, 3.3, 4.5
fn render_entry(ui: &mut egui::Ui, entry: &PaletteEntry, is_selected: bool, query: &str) -> bool {
    let bg = if is_selected {
        ui.visuals().selection.bg_fill
    } else {
        egui::Color32::TRANSPARENT
    };

    let (rect, resp) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::click());

    if ui.is_rect_visible(rect) {
        ui.painter().rect_filled(rect, 2.0, bg);

        let name_color = if entry.enabled {
            ui.visuals().text_color()
        } else {
            egui::Color32::DARK_GRAY // Validates: Requirement 4.5
        };
        let cat_color = egui::Color32::GRAY;
        let name_pos = rect.left_top() + egui::vec2(4.0, 3.0);

        // Display name with match highlighting -- Validates: Requirement 3.3
        let name_galley = build_highlighted_text(ui, &entry.display_name, query, name_color);
        ui.painter().galley(name_pos, name_galley, name_color);

        // Category label -- Validates: Requirement 3.1
        let cat_galley = ui.fonts(|f| {
            f.layout_no_wrap(
                entry.category.clone(),
                egui::FontId::proportional(10.0),
                cat_color,
            )
        });
        let cat_x = name_pos.x + (rect.width() * 0.5).min(200.0);
        ui.painter()
            .galley(egui::pos2(cat_x, name_pos.y), cat_galley, cat_color);

        // Shortcut right-aligned -- Validates: Requirement 3.1
        if let Some(ref sc) = entry.shortcut {
            let sc_galley = ui.fonts(|f| {
                f.layout_no_wrap(
                    sc.clone(),
                    egui::FontId::monospace(10.0),
                    egui::Color32::GRAY,
                )
            });
            let sc_x = rect.right() - sc_galley.rect.width() - 4.0;
            ui.painter()
                .galley(egui::pos2(sc_x, name_pos.y), sc_galley, egui::Color32::GRAY);
        }
    }

    resp.clicked()
}

/// Build a galley with matched characters highlighted in yellow.
///
/// Validates: Requirement 3.3
fn build_highlighted_text(
    ui: &egui::Ui,
    text: &str,
    query: &str,
    base_color: egui::Color32,
) -> Arc<egui::Galley> {
    let q_lower: Vec<char> = query.chars().flat_map(|c| c.to_lowercase()).collect();
    let t_lower: Vec<char> = text.chars().flat_map(|c| c.to_lowercase()).collect();

    let mut match_pos: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut ti = 0;
    for &qc in &q_lower {
        if let Some(i) = (ti..t_lower.len()).find(|&i| t_lower[i] == qc) {
            match_pos.insert(i);
            ti = i + 1;
        }
    }

    if match_pos.is_empty() || query.is_empty() {
        return ui.fonts(|f| {
            f.layout_no_wrap(text.to_string(), egui::FontId::monospace(12.0), base_color)
        });
    }

    let mut job = egui::text::LayoutJob::default();
    for (i, ch) in text.chars().enumerate() {
        let color = if match_pos.contains(&i) {
            egui::Color32::YELLOW
        } else {
            base_color
        };
        job.append(
            &ch.to_string(),
            0.0,
            egui::text::TextFormat {
                font_id: egui::FontId::monospace(12.0),
                color,
                ..Default::default()
            },
        );
    }
    ui.fonts(|f| f.layout_job(job))
}

/// Rebuild `state.filtered` from `all_entries` based on the current query.
///
/// Validates: Requirement 2.1, 2.3, 2.4, 5.1, 5.3
fn rebuild_filtered(
    state: &mut CommandPaletteState,
    all_entries: &[PaletteEntry],
    recent: &[String],
) {
    let query = state.query.trim().to_string();

    if query.is_empty() {
        // Recent first, then all alphabetically -- Validates: Requirement 2.4, 5.1
        let mut result: Vec<PaletteEntry> = Vec::new();
        for id in recent {
            if let Some(e) = all_entries.iter().find(|e| &e.command_id == id) {
                result.push(e.clone());
            }
        }
        let recent_set: std::collections::HashSet<&str> =
            recent.iter().map(|s| s.as_str()).collect();
        let mut rest: Vec<PaletteEntry> = all_entries
            .iter()
            .filter(|e| !recent_set.contains(e.command_id.as_str()))
            .cloned()
            .collect();
        rest.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        result.extend(rest);
        state.filtered = result;
    } else {
        // Fuzzy filter + sort by score desc, alpha tiebreak -- Validates: Req 2.1, 2.3
        let mut scored: Vec<PaletteEntry> = all_entries
            .iter()
            .filter(|e| fuzzy_match(&query, &e.display_name) || fuzzy_match(&query, &e.command_id))
            .map(|e| {
                let s =
                    fuzzy_score(&query, &e.display_name).max(fuzzy_score(&query, &e.command_id));
                let mut entry = e.clone();
                entry.score = s;
                entry
            })
            .collect();
        scored.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.display_name.cmp(&b.display_name))
        });
        state.filtered = scored;
    }

    if state.filtered.is_empty() || state.selected_index >= state.filtered.len() {
        state.selected_index = 0;
    }
}
