//! Keyword set management: sorted storage with first-character indexing.

use std::collections::HashMap;

/// A sorted list of keywords for a single keyword category (0–8).
///
/// Uses a first-character index for O(1) lookup of the starting position
/// for each initial ASCII character.
#[derive(Debug, Clone)]
pub struct KeywordSet {
    /// Keywords sorted alphabetically.
    words: Vec<String>,
    /// Index mapping first character (as byte) to starting position in `words`.
    /// `char_index[c]` holds the index of the first word starting with byte `c`,
    /// or `u32::MAX` if no words start with that character.
    char_index: [u32; 128],
    /// Optional semantic name for this keyword set.
    semantic_name: Option<String>,
}

impl KeywordSet {
    /// Create a new keyword set from a list of words.
    ///
    /// Words are sorted alphabetically. If `case_sensitive` is false,
    /// all words are stored in lowercase.
    pub fn new(mut words: Vec<String>, case_sensitive: bool) -> Self {
        if !case_sensitive {
            words = words.into_iter().map(|w| w.to_lowercase()).collect();
        }
        words.sort();
        words.dedup();

        let mut char_index = [u32::MAX; 128];
        for (i, word) in words.iter().enumerate() {
            if let Some(&first_byte) = word.as_bytes().first() {
                let idx = first_byte as usize;
                if idx < 128 && char_index[idx] == u32::MAX {
                    char_index[idx] = i as u32;
                }
            }
        }

        Self {
            words,
            char_index,
            semantic_name: None,
        }
    }

    /// Perform case-sensitive membership test.
    pub fn contains(&self, word: &str) -> bool {
        if word.is_empty() {
            return false;
        }
        let first_byte = word.as_bytes()[0] as usize;
        if first_byte >= 128 {
            return false;
        }
        let start = self.char_index[first_byte];
        if start == u32::MAX {
            return false;
        }
        let start = start as usize;
        // Binary search within the range of words starting with this character
        self.words[start..]
            .binary_search_by(|probe| {
                if probe.as_bytes().first().copied() != Some(first_byte as u8) {
                    std::cmp::Ordering::Greater
                } else {
                    probe.as_str().cmp(word)
                }
            })
            .is_ok()
    }

    /// Perform case-insensitive membership test.
    ///
    /// The query is lowercased and compared against the stored (potentially lowercased) keywords.
    pub fn contains_case_insensitive(&self, word: &str) -> bool {
        let lowered = word.to_lowercase();
        self.contains(&lowered)
    }

    /// Returns the number of keywords in this set.
    pub fn len(&self) -> usize {
        self.words.len()
    }

    /// Returns true if this set is empty.
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// Returns the semantic name for this keyword set.
    pub fn semantic_name(&self) -> Option<&str> {
        self.semantic_name.as_deref()
    }

    /// Sets the semantic name for this keyword set.
    pub fn set_semantic_name(&mut self, name: String) {
        self.semantic_name = Some(name);
    }
}

/// Default style name mappings for keyword set numbers.
const DEFAULT_SET_NAMES: [&str; 9] = [
    "keyword",
    "type",
    "builtin",
    "constant",
    "function",
    "preprocessor",
    "annotation",
    "operator",
    "reserved",
];

/// Collection of up to 9 keyword sets for a language definition.
#[derive(Debug, Clone)]
pub struct KeywordSets {
    /// The keyword sets (index corresponds to set number 0–8).
    sets: [Option<KeywordSet>; 9],
}

impl KeywordSets {
    /// Create an empty keyword sets collection.
    pub fn empty() -> Self {
        Self {
            sets: Default::default(),
        }
    }

    /// Parse keyword sets from a TOML keywords table.
    ///
    /// The table maps string keys "0"–"8" to arrays of keyword strings.
    pub fn from_toml_table(table: &HashMap<String, Vec<String>>, case_sensitive: bool) -> Self {
        let mut sets: [Option<KeywordSet>; 9] = Default::default();
        for (key, words) in table {
            if let Ok(num) = key.parse::<u8>() {
                if num <= 8 {
                    let mut ks = KeywordSet::new(words.clone(), case_sensitive);
                    ks.set_semantic_name(DEFAULT_SET_NAMES[num as usize].to_string());
                    sets[num as usize] = Some(ks);
                }
            }
        }
        Self { sets }
    }

    /// Check case-sensitive membership in a specific keyword set.
    pub fn in_keyword_set(&self, word: &str, set_number: u8) -> bool {
        if set_number > 8 {
            return false;
        }
        self.sets[set_number as usize]
            .as_ref()
            .is_some_and(|ks| ks.contains(word))
    }

    /// Check case-insensitive membership in a specific keyword set.
    pub fn in_keyword_set_case_insensitive(&self, word: &str, set_number: u8) -> bool {
        if set_number > 8 {
            return false;
        }
        self.sets[set_number as usize]
            .as_ref()
            .is_some_and(|ks| ks.contains_case_insensitive(word))
    }

    /// Returns the style name for a keyword set number.
    pub fn style_name_for_set(set_number: u8) -> &'static str {
        if (set_number as usize) < DEFAULT_SET_NAMES.len() {
            DEFAULT_SET_NAMES[set_number as usize]
        } else {
            "keyword"
        }
    }

    /// Returns the keyword set at the given index.
    pub fn get(&self, set_number: u8) -> Option<&KeywordSet> {
        if set_number > 8 {
            return None;
        }
        self.sets[set_number as usize].as_ref()
    }

    /// Returns the number of non-empty keyword sets.
    pub fn count(&self) -> usize {
        self.sets.iter().filter(|s| s.is_some()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyword_set_new_sorts_words() {
        // Validates: Requirement 5.3
        let ks = KeywordSet::new(
            vec!["while".to_string(), "for".to_string(), "if".to_string()],
            true,
        );
        assert_eq!(ks.words, vec!["for", "if", "while"]);
    }

    #[test]
    fn keyword_set_contains_finds_present_word() {
        // Validates: Requirement 5.4
        let ks = KeywordSet::new(
            vec!["fn".to_string(), "let".to_string(), "mut".to_string()],
            true,
        );
        assert!(ks.contains("fn"));
        assert!(ks.contains("let"));
        assert!(ks.contains("mut"));
    }

    #[test]
    fn keyword_set_contains_rejects_absent_word() {
        // Validates: Requirement 5.4
        let ks = KeywordSet::new(
            vec!["fn".to_string(), "let".to_string(), "mut".to_string()],
            true,
        );
        assert!(!ks.contains("var"));
        assert!(!ks.contains("const"));
    }

    #[test]
    fn keyword_set_contains_is_case_sensitive() {
        // Validates: Requirement 5.4
        let ks = KeywordSet::new(vec!["fn".to_string(), "Let".to_string()], true);
        assert!(ks.contains("fn"));
        assert!(ks.contains("Let"));
        assert!(!ks.contains("FN"));
        assert!(!ks.contains("let"));
    }

    #[test]
    fn keyword_set_contains_case_insensitive_matches_any_casing() {
        // Validates: Requirement 5.5
        let ks = KeywordSet::new(
            vec!["function".to_string(), "class".to_string()],
            false, // stored lowercase
        );
        assert!(ks.contains_case_insensitive("FUNCTION"));
        assert!(ks.contains_case_insensitive("Function"));
        assert!(ks.contains_case_insensitive("function"));
        assert!(ks.contains_case_insensitive("CLASS"));
    }

    #[test]
    fn keyword_set_empty_set_returns_false() {
        // Validates: Requirement 5.4
        let ks = KeywordSet::new(Vec::new(), true);
        assert!(!ks.contains("anything"));
        assert!(ks.is_empty());
    }

    #[test]
    fn keyword_set_contains_rejects_empty_word() {
        // Validates: Requirement 5.4
        let ks = KeywordSet::new(vec!["fn".to_string()], true);
        assert!(!ks.contains(""));
    }

    #[test]
    fn keyword_sets_from_toml_table_parses_correctly() {
        // Validates: Requirement 5.2
        let mut table = HashMap::new();
        table.insert("0".to_string(), vec!["if".to_string(), "else".to_string()]);
        table.insert(
            "1".to_string(),
            vec!["int".to_string(), "float".to_string()],
        );

        let sets = KeywordSets::from_toml_table(&table, true);
        assert!(sets.in_keyword_set("if", 0));
        assert!(sets.in_keyword_set("else", 0));
        assert!(sets.in_keyword_set("int", 1));
        assert!(!sets.in_keyword_set("if", 1));
    }

    #[test]
    fn keyword_sets_out_of_range_returns_false() {
        // Validates: Requirement 5.4
        let sets = KeywordSets::empty();
        assert!(!sets.in_keyword_set("test", 9));
        assert!(!sets.in_keyword_set("test", 255));
    }

    #[test]
    fn keyword_sets_style_name_for_set_returns_defaults() {
        // Validates: Requirement 5.7
        assert_eq!(KeywordSets::style_name_for_set(0), "keyword");
        assert_eq!(KeywordSets::style_name_for_set(1), "type");
        assert_eq!(KeywordSets::style_name_for_set(2), "builtin");
    }

    #[test]
    fn keyword_set_case_insensitive_storage_lowercases() {
        // Validates: Requirement 5.6
        let ks = KeywordSet::new(
            vec![
                "BEGIN".to_string(),
                "End".to_string(),
                "PERFORM".to_string(),
            ],
            false,
        );
        assert!(ks.contains("begin"));
        assert!(ks.contains("end"));
        assert!(ks.contains("perform"));
        assert!(!ks.contains("BEGIN")); // stored lowercase, case-sensitive won't match
    }

    #[test]
    fn keyword_sets_case_insensitive_lookup() {
        // Validates: Requirement 5.5
        let mut table = HashMap::new();
        table.insert(
            "0".to_string(),
            vec!["PERFORM".to_string(), "DISPLAY".to_string()],
        );

        let sets = KeywordSets::from_toml_table(&table, false);
        assert!(sets.in_keyword_set_case_insensitive("perform", 0));
        assert!(sets.in_keyword_set_case_insensitive("PERFORM", 0));
        assert!(sets.in_keyword_set_case_insensitive("Perform", 0));
    }
}
