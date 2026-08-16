//! WordList: hash-based keyword storage for O(1) average-case lookup during lexing.

use std::collections::HashSet;

use crate::types::StyleSlotIndex;

/// Hash-based keyword storage for O(1) average-case lookup during lexing.
/// Addresses: Requirement 5, criterion 5.3
pub struct WordList {
    /// Keywords stored in a HashSet for fast lookup.
    /// If case_insensitive, stored in lowercase.
    words: HashSet<String>,
    /// Whether lookups are case-insensitive.
    case_insensitive: bool,
    /// The style-slot index assigned when a keyword matches.
    style: StyleSlotIndex,
}

impl WordList {
    /// Create a new empty WordList.
    pub fn new(style: StyleSlotIndex, case_insensitive: bool) -> Self {
        Self {
            words: HashSet::new(),
            case_insensitive,
            style,
        }
    }

    /// Create a WordList pre-populated with keywords.
    pub fn with_words(words: &[&str], style: StyleSlotIndex, case_insensitive: bool) -> Self {
        let mut wl = Self::new(style, case_insensitive);
        for word in words {
            wl.add(word);
        }
        wl
    }

    /// Add a keyword to the list.
    /// Addresses: Requirement 5, criterion 5.8
    pub fn add(&mut self, word: &str) {
        if self.case_insensitive {
            self.words.insert(word.to_lowercase());
        } else {
            self.words.insert(word.to_string());
        }
    }

    /// Remove a keyword from the list.
    /// Addresses: Requirement 5, criterion 5.8
    pub fn remove(&mut self, word: &str) -> bool {
        if self.case_insensitive {
            self.words.remove(&word.to_lowercase())
        } else {
            self.words.remove(word)
        }
    }

    /// Check if an identifier matches a keyword in this set.
    /// Performs case-folded comparison when case_insensitive is true.
    /// Addresses: Requirement 5, criteria 5.6–5.7
    pub fn contains(&self, word: &str) -> bool {
        if self.case_insensitive {
            self.words.contains(&word.to_lowercase())
        } else {
            self.words.contains(word)
        }
    }

    /// Get the style-slot index for matches in this set.
    pub fn style(&self) -> StyleSlotIndex {
        self.style
    }

    /// Get the number of keywords in this set.
    pub fn len(&self) -> usize {
        self.words.len()
    }

    /// Check if the word list is empty.
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// Check if this word list is case-insensitive.
    pub fn is_case_insensitive(&self) -> bool {
        self.case_insensitive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_sensitive_exact_match() {
        // Validates: Requirement 5, criterion 5.6
        let wl = WordList::with_words(&["fn", "let", "mut"], StyleSlotIndex(1), false);
        assert!(wl.contains("fn"));
        assert!(wl.contains("let"));
        assert!(wl.contains("mut"));
        assert!(!wl.contains("Fn"));
        assert!(!wl.contains("FN"));
        assert!(!wl.contains("unknown"));
    }

    #[test]
    fn case_insensitive_match() {
        // Validates: Requirement 5, criterion 5.7
        let wl = WordList::with_words(&["begin", "end", "if"], StyleSlotIndex(2), true);
        assert!(wl.contains("begin"));
        assert!(wl.contains("BEGIN"));
        assert!(wl.contains("Begin"));
        assert!(wl.contains("bEgIn"));
        assert!(!wl.contains("unknown"));
    }

    #[test]
    fn add_and_remove() {
        // Validates: Requirement 5, criterion 5.8
        let mut wl = WordList::new(StyleSlotIndex(1), false);
        assert!(wl.is_empty());
        wl.add("hello");
        assert_eq!(wl.len(), 1);
        assert!(wl.contains("hello"));
        assert!(wl.remove("hello"));
        assert!(wl.is_empty());
        assert!(!wl.contains("hello"));
    }

    #[test]
    fn add_case_insensitive_stores_lowercase() {
        let mut wl = WordList::new(StyleSlotIndex(1), true);
        wl.add("HELLO");
        assert!(wl.contains("hello"));
        assert!(wl.contains("HELLO"));
        assert!(wl.contains("Hello"));
    }

    #[test]
    fn remove_case_insensitive() {
        let mut wl = WordList::new(StyleSlotIndex(1), true);
        wl.add("hello");
        assert!(wl.remove("HELLO"));
        assert!(!wl.contains("hello"));
    }

    #[test]
    fn style_returns_configured_style() {
        let wl = WordList::new(StyleSlotIndex(5), false);
        assert_eq!(wl.style(), StyleSlotIndex(5));
    }

    #[test]
    fn empty_word_list_contains_nothing() {
        let wl = WordList::new(StyleSlotIndex(1), false);
        assert!(!wl.contains("anything"));
        assert!(!wl.contains(""));
    }

    #[test]
    fn with_words_constructor() {
        let wl = WordList::with_words(&["a", "b", "c"], StyleSlotIndex(3), false);
        assert_eq!(wl.len(), 3);
        assert!(wl.contains("a"));
        assert!(wl.contains("b"));
        assert!(wl.contains("c"));
        assert!(!wl.contains("d"));
    }
}
