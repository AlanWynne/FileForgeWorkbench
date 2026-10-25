//! Pure filter predicate for the Alias Editor FILTER row (Req 5.15, design
//! 9.10.7).
//!
//! The filter is DISPLAY-only -- it never mutates `rows`. It matches on CE and
//! Root ONLY (never ALIAS / Parameters). The returned indices are the BACKING
//! `rows` indices (NOT display positions), so a line command on a filtered-visible
//! row targets the correct underlying row (the index hazard, design 9.10.7).

use super::state::AliasRowEdit;
use crate::shell::user_aliases::AliasEnvironment;

/// Compute the BACKING `rows` indices to display given the FILTER row state.
///
/// - An empty filter (`filter_ce` None AND `filter_root` empty) lists ALL rows.
/// - A CE filter restricts to rows whose `environment` matches.
/// - A Root filter restricts to rows whose `canonical` matches
///   case-insensitively (trimmed).
/// - CE and Root combine with AND.
///
/// Validates: command-configurator Requirement 5.15
pub fn displayed_indices(
    rows: &[AliasRowEdit],
    filter_ce: Option<AliasEnvironment>,
    filter_root: &str,
) -> Vec<usize> {
    let root = filter_root.trim();
    rows.iter()
        .enumerate()
        .filter(|(_, row)| {
            let ce_ok = match filter_ce {
                Some(ce) => row.environment == ce,
                None => true,
            };
            let root_ok = root.is_empty() || row.canonical.trim().eq_ignore_ascii_case(root);
            ce_ok && root_ok
        })
        .map(|(i, _)| i)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(surface: &str, canonical: &str, env: AliasEnvironment) -> AliasRowEdit {
        AliasRowEdit {
            surface: surface.to_string(),
            canonical: canonical.to_string(),
            environment: env,
            ..Default::default()
        }
    }

    fn fixture() -> Vec<AliasRowEdit> {
        vec![
            row("ECOB", "EDIT", AliasEnvironment::Ffcmd),
            row("VCOB", "VIEW", AliasEnvironment::Ffcmd),
            row("SOEK", "FIND", AliasEnvironment::Ffedit),
        ]
    }

    // Validates: command-configurator Requirement 5.15 -- empty filters list ALL
    // rows (in backing order).
    #[test]
    fn empty_filters_show_all_rows() {
        let rows = fixture();
        assert_eq!(displayed_indices(&rows, None, ""), vec![0, 1, 2]);
    }

    // Validates: command-configurator Requirement 5.15 -- a CE filter restricts
    // by environment only.
    #[test]
    fn ce_filter_restricts_by_environment() {
        let rows = fixture();
        assert_eq!(
            displayed_indices(&rows, Some(AliasEnvironment::Ffcmd), ""),
            vec![0, 1]
        );
        assert_eq!(
            displayed_indices(&rows, Some(AliasEnvironment::Ffedit), ""),
            vec![2]
        );
    }

    // Validates: command-configurator Requirement 5.15 -- a Root filter restricts
    // by canonical, case-insensitively and trimmed.
    #[test]
    fn root_filter_restricts_by_canonical() {
        let rows = fixture();
        assert_eq!(displayed_indices(&rows, None, "edit"), vec![0]);
        assert_eq!(displayed_indices(&rows, None, "  VIEW  "), vec![1]);
        assert_eq!(displayed_indices(&rows, None, "FIND"), vec![2]);
    }

    // Validates: command-configurator Requirement 5.15 -- CE and Root combine
    // with AND.
    #[test]
    fn ce_and_root_combine_with_and() {
        let rows = fixture();
        // FFCMD + EDIT -> only ECOB.
        assert_eq!(
            displayed_indices(&rows, Some(AliasEnvironment::Ffcmd), "EDIT"),
            vec![0]
        );
        // FFEDIT + EDIT -> nothing (SOEK's root is FIND, EDIT is FFCMD's).
        assert!(displayed_indices(&rows, Some(AliasEnvironment::Ffedit), "EDIT").is_empty());
    }

    // Validates: command-configurator Requirement 5.15 -- ALIAS and Parameters are
    // NOT filter inputs (a filter_root equal to a surface matches nothing).
    #[test]
    fn alias_and_parameters_are_not_filter_inputs() {
        let rows = fixture();
        // "ECOB" is a SURFACE, not a canonical root -> no match on the root filter.
        assert!(displayed_indices(&rows, None, "ECOB").is_empty());
    }

    // Validates: command-configurator Requirement 5.15 -- filtering returns
    // BACKING indices, not display positions, so a later-row match keeps its true
    // index.
    #[test]
    fn returns_backing_indices_not_display_positions() {
        let rows = fixture();
        // Only the third row (index 2) is FFEDIT; the returned index is 2, not 0.
        assert_eq!(
            displayed_indices(&rows, Some(AliasEnvironment::Ffedit), ""),
            vec![2]
        );
    }
}
