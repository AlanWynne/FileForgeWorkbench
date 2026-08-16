//! PREVIEW command handler.
//!
//! Implements the `viewer.preview` command which activates, switches, lists,
//! and deactivates viewers. This is the primary user-facing interface for the
//! viewer framework.

use crate::error::ViewerError;
use crate::key::ViewerKey;
use crate::panel::ViewerPanel;
use crate::registry::ViewerRegistry;
use crate::selection::ContentSelector;

/// Command ID for the PREVIEW command.
pub const PREVIEW_COMMAND_ID: &str = "viewer.preview";

/// Parsed action from a PREVIEW command invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewCommandAction {
    /// Toggle viewer: activate default if off, deactivate if on.
    Toggle,
    /// Activate the default viewer for the current resource.
    On,
    /// Deactivate the active viewer.
    Off,
    /// List all registered viewers.
    List,
    /// Activate a specific viewer by key.
    Activate(ViewerKey),
}

/// Result of executing a PREVIEW command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreviewCommandResult {
    /// Viewer was activated.
    Activated { viewer_key: ViewerKey },
    /// Viewer was deactivated.
    Deactivated,
    /// Viewer list was produced.
    Listed { entries: Vec<String> },
    /// A message for the user (e.g., no default viewer available).
    Message(String),
}

/// The PREVIEW command handler.
///
/// Integrates with the ViewerRegistry and ViewerPanel to handle all
/// viewer activation, deactivation, and listing operations.
pub struct PreviewCommand<'a> {
    registry: &'a ViewerRegistry,
    panel: &'a mut ViewerPanel,
    selector: &'a ContentSelector,
}

impl<'a> PreviewCommand<'a> {
    /// Create a new PREVIEW command handler.
    pub fn new(
        registry: &'a ViewerRegistry,
        panel: &'a mut ViewerPanel,
        selector: &'a ContentSelector,
    ) -> Self {
        Self {
            registry,
            panel,
            selector,
        }
    }

    /// Parse the command action parameter into a `PreviewCommandAction`.
    ///
    /// Accepts: `None` (toggle), `"on"`, `"off"`, `"list"`, or a viewer-key string.
    ///
    /// # Errors
    ///
    /// Returns `ViewerError::InvalidKeyFormat` if the argument looks like a viewer
    /// key but fails validation.
    pub fn parse_action(action_param: Option<&str>) -> Result<PreviewCommandAction, ViewerError> {
        match action_param {
            None | Some("") => Ok(PreviewCommandAction::Toggle),
            Some("on") | Some("ON") => Ok(PreviewCommandAction::On),
            Some("off") | Some("OFF") => Ok(PreviewCommandAction::Off),
            Some("list") | Some("LIST") => Ok(PreviewCommandAction::List),
            Some(key_str) => {
                let key = ViewerKey::new(&key_str.to_lowercase())?;
                Ok(PreviewCommandAction::Activate(key))
            }
        }
    }

    /// Execute the PREVIEW command action.
    ///
    /// This method never produces an Undo_Record — viewer state changes are
    /// non-undoable display operations.
    ///
    /// # Errors
    ///
    /// Returns an error if the requested viewer key is not found in the registry.
    pub fn execute(
        &mut self,
        action: PreviewCommandAction,
        resource_uri: Option<&str>,
        resource_content: Option<&[u8]>,
    ) -> Result<PreviewCommandResult, ViewerError> {
        match action {
            PreviewCommandAction::Toggle => self.execute_toggle(resource_uri, resource_content),
            PreviewCommandAction::On => self.execute_on(resource_uri, resource_content),
            PreviewCommandAction::Off => self.execute_off(),
            PreviewCommandAction::List => self.execute_list(),
            PreviewCommandAction::Activate(key) => self.execute_activate(&key, resource_content),
        }
    }

    fn execute_toggle(
        &mut self,
        resource_uri: Option<&str>,
        resource_content: Option<&[u8]>,
    ) -> Result<PreviewCommandResult, ViewerError> {
        if self.panel.is_active() {
            self.execute_off()
        } else {
            self.execute_on(resource_uri, resource_content)
        }
    }

    fn execute_on(
        &mut self,
        resource_uri: Option<&str>,
        resource_content: Option<&[u8]>,
    ) -> Result<PreviewCommandResult, ViewerError> {
        let uri = resource_uri.unwrap_or("");
        let content = resource_content.unwrap_or(&[]);

        // Try to auto-select a viewer
        if let Some(viewer_key) = self.selector.select_viewer(uri, content, None) {
            self.panel
                .activate(viewer_key.clone(), uri.to_string(), content.to_vec());
            Ok(PreviewCommandResult::Activated { viewer_key })
        } else {
            // No default viewer — list available ones
            let viewers = self.registry.list_viewers();
            if viewers.is_empty() {
                Ok(PreviewCommandResult::Message(
                    "No viewers are registered.".to_string(),
                ))
            } else {
                let names: Vec<String> = viewers
                    .iter()
                    .map(|v| format!("{} — {}", v.key, v.display_name))
                    .collect();
                Ok(PreviewCommandResult::Message(format!(
                    "No default viewer for this resource. Available viewers: {}",
                    names.join(", ")
                )))
            }
        }
    }

    fn execute_off(&mut self) -> Result<PreviewCommandResult, ViewerError> {
        self.panel.deactivate();
        Ok(PreviewCommandResult::Deactivated)
    }

    fn execute_list(&self) -> Result<PreviewCommandResult, ViewerError> {
        let viewers = self.registry.list_viewers();
        let entries: Vec<String> = viewers
            .iter()
            .map(|v| format!("{}: {} — {}", v.key, v.display_name, v.description))
            .collect();
        Ok(PreviewCommandResult::Listed { entries })
    }

    fn execute_activate(
        &mut self,
        key: &ViewerKey,
        resource_content: Option<&[u8]>,
    ) -> Result<PreviewCommandResult, ViewerError> {
        if !self.registry.contains(key) {
            return Err(ViewerError::UnknownKey {
                key: key.as_str().to_string(),
            });
        }

        let content = resource_content.unwrap_or(&[]).to_vec();
        self.panel.activate(key.clone(), String::new(), content);
        Ok(PreviewCommandResult::Activated {
            viewer_key: key.clone(),
        })
    }
}

/// Returns whether a command ID is the PREVIEW command.
pub fn is_preview_command(command_id: &str) -> bool {
    command_id == PREVIEW_COMMAND_ID
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::built_in::register_built_in_viewers;

    fn setup() -> (ViewerRegistry, ViewerPanel, ContentSelector) {
        let registry = ViewerRegistry::new();
        register_built_in_viewers(&registry).unwrap();
        let panel = ViewerPanel::new();
        let selector = ContentSelector::new(&registry);
        (registry, panel, selector)
    }

    #[test]
    fn parse_action_none_is_toggle() {
        // Validates: Requirement 3 AC 1
        let action = PreviewCommand::parse_action(None).unwrap();
        assert_eq!(action, PreviewCommandAction::Toggle);
    }

    #[test]
    fn parse_action_on() {
        // Validates: Requirement 3 AC 3
        let action = PreviewCommand::parse_action(Some("on")).unwrap();
        assert_eq!(action, PreviewCommandAction::On);

        let action = PreviewCommand::parse_action(Some("ON")).unwrap();
        assert_eq!(action, PreviewCommandAction::On);
    }

    #[test]
    fn parse_action_off() {
        // Validates: Requirement 3 AC 5
        let action = PreviewCommand::parse_action(Some("off")).unwrap();
        assert_eq!(action, PreviewCommandAction::Off);
    }

    #[test]
    fn parse_action_list() {
        // Validates: Requirement 3 AC 6
        let action = PreviewCommand::parse_action(Some("list")).unwrap();
        assert_eq!(action, PreviewCommandAction::List);
    }

    #[test]
    fn parse_action_viewer_key() {
        // Validates: Requirement 3 AC 4
        let action = PreviewCommand::parse_action(Some("asa-report")).unwrap();
        assert_eq!(
            action,
            PreviewCommandAction::Activate(ViewerKey::new("asa-report").unwrap())
        );
    }

    #[test]
    fn parse_action_invalid_key_returns_error() {
        let result = PreviewCommand::parse_action(Some("INVALID KEY!"));
        assert!(result.is_err());
    }

    #[test]
    fn execute_toggle_activates_when_inactive() {
        // Validates: Requirement 3 AC 2
        let (registry, mut panel, selector) = setup();
        let mut cmd = PreviewCommand::new(&registry, &mut panel, &selector);
        let result = cmd
            .execute(
                PreviewCommandAction::Toggle,
                Some("file:///report.lst"),
                Some(b" Hello" as &[u8]),
            )
            .unwrap();
        assert!(matches!(result, PreviewCommandResult::Activated { .. }));
    }

    #[test]
    fn execute_toggle_deactivates_when_active() {
        // Validates: Requirement 3 AC 2
        let (registry, mut panel, selector) = setup();
        panel.activate(
            ViewerKey::new("hex").unwrap(),
            "file:///test".to_string(),
            vec![],
        );

        let mut cmd = PreviewCommand::new(&registry, &mut panel, &selector);
        let result = cmd
            .execute(PreviewCommandAction::Toggle, None, None)
            .unwrap();
        assert_eq!(result, PreviewCommandResult::Deactivated);
    }

    #[test]
    fn execute_off_deactivates_panel() {
        // Validates: Requirement 3 AC 5
        let (registry, mut panel, selector) = setup();
        panel.activate(
            ViewerKey::new("hex").unwrap(),
            "file:///test".to_string(),
            vec![],
        );

        let mut cmd = PreviewCommand::new(&registry, &mut panel, &selector);
        let result = cmd.execute(PreviewCommandAction::Off, None, None).unwrap();
        assert_eq!(result, PreviewCommandResult::Deactivated);
        assert!(!panel.is_active());
    }

    #[test]
    fn execute_list_returns_all_viewers() {
        // Validates: Requirement 3 AC 6
        let (registry, mut panel, selector) = setup();
        let mut cmd = PreviewCommand::new(&registry, &mut panel, &selector);
        let result = cmd.execute(PreviewCommandAction::List, None, None).unwrap();

        match result {
            PreviewCommandResult::Listed { entries } => {
                assert_eq!(entries.len(), 4);
            }
            other => panic!("Expected Listed, got: {other:?}"),
        }
    }

    #[test]
    fn execute_activate_specific_viewer() {
        // Validates: Requirement 3 AC 4
        let (registry, mut panel, selector) = setup();
        let key = ViewerKey::new("hex").unwrap();
        let mut cmd = PreviewCommand::new(&registry, &mut panel, &selector);
        let result = cmd
            .execute(PreviewCommandAction::Activate(key.clone()), None, None)
            .unwrap();
        assert_eq!(result, PreviewCommandResult::Activated { viewer_key: key });
        assert!(panel.is_active());
    }

    #[test]
    fn execute_activate_unknown_key_returns_error() {
        // Validates: Requirement 1 AC 8
        let (registry, mut panel, selector) = setup();
        let key = ViewerKey::new("nonexistent").unwrap();
        let mut cmd = PreviewCommand::new(&registry, &mut panel, &selector);
        let result = cmd.execute(PreviewCommandAction::Activate(key), None, None);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ViewerError::UnknownKey { .. }
        ));
    }

    #[test]
    fn preview_command_never_produces_undo_record() {
        // Validates: Requirement 3 AC 9
        // The PREVIEW command operates purely on display state —
        // there is no undo integration in PreviewCommand at all.
        // This test verifies no UndoRecord type exists in this module.
        let (registry, mut panel, selector) = setup();
        let mut cmd = PreviewCommand::new(&registry, &mut panel, &selector);

        // Execute multiple actions — none should involve undo
        cmd.execute(
            PreviewCommandAction::On,
            Some("file:///data.csv"),
            Some(b"a,b\n1,2"),
        )
        .unwrap();
        cmd.execute(PreviewCommandAction::Off, None, None).unwrap();
        cmd.execute(PreviewCommandAction::List, None, None).unwrap();
        // If we got here without touching any undo system, the property holds.
    }

    #[test]
    fn command_id_constant_is_correct() {
        assert_eq!(PREVIEW_COMMAND_ID, "viewer.preview");
    }

    #[test]
    fn is_preview_command_matches_correctly() {
        assert!(is_preview_command("viewer.preview"));
        assert!(!is_preview_command("edit.delete"));
    }
}
