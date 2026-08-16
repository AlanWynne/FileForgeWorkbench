//! Property-based tests for Tab Group operations.
//! Feature: layout-and-docking, Property 3 and Property 4

use ff_layout::tabs::manager::TabGroupManager;
use proptest::prelude::*;

/// Strategy to generate tab names.
fn tab_name_strategy() -> impl Strategy<Value = String> {
    "[a-z_]{1,15}\\.rs"
}

proptest! {
    /// **Validates: Requirements 2.2, 2.3**
    ///
    /// Property 3: Tab Group Split Preserves Total Tab Count
    /// For any split (horizontal or vertical) on a group with N tabs,
    /// total tabs across resulting groups equals N.
    #[test]
    fn tab_group_split_preserves_total_tab_count(
        tabs in proptest::collection::vec(tab_name_strategy(), 1..20),
        split_horizontal in proptest::bool::ANY,
    ) {
        let mut mgr = TabGroupManager::new();
        for tab in &tabs {
            mgr.add_tab(tab, None).unwrap();
        }

        let original_count = mgr.total_tab_count();
        prop_assert_eq!(original_count, tabs.len());

        let result = if split_horizontal {
            mgr.split_horizontal()
        } else {
            mgr.split_vertical()
        };

        prop_assert!(result.is_ok(), "Split failed: {:?}", result);
        prop_assert_eq!(mgr.total_tab_count(), original_count,
            "Tab count changed after split: {} -> {}", original_count, mgr.total_tab_count());
    }

    /// **Validates: Requirements 2.5**
    ///
    /// Property 4: Empty Tab Group Elimination
    /// After any sequence of tab moves, no empty TabGroup exists.
    /// Moving the last tab from a group removes it.
    #[test]
    fn empty_tab_group_elimination(
        tabs in proptest::collection::vec(tab_name_strategy(), 2..10),
        moves in proptest::collection::vec(0usize..10, 1..5),
    ) {
        let mut mgr = TabGroupManager::new();
        for tab in &tabs {
            mgr.add_tab(tab, None).unwrap();
        }

        // Split to create two groups
        let _new_id = mgr.split_horizontal().unwrap();
        let _group_ids = mgr.tree().all_group_ids();

        // Perform random moves between groups
        for move_seed in &moves {
            let current_ids = mgr.tree().all_group_ids();
            if current_ids.len() < 2 {
                break; // Only one group left
            }

            let source_idx = move_seed % current_ids.len();
            let source_id = current_ids[source_idx];

            // Find a group with at least one tab
            if let Some(group) = mgr.tree().find_group(source_id) {
                if group.tab_count() > 0 {
                    // Pick a different target
                    let target_idx = (source_idx + 1) % current_ids.len();
                    let target_id = current_ids[target_idx];
                    let _ = mgr.move_tab(source_id, 0, target_id, 0);
                }
            }
        }

        // Invariant: no empty groups exist
        prop_assert!(!mgr.tree().has_empty_groups(),
            "Tree has empty groups after moves");
    }
}
