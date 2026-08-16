//! Built-in workflow definitions for common workbench operations.
//!
//! These are structural templates — their step implementations will be
//! provided by downstream crates (e.g., `file-operations`, `compare-and-merge`).

use crate::context::ContextValueType;
use crate::definition::{
    ParameterDeclaration, StepDefinition, Transition, WorkflowBuilder, WorkflowDefinition,
};

/// Creates the built-in data transfer workflow definition.
///
/// Steps: source → transform → destination
/// Addresses: Requirement 1, criterion 7
pub fn data_transfer_workflow() -> WorkflowDefinition {
    WorkflowBuilder::new("data-transfer")
        .display_name("Data Transfer")
        .description("Copy/move data between VFS locations with progress")
        .category("file-operation")
        .supports_cancellation(true)
        .supports_persistence(true)
        .parameter(ParameterDeclaration {
            name: "source_path".to_string(),
            value_type: ContextValueType::String,
            required: true,
            default: None,
            description: "Source VFS path".to_string(),
        })
        .parameter(ParameterDeclaration {
            name: "dest_path".to_string(),
            value_type: ContextValueType::String,
            required: true,
            default: None,
            description: "Destination VFS path".to_string(),
        })
        .step(StepDefinition {
            name: "source".to_string(),
            display_name: "Read Source".to_string(),
            ..Default::default()
        })
        .step(StepDefinition {
            name: "transform".to_string(),
            display_name: "Transform".to_string(),
            ..Default::default()
        })
        .step(StepDefinition {
            name: "destination".to_string(),
            display_name: "Write Destination".to_string(),
            has_compensation: true,
            ..Default::default()
        })
        .transition(Transition {
            from: "source".to_string(),
            to: "transform".to_string(),
            predicate: None,
            priority: 0,
        })
        .transition(Transition {
            from: "transform".to_string(),
            to: "destination".to_string(),
            predicate: None,
            priority: 0,
        })
        .initial_step("source")
        .terminal_step("destination")
        .build()
        .expect("data-transfer workflow definition is valid")
}

/// Creates the built-in file import/export workflow definition.
///
/// Steps: read → validate → convert → write
/// Addresses: Requirement 1, criterion 7
pub fn file_import_workflow() -> WorkflowDefinition {
    WorkflowBuilder::new("file-import")
        .display_name("File Import")
        .description("Import external file into workbench via VFS")
        .category("file-operation")
        .supports_cancellation(true)
        .parameter(ParameterDeclaration {
            name: "import_path".to_string(),
            value_type: ContextValueType::String,
            required: true,
            default: None,
            description: "Path to file to import".to_string(),
        })
        .step(StepDefinition {
            name: "read".to_string(),
            display_name: "Read File".to_string(),
            ..Default::default()
        })
        .step(StepDefinition {
            name: "validate".to_string(),
            display_name: "Validate".to_string(),
            ..Default::default()
        })
        .step(StepDefinition {
            name: "convert".to_string(),
            display_name: "Convert".to_string(),
            ..Default::default()
        })
        .step(StepDefinition {
            name: "write".to_string(),
            display_name: "Write".to_string(),
            has_compensation: true,
            ..Default::default()
        })
        .transition(Transition {
            from: "read".to_string(),
            to: "validate".to_string(),
            predicate: None,
            priority: 0,
        })
        .transition(Transition {
            from: "validate".to_string(),
            to: "convert".to_string(),
            predicate: None,
            priority: 0,
        })
        .transition(Transition {
            from: "convert".to_string(),
            to: "write".to_string(),
            predicate: None,
            priority: 0,
        })
        .initial_step("read")
        .terminal_step("write")
        .build()
        .expect("file-import workflow definition is valid")
}

/// Creates the built-in compare-merge workflow definition.
///
/// Steps: load-pair → diff → resolve → apply
/// Addresses: Requirement 1, criterion 7
pub fn compare_merge_workflow() -> WorkflowDefinition {
    WorkflowBuilder::new("compare-merge")
        .display_name("Compare and Merge")
        .description("Three-way compare and merge of document content")
        .category("refactoring")
        .supports_cancellation(true)
        .parameter(ParameterDeclaration {
            name: "left_path".to_string(),
            value_type: ContextValueType::String,
            required: true,
            default: None,
            description: "Left side document path".to_string(),
        })
        .parameter(ParameterDeclaration {
            name: "right_path".to_string(),
            value_type: ContextValueType::String,
            required: true,
            default: None,
            description: "Right side document path".to_string(),
        })
        .step(StepDefinition {
            name: "load-pair".to_string(),
            display_name: "Load Documents".to_string(),
            ..Default::default()
        })
        .step(StepDefinition {
            name: "diff".to_string(),
            display_name: "Compute Differences".to_string(),
            ..Default::default()
        })
        .step(StepDefinition {
            name: "resolve".to_string(),
            display_name: "Resolve Conflicts".to_string(),
            ..Default::default()
        })
        .step(StepDefinition {
            name: "apply".to_string(),
            display_name: "Apply Merge".to_string(),
            has_compensation: true,
            ..Default::default()
        })
        .transition(Transition {
            from: "load-pair".to_string(),
            to: "diff".to_string(),
            predicate: None,
            priority: 0,
        })
        .transition(Transition {
            from: "diff".to_string(),
            to: "resolve".to_string(),
            predicate: None,
            priority: 0,
        })
        .transition(Transition {
            from: "resolve".to_string(),
            to: "apply".to_string(),
            predicate: None,
            priority: 0,
        })
        .initial_step("load-pair")
        .terminal_step("apply")
        .build()
        .expect("compare-merge workflow definition is valid")
}

/// Creates the built-in bulk rename workflow definition.
///
/// Steps: scan → preview → confirm → apply
/// Addresses: Requirement 1, criterion 7
pub fn bulk_rename_workflow() -> WorkflowDefinition {
    WorkflowBuilder::new("bulk-rename")
        .display_name("Bulk Rename")
        .description("Rename multiple files/datasets according to a pattern")
        .category("file-operation")
        .supports_cancellation(true)
        .supports_persistence(true)
        .parameter(ParameterDeclaration {
            name: "pattern".to_string(),
            value_type: ContextValueType::String,
            required: true,
            default: None,
            description: "Rename pattern".to_string(),
        })
        .parameter(ParameterDeclaration {
            name: "directory".to_string(),
            value_type: ContextValueType::String,
            required: true,
            default: None,
            description: "Target directory".to_string(),
        })
        .step(StepDefinition {
            name: "scan".to_string(),
            display_name: "Scan Files".to_string(),
            ..Default::default()
        })
        .step(StepDefinition {
            name: "preview".to_string(),
            display_name: "Preview Changes".to_string(),
            ..Default::default()
        })
        .step(StepDefinition {
            name: "confirm".to_string(),
            display_name: "Confirm".to_string(),
            ..Default::default()
        })
        .step(StepDefinition {
            name: "apply".to_string(),
            display_name: "Apply Renames".to_string(),
            has_compensation: true,
            ..Default::default()
        })
        .transition(Transition {
            from: "scan".to_string(),
            to: "preview".to_string(),
            predicate: None,
            priority: 0,
        })
        .transition(Transition {
            from: "preview".to_string(),
            to: "confirm".to_string(),
            predicate: None,
            priority: 0,
        })
        .transition(Transition {
            from: "confirm".to_string(),
            to: "apply".to_string(),
            predicate: None,
            priority: 0,
        })
        .initial_step("scan")
        .terminal_step("apply")
        .build()
        .expect("bulk-rename workflow definition is valid")
}

/// Returns all built-in workflow definitions.
pub fn all_builtin_workflows() -> Vec<WorkflowDefinition> {
    vec![
        data_transfer_workflow(),
        file_import_workflow(),
        compare_merge_workflow(),
        bulk_rename_workflow(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::validate_definition;

    // Validates: Requirement 1.7 — built-in workflow definitions pass validation

    #[test]
    fn data_transfer_workflow_is_valid() {
        let def = data_transfer_workflow();
        assert!(validate_definition(&def).is_ok());
        assert_eq!(def.name, "data-transfer");
        assert!(def.categories.contains(&"file-operation".to_string()));
    }

    #[test]
    fn file_import_workflow_is_valid() {
        let def = file_import_workflow();
        assert!(validate_definition(&def).is_ok());
        assert_eq!(def.name, "file-import");
    }

    #[test]
    fn compare_merge_workflow_is_valid() {
        let def = compare_merge_workflow();
        assert!(validate_definition(&def).is_ok());
        assert_eq!(def.name, "compare-merge");
        assert!(def.categories.contains(&"refactoring".to_string()));
    }

    #[test]
    fn bulk_rename_workflow_is_valid() {
        let def = bulk_rename_workflow();
        assert!(validate_definition(&def).is_ok());
        assert_eq!(def.name, "bulk-rename");
        assert!(def.supports_persistence);
    }

    #[test]
    fn all_builtin_workflows_returns_four() {
        let all = all_builtin_workflows();
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn all_builtin_workflows_pass_validation() {
        for def in all_builtin_workflows() {
            assert!(
                validate_definition(&def).is_ok(),
                "workflow '{}' failed validation",
                def.name
            );
        }
    }
}
