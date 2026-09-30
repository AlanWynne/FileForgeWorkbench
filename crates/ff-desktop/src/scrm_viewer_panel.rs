//! SCRM Replay viewer Context (CR-NR-098, Wave 2).
//!
//! A Workspace Context that replays a captured screen Collection: First /
//! Previous / Next / Last navigation plus the current capture rendered as
//! SELECTABLE text (consistent with the text-first constraint, Requirement
//! 10.8). The shell stages the active Collection onto this panel before dispatch
//! (the same "stage data, then render" pattern the Search Results Context uses).
//!
//! Validates: screen-snapshot-scrm Requirement 10, 16.

use eframe::egui;
use ff_screen_model::{render as render_screen, RenderOptions, SnapshotFormat};
use ff_scrm::{ReplaySession, ScreenCollection};

/// Per-shell state for the SCRM Replay viewer Context.
#[derive(Default)]
pub struct ScrmViewerState {
    /// The Collection being replayed, staged by the shell before render.
    collection: Option<ScreenCollection>,
    /// The replay cursor over `collection`.
    replay: Option<ReplaySession>,
    /// The first interior control id reported to the shell Boundary_Policy.
    pub first_interior_id: Option<egui::Id>,
}

impl std::fmt::Debug for ScrmViewerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScrmViewerState")
            .field("has_collection", &self.collection.is_some())
            .field("position", &self.replay.as_ref().map(|r| r.position()))
            .finish()
    }
}

impl ScrmViewerState {
    /// Stage the Collection to replay (the shell calls this before dispatch each
    /// frame). Rebuilds the replay cursor only when the staged Collection changes
    /// identity, so navigation position is preserved across frames.
    pub fn set_collection(&mut self, collection: Option<ScreenCollection>) {
        let same = match (&self.collection, &collection) {
            (Some(a), Some(b)) => a.collection_id == b.collection_id && a.len() == b.len(),
            (None, None) => true,
            _ => false,
        };
        if !same {
            self.replay = collection.as_ref().map(ReplaySession::new);
        }
        self.collection = collection;
    }

    /// The current capture's screen rendered as plain selectable text, or a
    /// placeholder when the Collection is empty / absent.
    fn current_screen_text(&self) -> String {
        match (&self.collection, &self.replay) {
            (Some(c), Some(r)) => match r.current_index().and_then(|i| c.screens.get(i)) {
                Some(cap) => render_screen(
                    &cap.screen,
                    SnapshotFormat::PlainText,
                    RenderOptions::default(),
                ),
                None => "(empty collection)".to_string(),
            },
            _ => "(no collection to replay)".to_string(),
        }
    }
}

/// Render the SCRM viewer into `ui`, returning the id of the first interior
/// control (the First button) for the focus contract.
///
/// Validates: screen-snapshot-scrm Requirement 10.1-10.7, 16.2.
pub fn render(ui: &mut egui::Ui, state: &mut ScrmViewerState) -> Option<egui::Id> {
    let mut first_id: Option<egui::Id> = None;

    let (count, position) = match &state.replay {
        Some(r) => (r.len(), r.position()),
        None => (0, 0),
    };

    ui.horizontal(|ui| {
        // The FIRST interior control gets a STABLE id (workspace-conformance:
        // the first Tab from the command field must land here, no phantom stop).
        // We capture the button's fresh same-frame id and report it as `first`.
        let first = ui.button("|< First");
        first_id = Some(first.id);
        if first.clicked() {
            if let Some(r) = state.replay.as_mut() {
                r.first();
            }
        }
        if ui.button("< Prev").clicked() {
            if let Some(r) = state.replay.as_mut() {
                r.previous();
            }
        }
        if ui.button("Next >").clicked() {
            if let Some(r) = state.replay.as_mut() {
                r.next();
            }
        }
        if ui.button("Last >|").clicked() {
            if let Some(r) = state.replay.as_mut() {
                r.last();
            }
        }
    });

    // Position / timing line.
    if count > 0 {
        ui.label(format!("Capture {} of {}", position + 1, count));
    } else {
        ui.label("No captures");
    }

    ui.separator();

    // The current screen as SELECTABLE, copyable text (Req 10.8, 1.1).
    let text = state.current_screen_text();
    egui::ScrollArea::both().show(ui, |ui| {
        // A read-only multiline TextEdit keeps the content selectable/copyable.
        let mut buf = text;
        ui.add(
            egui::TextEdit::multiline(&mut buf)
                .font(egui::TextStyle::Monospace)
                .interactive(true)
                .desired_width(f32::INFINITY),
        );
    });

    state.first_interior_id = first_id;
    first_id
}

/// The SCRM viewer is a Workspace Context.
///
/// Validates: screen-snapshot-scrm Requirement 16.1, 16.2.
impl crate::shell::workspace_context::WorkspaceContext for ScrmViewerState {
    fn render(
        &mut self,
        ui: &mut egui::Ui,
        _services: &mut crate::shell::workspace_context::ShellServices<'_>,
    ) -> crate::shell::workspace_context::InteriorFocus {
        let first = render(ui, self);
        crate::shell::workspace_context::InteriorFocus { first, last: first }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use ff_screen_model::ScreenModel;

    fn collection_with(n: usize) -> ScreenCollection {
        let mut c = ScreenCollection::new("cid", "Test", "alan", Utc::now());
        for i in 0..n {
            c.append_capture(
                format!("cap{i}"),
                Utc::now(),
                ScreenModel::new(format!("Screen {i}")),
            );
        }
        c
    }

    // Validates: Req 10.2 -- staging a collection builds a replay cursor at start.
    #[test]
    fn staging_collection_builds_replay_at_first() {
        let mut s = ScrmViewerState::default();
        s.set_collection(Some(collection_with(3)));
        assert!(s.replay.is_some());
        assert_eq!(s.replay.as_ref().unwrap().position(), 0);
    }

    // Validates: Req 10.8 -- current screen text is the selectable capture text.
    #[test]
    fn current_screen_text_reflects_capture() {
        let mut s = ScrmViewerState::default();
        s.set_collection(Some(collection_with(2)));
        let text = s.current_screen_text();
        assert!(
            text.contains("Screen 0"),
            "shows first capture; got: {text}"
        );
    }

    // Validates: Req 10.2 -- re-staging the same collection preserves position.
    #[test]
    fn restaging_same_collection_preserves_position() {
        let mut s = ScrmViewerState::default();
        s.set_collection(Some(collection_with(3)));
        s.replay.as_mut().unwrap().next();
        assert_eq!(s.replay.as_ref().unwrap().position(), 1);
        // Re-stage an equal collection (same id + len): position kept.
        s.set_collection(Some(collection_with(3)));
        assert_eq!(s.replay.as_ref().unwrap().position(), 1);
    }

    // Validates: Req 10 -- absent collection yields a placeholder, no panic.
    #[test]
    fn no_collection_yields_placeholder() {
        let s = ScrmViewerState::default();
        assert!(s.current_screen_text().contains("no collection"));
    }
}
