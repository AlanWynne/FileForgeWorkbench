//! Property-based tests for Panel Registration.
//! Feature: layout-and-docking, Property 1: Panel Registration Uniqueness

use ff_layout::dock::zone::DockZone;
use ff_layout::error::LayoutError;
use ff_layout::panel::registry::PanelRegistry;
use proptest::prelude::*;

/// Strategy to generate valid panel IDs (1–64 ASCII alphanumeric/underscore).
fn valid_panel_id_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_]{1,64}"
}

/// Strategy to generate a dock zone.
fn dock_zone_strategy() -> impl Strategy<Value = DockZone> {
    prop_oneof![
        Just(DockZone::Left),
        Just(DockZone::Right),
        Just(DockZone::Bottom),
        Just(DockZone::Center),
    ]
}

proptest! {
    /// **Validates: Requirements 1.10**
    ///
    /// Property 1: Panel Registration Uniqueness
    /// For any sequence of panel registrations, the PanelRegistry contains
    /// at most one entry per `panel_id`. Duplicate attempts return
    /// `DuplicatePanelId` error without modifying state.
    #[test]
    fn panel_registration_uniqueness(
        registrations in proptest::collection::vec(
            (valid_panel_id_strategy(), dock_zone_strategy()),
            1..30
        )
    ) {
        let mut registry = PanelRegistry::new();
        let mut registered_ids: Vec<String> = Vec::new();

        for (panel_id, zone) in &registrations {
            let result = registry.register(panel_id, "Title", *zone);
            if registered_ids.contains(panel_id) {
                // Duplicate — must return error
                prop_assert!(matches!(result, Err(LayoutError::DuplicatePanelId { .. })),
                    "Expected DuplicatePanelId for '{}', got {:?}", panel_id, result);
                // Registry unchanged
                prop_assert_eq!(registry.count(), registered_ids.len());
            } else {
                // New registration — must succeed
                prop_assert!(result.is_ok(),
                    "Expected Ok for '{}', got {:?}", panel_id, result);
                registered_ids.push(panel_id.clone());
            }
        }

        // Final assertion: no duplicates in registry
        let all_ids = registry.list_all();
        let mut sorted_ids = all_ids.clone();
        sorted_ids.sort();
        sorted_ids.dedup();
        prop_assert_eq!(all_ids.len(), sorted_ids.len(),
            "Registry contains duplicate entries");
    }
}
