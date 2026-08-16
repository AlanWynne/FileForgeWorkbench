//! Built-in provider for command name completion.
//!
//! Queries the command registry for all registered commands and
//! offers them as candidates when the user is in command name position.

use crate::candidate::{CompletionCandidate, CompletionKind};
use crate::context::{CompletionContext, CompletionField};
use crate::error::CompletionError;
use crate::provider::CompletionProvider;

/// Provides command name completion candidates from the command registry.
///
/// Active when the cursor is in the first token (command name position)
/// of the primary command field.
pub struct CommandNameProvider {
    /// Mock command list for testing. In production this would query
    /// the actual CommandRegistry.
    commands: Vec<CommandEntry>,
}

/// A simplified command entry for the provider.
#[derive(Debug, Clone)]
struct CommandEntry {
    name: String,
    category: String,
    description: String,
}

impl CommandNameProvider {
    /// Creates a new provider with a default set of common commands.
    pub fn new() -> Self {
        Self {
            commands: default_commands(),
        }
    }

    /// Creates a provider with a custom command list (for testing).
    pub fn with_commands(commands: Vec<(String, String, String)>) -> Self {
        Self {
            commands: commands
                .into_iter()
                .map(|(name, category, description)| CommandEntry {
                    name,
                    category,
                    description,
                })
                .collect(),
        }
    }
}

impl Default for CommandNameProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl CompletionProvider for CommandNameProvider {
    fn id(&self) -> &str {
        "command_name"
    }

    fn is_applicable(&self, context: &CompletionContext) -> bool {
        context.field == CompletionField::PrimaryCommand && context.command_name.is_none()
    }

    fn provide_candidates(
        &self,
        _context: &CompletionContext,
    ) -> Result<Vec<CompletionCandidate>, CompletionError> {
        let candidates = self
            .commands
            .iter()
            .map(|cmd| {
                CompletionCandidate::new(
                    cmd.name.clone(),
                    cmd.name.to_uppercase(),
                    CompletionKind::Command,
                )
                .with_detail(cmd.category.clone())
                .with_description(cmd.description.clone())
            })
            .collect();
        Ok(candidates)
    }
}

/// Returns a default set of common commands for the workbench.
fn default_commands() -> Vec<CommandEntry> {
    vec![
        CommandEntry {
            name: "FIND".to_string(),
            category: "search".to_string(),
            description: "Find text in document".to_string(),
        },
        CommandEntry {
            name: "CHANGE".to_string(),
            category: "search".to_string(),
            description: "Find and replace text".to_string(),
        },
        CommandEntry {
            name: "SAVE".to_string(),
            category: "file".to_string(),
            description: "Save current document".to_string(),
        },
        CommandEntry {
            name: "EDIT".to_string(),
            category: "file".to_string(),
            description: "Open file for editing".to_string(),
        },
        CommandEntry {
            name: "SUBMIT".to_string(),
            category: "file".to_string(),
            description: "Submit changes".to_string(),
        },
        CommandEntry {
            name: "CANCEL".to_string(),
            category: "file".to_string(),
            description: "Cancel editing session".to_string(),
        },
        CommandEntry {
            name: "COPY".to_string(),
            category: "edit".to_string(),
            description: "Copy selected text".to_string(),
        },
        CommandEntry {
            name: "CUT".to_string(),
            category: "edit".to_string(),
            description: "Cut selected text".to_string(),
        },
        CommandEntry {
            name: "PASTE".to_string(),
            category: "edit".to_string(),
            description: "Paste from clipboard".to_string(),
        },
        CommandEntry {
            name: "UNDO".to_string(),
            category: "edit".to_string(),
            description: "Undo last change".to_string(),
        },
        CommandEntry {
            name: "REDO".to_string(),
            category: "edit".to_string(),
            description: "Redo undone change".to_string(),
        },
        CommandEntry {
            name: "RESET".to_string(),
            category: "edit".to_string(),
            description: "Reset document to saved state".to_string(),
        },
        CommandEntry {
            name: "SORT".to_string(),
            category: "edit".to_string(),
            description: "Sort lines in selection".to_string(),
        },
        CommandEntry {
            name: "FILTER".to_string(),
            category: "view".to_string(),
            description: "Filter displayed lines".to_string(),
        },
        CommandEntry {
            name: "EXCLUDE".to_string(),
            category: "view".to_string(),
            description: "Exclude lines from view".to_string(),
        },
        CommandEntry {
            name: "LOCATE".to_string(),
            category: "navigation".to_string(),
            description: "Locate line number".to_string(),
        },
        CommandEntry {
            name: "TOP".to_string(),
            category: "navigation".to_string(),
            description: "Go to top of file".to_string(),
        },
        CommandEntry {
            name: "BOTTOM".to_string(),
            category: "navigation".to_string(),
            description: "Go to bottom of file".to_string(),
        },
        CommandEntry {
            name: "MACRO".to_string(),
            category: "macro".to_string(),
            description: "Run or manage macros".to_string(),
        },
        CommandEntry {
            name: "HELP".to_string(),
            category: "help".to_string(),
            description: "Display help".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::CompletionContextBuilder;

    // Validates: Requirement 1.1 (command name completion in primary field)
    #[test]
    fn applicable_in_command_position() {
        let provider = CommandNameProvider::new();
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .prefix("fi")
            .build();
        assert!(provider.is_applicable(&ctx));
    }

    #[test]
    fn not_applicable_in_argument_position() {
        let provider = CommandNameProvider::new();
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .prefix("path")
            .command_name("FIND")
            .build();
        assert!(!provider.is_applicable(&ctx));
    }

    #[test]
    fn not_applicable_in_prefix_area() {
        let provider = CommandNameProvider::new();
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrefixArea)
            .prefix("C")
            .build();
        assert!(!provider.is_applicable(&ctx));
    }

    // Validates: Requirement 1.3 (candidate includes label, category, description)
    #[test]
    fn provides_command_candidates_with_metadata() {
        let provider = CommandNameProvider::new();
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .prefix("FI")
            .build();

        let candidates = provider.provide_candidates(&ctx).unwrap();
        assert!(!candidates.is_empty());

        let find_candidate = candidates.iter().find(|c| c.label == "FIND").unwrap();
        assert_eq!(find_candidate.kind, CompletionKind::Command);
        assert_eq!(find_candidate.detail.as_deref(), Some("search"));
        assert!(find_candidate.description.is_some());
    }

    // Validates: Requirement 1.5 (insertion value is canonical uppercase)
    #[test]
    fn insertion_value_is_uppercase_canonical() {
        let provider = CommandNameProvider::with_commands(vec![(
            "find".to_string(),
            "search".to_string(),
            "Find text".to_string(),
        )]);
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .build();

        let candidates = provider.provide_candidates(&ctx).unwrap();
        assert_eq!(candidates[0].insert_text, "FIND");
    }
}
