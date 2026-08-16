//! Structure association and auto-suggestion logic.
//!
//! Provides auto-suggestion of saved criteria sets when a matching
//! structure is activated.

use std::path::Path;

use crate::error::CriteriaError;
use crate::persistence::{CriteriaPersistence, CriteriaSetMetadata};

/// Provides auto-suggestion logic for applying saved criteria
/// when a matching structure is activated.
///
/// Addresses: Requirement 12
pub struct StructureAssociation;

impl StructureAssociation {
    /// Find saved criteria sets whose structure_association matches
    /// the given structure name (case-insensitive).
    ///
    /// Addresses: Requirement 12 AC 1, 5
    pub fn find_matching(
        location: &Path,
        structure_name: &str,
    ) -> Result<Vec<CriteriaSetMetadata>, CriteriaError> {
        let all_sets = CriteriaPersistence::list(location)?;
        let target = structure_name.to_lowercase();

        let matching: Vec<CriteriaSetMetadata> = all_sets
            .into_iter()
            .filter(|m| {
                m.structure_association
                    .as_ref()
                    .map(|sa| sa.to_lowercase() == target)
                    .unwrap_or(false)
            })
            .collect();

        Ok(matching)
    }

    /// Get the most recently modified matching criteria set.
    ///
    /// Returns the first match found (since we don't track modification times
    /// in metadata, returns the first match as a reasonable approximation).
    ///
    /// Addresses: Requirement 12 AC 1
    pub fn most_recent_match(
        location: &Path,
        structure_name: &str,
    ) -> Result<Option<CriteriaSetMetadata>, CriteriaError> {
        let matching = Self::find_matching(location, structure_name)?;
        Ok(matching.into_iter().next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{CriteriaOperator, CriteriaSet};
    use crate::persistence::CriteriaPersistence;
    use tempfile::TempDir;

    fn make_criteria_with_association(name: &str, association: &str) -> CriteriaSet {
        CriteriaSet {
            name: Some(name.to_string()),
            structure_association: Some(association.to_string()),
            ..CriteriaSet::single("FIELD", CriteriaOperator::Eq, "val")
        }
    }

    #[test]
    fn find_matching_returns_matching_sets() {
        let dir = TempDir::new().unwrap();
        CriteriaPersistence::save(
            dir.path(),
            &make_criteria_with_association("filter1", "MY_STRUCT"),
        )
        .unwrap();
        CriteriaPersistence::save(
            dir.path(),
            &make_criteria_with_association("filter2", "OTHER_STRUCT"),
        )
        .unwrap();
        CriteriaPersistence::save(
            dir.path(),
            &make_criteria_with_association("filter3", "MY_STRUCT"),
        )
        .unwrap();

        let matches = StructureAssociation::find_matching(dir.path(), "MY_STRUCT").unwrap();
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn find_matching_is_case_insensitive() {
        let dir = TempDir::new().unwrap();
        CriteriaPersistence::save(
            dir.path(),
            &make_criteria_with_association("filter1", "My_Struct"),
        )
        .unwrap();

        let matches = StructureAssociation::find_matching(dir.path(), "my_struct").unwrap();
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn find_matching_no_matches_returns_empty() {
        let dir = TempDir::new().unwrap();
        CriteriaPersistence::save(
            dir.path(),
            &make_criteria_with_association("filter1", "OTHER"),
        )
        .unwrap();

        let matches = StructureAssociation::find_matching(dir.path(), "MY_STRUCT").unwrap();
        assert!(matches.is_empty());
    }

    #[test]
    fn most_recent_match_returns_first_matching() {
        let dir = TempDir::new().unwrap();
        CriteriaPersistence::save(
            dir.path(),
            &make_criteria_with_association("filter1", "STRUCT_A"),
        )
        .unwrap();

        let result = StructureAssociation::most_recent_match(dir.path(), "STRUCT_A").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "filter1");
    }

    #[test]
    fn most_recent_match_no_match_returns_none() {
        let dir = TempDir::new().unwrap();
        let result = StructureAssociation::most_recent_match(dir.path(), "NONE").unwrap();
        assert!(result.is_none());
    }
}
