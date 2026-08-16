//! Built-in provider for keyword/modifier completion.
//!
//! Provides keyword completions for command arguments that accept a known set
//! of modifiers or keywords (e.g., FIND modifiers, scope modifiers).

use crate::candidate::{CompletionCandidate, CompletionKind};
use crate::context::{CompletionContext, CompletionField};
use crate::error::CompletionError;
use crate::provider::CompletionProvider;

/// Provides keyword/modifier completion candidates.
///
/// Active when the cursor is in argument position and the command
/// has known keyword sets for that argument position.
pub struct KeywordProvider;

impl KeywordProvider {
    /// Creates a new keyword provider.
    pub fn new() -> Self {
        Self
    }
}

impl Default for KeywordProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl CompletionProvider for KeywordProvider {
    fn id(&self) -> &str {
        "keyword"
    }

    fn is_applicable(&self, context: &CompletionContext) -> bool {
        context.field == CompletionField::PrimaryCommand
            && context.command_name.is_some()
            && context.argument_index.is_some()
    }

    fn provide_candidates(
        &self,
        context: &CompletionContext,
    ) -> Result<Vec<CompletionCandidate>, CompletionError> {
        let command_name = match &context.command_name {
            Some(name) => name.to_uppercase(),
            None => return Ok(vec![]),
        };

        let keywords = get_keywords_for_command(&command_name, context.argument_index.unwrap_or(0));
        let candidates = keywords
            .into_iter()
            .map(|(keyword, description)| {
                CompletionCandidate::new(keyword.clone(), keyword, CompletionKind::Keyword)
                    .with_description(description)
            })
            .collect();
        Ok(candidates)
    }
}

/// Returns the keyword set applicable for a command at a given argument position.
fn get_keywords_for_command(command: &str, argument_index: usize) -> Vec<(String, String)> {
    match (command, argument_index) {
        ("FIND", 1) | ("CHANGE", 2) => find_modifiers(),
        ("FIND", 2) | ("CHANGE", 3) | ("EXCLUDE", 1) | ("FILTER", 1) => scope_modifiers(),
        _ => vec![],
    }
}

/// FIND command modifiers for match type.
fn find_modifiers() -> Vec<(String, String)> {
    vec![
        (
            "CHARS".to_string(),
            "Match individual characters".to_string(),
        ),
        ("PREFIX".to_string(), "Match at word start".to_string()),
        ("SUFFIX".to_string(), "Match at word end".to_string()),
        ("WORD".to_string(), "Match whole word".to_string()),
    ]
}

/// Scope modifiers for filtering/search commands.
fn scope_modifiers() -> Vec<(String, String)> {
    vec![
        ("ALL".to_string(), "Search all lines".to_string()),
        (
            "VISIBLE".to_string(),
            "Search visible lines only".to_string(),
        ),
        (
            "EXCLUDED".to_string(),
            "Search excluded lines only".to_string(),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::CompletionContextBuilder;

    // Validates: Requirement 2.4 (keyword completion for known commands)
    #[test]
    fn applicable_in_argument_position() {
        let provider = KeywordProvider::new();
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .command_name("FIND")
            .argument_index(1)
            .prefix("CH")
            .build();
        assert!(provider.is_applicable(&ctx));
    }

    #[test]
    fn not_applicable_in_command_position() {
        let provider = KeywordProvider::new();
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .prefix("FIND")
            .build();
        assert!(!provider.is_applicable(&ctx));
    }

    // Validates: Requirement 2.4 (FIND modifiers)
    #[test]
    fn provides_find_modifiers_at_position_1() {
        let provider = KeywordProvider::new();
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .command_name("FIND")
            .argument_index(1)
            .build();

        let candidates = provider.provide_candidates(&ctx).unwrap();
        let labels: Vec<_> = candidates.iter().map(|c| c.label.as_str()).collect();
        assert!(labels.contains(&"CHARS"));
        assert!(labels.contains(&"PREFIX"));
        assert!(labels.contains(&"SUFFIX"));
        assert!(labels.contains(&"WORD"));
    }

    // Validates: Requirement 2.4 (scope modifiers)
    #[test]
    fn provides_scope_modifiers_for_find_position_2() {
        let provider = KeywordProvider::new();
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .command_name("FIND")
            .argument_index(2)
            .build();

        let candidates = provider.provide_candidates(&ctx).unwrap();
        let labels: Vec<_> = candidates.iter().map(|c| c.label.as_str()).collect();
        assert!(labels.contains(&"ALL"));
        assert!(labels.contains(&"VISIBLE"));
        assert!(labels.contains(&"EXCLUDED"));
    }

    #[test]
    fn returns_empty_for_unknown_command() {
        let provider = KeywordProvider::new();
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .command_name("UNKNOWN")
            .argument_index(0)
            .build();

        let candidates = provider.provide_candidates(&ctx).unwrap();
        assert!(candidates.is_empty());
    }
}
