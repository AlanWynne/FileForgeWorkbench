//! Property-based tests for Floating Window Management.
//! Feature: layout-and-docking, Property 2 and Property 5

use ff_layout::dock::zone::DockZone;
use ff_layout::error::LayoutError;
use ff_layout::floating::manager::FloatingWindowManager;
use ff_layout::{Size, MAX_FLOATING_WINDOWS};
use proptest::prelude::*;

/// Strategy for generating panel IDs.
fn panel_id_strategy() -> impl Strategy<Value = String> {
    "[a-z_]{1,20}"
}

/// Strategy for generating dock zones (dockable only).
fn dockable_zone_strategy() -> impl Strategy<Value = DockZone> {
    prop_oneof![
        Just(DockZone::Left),
        Just(DockZone::Right),
        Just(DockZone::Bottom),
        Just(DockZone::Center),
    ]
}

proptest! {
    /// **Validates: Requirements 3.1, 3.5, 3.7**
    ///
    /// Property 2: Dock/Undock Round-Trip Preserves Panel Identity
    /// For any panel undocked then redocked, the panel remains registered
    /// with its original panel_id, and the final zone matches pre-undock zone.
    #[test]
    fn dock_undock_round_trip_preserves_identity(
        panel_id in panel_id_strategy(),
        zone in dockable_zone_strategy(),
    ) {
        let mut mgr = FloatingWindowManager::new();

        // Undock (create floating window)
        let window_id = mgr.create_window(&panel_id, Size::new(400.0, 300.0), zone).unwrap();

        // Verify the panel is in the floating window
        let window = mgr.get(window_id).unwrap();
        prop_assert!(window.panels.contains(&panel_id));
        prop_assert_eq!(window.origin_zone, zone);

        // Redock (remove floating window)
        let removed = mgr.remove_window(window_id).unwrap();
        prop_assert!(removed.panels.contains(&panel_id));
        prop_assert_eq!(removed.origin_zone, zone);

        // Panel identity preserved — same panel_id and origin zone
        prop_assert_eq!(&removed.panels[0], &panel_id);
    }

    /// **Validates: Requirements 3.14**
    ///
    /// Property 5: Floating Window Count Bound
    /// Active floating windows never exceed MAX_FLOATING_WINDOWS (16).
    /// Attempts beyond limit return MaxFloatingWindows error.
    #[test]
    fn floating_window_count_bound(
        num_undocks in 1usize..25,
    ) {
        let mut mgr = FloatingWindowManager::new();

        for i in 0..num_undocks {
            let result = mgr.create_window(
                &format!("panel_{i}"),
                Size::new(400.0, 300.0),
                DockZone::Left,
            );

            if i < MAX_FLOATING_WINDOWS {
                prop_assert!(result.is_ok(),
                    "Expected Ok for window {i}, got {:?}", result);
            } else {
                prop_assert!(matches!(result, Err(LayoutError::MaxFloatingWindows { .. })),
                    "Expected MaxFloatingWindows for window {i}, got {:?}", result);
            }

            // Invariant: count never exceeds max
            prop_assert!(mgr.count() <= MAX_FLOATING_WINDOWS,
                "Window count {} exceeds max {}", mgr.count(), MAX_FLOATING_WINDOWS);
        }
    }
}
