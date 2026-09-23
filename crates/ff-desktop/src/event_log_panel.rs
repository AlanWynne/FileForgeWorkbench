//! Event Log Panel -- persistent log of all notifications.
//!
//! Validates: notification-system Requirement 2

use eframe::egui;
use std::sync::{Arc, Mutex};

use crate::notification::{NotificationLevel, NotificationQueue};

/// State for the Event Log panel.
///
/// Validates: notification-system Requirement 2.2
pub struct EventLogPanelState {
    /// Level filter: None = show all.
    pub level_filter: Option<NotificationLevel>,
    /// Text search filter.
    pub text_filter: String,
    /// Index of the selected entry (for detail view).
    pub selected_index: Option<usize>,
    /// Set to true when the user clicks Clear Log.
    pub clear_requested: bool,
    /// FIRST interior Tab stop (the level-filter combo), captured fresh each
    /// frame by the `WorkspaceContext` render and reported to the shell
    /// Boundary_Policy (CR-NR-078 WF.6, CR-CH-023 B059).
    pub first_interior_id: Option<egui::Id>,
}

impl EventLogPanelState {
    pub fn new() -> Self {
        Self {
            level_filter: None,
            text_filter: String::new(),
            selected_index: None,
            clear_requested: false,
            first_interior_id: None,
        }
    }
}

impl Default for EventLogPanelState {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the Event Log panel.
///
/// The caller must check `state.clear_requested` after this call and clear the queue.
/// Validates: notification-system Requirement 2.1-2.6
pub fn render(
    ui: &mut egui::Ui,
    state: &mut EventLogPanelState,
    queue: &Arc<Mutex<NotificationQueue>>,
) -> Option<egui::Id> {
    let queue_guard = queue.lock().expect("notification queue lock");
    // FIRST interior control (CR-CH-023, B059): the level-filter combo. Captured
    // from the combo's fresh response id each frame and returned to the shell so
    // the command-field -> first-interior Tab jump latches to a real widget.
    let mut first_interior: Option<egui::Id> = None;

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Event Log").monospace().strong());
        ui.separator();
        // Level filter -- Validates: Req 2.4
        let combo = egui::ComboBox::from_id_salt("event_log_level_filter")
            .selected_text(match state.level_filter {
                None => "All",
                Some(NotificationLevel::Info) => "Info",
                Some(NotificationLevel::Success) => "Success",
                Some(NotificationLevel::Warning) => "Warning",
                Some(NotificationLevel::Error) => "Error",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut state.level_filter, None, "All");
                ui.selectable_value(
                    &mut state.level_filter,
                    Some(NotificationLevel::Info),
                    "Info",
                );
                ui.selectable_value(
                    &mut state.level_filter,
                    Some(NotificationLevel::Success),
                    "Success",
                );
                ui.selectable_value(
                    &mut state.level_filter,
                    Some(NotificationLevel::Warning),
                    "Warning",
                );
                ui.selectable_value(
                    &mut state.level_filter,
                    Some(NotificationLevel::Error),
                    "Error",
                );
            });
        // The level-filter combo is the first interior Tab stop (B059).
        first_interior = Some(combo.response.id);
        ui.add(
            egui::TextEdit::singleline(&mut state.text_filter)
                .desired_width(150.0)
                .hint_text("search..."),
        );
        // Validates: Req 2.6
        if ui.small_button("Clear Log").clicked() {
            state.clear_requested = true;
        }
    });
    ui.separator();

    let text_lower = state.text_filter.to_lowercase();
    let entries: Vec<(usize, _)> = queue_guard
        .entries()
        .iter()
        .enumerate()
        .filter(|(_, n)| {
            if let Some(lvl) = state.level_filter {
                if n.level != lvl {
                    return false;
                }
            }
            if !text_lower.is_empty() && !n.title.to_lowercase().contains(&text_lower) {
                return false;
            }
            true
        })
        .collect();

    if entries.is_empty() {
        ui.label(egui::RichText::new("No log entries.").weak().italics());
        return first_interior;
    }

    // Validates: Req 2.3 -- reverse-chronological (queue already newest-first)
    egui::ScrollArea::vertical()
        .id_salt("event_log_scroll")
        .max_height(300.0)
        .show(ui, |ui| {
            for (orig_idx, n) in &entries {
                let is_selected = state.selected_index == Some(*orig_idx);
                let label = format!("[{}] {} {}", n.timestamp, n.level.label(), n.title);
                let colour = level_colour(n.level);
                let resp = ui.selectable_label(
                    is_selected,
                    egui::RichText::new(&label)
                        .monospace()
                        .color(colour)
                        .small(),
                );
                if resp.clicked() {
                    state.selected_index = Some(*orig_idx);
                }
            }
        });

    // Detail area -- Validates: Req 2.5
    if let Some(idx) = state.selected_index {
        if let Some(n) = queue_guard.entries().get(idx) {
            ui.separator();
            ui.label(egui::RichText::new(&n.title).strong());
            if let Some(detail) = &n.detail {
                ui.label(detail.as_str());
            }
        }
    }
    first_interior
}

fn level_colour(level: NotificationLevel) -> egui::Color32 {
    match level {
        NotificationLevel::Info => egui::Color32::from_rgb(0x21, 0x96, 0xF3),
        NotificationLevel::Success => egui::Color32::from_rgb(0x4C, 0xAF, 0x50),
        NotificationLevel::Warning => egui::Color32::from_rgb(0xFF, 0x98, 0x00),
        NotificationLevel::Error => egui::Color32::from_rgb(0xF4, 0x43, 0x36),
    }
}

/// `WorkspaceContext` impl (CR-NR-078 WF.6): render the Event Log against the
/// shared notification queue, apply a pending Clear-Log request in place (via
/// `services.notifications`, identical to the pre-framework shell handling), and
/// report the level-filter combo as the FIRST and LAST interior control.
///
/// Validates: workspace-framework Requirement 1.4, 1.5, 6.1;
/// notification-system Requirement 2.6.
impl crate::shell::workspace_context::WorkspaceContext for EventLogPanelState {
    fn render(
        &mut self,
        ui: &mut egui::Ui,
        services: &mut crate::shell::workspace_context::ShellServices<'_>,
    ) -> crate::shell::workspace_context::InteriorFocus {
        let first_interior = render(ui, self, services.notifications);
        self.first_interior_id = first_interior;
        if self.clear_requested {
            self.clear_requested = false;
            services
                .notifications
                .lock()
                .expect("notification queue lock")
                .clear();
        }
        crate::shell::workspace_context::InteriorFocus {
            first: first_interior,
            last: first_interior,
        }
    }
}
