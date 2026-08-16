//! Workflow registry — central lookup for workflow definitions.
//!
//! Thread-safe registration, unregistration, and querying of workflows
//! by name, category, and input parameter type.

use std::collections::HashMap;
use std::sync::RwLock;

use crate::context::ContextValueType;
use crate::definition::WorkflowDefinition;
use crate::error::WorkflowError;

/// Metadata exposed by the registry for UI consumption.
///
/// Addresses: Requirement 6, criterion 6
#[derive(Debug, Clone)]
pub struct WorkflowMetadata {
    /// Workflow name (registry key).
    pub name: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Description of what the workflow does.
    pub description: String,
    /// Category tags.
    pub categories: Vec<String>,
    /// Input parameters with types and descriptions.
    pub parameters: Vec<crate::definition::ParameterDeclaration>,
    /// Whether the workflow supports cancellation.
    pub supports_cancellation: bool,
    /// Whether the workflow supports pause/resume.
    pub supports_pause: bool,
    /// Whether the workflow supports persistence.
    pub supports_persistence: bool,
}

impl From<&WorkflowDefinition> for WorkflowMetadata {
    fn from(def: &WorkflowDefinition) -> Self {
        Self {
            name: def.name.clone(),
            display_name: def.display_name.clone(),
            description: def.description.clone(),
            categories: def.categories.clone(),
            parameters: def.parameters.clone(),
            supports_cancellation: def.supports_cancellation,
            supports_pause: def.supports_pause,
            supports_persistence: def.supports_persistence,
        }
    }
}

/// An entry in the workflow registry.
#[derive(Debug, Clone)]
struct RegistryEntry {
    definition: WorkflowDefinition,
    /// Owner identifier (plugin name, or None for built-in).
    owner: Option<String>,
}

/// Central registry for workflow definitions. Thread-safe.
///
/// Addresses: Requirement 6, all criteria
pub struct WorkflowRegistry {
    entries: RwLock<HashMap<String, RegistryEntry>>,
}

impl Default for WorkflowRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a workflow definition. Returns error if name exists.
    ///
    /// Addresses: Requirement 6, criterion 1
    pub fn register(
        &self,
        definition: WorkflowDefinition,
        owner: Option<String>,
    ) -> Result<(), WorkflowError> {
        let mut entries = self.entries.write().expect("registry lock poisoned");
        if entries.contains_key(&definition.name) {
            return Err(WorkflowError::DuplicateName {
                name: definition.name.clone(),
            });
        }
        let name = definition.name.clone();
        entries.insert(name, RegistryEntry { definition, owner });
        Ok(())
    }

    /// Unregisters a workflow by name. Returns true if removed.
    pub fn unregister(&self, name: &str) -> bool {
        let mut entries = self.entries.write().expect("registry lock poisoned");
        entries.remove(name).is_some()
    }

    /// Removes all workflows owned by a specific plugin.
    ///
    /// Addresses: Requirement 6, criterion 3
    pub fn unregister_by_owner(&self, owner: &str) {
        let mut entries = self.entries.write().expect("registry lock poisoned");
        entries.retain(|_, entry| entry.owner.as_deref() != Some(owner));
    }

    /// Looks up a workflow definition by exact name.
    ///
    /// Addresses: Requirement 6, criterion 4
    pub fn get(&self, name: &str) -> Option<WorkflowDefinition> {
        let entries = self.entries.read().expect("registry lock poisoned");
        entries.get(name).map(|e| e.definition.clone())
    }

    /// Queries all workflows in a given category.
    ///
    /// Addresses: Requirement 6, criterion 4
    pub fn query_by_category(&self, category: &str) -> Vec<WorkflowDefinition> {
        let entries = self.entries.read().expect("registry lock poisoned");
        entries
            .values()
            .filter(|e| e.definition.categories.contains(&category.to_string()))
            .map(|e| e.definition.clone())
            .collect()
    }

    /// Queries workflows that accept a given input parameter type.
    ///
    /// Addresses: Requirement 6, criterion 4
    pub fn query_by_parameter_type(&self, param_type: ContextValueType) -> Vec<WorkflowDefinition> {
        let entries = self.entries.read().expect("registry lock poisoned");
        entries
            .values()
            .filter(|e| {
                e.definition
                    .parameters
                    .iter()
                    .any(|p| p.value_type == param_type)
            })
            .map(|e| e.definition.clone())
            .collect()
    }

    /// Gets metadata for a workflow (for UI display).
    ///
    /// Addresses: Requirement 6, criterion 6
    pub fn metadata(&self, name: &str) -> Option<WorkflowMetadata> {
        let entries = self.entries.read().expect("registry lock poisoned");
        entries
            .get(name)
            .map(|e| WorkflowMetadata::from(&e.definition))
    }

    /// Lists all registered workflow names.
    pub fn list_all(&self) -> Vec<String> {
        let entries = self.entries.read().expect("registry lock poisoned");
        entries.keys().cloned().collect()
    }

    /// Returns the total number of registered workflows.
    pub fn count(&self) -> usize {
        let entries = self.entries.read().expect("registry lock poisoned");
        entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::{StepDefinition, WorkflowBuilder};

    fn make_step(name: &str) -> StepDefinition {
        StepDefinition {
            name: name.to_string(),
            display_name: name.to_string(),
            ..Default::default()
        }
    }

    fn make_workflow(name: &str) -> WorkflowDefinition {
        WorkflowBuilder::new(name)
            .step(make_step("start"))
            .initial_step("start")
            .terminal_step("start")
            .build()
            .unwrap()
    }

    // Validates: Requirement 6.1 — unique names, duplicate rejection

    #[test]
    fn register_workflow_succeeds() {
        let registry = WorkflowRegistry::new();
        let def = make_workflow("wf1");
        assert!(registry.register(def, None).is_ok());
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn duplicate_name_registration_returns_error() {
        let registry = WorkflowRegistry::new();
        let def1 = make_workflow("wf1");
        let def2 = make_workflow("wf1");
        assert!(registry.register(def1, None).is_ok());
        let result = registry.register(def2, None);
        assert!(matches!(result, Err(WorkflowError::DuplicateName { .. })));
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn unregister_removes_workflow() {
        let registry = WorkflowRegistry::new();
        registry.register(make_workflow("wf1"), None).unwrap();
        assert!(registry.unregister("wf1"));
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn unregister_nonexistent_returns_false() {
        let registry = WorkflowRegistry::new();
        assert!(!registry.unregister("nonexistent"));
    }

    // Validates: Requirement 6.3 — plugin lifecycle

    #[test]
    fn unregister_by_owner_removes_plugin_workflows() {
        let registry = WorkflowRegistry::new();
        registry.register(make_workflow("builtin"), None).unwrap();
        registry
            .register(make_workflow("plugin-a"), Some("plugin-a".to_string()))
            .unwrap();
        registry
            .register(make_workflow("plugin-b"), Some("plugin-a".to_string()))
            .unwrap();

        registry.unregister_by_owner("plugin-a");
        assert_eq!(registry.count(), 1);
        assert!(registry.get("builtin").is_some());
        assert!(registry.get("plugin-a").is_none());
    }

    // Validates: Requirement 6.4 — query by name and category

    #[test]
    fn get_by_name_returns_definition() {
        let registry = WorkflowRegistry::new();
        let def = make_workflow("test");
        registry.register(def.clone(), None).unwrap();
        let found = registry.get("test").unwrap();
        assert_eq!(found.name, "test");
    }

    #[test]
    fn get_missing_name_returns_none() {
        let registry = WorkflowRegistry::new();
        assert!(registry.get("missing").is_none());
    }

    #[test]
    fn query_by_category_returns_matching() {
        let registry = WorkflowRegistry::new();
        let def = WorkflowBuilder::new("transfer")
            .category("file-operation")
            .step(make_step("start"))
            .initial_step("start")
            .terminal_step("start")
            .build()
            .unwrap();
        registry.register(def, None).unwrap();
        registry.register(make_workflow("other"), None).unwrap();

        let results = registry.query_by_category("file-operation");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "transfer");
    }

    // Validates: Requirement 6.6 — metadata exposure

    #[test]
    fn metadata_returns_workflow_info() {
        let registry = WorkflowRegistry::new();
        let def = WorkflowBuilder::new("meta-test")
            .display_name("Meta Test")
            .description("A test workflow")
            .category("testing")
            .supports_persistence(true)
            .step(make_step("s"))
            .initial_step("s")
            .terminal_step("s")
            .build()
            .unwrap();
        registry.register(def, None).unwrap();

        let meta = registry.metadata("meta-test").unwrap();
        assert_eq!(meta.display_name, "Meta Test");
        assert_eq!(meta.description, "A test workflow");
        assert!(meta.supports_persistence);
    }

    // Validates: Requirement 6.7 — thread-safety

    #[test]
    fn registry_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<WorkflowRegistry>();
    }
}
