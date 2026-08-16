//! Built-in provider for macro name completion.
//!
//! Queries the Lua macro engine for registered macro names.
//! Returns an empty list when no macros are registered.

use crate::candidate::{CompletionCandidate, CompletionKind};
use crate::context::{CompletionContext, CompletionField};
use crate::error::CompletionError;
use crate::provider::CompletionProvider;

/// Provides macro name completion candidates from the Lua macro engine.
///
/// Active when the command is a macro invocation context
/// (e.g., command name is "MACRO" and argument position is the macro name).
pub struct MacroNameProvider {
    /// Mock macro list. In production this queries the actual macro engine.
    macros: Vec<MacroEntry>,
}

/// A simplified macro entry.
#[derive(Debug, Clone)]
struct MacroEntry {
    name: String,
    file_path: String,
    description: Option<String>,
}

impl MacroNameProvider {
    /// Creates a new macro name provider with an empty macro list.
    ///
    /// Returns no candidates until macros are registered.
    pub fn new() -> Self {
        Self { macros: vec![] }
    }

    /// Creates a provider with a custom macro list (for testing).
    pub fn with_macros(macros: Vec<(String, String, Option<String>)>) -> Self {
        Self {
            macros: macros
                .into_iter()
                .map(|(name, path, desc)| MacroEntry {
                    name,
                    file_path: path,
                    description: desc,
                })
                .collect(),
        }
    }
}

impl Default for MacroNameProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl CompletionProvider for MacroNameProvider {
    fn id(&self) -> &str {
        "macro_name"
    }

    fn is_applicable(&self, context: &CompletionContext) -> bool {
        context.field == CompletionField::PrimaryCommand
            && context.command_name.as_deref() == Some("MACRO")
            && context.argument_index == Some(0)
    }

    fn provide_candidates(
        &self,
        _context: &CompletionContext,
    ) -> Result<Vec<CompletionCandidate>, CompletionError> {
        if self.macros.is_empty() {
            return Ok(vec![]);
        }

        let candidates = self
            .macros
            .iter()
            .map(|m| {
                let mut candidate =
                    CompletionCandidate::new(m.name.clone(), m.name.clone(), CompletionKind::Macro)
                        .with_detail(m.file_path.clone());

                if let Some(desc) = &m.description {
                    candidate = candidate.with_description(desc.clone());
                }
                candidate
            })
            .collect();
        Ok(candidates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::CompletionContextBuilder;

    // Validates: Requirement 8.1 (macro name completion in MACRO command context)
    #[test]
    fn applicable_for_macro_command_argument() {
        let provider = MacroNameProvider::new();
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .command_name("MACRO")
            .argument_index(0)
            .prefix("my")
            .build();
        assert!(provider.is_applicable(&ctx));
    }

    #[test]
    fn not_applicable_for_other_commands() {
        let provider = MacroNameProvider::new();
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .command_name("FIND")
            .argument_index(0)
            .build();
        assert!(!provider.is_applicable(&ctx));
    }

    // Validates: Requirement 8.5 (empty list when no macros registered)
    #[test]
    fn returns_empty_when_no_macros_registered() {
        let provider = MacroNameProvider::new();
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .command_name("MACRO")
            .argument_index(0)
            .build();

        let candidates = provider.provide_candidates(&ctx).unwrap();
        assert!(candidates.is_empty());
    }

    // Validates: Requirement 8.2 (macro candidates include name and path)
    #[test]
    fn provides_macro_candidates_with_metadata() {
        let provider = MacroNameProvider::with_macros(vec![
            (
                "format_code".to_string(),
                "/macros/format_code.lua".to_string(),
                Some("Format source code".to_string()),
            ),
            (
                "lint_check".to_string(),
                "/macros/lint_check.lua".to_string(),
                None,
            ),
        ]);
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .command_name("MACRO")
            .argument_index(0)
            .build();

        let candidates = provider.provide_candidates(&ctx).unwrap();
        assert_eq!(candidates.len(), 2);

        let format = &candidates[0];
        assert_eq!(format.label, "format_code");
        assert_eq!(format.insert_text, "format_code");
        assert_eq!(format.kind, CompletionKind::Macro);
        assert_eq!(format.detail.as_deref(), Some("/macros/format_code.lua"));
        assert_eq!(format.description.as_deref(), Some("Format source code"));

        let lint = &candidates[1];
        assert_eq!(lint.label, "lint_check");
        assert!(lint.description.is_none());
    }
}
