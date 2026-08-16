//! Workflow definition — declarative state machine descriptions.
//!
//! A `WorkflowDefinition` describes a workflow as a directed graph of states
//! and transitions. Definitions are constructed via the `WorkflowBuilder` and
//! validated for structural correctness (exactly one initial state, at least
//! one terminal state, no unreachable states).

use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Duration;

use crate::context::{ContextValue, ContextValueType};
use crate::error::WorkflowError;
use crate::error_policy::ErrorPolicy;

/// A declarative description of a workflow's structure: states, transitions,
/// steps, error policy, and cancellation behaviour.
///
/// Addresses: Requirement 1, criteria 1/4/6
#[derive(Debug, Clone)]
pub struct WorkflowDefinition {
    /// Unique workflow name (used as registry key).
    pub name: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Description of what the workflow does.
    pub description: String,
    /// Category tags for registry queries.
    pub categories: Vec<String>,
    /// The ordered set of step definitions forming the state graph.
    pub steps: Vec<StepDefinition>,
    /// Transitions between steps (directed edges in the state graph).
    pub transitions: Vec<Transition>,
    /// The name of the initial step (exactly one required).
    pub initial_step: String,
    /// The names of terminal steps (at least one required).
    pub terminal_steps: Vec<String>,
    /// Input parameters this workflow requires.
    pub parameters: Vec<ParameterDeclaration>,
    /// Default error policy for all steps.
    pub error_policy: ErrorPolicy,
    /// Whether this workflow supports persistence/checkpoint.
    pub supports_persistence: bool,
    /// Whether this workflow supports cancellation.
    pub supports_cancellation: bool,
    /// Whether this workflow supports pause/resume.
    pub supports_pause: bool,
}

/// A single step within a workflow definition.
///
/// Addresses: Requirement 1, criteria 1/2/5
#[derive(Debug, Clone)]
pub struct StepDefinition {
    /// Unique name within this workflow.
    pub name: String,
    /// Human-readable display name (for progress reporting).
    pub display_name: String,
    /// The kind of step execution.
    pub kind: StepKind,
    /// Expected input keys from the WorkflowContext (for validation).
    pub expected_inputs: Vec<ContextKeyDeclaration>,
    /// Output keys this step will write to the WorkflowContext.
    pub declared_outputs: Vec<ContextKeyDeclaration>,
    /// Per-step error policy override.
    pub error_policy_override: Option<ErrorPolicy>,
    /// Whether this step has a compensating action for rollback.
    pub has_compensation: bool,
    /// Cancellation timeout for this step.
    pub cancellation_timeout: Duration,
}

impl Default for StepDefinition {
    fn default() -> Self {
        Self {
            name: String::new(),
            display_name: String::new(),
            kind: StepKind::Sequential,
            expected_inputs: Vec::new(),
            declared_outputs: Vec::new(),
            error_policy_override: None,
            has_compensation: false,
            cancellation_timeout: Duration::from_secs(5),
        }
    }
}

/// The execution mode of a step.
///
/// Addresses: Requirement 1, criterion 2
#[derive(Debug, Clone)]
pub enum StepKind {
    /// A single sequential step (sync or async).
    Sequential,
    /// A group of steps that execute concurrently with a join barrier.
    Parallel {
        /// Names of member steps in this parallel group.
        member_steps: Vec<String>,
    },
    /// A conditional branch point — routes transitions based on predicates.
    Conditional,
}

/// A directed edge between steps in the workflow graph.
///
/// Addresses: Requirement 1, criteria 1/2
#[derive(Debug, Clone)]
pub struct Transition {
    /// Source step name.
    pub from: String,
    /// Target step name.
    pub to: String,
    /// Condition for this transition (None = unconditional / default).
    pub predicate: Option<TransitionPredicate>,
    /// Priority for ordering when multiple predicates are evaluated.
    pub priority: u32,
}

/// A predicate evaluated against the WorkflowContext to determine
/// which conditional branch to follow.
///
/// Addresses: Requirement 2, criterion 5
#[derive(Debug, Clone)]
pub enum TransitionPredicate {
    /// Context key equals a specific value.
    Equals {
        /// The context key to check.
        key: String,
        /// The expected value.
        value: ContextValue,
    },
    /// Context key exists and is truthy (non-null, non-false, non-zero).
    IsTrue {
        /// The context key to check.
        key: String,
    },
    /// Context key does not exist or is falsy.
    IsFalse {
        /// The context key to check.
        key: String,
    },
    /// Custom predicate (description for diagnostics).
    Custom {
        /// Human-readable description of the predicate.
        description: String,
    },
}

/// Declares an input parameter for a workflow.
///
/// Addresses: Requirement 1, criterion 6
#[derive(Debug, Clone)]
pub struct ParameterDeclaration {
    /// Parameter name.
    pub name: String,
    /// Expected type (for validation at invocation time).
    pub value_type: ContextValueType,
    /// Whether this parameter is required.
    pub required: bool,
    /// Optional default value.
    pub default: Option<ContextValue>,
    /// Human-readable description (for UI and help).
    pub description: String,
}

/// Declares a key expected or produced by a step in the WorkflowContext.
///
/// Addresses: Requirement 1, criterion 5
#[derive(Debug, Clone)]
pub struct ContextKeyDeclaration {
    /// The key name.
    pub key: String,
    /// The expected value type.
    pub value_type: ContextValueType,
    /// Human-readable description.
    pub description: String,
}

/// Fluent builder API for constructing workflow definitions.
///
/// Addresses: Requirement 1, criterion 4
pub struct WorkflowBuilder {
    name: String,
    display_name: String,
    description: String,
    categories: Vec<String>,
    steps: Vec<StepDefinition>,
    transitions: Vec<Transition>,
    initial_step: Option<String>,
    terminal_steps: Vec<String>,
    parameters: Vec<ParameterDeclaration>,
    error_policy: ErrorPolicy,
    supports_persistence: bool,
    supports_cancellation: bool,
    supports_pause: bool,
}

impl WorkflowBuilder {
    /// Starts building a new workflow with the given unique name.
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            display_name: name.clone(),
            name,
            description: String::new(),
            categories: Vec::new(),
            steps: Vec::new(),
            transitions: Vec::new(),
            initial_step: None,
            terminal_steps: Vec::new(),
            parameters: Vec::new(),
            error_policy: ErrorPolicy::default(),
            supports_persistence: false,
            supports_cancellation: true,
            supports_pause: false,
        }
    }

    /// Sets the display name.
    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = name.into();
        self
    }

    /// Sets the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Adds a category tag.
    pub fn category(mut self, cat: impl Into<String>) -> Self {
        self.categories.push(cat.into());
        self
    }

    /// Adds an input parameter declaration.
    pub fn parameter(mut self, param: ParameterDeclaration) -> Self {
        self.parameters.push(param);
        self
    }

    /// Adds a step definition.
    pub fn step(mut self, step: StepDefinition) -> Self {
        self.steps.push(step);
        self
    }

    /// Adds a transition between steps.
    pub fn transition(mut self, transition: Transition) -> Self {
        self.transitions.push(transition);
        self
    }

    /// Sets the initial step name.
    pub fn initial_step(mut self, name: impl Into<String>) -> Self {
        self.initial_step = Some(name.into());
        self
    }

    /// Marks a step as terminal (success endpoint).
    pub fn terminal_step(mut self, name: impl Into<String>) -> Self {
        self.terminal_steps.push(name.into());
        self
    }

    /// Sets the default error policy for all steps.
    pub fn error_policy(mut self, policy: ErrorPolicy) -> Self {
        self.error_policy = policy;
        self
    }

    /// Enables or disables persistence support.
    pub fn supports_persistence(mut self, enabled: bool) -> Self {
        self.supports_persistence = enabled;
        self
    }

    /// Enables or disables cancellation support.
    pub fn supports_cancellation(mut self, enabled: bool) -> Self {
        self.supports_cancellation = enabled;
        self
    }

    /// Enables or disables pause/resume support.
    pub fn supports_pause(mut self, enabled: bool) -> Self {
        self.supports_pause = enabled;
        self
    }

    /// Validates and builds the workflow definition.
    ///
    /// Returns an error if the definition is structurally invalid:
    /// - No initial state
    /// - No terminal states
    /// - Unreachable states from initial
    ///
    /// Addresses: Requirement 1, criterion 3
    pub fn build(self) -> Result<WorkflowDefinition, WorkflowError> {
        let initial_step = self
            .initial_step
            .ok_or_else(|| WorkflowError::NoInitialState {
                name: self.name.clone(),
            })?;

        if self.terminal_steps.is_empty() {
            return Err(WorkflowError::NoTerminalStates {
                name: self.name.clone(),
            });
        }

        // Validate that initial_step exists in steps
        let step_names: HashSet<&str> = self.steps.iter().map(|s| s.name.as_str()).collect();
        if !step_names.contains(initial_step.as_str()) {
            return Err(WorkflowError::InvalidDefinition {
                description: format!("initial step '{}' is not in the step list", initial_step),
            });
        }

        // Validate terminal steps exist
        for t in &self.terminal_steps {
            if !step_names.contains(t.as_str()) {
                return Err(WorkflowError::InvalidDefinition {
                    description: format!("terminal step '{}' is not in the step list", t),
                });
            }
        }

        // BFS reachability check from initial step
        let unreachable = find_unreachable_states(
            &initial_step,
            &self.steps,
            &self.transitions,
            &self.terminal_steps,
        );

        if !unreachable.is_empty() {
            return Err(WorkflowError::UnreachableStates {
                name: self.name.clone(),
                states: unreachable,
            });
        }

        Ok(WorkflowDefinition {
            name: self.name,
            display_name: self.display_name,
            description: self.description,
            categories: self.categories,
            steps: self.steps,
            transitions: self.transitions,
            initial_step,
            terminal_steps: self.terminal_steps,
            parameters: self.parameters,
            error_policy: self.error_policy,
            supports_persistence: self.supports_persistence,
            supports_cancellation: self.supports_cancellation,
            supports_pause: self.supports_pause,
        })
    }
}

/// Finds all states that are unreachable from the initial state via transitions.
fn find_unreachable_states(
    initial: &str,
    steps: &[StepDefinition],
    transitions: &[Transition],
    terminal_steps: &[String],
) -> Vec<String> {
    let all_step_names: HashSet<&str> = steps.iter().map(|s| s.name.as_str()).collect();

    // Build adjacency list
    let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
    for t in transitions {
        adjacency
            .entry(t.from.as_str())
            .or_default()
            .push(t.to.as_str());
    }

    // BFS from initial
    let mut visited: HashSet<&str> = HashSet::new();
    let mut queue: VecDeque<&str> = VecDeque::new();
    queue.push_back(initial);
    visited.insert(initial);

    while let Some(current) = queue.pop_front() {
        if let Some(neighbors) = adjacency.get(current) {
            for &next in neighbors {
                if visited.insert(next) {
                    queue.push_back(next);
                }
            }
        }
    }

    // Terminal steps are always considered reachable (they don't need outgoing transitions)
    for t in terminal_steps {
        visited.insert(t.as_str());
    }

    // Find unreachable states
    all_step_names
        .iter()
        .filter(|&&name| !visited.contains(name))
        .map(|&name| name.to_string())
        .collect()
}

/// Validates a workflow definition for structural correctness.
///
/// Checks:
/// - Exactly one initial state
/// - At least one terminal state
/// - No unreachable states from the initial state
///
/// Addresses: Requirement 1, criterion 3
pub fn validate_definition(def: &WorkflowDefinition) -> Result<(), WorkflowError> {
    let step_names: HashSet<&str> = def.steps.iter().map(|s| s.name.as_str()).collect();

    // Check initial step exists
    if !step_names.contains(def.initial_step.as_str()) {
        return Err(WorkflowError::InvalidDefinition {
            description: format!(
                "initial step '{}' is not in the step list",
                def.initial_step
            ),
        });
    }

    // Check at least one terminal
    if def.terminal_steps.is_empty() {
        return Err(WorkflowError::NoTerminalStates {
            name: def.name.clone(),
        });
    }

    // Check terminal steps exist
    for t in &def.terminal_steps {
        if !step_names.contains(t.as_str()) {
            return Err(WorkflowError::InvalidDefinition {
                description: format!("terminal step '{}' is not in the step list", t),
            });
        }
    }

    // Check reachability
    let unreachable = find_unreachable_states(
        &def.initial_step,
        &def.steps,
        &def.transitions,
        &def.terminal_steps,
    );

    if !unreachable.is_empty() {
        return Err(WorkflowError::UnreachableStates {
            name: def.name.clone(),
            states: unreachable,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_step(name: &str) -> StepDefinition {
        StepDefinition {
            name: name.to_string(),
            display_name: name.to_string(),
            ..Default::default()
        }
    }

    fn make_transition(from: &str, to: &str) -> Transition {
        Transition {
            from: from.to_string(),
            to: to.to_string(),
            predicate: None,
            priority: 0,
        }
    }

    // Validates: Requirement 1.3 — definition validation: exactly one initial, at least one terminal

    #[test]
    fn valid_linear_workflow_builds_successfully() {
        let def = WorkflowBuilder::new("test-workflow")
            .step(make_step("start"))
            .step(make_step("process"))
            .step(make_step("end"))
            .transition(make_transition("start", "process"))
            .transition(make_transition("process", "end"))
            .initial_step("start")
            .terminal_step("end")
            .build();
        assert!(def.is_ok());
    }

    #[test]
    fn missing_initial_step_returns_error() {
        let result = WorkflowBuilder::new("no-initial")
            .step(make_step("a"))
            .terminal_step("a")
            .build();
        assert!(matches!(result, Err(WorkflowError::NoInitialState { .. })));
    }

    #[test]
    fn missing_terminal_steps_returns_error() {
        let result = WorkflowBuilder::new("no-terminal")
            .step(make_step("a"))
            .initial_step("a")
            .build();
        assert!(matches!(
            result,
            Err(WorkflowError::NoTerminalStates { .. })
        ));
    }

    #[test]
    fn unreachable_state_returns_error() {
        let result = WorkflowBuilder::new("unreachable")
            .step(make_step("start"))
            .step(make_step("end"))
            .step(make_step("orphan"))
            .transition(make_transition("start", "end"))
            .initial_step("start")
            .terminal_step("end")
            .build();
        assert!(matches!(
            result,
            Err(WorkflowError::UnreachableStates { .. })
        ));
    }

    #[test]
    fn initial_step_not_in_steps_returns_error() {
        let result = WorkflowBuilder::new("bad-initial")
            .step(make_step("a"))
            .initial_step("nonexistent")
            .terminal_step("a")
            .build();
        assert!(matches!(
            result,
            Err(WorkflowError::InvalidDefinition { .. })
        ));
    }

    #[test]
    fn terminal_step_not_in_steps_returns_error() {
        let result = WorkflowBuilder::new("bad-terminal")
            .step(make_step("a"))
            .initial_step("a")
            .terminal_step("nonexistent")
            .build();
        assert!(matches!(
            result,
            Err(WorkflowError::InvalidDefinition { .. })
        ));
    }

    // Validates: Requirement 1.2 — three step-sequencing modes

    #[test]
    fn workflow_with_conditional_step_builds() {
        let def = WorkflowBuilder::new("conditional")
            .step(StepDefinition {
                name: "check".to_string(),
                display_name: "Check".to_string(),
                kind: StepKind::Conditional,
                ..Default::default()
            })
            .step(make_step("branch_a"))
            .step(make_step("branch_b"))
            .transition(Transition {
                from: "check".to_string(),
                to: "branch_a".to_string(),
                predicate: Some(TransitionPredicate::IsTrue {
                    key: "flag".to_string(),
                }),
                priority: 1,
            })
            .transition(Transition {
                from: "check".to_string(),
                to: "branch_b".to_string(),
                predicate: None,
                priority: 0,
            })
            .initial_step("check")
            .terminal_step("branch_a")
            .terminal_step("branch_b")
            .build();
        assert!(def.is_ok());
    }

    // Validates: Requirement 1.4 — builder API (data-driven definitions)

    #[test]
    fn builder_sets_all_metadata() {
        let def = WorkflowBuilder::new("my-workflow")
            .display_name("My Workflow")
            .description("A test workflow")
            .category("file-operation")
            .category("data-transfer")
            .supports_persistence(true)
            .supports_cancellation(true)
            .supports_pause(true)
            .step(make_step("only"))
            .initial_step("only")
            .terminal_step("only")
            .build()
            .expect("should build");

        assert_eq!(def.name, "my-workflow");
        assert_eq!(def.display_name, "My Workflow");
        assert_eq!(def.description, "A test workflow");
        assert_eq!(def.categories, vec!["file-operation", "data-transfer"]);
        assert!(def.supports_persistence);
        assert!(def.supports_cancellation);
        assert!(def.supports_pause);
    }

    // Validates: Requirement 1.6 — parameterization

    #[test]
    fn builder_accepts_parameters() {
        let def = WorkflowBuilder::new("parameterized")
            .parameter(ParameterDeclaration {
                name: "source_path".to_string(),
                value_type: ContextValueType::String,
                required: true,
                default: None,
                description: "Source file path".to_string(),
            })
            .step(make_step("only"))
            .initial_step("only")
            .terminal_step("only")
            .build()
            .expect("should build");

        assert_eq!(def.parameters.len(), 1);
        assert_eq!(def.parameters[0].name, "source_path");
        assert!(def.parameters[0].required);
    }

    #[test]
    fn single_step_workflow_that_is_both_initial_and_terminal() {
        let def = WorkflowBuilder::new("single")
            .step(make_step("only"))
            .initial_step("only")
            .terminal_step("only")
            .build();
        assert!(def.is_ok());
    }

    #[test]
    fn validate_definition_passes_for_valid_definition() {
        let def = WorkflowBuilder::new("valid")
            .step(make_step("a"))
            .step(make_step("b"))
            .transition(make_transition("a", "b"))
            .initial_step("a")
            .terminal_step("b")
            .build()
            .expect("should build");

        assert!(validate_definition(&def).is_ok());
    }
}
