//! Property-based tests for Layout Serialization.
//! Feature: layout-and-docking, Property 7: Layout Serialization Round-Trip

use ff_layout::dock::zone::DockZone;
use ff_layout::state::layout_state::{DockedPanelState, LayoutState};
use ff_layout::state::serializer::{deserialize_layout_state, serialize_layout_state};
use ff_layout::tabs::group::{TabGroup, TabGroupId, TabGroupTree};
use ff_layout::SCHEMA_VERSION;
use proptest::prelude::*;
use std::collections::HashMap;

/// Strategy for generating valid panel IDs.
fn panel_id_strategy() -> impl Strategy<Value = String> {
    "[a-z_]{1,20}"
}

/// Strategy for generating a dockable zone.
fn dockable_zone_strategy() -> impl Strategy<Value = DockZone> {
    prop_oneof![
        Just(DockZone::Left),
        Just(DockZone::Right),
        Just(DockZone::Bottom),
        Just(DockZone::Center),
    ]
}

/// Strategy for generating a tab group tree (leaf only for simplicity).
fn tab_group_tree_strategy() -> impl Strategy<Value = TabGroupTree> {
    proptest::collection::vec("[a-z]{1,10}\\.rs", 0..5)
        .prop_map(|tabs| TabGroupTree::Leaf(TabGroup::new(TabGroupId::new(1), tabs)))
}

/// Strategy for generating a valid LayoutState.
fn layout_state_strategy() -> impl Strategy<Value = LayoutState> {
    (
        proptest::collection::vec(
            (
                panel_id_strategy(),
                dockable_zone_strategy(),
                100.0f32..500.0,
            ),
            0..5,
        ),
        tab_group_tree_strategy(),
        proptest::collection::hash_map(panel_id_strategy(), proptest::bool::ANY, 0..5),
    )
        .prop_map(|(panels, tab_groups, visibility)| {
            let docked_panels: Vec<DockedPanelState> = panels
                .into_iter()
                .map(|(id, zone, dim)| DockedPanelState {
                    panel_id: id,
                    zone,
                    zone_dimension: dim,
                })
                .collect();

            LayoutState {
                schema_version: SCHEMA_VERSION,
                docked_panels,
                tab_groups,
                floating_windows: Vec::new(),
                splitter_positions: HashMap::new(),
                panel_visibility: visibility,
                panel_display_states: HashMap::new(),
            }
        })
}

proptest! {
    /// **Validates: Requirements 6.1, 6.2, 6.4**
    ///
    /// Property 7: Layout Serialization Round-Trip
    /// For any valid LayoutState, serialize → deserialize produces equivalent state.
    #[test]
    fn layout_serialization_round_trip(
        state in layout_state_strategy(),
    ) {
        let serialized = serialize_layout_state(&state)
            .expect("Serialization should not fail for valid state");

        let deserialized = deserialize_layout_state(&serialized)
            .expect("Deserialization should not fail for valid TOML");

        prop_assert_eq!(&state, &deserialized,
            "Round-trip produced different state");
    }
}
