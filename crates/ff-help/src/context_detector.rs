//! Context detection for F1 help activation.
//!
//! The `ContextDetector` inspects the current editor state and resolves the
//! most relevant `TopicKey` for the Help Panel to display.

use crate::registry::HelpTopicRegistry;
use crate::topic_key::TopicKey;

/// Editor mode identifiers for context detection.
///
/// Represents the currently active editing mode. Special modes (Hex, Preview,
/// Grid_*) contribute to context resolution when no more specific context
/// is available.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    /// Standard browsing mode.
    Browse,
    /// Standard editing mode.
    Edit,
    /// Read-only view mode.
    View,
    /// Hexadecimal display mode.
    Hex,
    /// Preview/render mode.
    Preview,
    /// FileForge grid browse mode.
    GridBrowse,
    /// FileForge grid edit mode.
    GridEdit,
}

impl EditorMode {
    /// Returns the mode name as used in TopicKey (lowercase).
    pub fn as_topic_name(&self) -> &str {
        match self {
            Self::Browse => "browse",
            Self::Edit => "edit",
            Self::View => "view",
            Self::Hex => "hex",
            Self::Preview => "preview",
            Self::GridBrowse => "grid_browse",
            Self::GridEdit => "grid_edit",
        }
    }

    /// Returns true if this is a "special" mode that provides context
    /// for help resolution (Hex, Preview, Grid_*).
    pub fn is_special(&self) -> bool {
        matches!(
            self,
            Self::Hex | Self::Preview | Self::GridBrowse | Self::GridEdit
        )
    }
}

/// Snapshot of the current editor context used by `ContextDetector` to resolve a `TopicKey`.
///
/// Provided by the shell layer at the moment F1 is pressed or HELP is invoked.
#[derive(Debug, Clone)]
pub struct EditorContext {
    /// Current content of the command input field (trimmed).
    pub command_line_text: String,
    /// Whether the command input field currently has keyboard focus.
    pub command_line_has_focus: bool,
    /// Content of the focused prefix area cell (if any).
    pub prefix_area_text: Option<String>,
    /// Whether a prefix area cell currently has keyboard focus.
    pub prefix_area_has_focus: bool,
    /// The currently active editor mode.
    pub active_mode: EditorMode,
    /// Whether the Help Panel is currently open.
    pub help_panel_open: bool,
    /// The currently displayed topic in the Help Panel (for toggle detection).
    pub current_help_topic: Option<TopicKey>,
}

/// Inspects the current editor state and resolves the most relevant `TopicKey`.
///
/// Implements best-effort context detection with the following priority order:
/// 1. Command input field focused + contains recognisable command → `cmd:<NAME>`
/// 2. Command input field focused + empty → `index`
/// 3. Prefix area focused + contains line command → `line:<CMD>`
/// 4. Active special mode (Hex, Preview, Grid_*) → `mode:<MODE>`
/// 5. Fallback → `index`
pub struct ContextDetector;

impl ContextDetector {
    /// Resolve the most relevant `TopicKey` from the given context state.
    ///
    /// # Priority Order
    ///
    /// 1. Command input field focused + contains recognisable command name
    ///    (first whitespace-delimited token) → `"cmd:<COMMAND_NAME>"`
    /// 2. Command input field focused + empty/whitespace → `"index"`
    /// 3. Prefix area focused + contains line command text → `"line:<COMMAND>"`
    /// 4. Active special mode (Hex, Preview, Grid_*) → `"mode:<mode>"`
    /// 5. Fallback → `"index"`
    pub fn resolve(ctx: &EditorContext) -> TopicKey {
        // Priority 1 & 2: Command input field has focus
        if ctx.command_line_has_focus {
            let trimmed = ctx.command_line_text.trim();
            if trimmed.is_empty() {
                return TopicKey::index();
            }
            // Extract first token as the command name
            let command_name = trimmed.split_whitespace().next().unwrap_or("");
            if !command_name.is_empty() {
                return TopicKey::command(&command_name.to_uppercase());
            }
            return TopicKey::index();
        }

        // Priority 3: Prefix area has focus with line command
        if ctx.prefix_area_has_focus {
            if let Some(ref text) = ctx.prefix_area_text {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    return TopicKey::line_command(&trimmed.to_uppercase());
                }
            }
            return TopicKey::index();
        }

        // Priority 4: Active special mode
        if ctx.active_mode.is_special() {
            return TopicKey::mode(ctx.active_mode.as_topic_name());
        }

        // Priority 5: Fallback to index
        TopicKey::index()
    }

    /// Resolve a `TopicKey` and check whether content exists in the registry.
    ///
    /// Returns `Ok(key)` when the resolved topic exists in `registry`.
    /// Returns `Err(message)` with a human-readable "not available yet" message
    /// that identifies the exact context so missing topics can be tracked.
    ///
    /// Validates: Requirement 18.1, 18.2
    pub fn resolve_with_fallback(
        ctx: &EditorContext,
        registry: &HelpTopicRegistry,
    ) -> Result<TopicKey, String> {
        let key = Self::resolve(ctx);
        if registry.contains(&key) {
            Ok(key)
        } else {
            let label = Self::human_label(&key);
            Err(format!(
                "Help not yet available for {label} [topic-key: {}]",
                key.as_str()
            ))
        }
    }

    /// Converts a `TopicKey` into a human-readable context description.
    ///
    /// Examples: `cmd:FIND` → `command "FIND"`, `line:CC` → `line command "CC"`,
    /// `mode:hex` → `mode "hex"`, `index` → `the Help Index`.
    fn human_label(key: &TopicKey) -> String {
        use crate::topic_key::TopicCategory;
        match key.category() {
            TopicCategory::Command => format!("command \"{}\"", key.identifier()),
            TopicCategory::LineCommand => format!("line command \"{}\"", key.identifier()),
            TopicCategory::Mode => format!("mode \"{}\"", key.identifier()),
            TopicCategory::Feature => format!("feature \"{}\"", key.identifier()),
            TopicCategory::Config => format!("config key \"{}\"", key.identifier()),
            TopicCategory::Api => format!("API function \"{}\"", key.identifier()),
            TopicCategory::Index | TopicCategory::GettingStarted => "the Help Index".to_string(),
        }
    }

    /// Determine if F1 should toggle the Help Panel closed (same topic redisplay).
    ///
    /// Returns `true` if the panel is open and the resolved topic matches
    /// the currently displayed topic.
    pub fn should_toggle_close(ctx: &EditorContext, resolved: &TopicKey) -> bool {
        ctx.help_panel_open && ctx.current_help_topic.as_ref() == Some(resolved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_context() -> EditorContext {
        EditorContext {
            command_line_text: String::new(),
            command_line_has_focus: false,
            prefix_area_text: None,
            prefix_area_has_focus: false,
            active_mode: EditorMode::Edit,
            help_panel_open: false,
            current_help_topic: None,
        }
    }

    // Validates: Requirement 1.2 — Command field with command resolves to cmd:<NAME>
    #[test]
    fn resolve_command_field_with_command_name() {
        let ctx = EditorContext {
            command_line_text: "CHANGE 'foo' 'bar'".to_string(),
            command_line_has_focus: true,
            ..default_context()
        };
        let key = ContextDetector::resolve(&ctx);
        assert_eq!(key, TopicKey::command("CHANGE"));
    }

    // Validates: Requirement 1.3 — Empty command field resolves to index
    #[test]
    fn resolve_empty_command_field_to_index() {
        let ctx = EditorContext {
            command_line_text: "   ".to_string(),
            command_line_has_focus: true,
            ..default_context()
        };
        let key = ContextDetector::resolve(&ctx);
        assert_eq!(key, TopicKey::index());
    }

    // Validates: Requirement 1.4 — Prefix area with line command
    #[test]
    fn resolve_prefix_area_with_line_command() {
        let ctx = EditorContext {
            prefix_area_text: Some("cc".to_string()),
            prefix_area_has_focus: true,
            ..default_context()
        };
        let key = ContextDetector::resolve(&ctx);
        assert_eq!(key, TopicKey::line_command("CC"));
    }

    // Validates: Requirement 1.5 — Special mode provides context
    #[test]
    fn resolve_special_mode_when_no_other_context() {
        let ctx = EditorContext {
            active_mode: EditorMode::Hex,
            ..default_context()
        };
        let key = ContextDetector::resolve(&ctx);
        assert_eq!(key, TopicKey::mode("hex"));
    }

    // Validates: Requirement 1.7 — No specific context falls back to index
    #[test]
    fn resolve_no_context_to_index() {
        let ctx = default_context();
        let key = ContextDetector::resolve(&ctx);
        assert_eq!(key, TopicKey::index());
    }

    // Validates: Requirement 1.2 — Command field priority over prefix area
    #[test]
    fn command_field_focus_takes_priority_over_prefix_area() {
        let ctx = EditorContext {
            command_line_text: "find text".to_string(),
            command_line_has_focus: true,
            prefix_area_text: Some("cc".to_string()),
            prefix_area_has_focus: true,
            active_mode: EditorMode::Hex,
            ..default_context()
        };
        let key = ContextDetector::resolve(&ctx);
        assert_eq!(key, TopicKey::command("FIND"));
    }

    // Validates: Requirement 1.6 — Toggle detection when same topic
    #[test]
    fn should_toggle_close_when_same_topic() {
        let resolved = TopicKey::command("FIND");
        let ctx = EditorContext {
            help_panel_open: true,
            current_help_topic: Some(TopicKey::command("FIND")),
            ..default_context()
        };
        assert!(ContextDetector::should_toggle_close(&ctx, &resolved));
    }

    // Validates: Requirement 1.6 — No toggle when different topic
    #[test]
    fn should_not_toggle_close_when_different_topic() {
        let resolved = TopicKey::command("CHANGE");
        let ctx = EditorContext {
            help_panel_open: true,
            current_help_topic: Some(TopicKey::command("FIND")),
            ..default_context()
        };
        assert!(!ContextDetector::should_toggle_close(&ctx, &resolved));
    }

    // Validates: Requirement 1.6 — No toggle when panel closed
    #[test]
    fn should_not_toggle_close_when_panel_closed() {
        let resolved = TopicKey::command("FIND");
        let ctx = EditorContext {
            help_panel_open: false,
            current_help_topic: Some(TopicKey::command("FIND")),
            ..default_context()
        };
        assert!(!ContextDetector::should_toggle_close(&ctx, &resolved));
    }

    // Validates: Requirement 18.1 — missing topic produces human-readable fallback message
    #[test]
    fn resolve_with_fallback_missing_topic_returns_err() {
        use crate::registry::HelpTopicRegistry;
        let registry = HelpTopicRegistry::new(); // empty — no topics registered
        let ctx = EditorContext {
            command_line_text: "FIND".to_string(),
            command_line_has_focus: true,
            ..default_context()
        };
        let result = ContextDetector::resolve_with_fallback(&ctx, &registry);
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("Help not yet available for"), "msg: {msg}");
        assert!(msg.contains("command \"FIND\""), "msg: {msg}");
        assert!(
            msg.contains("cmd:FIND"),
            "topic-key missing from msg: {msg}"
        );
    }

    // Validates: Requirement 18.1 — existing topic returns Ok
    #[test]
    fn resolve_with_fallback_existing_topic_returns_ok() {
        use crate::registry::HelpTopicRegistry;
        use crate::topic::{HelpTopic, TopicSource};
        use std::path::PathBuf;
        let registry = HelpTopicRegistry::new();
        let key = TopicKey::command("FIND");
        registry.register_file_topic(HelpTopic::new(
            key.clone(),
            "FIND".to_string(),
            "Find help body".to_string(),
            TopicSource::FileBased {
                file_path: PathBuf::from("find.help.md"),
            },
        ));
        let ctx = EditorContext {
            command_line_text: "FIND".to_string(),
            command_line_has_focus: true,
            ..default_context()
        };
        let result = ContextDetector::resolve_with_fallback(&ctx, &registry);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), key);
    }

    // Validates: Requirement 1.9 — All editor modes supported
    #[test]
    fn all_editor_modes_have_topic_names() {
        let modes = [
            EditorMode::Browse,
            EditorMode::Edit,
            EditorMode::View,
            EditorMode::Hex,
            EditorMode::Preview,
            EditorMode::GridBrowse,
            EditorMode::GridEdit,
        ];
        for mode in &modes {
            let name = mode.as_topic_name();
            assert!(!name.is_empty());
        }
    }
}
