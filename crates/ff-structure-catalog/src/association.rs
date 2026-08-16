//! Auto-association — file pattern matching.
//!
//! Provides [`FileAssociationMap`] which maps glob patterns to structure names,
//! enabling automatic structure application when files are opened.

use glob::Pattern;

use crate::model::StructureDefinition;

/// Result of matching a filename against the association map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssociationResult {
    /// No matching structure found.
    None,
    /// Exactly one matching structure found.
    Single(String),
    /// Multiple matching structures found (operator must choose).
    Multiple(Vec<String>),
}

/// A single pattern conflict — same glob in multiple structures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternConflict {
    /// The glob pattern that appears in multiple structures.
    pub pattern: String,
    /// Names of the conflicting structures.
    pub structure_names: Vec<String>,
}

/// Maps file glob patterns to structure definition names.
///
/// Built from all loaded structure definitions' file_patterns associations.
/// Used for auto-association when a file is opened.
#[derive(Debug, Default)]
pub struct FileAssociationMap {
    /// Entries: (compiled glob pattern, pattern string, structure name).
    entries: Vec<AssociationEntry>,
}

/// A single entry in the association map.
#[derive(Debug)]
struct AssociationEntry {
    /// The compiled glob pattern.
    pattern: Pattern,
    /// The original pattern string.
    pattern_str: String,
    /// The structure name this pattern maps to.
    structure_name: String,
}

impl FileAssociationMap {
    /// Create an empty association map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Rebuild the map from a set of structure definitions.
    ///
    /// Scans all definitions for file_patterns and builds the lookup index.
    /// Invalid glob patterns are silently skipped.
    pub fn rebuild(&mut self, definitions: &[&StructureDefinition]) {
        self.entries.clear();

        for def in definitions {
            if let Some(ref assoc) = def.associations {
                for pattern_str in &assoc.file_patterns {
                    if let Ok(pattern) = Pattern::new(pattern_str) {
                        self.entries.push(AssociationEntry {
                            pattern,
                            pattern_str: pattern_str.clone(),
                            structure_name: def.metadata.name.clone(),
                        });
                    }
                }
            }
        }
    }

    /// Match a filename against all patterns in the map.
    ///
    /// Returns the association result: None, Single, or Multiple matches.
    pub fn match_file(&self, filename: &str) -> AssociationResult {
        let mut matches: Vec<String> = self
            .entries
            .iter()
            .filter(|entry| entry.pattern.matches(filename))
            .map(|entry| entry.structure_name.clone())
            .collect();

        // Deduplicate (same structure could match via multiple patterns)
        matches.sort();
        matches.dedup();

        match matches.len() {
            0 => AssociationResult::None,
            1 => AssociationResult::Single(matches.into_iter().next().unwrap()),
            _ => AssociationResult::Multiple(matches),
        }
    }

    /// Detect conflicting patterns — same pattern in multiple structures.
    ///
    /// A conflict occurs when the same glob pattern string appears in more
    /// than one structure definition. Per Requirement 10.7, the first match
    /// alphabetically is used.
    pub fn find_conflicts(&self) -> Vec<PatternConflict> {
        use std::collections::HashMap;

        let mut pattern_to_names: HashMap<&str, Vec<&str>> = HashMap::new();
        for entry in &self.entries {
            pattern_to_names
                .entry(&entry.pattern_str)
                .or_default()
                .push(&entry.structure_name);
        }

        let mut conflicts = Vec::new();
        for (pattern, mut names) in pattern_to_names {
            names.sort();
            names.dedup();
            if names.len() > 1 {
                conflicts.push(PatternConflict {
                    pattern: pattern.to_string(),
                    structure_names: names.into_iter().map(String::from).collect(),
                });
            }
        }
        conflicts.sort_by(|a, b| a.pattern.cmp(&b.pattern));
        conflicts
    }

    /// Return the number of entries in the map.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Return whether the map is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::{FieldDefinition, FieldType};
    use crate::model::{FileAssociations, RecordStructure, StructureMetadata};

    fn def_with_patterns(name: &str, patterns: Vec<&str>) -> StructureDefinition {
        StructureDefinition {
            metadata: StructureMetadata::new(name),
            associations: Some(FileAssociations {
                file_patterns: patterns.into_iter().map(String::from).collect(),
            }),
            record_structures: vec![RecordStructure::with_fields(
                "Default",
                vec![FieldDefinition::new("F1", 0, 10, FieldType::Alphanumeric)],
            )],
        }
    }

    // Validates: Requirement 10.3 — single match auto-applies
    #[test]
    fn match_file_returns_single_for_one_match() {
        let def = def_with_patterns("CUSTOMER", vec!["*.dat"]);
        let mut map = FileAssociationMap::new();
        map.rebuild(&[&def]);

        assert_eq!(
            map.match_file("invoice.dat"),
            AssociationResult::Single("CUSTOMER".to_string())
        );
    }

    // Validates: Requirement 10.6 — no match returns None
    #[test]
    fn match_file_returns_none_for_no_match() {
        let def = def_with_patterns("CUSTOMER", vec!["*.dat"]);
        let mut map = FileAssociationMap::new();
        map.rebuild(&[&def]);

        assert_eq!(map.match_file("readme.txt"), AssociationResult::None);
    }

    // Validates: Requirement 10.5 — multiple matches
    #[test]
    fn match_file_returns_multiple_for_overlapping_patterns() {
        let def1 = def_with_patterns("ALPHA", vec!["*.dat"]);
        let def2 = def_with_patterns("BETA", vec!["invoice.*"]);
        let mut map = FileAssociationMap::new();
        map.rebuild(&[&def1, &def2]);

        let result = map.match_file("invoice.dat");
        assert!(matches!(result, AssociationResult::Multiple(_)));
        if let AssociationResult::Multiple(names) = result {
            assert!(names.contains(&"ALPHA".to_string()));
            assert!(names.contains(&"BETA".to_string()));
        }
    }

    // Validates: Requirement 10.7 — conflict detection
    #[test]
    fn find_conflicts_detects_same_pattern_in_multiple_structures() {
        let def1 = def_with_patterns("A_STRUCT", vec!["*.dat"]);
        let def2 = def_with_patterns("B_STRUCT", vec!["*.dat"]);
        let mut map = FileAssociationMap::new();
        map.rebuild(&[&def1, &def2]);

        let conflicts = map.find_conflicts();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].pattern, "*.dat");
        assert_eq!(conflicts[0].structure_names.len(), 2);
    }

    // Validates: Requirement 10.8 — no patterns is valid
    #[test]
    fn definition_without_patterns_produces_no_entries() {
        let def = StructureDefinition {
            metadata: StructureMetadata::new("NO_PATTERNS"),
            associations: None,
            record_structures: vec![RecordStructure::new("Default")],
        };
        let mut map = FileAssociationMap::new();
        map.rebuild(&[&def]);
        assert!(map.is_empty());
    }

    // Validates: Requirement 10.1 — glob pattern matching
    #[test]
    fn glob_patterns_match_correctly() {
        let def = def_with_patterns("STRUCTURED", vec!["CUST_*.dat", "INV??????.txt"]);
        let mut map = FileAssociationMap::new();
        map.rebuild(&[&def]);

        assert_eq!(
            map.match_file("CUST_MASTER.dat"),
            AssociationResult::Single("STRUCTURED".to_string())
        );
        assert_eq!(
            map.match_file("INV123456.txt"),
            AssociationResult::Single("STRUCTURED".to_string())
        );
        assert_eq!(map.match_file("OTHER.dat"), AssociationResult::None);
    }

    // Validates: Requirement 10.2 — map rebuilt on reload
    #[test]
    fn rebuild_replaces_previous_entries() {
        let def1 = def_with_patterns("FIRST", vec!["*.dat"]);
        let def2 = def_with_patterns("SECOND", vec!["*.txt"]);
        let mut map = FileAssociationMap::new();

        map.rebuild(&[&def1]);
        assert_eq!(map.len(), 1);

        map.rebuild(&[&def2]);
        assert_eq!(map.len(), 1);
        assert_eq!(
            map.match_file("file.txt"),
            AssociationResult::Single("SECOND".to_string())
        );
        assert_eq!(map.match_file("file.dat"), AssociationResult::None);
    }
}
