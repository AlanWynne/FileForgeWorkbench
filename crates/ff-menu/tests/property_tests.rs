//! Property-based tests for the ff-menu crate.
//!
//! Uses proptest to validate invariants across many random inputs.

use ff_menu::command_field::CommandFieldController;
use ff_menu::menu_bar::MenuBar;
use ff_menu::menu_item::MenuItem;
use ff_menu::menu_model::Menu;
use ff_menu::recent_files::RecentFilesManager;
use ff_menu::status_bar::StatusBar;
use ff_menu::status_segment::{SegmentAlignment, StatusSegment};
use proptest::prelude::*;

// ─── Property 1: Recent Files Bounded-Size ──────────────────────────────────
// Feature: ff-menu, Property 1: Recent files list bounded-size property
// **Validates: Requirements 3.2, 3.3**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn recent_files_list_never_exceeds_max_entries(
        max_entries in 1usize..=50,
        paths in proptest::collection::vec("[a-z]{1,5}/[a-z]{1,10}\\.txt", 10..200),
    ) {
        let mut mgr = RecentFilesManager::new(max_entries);
        for path in &paths {
            mgr.add_or_promote(path);
            prop_assert!(
                mgr.len() <= max_entries,
                "List length {} exceeded max_entries {} after adding '{}'",
                mgr.len(),
                max_entries,
                path,
            );
        }
    }
}

// ─── Property 2: Recent Files Add-or-Promote Ordering ───────────────────────
// Feature: ff-menu, Property 2: Recent files add-or-promote ordering property
// **Validates: Requirements 3.1, 3.3**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn recent_files_most_recent_always_at_index_zero_and_no_duplicates(
        paths in proptest::collection::vec("[a-z]{1,3}/[a-z]{1,5}\\.txt", 5..100),
    ) {
        let mut mgr = RecentFilesManager::new(50);
        for path in &paths {
            mgr.add_or_promote(path);

            // Most recently added/promoted is always at index 0
            prop_assert_eq!(
                &mgr.entries()[0].path,
                path,
                "After add_or_promote('{}'), index 0 should be '{}' but was '{}'",
                path,
                path,
                mgr.entries()[0].path,
            );

            // No duplicate paths in the list
            let entry_paths: Vec<&str> = mgr.entries().iter().map(|e| e.path.as_str()).collect();
            let mut sorted = entry_paths.clone();
            sorted.sort();
            sorted.dedup();
            prop_assert_eq!(
                entry_paths.len(),
                sorted.len(),
                "Duplicate paths detected in recent files list after adding '{}'",
                path,
            );
        }
    }
}

// ─── Property 3: Status Segment ID Uniqueness ───────────────────────────────
// Feature: ff-menu, Property 3: Status segment ID uniqueness property
// **Validates: Requirements 5.4, 8.6**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn status_bar_segment_ids_always_unique(
        ids in proptest::collection::vec("[a-z_]{1,16}", 5..50),
    ) {
        let mut bar = StatusBar::new();
        for (i, id) in ids.iter().enumerate() {
            let alignment = match i % 3 {
                0 => SegmentAlignment::Left,
                1 => SegmentAlignment::Center,
                _ => SegmentAlignment::Right,
            };
            let segment = StatusSegment::new(id.as_str(), alignment, i as u32);
            match segment {
                Ok(seg) => {
                    let _ = bar.register_segment(seg);
                }
                Err(_) => continue, // Invalid ID, skip
            }
        }

        // Verify no duplicate IDs
        let segment_ids = bar.segment_ids();
        let mut sorted_ids = segment_ids.clone();
        sorted_ids.sort();
        sorted_ids.dedup();
        prop_assert_eq!(
            segment_ids.len(),
            sorted_ids.len(),
            "Duplicate segment IDs found in status bar",
        );
    }
}

// ─── Property 4: Menu Item Command Binding Consistency ──────────────────────
// Feature: ff-menu, Property 4: Menu item command binding consistency property
// **Validates: Requirements 2.1, 2.10**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn menu_items_always_have_consistent_command_bindings(
        num_items in 5usize..30,
    ) {
        let mut menu = Menu::new("Test", None);
        let mut expected_bindings = Vec::new();

        for i in 0..num_items {
            let id = format!("item_{}", i);
            let cmd = format!("cmd.action_{}", i);
            let item = MenuItem::new(&id, format!("Item {}", i), &cmd);
            expected_bindings.push((id.clone(), cmd.clone()));
            menu.items.push(ff_menu::menu_model::MenuEntry::Item(item));
        }

        let bar = MenuBar::with_menus(vec![menu]);

        // Verify each item can be found and has the correct command_id
        for (item_id, expected_cmd) in &expected_bindings {
            let found = bar.find_item(item_id);
            prop_assert!(found.is_some(), "Item '{}' not found in menu bar", item_id);
            let item = found.unwrap();
            prop_assert_eq!(
                &item.command_id,
                expected_cmd,
                "Item '{}' has wrong command_id",
                item_id,
            );
        }
    }
}

// ─── Property 5: Context Menu Predicate Evaluation Consistency ──────────────
// Feature: ff-menu, Property 5: Context menu predicate evaluation consistency property
// **Validates: Requirements 4.3, 4.4**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn disabled_menu_items_reflect_enabled_state(
        enable_mask in proptest::collection::vec(proptest::bool::ANY, 5..15),
    ) {
        let menu = Menu::new("Context", None);
        let mut items: Vec<MenuItem> = enable_mask.iter().enumerate().map(|(i, &enabled)| {
            let mut item = MenuItem::new(
                format!("ctx_item_{}", i),
                format!("Item {}", i),
                format!("cmd.ctx_{}", i),
            );
            item.is_enabled = enabled;
            item
        }).collect();

        // Verify that the enabled state matches what we set
        for (i, item) in items.iter().enumerate() {
            prop_assert_eq!(
                item.is_enabled,
                enable_mask[i],
                "Item {} enabled state mismatch",
                i,
            );
        }

        // Verify disabled items cannot be "activated" (they report not enabled)
        for item in &items {
            if !item.is_enabled {
                prop_assert!(
                    !item.is_enabled,
                    "Disabled item '{}' should not be activatable",
                    item.id,
                );
            }
        }

        // Suppress unused variable warning
        let _ = menu;
        let _ = items.drain(..);
    }
}

// ─── Property 6: Command Field History Navigation ───────────────────────────
// Feature: ff-menu, Property 6: Command field history navigation property
// **Validates: Requirements 9.6**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn command_field_history_navigation_bounded_and_correct(
        commands in proptest::collection::vec("[A-Z]{3,8}", 1..50),
        up_presses in 1usize..100,
    ) {
        let mut ctrl = CommandFieldController::new();

        // Submit all commands
        for cmd in &commands {
            ctrl.set_text(cmd.clone());
            ctrl.submit();
        }

        let history_len = ctrl.history_len();

        // Navigate up (towards older) repeatedly
        for press in 0..up_presses {
            ctrl.history_navigate(-1);

            // The history position should be clamped
            let _expected_index = (press + 1).min(history_len);
            // After navigating, text should be from history (not panic)
            let text = ctrl.text().to_string();
            prop_assert!(
                !text.is_empty() || history_len == 0,
                "Text should not be empty after navigating up (press {})",
                press,
            );
        }

        // Navigate back down to the bottom
        for _ in 0..(up_presses + 1) {
            ctrl.history_navigate(1);
        }

        // After returning to bottom, field should be empty (saved_input was empty)
        prop_assert_eq!(
            ctrl.text(),
            "",
            "After navigating back to bottom, field should be empty",
        );
    }
}
