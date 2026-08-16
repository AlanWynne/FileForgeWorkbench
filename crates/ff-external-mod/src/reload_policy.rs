//! Reload policy definitions.
//!
//! Defines the `ReloadPolicy` enum representing the configurable strategy
//! for responding to external modifications.

use serde::Deserialize;

/// Configurable strategy for responding to external modifications.
///
/// Determines how the system handles detected external file changes:
/// - `Prompt`: Always ask the user what to do
/// - `Auto`: Auto-reload if buffer is clean; prompt if dirty
/// - `Ignore`: Never notify — keep in-memory content as-is
///
/// Addresses: Requirement 10, criterion 2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ReloadPolicy {
    /// Always ask the user what to do.
    #[default]
    Prompt,
    /// Auto-reload if buffer is clean; prompt if dirty.
    Auto,
    /// Never notify — keep in-memory content as-is.
    Ignore,
}

impl std::fmt::Display for ReloadPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Prompt => write!(f, "prompt"),
            Self::Auto => write!(f, "auto"),
            Self::Ignore => write!(f, "ignore"),
        }
    }
}

impl std::str::FromStr for ReloadPolicy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "prompt" => Ok(Self::Prompt),
            "auto" => Ok(Self::Auto),
            "ignore" => Ok(Self::Ignore),
            _ => Err(format!(
                "invalid reload policy '{s}': expected 'prompt', 'auto', or 'ignore'"
            )),
        }
    }
}

/// The action determined by the policy engine after evaluating an external change.
///
/// Addresses: Requirement 3, criteria 2–5
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyAction {
    /// Show a prompt to the user with available options.
    ShowPrompt,
    /// Automatically reload the document content from disk.
    AutoReload,
    /// Suppress the notification entirely.
    Suppress,
    /// Only update the mtime snapshot without user interaction.
    UpdateSnapshotOnly,
}

/// The reload policy engine evaluates external change events against the
/// configured policy and document dirty state to determine the appropriate action.
///
/// Addresses: Requirement 3 AC 2–5, Requirement 5 AC 1
#[derive(Debug)]
pub struct ReloadPolicyEngine;

impl ReloadPolicyEngine {
    /// Evaluate the policy action for an external change event.
    ///
    /// The decision is deterministic given the same inputs:
    /// - `Ignore` → always `UpdateSnapshotOnly`
    /// - `Auto` + not dirty → always `AutoReload`
    /// - `Auto` + dirty → always `ShowPrompt` (data loss prevention)
    /// - `Prompt` → always `ShowPrompt`
    ///
    /// Addresses: Requirement 3, criteria 2–5
    pub fn evaluate(
        policy: ReloadPolicy,
        is_dirty: bool,
        _change_type: &crate::change_event::ChangeType,
    ) -> PolicyAction {
        match policy {
            ReloadPolicy::Ignore => PolicyAction::UpdateSnapshotOnly,
            ReloadPolicy::Auto => {
                if is_dirty {
                    // Dirty buffers always require confirmation (Req 3 AC 4)
                    PolicyAction::ShowPrompt
                } else {
                    PolicyAction::AutoReload
                }
            }
            ReloadPolicy::Prompt => PolicyAction::ShowPrompt,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reload_policy_default_is_prompt() {
        assert_eq!(ReloadPolicy::default(), ReloadPolicy::Prompt);
    }

    #[test]
    fn reload_policy_display_shows_lowercase() {
        assert_eq!(ReloadPolicy::Prompt.to_string(), "prompt");
        assert_eq!(ReloadPolicy::Auto.to_string(), "auto");
        assert_eq!(ReloadPolicy::Ignore.to_string(), "ignore");
    }

    #[test]
    fn reload_policy_from_str_parses_valid_values() {
        assert_eq!(
            "prompt".parse::<ReloadPolicy>().unwrap(),
            ReloadPolicy::Prompt
        );
        assert_eq!("auto".parse::<ReloadPolicy>().unwrap(), ReloadPolicy::Auto);
        assert_eq!(
            "ignore".parse::<ReloadPolicy>().unwrap(),
            ReloadPolicy::Ignore
        );
    }

    #[test]
    fn reload_policy_from_str_is_case_insensitive() {
        assert_eq!(
            "Prompt".parse::<ReloadPolicy>().unwrap(),
            ReloadPolicy::Prompt
        );
        assert_eq!("AUTO".parse::<ReloadPolicy>().unwrap(), ReloadPolicy::Auto);
        assert_eq!(
            "IGNORE".parse::<ReloadPolicy>().unwrap(),
            ReloadPolicy::Ignore
        );
    }

    #[test]
    fn reload_policy_from_str_rejects_invalid_values() {
        let result = "invalid".parse::<ReloadPolicy>();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid reload policy"));
    }

    #[test]
    fn policy_action_variants_are_distinct() {
        assert_ne!(PolicyAction::ShowPrompt, PolicyAction::AutoReload);
        assert_ne!(PolicyAction::AutoReload, PolicyAction::Suppress);
        assert_ne!(PolicyAction::Suppress, PolicyAction::UpdateSnapshotOnly);
    }

    // --- ReloadPolicyEngine tests ---

    #[test]
    fn policy_prompt_clean_content_changed_shows_prompt() {
        // Validates: Requirement 3.2
        let result = ReloadPolicyEngine::evaluate(
            ReloadPolicy::Prompt,
            false,
            &crate::change_event::ChangeType::ContentChanged,
        );
        assert_eq!(result, PolicyAction::ShowPrompt);
    }

    #[test]
    fn policy_prompt_dirty_content_changed_shows_prompt() {
        // Validates: Requirement 3.2
        let result = ReloadPolicyEngine::evaluate(
            ReloadPolicy::Prompt,
            true,
            &crate::change_event::ChangeType::ContentChanged,
        );
        assert_eq!(result, PolicyAction::ShowPrompt);
    }

    #[test]
    fn policy_auto_clean_content_changed_auto_reloads() {
        // Validates: Requirement 3.3, Requirement 5.1
        let result = ReloadPolicyEngine::evaluate(
            ReloadPolicy::Auto,
            false,
            &crate::change_event::ChangeType::ContentChanged,
        );
        assert_eq!(result, PolicyAction::AutoReload);
    }

    #[test]
    fn policy_auto_dirty_content_changed_shows_prompt() {
        // Validates: Requirement 3.4 — dirty buffer protection
        let result = ReloadPolicyEngine::evaluate(
            ReloadPolicy::Auto,
            true,
            &crate::change_event::ChangeType::ContentChanged,
        );
        assert_eq!(result, PolicyAction::ShowPrompt);
    }

    #[test]
    fn policy_ignore_clean_content_changed_updates_snapshot_only() {
        // Validates: Requirement 3.5
        let result = ReloadPolicyEngine::evaluate(
            ReloadPolicy::Ignore,
            false,
            &crate::change_event::ChangeType::ContentChanged,
        );
        assert_eq!(result, PolicyAction::UpdateSnapshotOnly);
    }

    #[test]
    fn policy_ignore_dirty_content_changed_updates_snapshot_only() {
        // Validates: Requirement 3.5 — ignore always suppresses
        let result = ReloadPolicyEngine::evaluate(
            ReloadPolicy::Ignore,
            true,
            &crate::change_event::ChangeType::ContentChanged,
        );
        assert_eq!(result, PolicyAction::UpdateSnapshotOnly);
    }

    #[test]
    fn policy_prompt_deleted_shows_prompt() {
        let result = ReloadPolicyEngine::evaluate(
            ReloadPolicy::Prompt,
            false,
            &crate::change_event::ChangeType::FileDeleted,
        );
        assert_eq!(result, PolicyAction::ShowPrompt);
    }

    #[test]
    fn policy_auto_clean_deleted_auto_reloads() {
        let result = ReloadPolicyEngine::evaluate(
            ReloadPolicy::Auto,
            false,
            &crate::change_event::ChangeType::FileDeleted,
        );
        assert_eq!(result, PolicyAction::AutoReload);
    }

    #[test]
    fn policy_ignore_renamed_updates_snapshot_only() {
        use ff_vfs::ResourceUri;
        let result = ReloadPolicyEngine::evaluate(
            ReloadPolicy::Ignore,
            false,
            &crate::change_event::ChangeType::FileRenamed {
                old_uri: ResourceUri::new("local", "/old.rs"),
                new_uri: ResourceUri::new("local", "/new.rs"),
            },
        );
        assert_eq!(result, PolicyAction::UpdateSnapshotOnly);
    }

    #[test]
    fn policy_auto_dirty_renamed_shows_prompt() {
        use ff_vfs::ResourceUri;
        let result = ReloadPolicyEngine::evaluate(
            ReloadPolicy::Auto,
            true,
            &crate::change_event::ChangeType::FileRenamed {
                old_uri: ResourceUri::new("local", "/old.rs"),
                new_uri: ResourceUri::new("local", "/new.rs"),
            },
        );
        assert_eq!(result, PolicyAction::ShowPrompt);
    }

    /// Exhaustive test: all 18 combinations of (policy, dirty, change_type) produce valid actions.
    #[test]
    fn policy_evaluation_completeness_all_combinations() {
        // Validates: Requirement 3.2–3.5 — every combination has a defined action
        use ff_vfs::ResourceUri;
        let policies = [
            ReloadPolicy::Prompt,
            ReloadPolicy::Auto,
            ReloadPolicy::Ignore,
        ];
        let dirty_states = [false, true];
        let change_types = [
            crate::change_event::ChangeType::ContentChanged,
            crate::change_event::ChangeType::FileDeleted,
            crate::change_event::ChangeType::FileRenamed {
                old_uri: ResourceUri::new("local", "/a"),
                new_uri: ResourceUri::new("local", "/b"),
            },
        ];

        for policy in &policies {
            for &dirty in &dirty_states {
                for change_type in &change_types {
                    let action = ReloadPolicyEngine::evaluate(*policy, dirty, change_type);
                    // Verify the invariants
                    match policy {
                        ReloadPolicy::Ignore => {
                            assert_eq!(action, PolicyAction::UpdateSnapshotOnly);
                        }
                        ReloadPolicy::Auto if dirty => {
                            assert_eq!(action, PolicyAction::ShowPrompt);
                        }
                        ReloadPolicy::Auto => {
                            assert_eq!(action, PolicyAction::AutoReload);
                        }
                        ReloadPolicy::Prompt => {
                            assert_eq!(action, PolicyAction::ShowPrompt);
                        }
                    }
                }
            }
        }
    }
}
