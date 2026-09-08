//! Integration tests for the unified CommandTarget model.
//!
//! Validates: command-framework Requirement 8.

use ff_command::{
    execute_target, resolve_target, target_from_toml, target_to_toml, CommandId, CommandResult,
    CommandTarget, ExternalMode, MacroSource, TargetExecution, TargetParams, TargetResolver,
    TargetValue,
};

// === Mock resolver =========================================================

/// A resolver whose lookups are driven by explicit fixtures, so each test
/// controls exactly which strings resolve as user commands, built-in verbs,
/// or registered command ids.
struct MockResolver {
    user: Vec<(String, CommandTarget)>,
    builtins: Vec<(String, CommandTarget)>,
    registered: Vec<String>,
}

impl MockResolver {
    fn empty() -> Self {
        Self {
            user: Vec::new(),
            builtins: Vec::new(),
            registered: Vec::new(),
        }
    }
}

impl TargetResolver for MockResolver {
    fn user_command_target(&self, input: &str) -> Option<CommandTarget> {
        self.user
            .iter()
            .find(|(k, _)| k == input)
            .map(|(_, t)| t.clone())
    }

    fn builtin_workspace_target(&self, input: &str) -> Option<CommandTarget> {
        self.builtins
            .iter()
            .find(|(k, _)| k == input)
            .map(|(_, t)| t.clone())
    }

    fn is_registered_command(&self, input: &str) -> bool {
        self.registered.iter().any(|k| k == input)
    }
}

// === Requirement 8.1: five variants exist =================================

#[test]
fn command_target_has_five_constructible_variants() {
    // Validates: Requirement 8.1 -- Menu/CustomWorkspace/Function/Macro/External
    let _menu = CommandTarget::Menu {
        name: "pom".to_string(),
    };
    let _custom = CommandTarget::CustomWorkspace {
        workspace_kind: "settings".to_string(),
        params: TargetParams::new(),
    };
    let _function = CommandTarget::Function {
        command_id: "file.save".to_string(),
        params: TargetParams::new(),
    };
    let _macro = CommandTarget::Macro {
        source: MacroSource::Name("format".to_string()),
    };
    let _external = CommandTarget::External {
        program: "pwsh".to_string(),
        args: vec!["-File".to_string(), "b.ps1".to_string()],
        working_dir: None,
        mode: ExternalMode::Detached,
    };
    assert_eq!(_menu.variant_name(), "menu");
    assert_eq!(_custom.variant_name(), "custom_workspace");
    assert_eq!(_function.variant_name(), "function");
    assert_eq!(_macro.variant_name(), "macro");
    assert_eq!(_external.variant_name(), "external");
}

// === Requirement 8.9: visible-workspace classification ====================

#[test]
fn menu_and_custom_workspace_and_captured_external_are_visible() {
    // Validates: Requirement 8.9
    assert!(CommandTarget::Menu {
        name: "pom".to_string()
    }
    .produces_visible_workspace());
    assert!(CommandTarget::CustomWorkspace {
        workspace_kind: "files".to_string(),
        params: TargetParams::new(),
    }
    .produces_visible_workspace());
    assert!(CommandTarget::External {
        program: "pwsh".to_string(),
        args: vec![],
        working_dir: None,
        mode: ExternalMode::Captured,
    }
    .produces_visible_workspace());
}

#[test]
fn function_macro_and_detached_external_are_not_visible() {
    // Validates: Requirement 8.9
    assert!(!CommandTarget::Function {
        command_id: "edit.undo".to_string(),
        params: TargetParams::new(),
    }
    .produces_visible_workspace());
    assert!(!CommandTarget::Macro {
        source: MacroSource::Path("/tmp/m.lua".to_string()),
    }
    .produces_visible_workspace());
    assert!(!CommandTarget::External {
        program: "backup.exe".to_string(),
        args: vec![],
        working_dir: None,
        mode: ExternalMode::Detached,
    }
    .produces_visible_workspace());
}

// === Requirement 8.3 / 8.4: resolution order & backward compat ============

#[test]
fn user_command_definition_resolves_first() {
    // Validates: Requirement 8.3 -- a string equal to a user-defined id wins
    let target = CommandTarget::External {
        program: "pwsh".to_string(),
        args: vec!["-File".to_string(), "build.ps1".to_string()],
        working_dir: None,
        mode: ExternalMode::Captured,
    };
    let resolver = MockResolver {
        user: vec![("build.release".to_string(), target.clone())],
        builtins: vec![],
        // Even if it were also a registered id, the user definition wins.
        registered: vec!["build.release".to_string()],
    };
    let resolved = resolve_target("build.release", &resolver).expect("resolves");
    assert_eq!(resolved, target);
}

#[test]
fn builtin_workspace_verb_resolves_to_custom_workspace() {
    // Validates: Requirement 8.3 -- built-in verb before registered command id
    let files = CommandTarget::CustomWorkspace {
        workspace_kind: "files".to_string(),
        params: TargetParams::new(),
    };
    let resolver = MockResolver {
        user: vec![],
        builtins: vec![("FILES".to_string(), files.clone())],
        registered: vec![],
    };
    assert_eq!(resolve_target("FILES", &resolver).unwrap(), files);
}

#[test]
fn bare_registered_command_resolves_to_function_target() {
    // Validates: Requirement 8.3, 8.4 -- a registered id becomes a Function target
    let resolver = MockResolver {
        user: vec![],
        builtins: vec![],
        registered: vec!["file.save".to_string()],
    };
    let resolved = resolve_target("file.save", &resolver).unwrap();
    assert_eq!(
        resolved,
        CommandTarget::Function {
            command_id: "file.save".to_string(),
            params: TargetParams::new(),
        }
    );
}

#[test]
fn resolution_trims_surrounding_whitespace() {
    // Validates: Requirement 8.4 -- same result regardless of padding
    let resolver = MockResolver {
        user: vec![],
        builtins: vec![],
        registered: vec!["edit.copy".to_string()],
    };
    let padded = resolve_target("   edit.copy  ", &resolver).unwrap();
    assert_eq!(
        padded,
        CommandTarget::Function {
            command_id: "edit.copy".to_string(),
            params: TargetParams::new(),
        }
    );
}

// === Requirement 8.8: unresolved input errors ==============================

#[test]
fn unresolved_string_returns_error_naming_input() {
    // Validates: Requirement 8.8
    let resolver = MockResolver::empty();
    let err = resolve_target("totally.unknown", &resolver).unwrap_err();
    assert_eq!(err.input, "totally.unknown");
    assert!(err.to_string().contains("totally.unknown"));
}

// === Requirement 8.2: execute_target routing ==============================

#[test]
fn execute_function_target_dispatches_via_callback() {
    // Validates: Requirement 8.2 -- Function routes through the dispatcher
    let target = CommandTarget::Function {
        command_id: "file.save".to_string(),
        params: TargetParams::new(),
    };
    let mut seen: Option<String> = None;
    let exec = execute_target(&target, |id: &CommandId, _params| {
        seen = Some(id.as_str().to_string());
        CommandResult::Ok
    });
    match exec {
        TargetExecution::Function(res) => assert!(res.is_ok()),
        other => panic!("expected Function execution, got {other:?}"),
    }
    assert_eq!(seen.as_deref(), Some("file.save"));
}

#[test]
fn execute_function_target_with_invalid_id_fails() {
    // Validates: Requirement 8.2 / 8.8 -- invalid Function id does not panic
    let target = CommandTarget::Function {
        command_id: "Not A Valid Id".to_string(),
        params: TargetParams::new(),
    };
    let exec = execute_target(&target, |_id, _params| CommandResult::Ok);
    assert!(matches!(exec, TargetExecution::Failed(_)));
}

#[test]
fn execute_non_function_targets_are_deferred_to_shell() {
    // Validates: Requirement 8.2 -- Menu/CustomWorkspace/Macro/External deferred
    for target in [
        CommandTarget::Menu {
            name: "settings".to_string(),
        },
        CommandTarget::CustomWorkspace {
            workspace_kind: "editor".to_string(),
            params: TargetParams::new(),
        },
        CommandTarget::Macro {
            source: MacroSource::Name("m".to_string()),
        },
        CommandTarget::External {
            program: "notepad.exe".to_string(),
            args: vec![],
            working_dir: None,
            mode: ExternalMode::Detached,
        },
    ] {
        let exec = execute_target(&target, |_id, _params| CommandResult::Ok);
        match exec {
            TargetExecution::Deferred(t) => assert_eq!(t, target),
            other => panic!("expected Deferred, got {other:?}"),
        }
    }
}

// === Requirement 8.7: TOML round-trip =====================================

#[test]
fn menu_target_toml_round_trips() {
    // Validates: Requirement 8.7
    let target = CommandTarget::Menu {
        name: "notes".to_string(),
    };
    let toml = target_to_toml(&target).unwrap();
    assert!(toml.contains("kind = \"menu\""));
    assert_eq!(target_from_toml(&toml).unwrap(), target);
}

#[test]
fn external_target_toml_round_trips_with_all_fields() {
    // Validates: Requirement 8.7
    let target = CommandTarget::External {
        program: "pwsh".to_string(),
        args: vec!["-File".to_string(), "scripts/build.ps1".to_string()],
        working_dir: Some("${workspace_root}".to_string()),
        mode: ExternalMode::Captured,
    };
    let toml = target_to_toml(&target).unwrap();
    let back = target_from_toml(&toml).unwrap();
    assert_eq!(back, target);
}

#[test]
fn custom_workspace_target_with_params_round_trips() {
    // Validates: Requirement 8.7 -- Settings namespace param survives round-trip
    let mut params = TargetParams::new();
    params.insert("namespace".to_string(), TargetValue::from("editor"));
    let target = CommandTarget::CustomWorkspace {
        workspace_kind: "settings".to_string(),
        params,
    };
    let toml = target_to_toml(&target).unwrap();
    assert_eq!(target_from_toml(&toml).unwrap(), target);
}

#[test]
fn external_target_parses_from_authored_toml() {
    // Validates: Requirement 8.7 -- the data-file shape in the design doc parses
    let src = r#"
kind = "external"
program = "pwsh"
args = ["-File", "scripts/build.ps1", "-Config", "Release"]
working_dir = "${workspace_root}"
mode = "captured"
"#;
    let target = target_from_toml(src).unwrap();
    match target {
        CommandTarget::External {
            program,
            args,
            working_dir,
            mode,
        } => {
            assert_eq!(program, "pwsh");
            assert_eq!(args.len(), 4);
            assert_eq!(working_dir.as_deref(), Some("${workspace_root}"));
            assert_eq!(mode, ExternalMode::Captured);
        }
        other => panic!("expected External, got {other:?}"),
    }
}

#[test]
fn function_target_defaults_params_when_omitted() {
    // Validates: Requirement 8.7 -- params is optional in the data file
    let src = r#"
kind = "function"
command_id = "edit.undo"
"#;
    let target = target_from_toml(src).unwrap();
    assert_eq!(
        target,
        CommandTarget::Function {
            command_id: "edit.undo".to_string(),
            params: TargetParams::new(),
        }
    );
}
