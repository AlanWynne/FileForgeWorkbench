//! Unit tests for the Kinds Editor panel state (CR-NR-090 B.4).

use super::*;
use crate::workspace_kind::{BaseKind, KindConfig};

/// Validates: workspace-kinds Requirement 6.1 -- load_kind sets the selection
/// and working copy.
#[test]
fn load_kind_sets_selection_and_working() {
    let mut state = KindsEditorState::default();
    let cfg = KindConfig::builtin_default(BuiltinKind::Editor);
    state.load_kind("editor", cfg.clone());
    assert_eq!(state.selected.as_deref(), Some("editor"));
    assert_eq!(state.working, Some(cfg));
    assert!(state.error.is_none());
}

/// Validates: workspace-kinds Requirement 6.2 -- the new-kind sub-form defaults
/// to a valid built-in base.
#[test]
fn new_kind_base_defaults_to_a_builtin() {
    let state = KindsEditorState::default();
    assert_eq!(state.new_base, BuiltinKind::Editor);
    // Sanity: the default working copy is None until a Kind is selected.
    assert!(state.working.is_none());
    let _ = BaseKind::Builtin(state.new_base); // base is a builtin
}
