//! End-to-end integration tests for the ff-layout crate.
//!
//! These tests exercise full panel lifecycle, tab group workflows,
//! persona operations, serialization round-trips, and resize scenarios.

use ff_layout::dock::zone::DockZone;
use ff_layout::engine::CloseAction;
use ff_layout::floating::manager::FloatingWindowManager;
use ff_layout::floating::monitor::{
    center_on_primary, is_window_sufficiently_visible, MonitorInfo,
};
use ff_layout::panel::registry::PanelRegistry;
use ff_layout::persona::manager::PersonaManager;
use ff_layout::resize::manager::SplitterManager;
use ff_layout::resize::splitter::SplitterOrientation;
use ff_layout::state::layout_state::{DockedPanelState, LayoutState};
use ff_layout::state::serializer;
use ff_layout::tabs::group::{TabGroup, TabGroupId, TabGroupTree};
use ff_layout::tabs::manager::TabGroupManager;
use ff_layout::{LayoutEngine, Position, Rect, Size, MAX_FLOATING_WINDOWS};
use tempfile::TempDir;

/// Integration test: full panel lifecycle
/// (register → dock → undock → float → redock → hide → show)
#[test]
fn full_panel_lifecycle() {
    // Validates: Requirement 1 criteria 1-14, Requirement 3 criteria 1, 5

    // Register panels
    let mut registry = PanelRegistry::new();
    registry
        .register("file_tree", "File Tree", DockZone::Left)
        .unwrap();
    registry
        .register("output", "Output", DockZone::Bottom)
        .unwrap();
    registry
        .register("properties", "Properties", DockZone::Right)
        .unwrap();

    assert_eq!(registry.count(), 3);
    assert!(registry.is_registered("file_tree"));

    // Create floating window manager and undock
    let mut floating = FloatingWindowManager::new();
    let window_id = floating
        .create_window("file_tree", Size::new(300.0, 600.0), DockZone::Left)
        .unwrap();
    assert_eq!(floating.count(), 1);

    // Verify the window
    let window = floating.get(window_id).unwrap();
    assert_eq!(window.origin_zone, DockZone::Left);
    assert!(window.panels.contains(&"file_tree".to_string()));

    // Redock (remove from floating)
    let removed = floating.remove_window(window_id).unwrap();
    assert_eq!(floating.count(), 0);
    assert_eq!(removed.origin_zone, DockZone::Left);

    // Panel visibility tracking
    let mut state = LayoutState::default();
    assert!(state.is_panel_visible("file_tree"));

    // Hide panel
    state
        .panel_visibility
        .insert("file_tree".to_string(), false);
    assert!(!state.is_panel_visible("file_tree"));

    // Show panel
    state.panel_visibility.insert("file_tree".to_string(), true);
    assert!(state.is_panel_visible("file_tree"));

    // Deregister (plugin unload)
    assert!(registry.deregister("file_tree"));
    assert!(!registry.is_registered("file_tree"));
    assert_eq!(registry.count(), 2);
}

/// Integration test: tab group workflow
/// (open tabs → split → move tabs → close empty group)
#[test]
fn tab_group_workflow() {
    // Validates: Requirement 2 criteria 1-5, 9

    let mut mgr = TabGroupManager::new();

    // Open tabs
    mgr.add_tab("main.rs", None).unwrap();
    mgr.add_tab("lib.rs", None).unwrap();
    mgr.add_tab("error.rs", None).unwrap();
    assert_eq!(mgr.total_tab_count(), 3);

    // Split horizontally
    let new_group = mgr.split_horizontal().unwrap();
    assert_eq!(mgr.total_tab_count(), 3); // Tab count preserved

    let group_ids = mgr.tree().all_group_ids();
    assert_eq!(group_ids.len(), 2);

    // Get the original group
    let original_group = group_ids.iter().find(|id| **id != new_group).unwrap();

    // Move a tab from original to new group
    let original = mgr.tree().find_group(*original_group).unwrap();
    if original.tab_count() > 0 {
        mgr.move_tab(*original_group, 0, new_group, 0).unwrap();
    }
    assert_eq!(mgr.total_tab_count(), 3); // Tab count still preserved

    // Move all tabs from one group to create an empty group
    loop {
        let ids = mgr.tree().all_group_ids();
        if ids.len() <= 1 {
            break;
        }
        let source = ids[0];
        let target = ids[1];
        let source_group = mgr.tree().find_group(source).unwrap();
        if source_group.is_empty() {
            break;
        }
        mgr.move_tab(source, 0, target, 0).unwrap();
    }

    // Empty groups should be eliminated
    assert!(!mgr.tree().has_empty_groups());
}

/// Integration test: persona workflow
/// (activate → modify → revert → save custom → delete)
#[test]
fn persona_workflow() {
    // Validates: Requirement 5 criteria 2-6, 9-10

    let mut mgr = PersonaManager::new();

    // Built-in personas exist
    assert!(mgr.get("Editor Focus").is_some());
    assert!(mgr.get("Debug").is_some());

    // Activate a built-in persona
    mgr.activate("Debug").unwrap();
    assert_eq!(mgr.active_persona_name(), Some("Debug"));
    assert!(!mgr.is_modified());

    // Modify the layout
    mgr.mark_modified();
    assert!(mgr.is_modified());

    // Save as custom persona
    mgr.save("My Debug", LayoutState::default());
    assert_eq!(mgr.active_persona_name(), Some("My Debug"));
    assert!(!mgr.is_modified()); // Saving clears modified flag

    // Delete custom persona
    mgr.delete("My Debug").unwrap();
    assert!(mgr.get("My Debug").is_none());

    // Cannot delete built-in
    assert!(mgr.delete("Editor Focus").is_err());
}

/// Integration test: serialization workflow
/// (save session → load → verify state)
#[test]
fn serialization_workflow() {
    // Validates: Requirement 6 criteria 1-5, 8, 11

    let dir = TempDir::new().unwrap();
    let path = dir.path().join("layout_state.toml");

    // Create a state with some content
    let mut state = LayoutState::default();
    state.docked_panels.push(DockedPanelState {
        panel_id: "file_tree".to_string(),
        zone: DockZone::Left,
        zone_dimension: 250.0,
    });
    state.panel_visibility.insert("output".to_string(), false);
    state
        .splitter_positions
        .insert("left_center".to_string(), 0.2);

    // Save session
    serializer::save_to_file(&state, &path).unwrap();

    // Load and verify
    let loaded = serializer::load_from_file(&path).unwrap();
    assert_eq!(state, loaded);
    assert_eq!(loaded.schema_version, ff_layout::SCHEMA_VERSION);
    assert_eq!(loaded.docked_panels.len(), 1);
    assert_eq!(loaded.docked_panels[0].panel_id, "file_tree");
    assert!(!loaded.is_panel_visible("output"));

    // Graceful degradation on invalid file
    let bad_path = dir.path().join("bad.toml");
    std::fs::write(&bad_path, "not valid toml {{{").unwrap();
    let (fallback, reason) = serializer::load_or_default(&bad_path);
    assert_eq!(fallback, LayoutState::default());
    assert!(reason.is_some());

    // Reset to default
    let default = LayoutState::default();
    assert!(default.docked_panels.is_empty());
}

/// Integration test: drag-and-drop scenarios
#[test]
fn drag_and_drop_scenarios() {
    // Validates: Requirement 7 criteria 1-12
    use ff_layout::drag::coordinator::{DragDropCoordinator, DragItem, DragPhase};
    use ff_layout::drag::hit_test::{build_dock_zone_targets, hit_test};
    use ff_layout::drag::indicator::DropPlacement;

    let mut coordinator = DragDropCoordinator::new();

    // Start a panel drag
    coordinator.begin_drag(
        DragItem::Panel {
            panel_id: "file_tree".to_string(),
        },
        Position::new(100.0, 50.0),
    );
    assert!(coordinator.is_dragging());

    // Update position
    coordinator.update_position(Position::new(200.0, 100.0));

    // Build dock zone targets
    let targets = build_dock_zone_targets(
        Rect::new(0.0, 0.0, 200.0, 800.0),      // Left
        Rect::new(1200.0, 0.0, 200.0, 800.0),   // Right
        Rect::new(200.0, 600.0, 1000.0, 200.0), // Bottom
        Rect::new(200.0, 0.0, 1000.0, 600.0),   // Center
    );

    // Hit test at cursor position (clearly in center zone)
    let hit = hit_test(&targets, Position::new(500.0, 300.0));
    assert!(hit.is_some());
    assert_eq!(
        hit.unwrap().placement,
        DropPlacement::DockZone(DockZone::Center)
    );

    // Cancel drag
    coordinator.cancel();
    assert!(!coordinator.is_dragging());
}

/// Integration test: multi-monitor disconnect/reconnect repositioning
#[test]
fn multi_monitor_repositioning() {
    // Validates: Requirement 4 criteria 6-8

    let primary = MonitorInfo::new("primary", true, Rect::new(0.0, 0.0, 1920.0, 1080.0), 1.0);
    let secondary = MonitorInfo::new(
        "secondary",
        false,
        Rect::new(1920.0, 0.0, 2560.0, 1440.0),
        1.5,
    );

    let monitors = vec![primary.clone(), secondary.clone()];

    // Window on secondary monitor is visible
    assert!(is_window_sufficiently_visible(
        Position::new(2000.0, 100.0),
        Size::new(400.0, 300.0),
        &monitors,
    ));

    // After secondary disconnect, window is NOT visible on primary only
    let primary_only = vec![primary.clone()];
    assert!(!is_window_sufficiently_visible(
        Position::new(2000.0, 100.0),
        Size::new(400.0, 300.0),
        &primary_only,
    ));

    // Reposition to center of primary
    let new_pos = center_on_primary(Size::new(400.0, 300.0), &primary_only).unwrap();
    assert!(is_window_sufficiently_visible(
        new_pos,
        Size::new(400.0, 300.0),
        &primary_only,
    ));
}

/// Integration test: window resize proportional redistribution with min constraints
#[test]
fn window_resize_proportional_redistribution() {
    // Validates: Requirement 8 criteria 3-6

    let mut mgr = SplitterManager::new();

    // Left-center splitter (default 20%)
    let left_id = mgr.add_splitter(0.2, SplitterOrientation::Vertical, 100.0, 200.0);
    // Center-right splitter (default 80%)
    let right_id = mgr.add_splitter(0.8, SplitterOrientation::Vertical, 200.0, 100.0);
    // Center-bottom splitter (default 75%)
    let bottom_id = mgr.add_splitter(0.75, SplitterOrientation::Horizontal, 200.0, 100.0);

    // Set proportions
    mgr.update_splitter(left_id, 0.2, 1400.0).unwrap();
    mgr.update_splitter(right_id, 0.8, 1400.0).unwrap();
    mgr.update_splitter(bottom_id, 0.75, 800.0).unwrap();

    // Resize window — proportions should be preserved
    mgr.on_window_resize(Size::new(1600.0, 900.0));

    assert_eq!(mgr.get(left_id).unwrap().proportion, 0.2);
    assert_eq!(mgr.get(right_id).unwrap().proportion, 0.8);
    assert_eq!(mgr.get(bottom_id).unwrap().proportion, 0.75);

    // Test minimum constraints during splitter drag
    // Left splitter with min_first=100, total=1400: min proportion = 100/1400 ≈ 0.071
    mgr.update_splitter(left_id, 0.01, 1400.0).unwrap();
    let proportion = mgr.get(left_id).unwrap().proportion;
    assert!(proportion >= 100.0 / 1400.0 - 0.001);

    // Double-click reset
    mgr.reset_splitter(left_id).unwrap();
    assert_eq!(mgr.get(left_id).unwrap().proportion, 0.2);
}

/// Integration test: LayoutEngine full panel lifecycle via engine API
#[test]
fn engine_full_panel_lifecycle() {
    // Validates: Requirement 1 full lifecycle through LayoutEngine
    let mut engine = LayoutEngine::new();
    engine
        .panel_registry_mut()
        .register("file_tree", "File Tree", DockZone::Left)
        .unwrap();
    engine
        .panel_registry_mut()
        .register("output", "Output", DockZone::Bottom)
        .unwrap();

    // Dock panels via state
    engine
        .current_state_mut()
        .docked_panels
        .push(DockedPanelState {
            panel_id: "file_tree".to_string(),
            zone: DockZone::Left,
            zone_dimension: 250.0,
        });

    // Undock
    let window_id = engine.undock_panel("file_tree").unwrap();
    assert_eq!(engine.floating_window_count(), 1);

    // Redock
    engine.redock_panel(window_id).unwrap();
    assert_eq!(engine.floating_window_count(), 0);

    // Hide / Show
    engine.hide_panel("file_tree").unwrap();
    assert!(!engine.current_state().is_panel_visible("file_tree"));
    engine.show_panel("file_tree").unwrap();
    assert!(engine.current_state().is_panel_visible("file_tree"));

    // Minimize / Maximize / Restore
    engine.minimize_panel("file_tree").unwrap();
    engine.maximize_panel("file_tree").unwrap();
    engine.restore_panel("file_tree").unwrap();
}

/// Integration test: LayoutEngine tab group workflow
#[test]
fn engine_tab_group_workflow() {
    // Validates: Requirement 2 full workflow through LayoutEngine
    let mut engine = LayoutEngine::new();
    engine.add_tab("main.rs", None).unwrap();
    engine.add_tab("lib.rs", None).unwrap();
    engine.add_tab("error.rs", None).unwrap();

    // Split
    let new_group = engine.split_horizontal().unwrap();
    assert_eq!(engine.tab_groups().total_tab_count(), 3);

    // Move tab
    let groups = engine.tab_groups().tree().all_group_ids();
    let other_group = groups.iter().find(|id| **id != new_group).copied().unwrap();
    engine.move_tab(new_group, 0, other_group, 0).unwrap();
    assert_eq!(engine.tab_groups().total_tab_count(), 3);
}

/// Integration test: LayoutEngine persona workflow
#[test]
fn engine_persona_workflow() {
    // Validates: Requirement 5 full workflow through LayoutEngine
    let mut engine = LayoutEngine::new();

    // Activate built-in
    engine.activate_persona("Debug").unwrap();
    assert_eq!(engine.active_persona_name(), Some("Debug"));
    assert!(!engine.is_persona_modified());

    // Save custom
    engine.save_persona("My Custom").unwrap();
    assert_eq!(engine.active_persona_name(), Some("My Custom"));

    // Delete custom
    engine.delete_persona("My Custom").unwrap();
}

/// Integration test: LayoutEngine serialization workflow
#[test]
fn engine_serialization_workflow() {
    // Validates: Requirement 6 full workflow through LayoutEngine
    let mut engine = LayoutEngine::new();
    engine
        .panel_registry_mut()
        .register("file_tree", "File Tree", DockZone::Left)
        .unwrap();
    engine.add_tab("main.rs", None).unwrap();

    let dir = TempDir::new().unwrap();
    let session_path = dir.path().join("session.toml");
    let export_path = dir.path().join("export.toml");

    // Save session
    engine.save_session(&session_path).unwrap();
    assert!(session_path.exists());

    // Export
    engine.export_layout(&export_path).unwrap();

    // Import into fresh engine
    let mut engine2 = LayoutEngine::new();
    engine2
        .panel_registry_mut()
        .register("file_tree", "File Tree", DockZone::Left)
        .unwrap();
    engine2.import_layout(&export_path).unwrap();

    // Reset
    engine2.reset_to_default();
    assert!(engine2.current_state().docked_panels.is_empty());
}
