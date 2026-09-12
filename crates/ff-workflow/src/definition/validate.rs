use std::collections::{HashMap, HashSet, VecDeque};

use crate::error::WorkflowError;

use super::{StepDefinition, Transition, WorkflowDefinition};

/// Finds all states that are unreachable from the initial state via transitions.
pub(super) fn find_unreachable_states(
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
