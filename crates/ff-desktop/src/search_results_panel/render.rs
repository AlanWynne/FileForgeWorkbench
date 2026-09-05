//! Rendering for the Global Search Results panel.
//!
//! Addresses: global-search Requirement 1, 2, 3, 4, 5

use eframe::egui;

use super::state::{ReplaceConfirm, SearchPhase, SearchResultsPanelState};

/// Outcome returned from `render()` each frame.
pub enum SearchPanelOutcome {
    /// Nothing happened.
    None,
    /// User clicked a match -- open file at this path and line.
    OpenMatch { path: String, line: u64 },
    /// User confirmed Replace All -- apply replacement.
    ReplaceAll,
    /// User cancelled a running search.
    Cancel,
}

/// Render the Search Results panel into the current egui UI.
///
/// `roots` is the list of directories to search (workspace roots or native catalogs).
/// Returns an outcome the shell should act on.
///
/// Addresses: Requirement 1.1, 2.1, 3.1, 4.1, 5.1
pub fn render(
    ui: &mut egui::Ui,
    state: &mut SearchResultsPanelState,
    roots: &[String],
    runtime: &tokio::runtime::Runtime,
) -> SearchPanelOutcome {
    let mut outcome = SearchPanelOutcome::None;

    // Poll background task events every frame while running.
    if matches!(state.phase, SearchPhase::Running { .. }) {
        state.poll_events();
        ui.ctx().request_repaint();
    }

    // ── Search input row ──────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.label("Search:");
        let resp = ui.add(
            egui::TextEdit::singleline(&mut state.query)
                .hint_text("Search across files...")
                .desired_width(280.0),
        );
        // Trigger search on Enter.
        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            start_search(state, roots, runtime);
        }

        // History dropdown button.
        if ui
            .small_button("v")
            .on_hover_text("Search history")
            .clicked()
        {
            state.history_open = !state.history_open;
        }

        if ui.button("Search").clicked() {
            start_search(state, roots, runtime);
        }

        // Cancel button while running.
        if matches!(state.phase, SearchPhase::Running { .. })
            && ui.button("Cancel").clicked()
        {
            if let SearchPhase::Running { ref cancel, .. } = state.phase {
                cancel.cancel();
            }
            outcome = SearchPanelOutcome::Cancel;
        }
    });

    // History dropdown.
    if state.history_open && !state.history.is_empty() {
        egui::Frame::popup(ui.style()).show(ui, |ui| {
            let mut selected: Option<String> = None;
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for entry in &state.history {
                        if ui
                            .selectable_label(false, egui::RichText::new(entry).monospace())
                            .clicked()
                        {
                            selected = Some(entry.clone());
                        }
                    }
                });
            if let Some(q) = selected {
                state.query = q;
                state.history_open = false;
            }
        });
    }

    // ── Options row ───────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.checkbox(&mut state.options.case_sensitive, "Aa")
            .on_hover_text("Case sensitive");
        ui.checkbox(&mut state.options.whole_word, "W")
            .on_hover_text("Whole word");
        ui.checkbox(&mut state.options.use_regex, ".*")
            .on_hover_text("Use regex");
        ui.label("Include:");
        ui.add(
            egui::TextEdit::singleline(&mut state.options.include_globs)
                .hint_text("**/*.rs")
                .desired_width(100.0),
        );
        ui.label("Exclude:");
        ui.add(
            egui::TextEdit::singleline(&mut state.options.exclude_globs)
                .hint_text("**/target/**")
                .desired_width(100.0),
        );
    });

    // Inline regex error.
    if let Some(ref err) = state.regex_error.clone() {
        ui.colored_label(egui::Color32::RED, err);
    }

    // ── Replace row ───────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        let expand_label = if state.replace_expanded {
            "v Replace"
        } else {
            "> Replace"
        };
        if ui.small_button(expand_label).clicked() {
            state.replace_expanded = !state.replace_expanded;
        }
        if state.replace_expanded {
            ui.add(
                egui::TextEdit::singleline(&mut state.replace_text)
                    .hint_text("Replacement text")
                    .desired_width(280.0),
            );
            let has_results = !state.results.is_empty();
            ui.add_enabled_ui(has_results, |ui| {
                if ui.button("Replace All").clicked() {
                    let file_count = state.results.len();
                    let match_count = state.total_match_count();
                    state.replace_confirm = ReplaceConfirm::Pending {
                        file_count,
                        match_count,
                    };
                }
            });
        }
    });

    // ── Replace confirmation dialog ───────────────────────────────────────
    if let ReplaceConfirm::Pending {
        file_count,
        match_count,
    } = &state.replace_confirm
    {
        let fc = *file_count;
        let mc = *match_count;
        egui::Window::new("Confirm Replace All")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label(format!("Replace {mc} occurrence(s) in {fc} file(s)?"));
                ui.horizontal(|ui| {
                    if ui.button("Replace").clicked() {
                        state.replace_confirm = ReplaceConfirm::None;
                        outcome = SearchPanelOutcome::ReplaceAll;
                    }
                    if ui.button("Cancel").clicked() {
                        state.replace_confirm = ReplaceConfirm::None;
                    }
                });
            });
    }

    // ── Status bar ────────────────────────────────────────────────────────
    match &state.phase {
        SearchPhase::Idle => {}
        SearchPhase::Running {
            files_scanned,
            matches_found,
            ..
        } => {
            ui.label(format!(
                "Searching... {files_scanned} files scanned, {matches_found} matches"
            ));
        }
        SearchPhase::Done {
            total_files,
            total_matches,
        } => {
            ui.label(format!("{total_matches} matches in {total_files} files"));
        }
        SearchPhase::Cancelled => {
            ui.label("Search cancelled.");
        }
    }

    // ── Results list ──────────────────────────────────────────────────────
    egui::ScrollArea::vertical().show(ui, |ui| {
        let mut open_match: Option<(String, u64)> = None;

        for (fi, fm) in state.results.iter().enumerate() {
            let is_collapsed = state.collapsed_files.contains(&fm.file_path);
            let file_name = std::path::Path::new(&fm.file_path)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| fm.file_path.clone());
            let header_label = format!(
                "{} ({} match{})",
                file_name,
                fm.matches.len(),
                if fm.matches.len() == 1 { "" } else { "es" }
            );

            // File section header -- click to collapse/expand.
            let header_resp = ui.selectable_label(
                false,
                egui::RichText::new(if is_collapsed {
                    format!("> {header_label}")
                } else {
                    format!("v {header_label}")
                })
                .strong(),
            );
            if header_resp.clicked() {
                if is_collapsed {
                    state.collapsed_files.remove(&fm.file_path);
                } else {
                    state.collapsed_files.insert(fm.file_path.clone());
                }
            }

            if !is_collapsed {
                for (mi, sr) in fm.matches.iter().enumerate() {
                    let is_selected = state.selected == Some((fi, mi));
                    let row_text = format!("  {:>5}: {}", sr.line_number, sr.line_text.trim_end());
                    let resp = ui.selectable_label(
                        is_selected,
                        egui::RichText::new(&row_text).monospace().small(),
                    );
                    if resp.clicked() {
                        state.selected = Some((fi, mi));
                        open_match = Some((fm.file_path.clone(), sr.line_number));
                    }
                }
            }
        }

        if let Some((path, line)) = open_match {
            outcome = SearchPanelOutcome::OpenMatch { path, line };
        }
    });

    // ── Keyboard navigation ───────────────────────────────────────────────
    if ui.ctx().input(|i| i.key_pressed(egui::Key::ArrowDown)) {
        state.select_next();
    }
    if ui.ctx().input(|i| i.key_pressed(egui::Key::ArrowUp)) {
        state.select_prev();
    }
    if ui.ctx().input(|i| i.key_pressed(egui::Key::Enter)) {
        if let Some((fm, sr)) = state.selected_result() {
            outcome = SearchPanelOutcome::OpenMatch {
                path: fm.file_path.clone(),
                line: sr.line_number,
            };
        }
    }

    outcome
}

/// Spawn a background search task and transition the panel to Running.
///
/// Addresses: Requirement 3.1, 3.3
fn start_search(
    state: &mut SearchResultsPanelState,
    roots: &[String],
    runtime: &tokio::runtime::Runtime,
) {
    if state.query.is_empty() {
        return;
    }
    let roots_vec: Vec<String> = roots.to_vec();
    let request = match state.build_request(roots_vec) {
        Ok(r) => r,
        Err(_) => return,
    };

    // Record in history -- Addresses: Requirement 6.1
    state.push_history(&state.query.clone());

    // Clear previous results -- Addresses: Requirement 4.6
    state.results.clear();
    state.selected = None;
    state.collapsed_files.clear();

    let (tx, rx) = tokio::sync::mpsc::channel(256);
    let cancel = ff_global_search::CancellationToken::new();
    let cancel_clone = cancel.clone();

    runtime.spawn(async move {
        let _ = ff_global_search::GlobalSearchEngine::search(request, tx, cancel_clone).await;
    });

    state.phase = SearchPhase::Running {
        receiver: rx,
        cancel,
        files_scanned: 0,
        matches_found: 0,
    };
}
