//! Built-in provider for line command completion.
//!
//! Provides line command kinds (C, CC, M, MM, D, DD, etc.) as candidates
//! when the user is typing in the prefix area.

use crate::candidate::{CompletionCandidate, CompletionKind};
use crate::context::{CompletionContext, CompletionField};
use crate::error::CompletionError;
use crate::provider::CompletionProvider;

/// Provides line command completion candidates for the prefix area.
///
/// Active only when the completion field is `PrefixArea`.
pub struct LineCommandProvider;

impl LineCommandProvider {
    /// Creates a new line command provider.
    pub fn new() -> Self {
        Self
    }
}

impl Default for LineCommandProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl CompletionProvider for LineCommandProvider {
    fn id(&self) -> &str {
        "line_command"
    }

    fn is_applicable(&self, context: &CompletionContext) -> bool {
        context.field == CompletionField::PrefixArea
    }

    fn provide_candidates(
        &self,
        _context: &CompletionContext,
    ) -> Result<Vec<CompletionCandidate>, CompletionError> {
        let candidates = LINE_COMMANDS
            .iter()
            .map(|(kind, description)| {
                CompletionCandidate::new(
                    kind.to_string(),
                    kind.to_string(),
                    CompletionKind::LineCommand,
                )
                .with_description(description.to_string())
            })
            .collect();
        Ok(candidates)
    }
}

/// The complete set of line command kinds with descriptions.
const LINE_COMMANDS: &[(&str, &str)] = &[
    ("C", "Copy line"),
    ("CC", "Copy block start/end"),
    ("M", "Move line"),
    ("MM", "Move block start/end"),
    ("D", "Delete line"),
    ("DD", "Delete block start/end"),
    ("R", "Repeat (duplicate) line"),
    ("RR", "Repeat block start/end"),
    ("X", "Exclude line from display"),
    ("XX", "Exclude block start/end"),
    ("I", "Insert line after"),
    ("A", "Insert line after (alias)"),
    ("B", "Insert line before"),
    ("O", "Overlay line"),
    ("W", "Write line to dataset"),
    ("S", "Show excluded line"),
    ("T", "Shift text right (tab)"),
    ("TT", "Shift block right start/end"),
    ("U", "Shift text left (untab)"),
    ("UU", "Shift block left start/end"),
    (">", "Shift data right"),
    (">>", "Shift data right block start/end"),
    ("<", "Shift data left"),
    ("<<", "Shift data left block start/end"),
    (")", "Indent right"),
    ("))", "Indent right block start/end"),
    ("(", "Indent left"),
    ("((", "Indent left block start/end"),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::CompletionContextBuilder;

    // Validates: Requirement 7.1 (line command completion in prefix area)
    #[test]
    fn applicable_in_prefix_area() {
        let provider = LineCommandProvider::new();
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrefixArea)
            .prefix("C")
            .build();
        assert!(provider.is_applicable(&ctx));
    }

    #[test]
    fn not_applicable_in_primary_command() {
        let provider = LineCommandProvider::new();
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .prefix("C")
            .build();
        assert!(!provider.is_applicable(&ctx));
    }

    // Validates: Requirement 7.1 (all valid line command kinds)
    #[test]
    fn provides_all_line_command_kinds() {
        let provider = LineCommandProvider::new();
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrefixArea)
            .build();

        let candidates = provider.provide_candidates(&ctx).unwrap();
        assert_eq!(candidates.len(), LINE_COMMANDS.len());

        let labels: Vec<_> = candidates.iter().map(|c| c.label.as_str()).collect();
        assert!(labels.contains(&"C"));
        assert!(labels.contains(&"CC"));
        assert!(labels.contains(&"D"));
        assert!(labels.contains(&"DD"));
        assert!(labels.contains(&"M"));
        assert!(labels.contains(&"MM"));
        assert!(labels.contains(&">"));
        assert!(labels.contains(&">>"));
        assert!(labels.contains(&"("));
        assert!(labels.contains(&"(("));
    }

    // Validates: Requirement 7.2 (candidates include description)
    #[test]
    fn candidates_have_descriptions() {
        let provider = LineCommandProvider::new();
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrefixArea)
            .build();

        let candidates = provider.provide_candidates(&ctx).unwrap();
        for c in &candidates {
            assert!(c.description.is_some());
            assert_eq!(c.kind, CompletionKind::LineCommand);
        }
    }
}
