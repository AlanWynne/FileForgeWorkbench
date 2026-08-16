//! Core `Lexer` trait that language-specific implementations must satisfy.

use crate::fold::context::FoldContext;
use crate::style::context::StyleContext;
use crate::types::{KeywordSetDescriptor, PropertyDescriptor, StyleSlotIndex};

/// The core lexer trait that language-specific implementations must satisfy.
/// Each supported language has one or more Lexer implementations.
/// Addresses: Requirement 1
pub trait Lexer: Send + Sync {
    /// Returns the unique identifier of this lexer (e.g., "rust", "cpp", "cobol").
    /// Addresses: Requirement 1, criterion 1.3
    fn name(&self) -> &str;

    /// Perform lexical analysis on the specified text range, assigning
    /// StyleSlotIndex values to each character position via the StyleContext.
    /// Addresses: Requirement 1, criterion 1.1
    fn style_text(&self, context: &mut StyleContext);

    /// Compute FoldLevel values for each line within the specified range.
    /// Addresses: Requirement 1, criterion 1.2
    fn fold_text(&self, context: &mut FoldContext);

    /// Returns the default style-slot index for unstyled text in this language.
    /// Addresses: Requirement 1, criterion 1.4
    fn default_style(&self) -> StyleSlotIndex;

    /// Returns metadata about the keyword sets this lexer supports.
    /// Addresses: Requirement 1, criterion 1.5
    fn keyword_sets(&self) -> &[KeywordSetDescriptor];

    /// Returns the base style indices that support sub-style differentiation.
    /// Addresses: Requirement 1, criterion 1.6
    fn sub_style_bases(&self) -> &[StyleSlotIndex];

    /// Get a lexer-specific property value.
    /// Addresses: Requirement 1, criterion 1.7
    fn get_property(&self, key: &str) -> Option<&str>;

    /// Set a lexer-specific property value.
    /// Addresses: Requirement 1, criterion 1.7
    fn set_property(&mut self, key: &str, value: &str);

    /// Returns metadata about all supported properties for auto-discovery.
    /// Addresses: Requirement 10, criterion 10.6
    fn property_names(&self) -> &[PropertyDescriptor];

    /// Returns the number of base style indices this lexer uses.
    /// Addresses: Requirement 12, criterion 12.4
    fn style_slot_count(&self) -> u8;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::LexerState;
    use std::collections::HashMap;

    /// A minimal test lexer that verifies trait object safety.
    struct TestLexer {
        properties: HashMap<String, String>,
    }

    impl TestLexer {
        fn new() -> Self {
            Self {
                properties: HashMap::new(),
            }
        }
    }

    impl Lexer for TestLexer {
        fn name(&self) -> &str {
            "test"
        }

        fn style_text(&self, context: &mut StyleContext) {
            // Simple: assign default style to everything
            while context.more() {
                context.forward();
            }
            context.set_state(LexerState::INITIAL);
        }

        fn fold_text(&self, _context: &mut FoldContext) {
            // No-op for test
        }

        fn default_style(&self) -> StyleSlotIndex {
            StyleSlotIndex::DEFAULT
        }

        fn keyword_sets(&self) -> &[KeywordSetDescriptor] {
            &[]
        }

        fn sub_style_bases(&self) -> &[StyleSlotIndex] {
            &[]
        }

        fn get_property(&self, key: &str) -> Option<&str> {
            self.properties.get(key).map(|v| v.as_str())
        }

        fn set_property(&mut self, key: &str, value: &str) {
            self.properties.insert(key.to_string(), value.to_string());
        }

        fn property_names(&self) -> &[PropertyDescriptor] {
            &[]
        }

        fn style_slot_count(&self) -> u8 {
            1
        }
    }

    #[test]
    fn lexer_trait_is_object_safe() {
        // Validates: Requirement 1 — Lexer trait can be used as trait object
        let lexer: Box<dyn Lexer> = Box::new(TestLexer::new());
        assert_eq!(lexer.name(), "test");
        assert_eq!(lexer.default_style(), StyleSlotIndex::DEFAULT);
        assert_eq!(lexer.style_slot_count(), 1);
        assert!(lexer.keyword_sets().is_empty());
        assert!(lexer.sub_style_bases().is_empty());
        assert!(lexer.property_names().is_empty());
    }

    #[test]
    fn lexer_property_set_and_get() {
        // Validates: Requirement 1, criterion 1.7
        let mut lexer = TestLexer::new();
        assert_eq!(lexer.get_property("fold.comment"), None);
        lexer.set_property("fold.comment", "1");
        assert_eq!(lexer.get_property("fold.comment"), Some("1"));
    }

    #[test]
    fn lexer_trait_send_sync() {
        // Validates: Requirement 11, criterion 11.5 — thread safety
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Box<dyn Lexer>>();
    }
}
