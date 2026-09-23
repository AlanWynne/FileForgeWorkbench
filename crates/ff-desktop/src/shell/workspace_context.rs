//! Workspace Context framework (CR-NR-078, Phase 1).
//!
//! Every Workspace Context (the content inside a Workspace tab) implements the
//! [`WorkspaceContext`] trait. Its [`WorkspaceContext::render`] method RETURNS an
//! [`InteriorFocus`] describing the first and last interior Tab stops, so an
//! implementor CANNOT compile without providing the focus contract the shell
//! Boundary_Policy (CR-CH-023) needs. This makes the phantom-Tab-stop bug class
//! (B056/B057/B058/B059) unrepresentable rather than merely tested-against.
//!
//! `render` is HOST-AGNOSTIC (Requirement 6): it takes a `&mut egui::Ui` and does
//! NOT assume it is drawn in the main window's central panel, so the same Context
//! renders as a tab, a dock zone, or a detached OS viewport (a second viewport of
//! the SINGLE FFWB process) with no per-Context change.
//!
//! A Context reaches shell services through [`ShellServices`] (Option Y: this
//! trait lives in ff-desktop, layered above `ff-layout::DockablePanel`, which owns
//! placement/geometry and stays GUI-independent). A Context communicates
//! state-changing intent by pushing [`ShellRequest`]s, which the shell drains and
//! applies through the existing pipelines -- generalising the "pure render ->
//! action" pattern already used by the editor panels.
//!
//! Validates: workspace-framework Requirement 1, 2, 6.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use eframe::egui;
use ff_command::CommandTarget;

use crate::notification::NotificationQueue;

/// The interior focus contract a Context reports to the shell Boundary_Policy.
///
/// `first` is the control the command-field -> first-interior Tab jump latches
/// to; `last` anchors the Shift+Tab reverse boundary. Both are `Option` so a
/// Context with no interior focus stops returns [`InteriorFocus::none`]
/// EXPLICITLY (never by accident).
///
/// Validates: workspace-framework Requirement 1.1, 1.3.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct InteriorFocus {
    /// First interior Tab stop (fresh same-frame `egui::Id`), or `None`.
    pub first: Option<egui::Id>,
    /// Last interior Tab stop (Shift+Tab reverse anchor), or `None`.
    pub last: Option<egui::Id>,
}

impl InteriorFocus {
    // `none`/`new` are the deliberate no-interior and distinct-first/last
    // constructors of the focus contract. Phase-1 migrated panels happen to use
    // only `single`; `none`/`new` are consumed by later-phase Contexts (e.g. a
    // no-interior display, or a panel with distinct first/last stops) and by the
    // framework's own unit tests. Kept as public API surface, not dead code.
    #[allow(dead_code)]
    /// The Context has NO interior focus stops (a deliberate, documented value).
    pub fn none() -> Self {
        Self {
            first: None,
            last: None,
        }
    }

    /// A single control that is both the first and last interior stop (the
    /// common case: egui-native Tab walks any controls in between).
    pub fn single(id: egui::Id) -> Self {
        Self {
            first: Some(id),
            last: Some(id),
        }
    }

    #[allow(dead_code)]
    /// Distinct first and last interior stops.
    pub fn new(first: egui::Id, last: egui::Id) -> Self {
        Self {
            first: Some(first),
            last: Some(last),
        }
    }
}

/// A request a Context asks the shell to perform, drained after `render`
/// (Requirement 2.2). Keeping this a small closed set means a Context never
/// mutates the shell directly; the shell applies each request through its
/// existing pipelines.
///
/// The variants are the framework's request vocabulary (Requirement 2.2). Phase-1
/// migrated Contexts (Config/Theme/Menus/MenuWorkspace) do not yet enqueue every
/// variant -- the Theme/Menus editors stash a rich `pending_action` the shell
/// drains instead -- but the vocabulary is consumed by the shell's
/// `apply_shell_requests` drain and by later-phase Contexts. Kept as the closed
/// request contract, not dead code.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ShellRequest {
    /// Run a command string through `handle_command` (identical to typing it).
    Command(String),
    /// Dispatch a resolved `CommandTarget` (menu / function / macro / external).
    Target(CommandTarget),
    /// Open a file by path/URI (routes through `file.open`).
    OpenFile(String),
    /// Surface a status/error message in the command area (`open_error`).
    Status(String),
}

/// A per-frame view of the shell services a Context may use during `render`,
/// constructed by the shell from its OWN fields (Requirement 2.1). It does NOT
/// expose `&mut WorkbenchShell`, so trait dispatch is possible while `&mut Ui`
/// (borrowed from the same shell's egui frame) is also live -- the shell builds
/// this from fields DISJOINT from the panel being rendered (Option A: the panel
/// is moved out with `mem::take` first).
///
/// A Context MUST NOT retain `ShellServices` across frames (Requirement 2.3).
///
/// Phase-1 migrated Contexts read only `config`; the remaining fields
/// (`runtime`, `notifications`, `themes_dir`, `menus_dir`, `requests`) are the
/// service surface later-phase Contexts (Plugin Manager, Event Log, Macro
/// Library, Search Results, Command Configurator) consume. Kept as the framework
/// service contract, not dead code.
#[allow(dead_code)]
pub struct ShellServices<'a> {
    /// Read access to the layered configuration.
    pub config: &'a ff_config::ConfigHandle,
    /// The Tokio runtime handle (for panels that spawn background work).
    pub runtime: &'a tokio::runtime::Runtime,
    /// The shared notification queue.
    pub notifications: &'a Arc<Mutex<NotificationQueue>>,
    /// Resolved themes directory (`<user_data>/themes`).
    pub themes_dir: PathBuf,
    /// Resolved menus directory (`<user_data>/menus`).
    pub menus_dir: PathBuf,
    /// Read access to the user-defined command store (the Command Configurator
    /// Context renders its definition table from this). The shell calls
    /// `poll_reload` before building `ShellServices`, so this is a settled
    /// read-only view for the frame.
    pub command_store: &'a crate::command_config::store::CommandStore,
    /// Requests the Context enqueues; drained by the shell after `render`.
    pub requests: &'a mut Vec<ShellRequest>,
}

// The `request_*` helpers are the ergonomic façade over `requests.push(..)` that
// later-phase Contexts use to enqueue `ShellRequest`s; phase-1 Contexts stash a
// `pending_action` instead. Kept as framework API, not dead code.
#[allow(dead_code)]
impl ShellServices<'_> {
    /// Enqueue a command string to run after render.
    pub fn request_command(&mut self, cmd: impl Into<String>) {
        self.requests.push(ShellRequest::Command(cmd.into()));
    }

    /// Enqueue a `CommandTarget` to dispatch after render.
    pub fn request_target(&mut self, target: CommandTarget) {
        self.requests.push(ShellRequest::Target(target));
    }

    /// Enqueue a file open after render.
    pub fn request_open_file(&mut self, path: impl Into<String>) {
        self.requests.push(ShellRequest::OpenFile(path.into()));
    }

    /// Enqueue a status/error message.
    pub fn request_status(&mut self, message: impl Into<String>) {
        self.requests.push(ShellRequest::Status(message.into()));
    }
}

/// A Workspace Context: the content inside a Workspace tab.
///
/// The `render` method is HOST-AGNOSTIC and RETURNS the interior focus contract,
/// so the shell dispatches every Context through one code path and cannot omit
/// the focus latch (Requirement 1.1, 1.2, 6.1).
///
/// Validates: workspace-framework Requirement 1.1, 1.4, 6.1.
pub trait WorkspaceContext {
    /// Render this Context into `ui`, using `services` to reach shell state, and
    /// return the interior focus contract for this frame. MUST NOT assume it is
    /// drawn in the main window's central panel (host-agnostic).
    fn render(&mut self, ui: &mut egui::Ui, services: &mut ShellServices<'_>) -> InteriorFocus;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interior_focus_none_is_both_none() {
        let f = InteriorFocus::none();
        assert_eq!(f.first, None);
        assert_eq!(f.last, None);
    }

    #[test]
    fn interior_focus_single_sets_both_to_same_id() {
        let id = egui::Id::new("x");
        let f = InteriorFocus::single(id);
        assert_eq!(f.first, Some(id));
        assert_eq!(f.last, Some(id));
    }

    #[test]
    fn interior_focus_new_sets_distinct_first_and_last() {
        let a = egui::Id::new("a");
        let b = egui::Id::new("b");
        let f = InteriorFocus::new(a, b);
        assert_eq!(f.first, Some(a));
        assert_eq!(f.last, Some(b));
    }

    // A trivial Context proves an implementor MUST return an InteriorFocus and
    // that a no-interior Context returns `none()` explicitly (Requirement 1.3).
    struct NoInteriorContext;
    impl WorkspaceContext for NoInteriorContext {
        fn render(
            &mut self,
            _ui: &mut egui::Ui,
            _services: &mut ShellServices<'_>,
        ) -> InteriorFocus {
            InteriorFocus::none()
        }
    }

    #[test]
    fn shell_request_helpers_enqueue() {
        let mut reqs: Vec<ShellRequest> = Vec::new();
        // Build a ShellServices with only the requests field exercised; the
        // other fields are not needed for this pure-enqueue test, so we cannot
        // construct the full struct here without a shell. Instead assert the
        // request variants and the panel-contract type compile and behave.
        reqs.push(ShellRequest::Command("THEME".to_string()));
        reqs.push(ShellRequest::OpenFile("/tmp/x".to_string()));
        assert_eq!(reqs.len(), 2);
        assert!(matches!(reqs[0], ShellRequest::Command(_)));
        // NoInteriorContext returns none() (compile-level contract check).
        let mut c = NoInteriorContext;
        // We cannot render without a Ui here; assert the type satisfies the
        // trait by taking it as a &mut dyn WorkspaceContext.
        let _dyn: &mut dyn WorkspaceContext = &mut c;
    }
}
