//! Property-based tests for the ff-lua crate.
//!
//! Uses proptest to verify correctness properties across many inputs.

use proptest::prelude::*;
use std::path::PathBuf;

use ff_lua::*;

// ─── Property 1: Security Mode Decision Consistency ─────────────────────────
// Feature: ff-lua, Property 2: Security mode decision consistency
// **Validates: Requirements 7.1, 7.2, 7.3, 7.4, 7.5**

fn arb_path() -> impl Strategy<Value = PathBuf> {
    prop_oneof![
        Just(PathBuf::from("/trusted/macros/script.lua")),
        Just(PathBuf::from("/user/macros/test.lua")),
        Just(PathBuf::from("/untrusted/random.lua")),
        Just(PathBuf::from("/other/path/file.lua")),
    ]
}

proptest! {
    // Validates: Requirement 7.2
    #[test]
    fn disabled_mode_always_denies(
        path in arb_path()
    ) {
        let gate = SecurityGate::new(
            SecurityMode::Disabled,
            vec![PathBuf::from("/trusted/macros")],
            vec![PathBuf::from("/user/macros")],
        );
        let result = gate.check_permission(&path);
        let is_denied = matches!(result, SecurityPermission::Denied { .. });
        prop_assert!(is_denied, "Expected Denied, got {:?}", result);
    }

    // Validates: Requirement 7.5
    #[test]
    fn enabled_mode_always_allows(
        path in arb_path()
    ) {
        let gate = SecurityGate::new(
            SecurityMode::Enabled,
            vec![],
            vec![],
        );
        let result = gate.check_permission(&path);
        prop_assert_eq!(result, SecurityPermission::Allowed);
    }
}

proptest! {
    // Validates: Requirement 7.1, 7.4
    #[test]
    fn trusted_only_mode_allows_iff_in_trusted_or_user_dirs(
        use_trusted_path in any::<bool>(),
    ) {
        let trusted = vec![PathBuf::from("/trusted/macros")];
        let user_dirs = vec![PathBuf::from("/user/macros")];
        let gate = SecurityGate::new(SecurityMode::TrustedOnly, trusted, user_dirs);

        let path = if use_trusted_path {
            PathBuf::from("/trusted/macros/script.lua")
        } else {
            PathBuf::from("/untrusted/random.lua")
        };

        let result = gate.check_permission(&path);
        if use_trusted_path {
            prop_assert_eq!(result, SecurityPermission::Allowed);
        } else {
            let is_denied = matches!(result, SecurityPermission::Denied { .. });
            prop_assert!(is_denied, "Expected Denied for untrusted path");
        }
    }
}

// ─── Property 3: Hook Invocation Order Preservation ─────────────────────────
// Feature: ff-lua, Property 3: Hook invocation order preservation
// **Validates: Requirements 3.3**

proptest! {
    #[test]
    fn hook_registration_preserves_order(
        num_scripts in 1usize..20,
    ) {
        let mut registry = HookRegistry::new();

        for i in 0..num_scripts {
            registry.register(
                "OnOpen",
                PathBuf::from(format!("script{i}.lua")),
                "OnOpen".to_string(),
            );
        }

        let handlers = registry.handlers_for("OnOpen");
        prop_assert_eq!(handlers.len(), num_scripts);

        // Verify ordering
        for (idx, handler) in handlers.iter().enumerate() {
            prop_assert_eq!(handler.registration_order, idx as u64);
            prop_assert_eq!(
                &handler.script_path,
                &PathBuf::from(format!("script{idx}.lua"))
            );
        }
    }
}

// ─── Property 7: Directory Scan Name Resolution ─────────────────────────────
// Feature: ff-lua, Property 7: Directory scan name resolution
// **Validates: Requirements 9.3, 9.4**

proptest! {
    #[test]
    fn workspace_priority_always_wins_over_user(
        macro_name in "[a-z]{3,10}",
    ) {
        let user_dir = tempfile::TempDir::new().unwrap();
        let workspace_dir = tempfile::TempDir::new().unwrap();

        let user_file = user_dir.path().join(format!("{macro_name}.lua"));
        let workspace_file = workspace_dir.path().join(format!("{macro_name}.lua"));

        std::fs::write(&user_file, "-- user version").unwrap();
        std::fs::write(&workspace_file, "-- workspace version").unwrap();

        let dirs = vec![
            (user_dir.path().to_path_buf(), DirectoryPriority::User),
            (workspace_dir.path().to_path_buf(), DirectoryPriority::Workspace),
        ];

        let result = ff_lua::scanner::scan_directories(&dirs).unwrap();
        let script = result.get(&macro_name).unwrap();

        prop_assert_eq!(script.priority, DirectoryPriority::Workspace);
        prop_assert!(script.path.starts_with(workspace_dir.path()));
    }
}

// ─── Property 8: Auto-Reload Hook Deduplication ─────────────────────────────
// Feature: ff-lua, Property 8: Auto-reload hook deduplication
// **Validates: Requirements 8.3**

proptest! {
    #[test]
    fn unregister_then_reregister_maintains_single_set(
        reload_count in 1usize..20,
        hook_count in 1usize..5,
    ) {
        let mut registry = HookRegistry::new();
        let script_path = PathBuf::from("my_script.lua");

        for _ in 0..reload_count {
            // Simulate reload: unregister then re-register
            registry.unregister_by_script(&script_path);

            for i in 0..hook_count {
                let event_name = format!("Event{i}");
                registry.register(
                    &event_name,
                    script_path.clone(),
                    event_name.clone(),
                );
            }
        }

        // Total handlers should be exactly hook_count, not hook_count * reload_count
        prop_assert_eq!(registry.total_handler_count(), hook_count);
    }
}
