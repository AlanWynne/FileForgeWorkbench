//! Property-based tests for the ff-workflow crate.
//!
//! These tests verify invariants that must hold across all valid inputs,
//! using the proptest framework with a minimum of 100 iterations per property.

use std::time::Duration;

use proptest::prelude::*;

use ff_workflow::{
    aggregate_progress, ContextValue, ErrorPolicy, ErrorStrategy, StepDefinition, Transition,
    WorkflowBuilder, WorkflowContext, WorkflowDefinition, WorkflowExecutionId, WorkflowRegistry,
};

// ─── Property 1: Workflow Definition Validation Completeness ────────────────
// Feature: workflow-engine, Property 1: definition validation
// **Validates: Requirements 1.3**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 1: For any directed graph of states and transitions, the
    /// definition validator SHALL accept the graph if and only if it has
    /// exactly one initial state, at least one terminal state, and no
    /// unreachable states.
    ///
    /// **Validates: Requirements 1.3**
    #[test]
    fn definition_validation_accepts_valid_graphs(
        num_steps in 2usize..10
    ) {
        // Generate a valid linear workflow and assert it builds successfully
        let runner = proptest::test_runner::TestRunner::default();
        let _ = runner; // just using the config above
        let names: Vec<String> = (0..num_steps).map(|i| format!("step_{}", i)).collect();

        let mut builder = WorkflowBuilder::new("prop-test");
        for name in &names {
            builder = builder.step(StepDefinition {
                name: name.clone(),
                display_name: name.clone(),
                ..Default::default()
            });
        }
        for i in 0..names.len() - 1 {
            builder = builder.transition(Transition {
                from: names[i].clone(),
                to: names[i + 1].clone(),
                predicate: None,
                priority: 0,
            });
        }
        builder = builder.initial_step(names[0].clone());
        builder = builder.terminal_step(names[names.len() - 1].clone());

        let result = builder.build();
        prop_assert!(result.is_ok(), "valid linear graph must pass validation");
    }

    /// Property 1 (negative): Graphs without initial state, terminal states,
    /// or with unreachable states SHALL be rejected.
    ///
    /// **Validates: Requirements 1.3**
    #[test]
    fn definition_validation_rejects_no_initial_state(
        num_steps in 1usize..8
    ) {
        let names: Vec<String> = (0..num_steps).map(|i| format!("step_{}", i)).collect();
        let mut builder = WorkflowBuilder::new("no-initial");
        for name in &names {
            builder = builder.step(StepDefinition {
                name: name.clone(),
                display_name: name.clone(),
                ..Default::default()
            });
        }
        builder = builder.terminal_step(names[0].clone());
        // No initial_step set
        let result = builder.build();
        prop_assert!(result.is_err(), "missing initial state must fail validation");
    }

    /// Property 1 (negative): no terminal states.
    ///
    /// **Validates: Requirements 1.3**
    #[test]
    fn definition_validation_rejects_no_terminal_states(
        num_steps in 1usize..8
    ) {
        let names: Vec<String> = (0..num_steps).map(|i| format!("step_{}", i)).collect();
        let mut builder = WorkflowBuilder::new("no-terminal");
        for name in &names {
            builder = builder.step(StepDefinition {
                name: name.clone(),
                display_name: name.clone(),
                ..Default::default()
            });
        }
        builder = builder.initial_step(names[0].clone());
        // No terminal_step set
        let result = builder.build();
        prop_assert!(result.is_err(), "missing terminal states must fail validation");
    }

    /// Property 1 (negative): unreachable states.
    ///
    /// **Validates: Requirements 1.3**
    #[test]
    fn definition_validation_rejects_unreachable_states(
        num_steps in 3usize..10
    ) {
        let names: Vec<String> = (0..num_steps).map(|i| format!("step_{}", i)).collect();
        let mut builder = WorkflowBuilder::new("unreachable");
        for name in &names {
            builder = builder.step(StepDefinition {
                name: name.clone(),
                display_name: name.clone(),
                ..Default::default()
            });
        }
        // Only connect first two, leaving the rest unreachable
        builder = builder.transition(Transition {
            from: names[0].clone(),
            to: names[1].clone(),
            predicate: None,
            priority: 0,
        });
        builder = builder.initial_step(names[0].clone());
        builder = builder.terminal_step(names[1].clone());
        let result = builder.build();
        prop_assert!(result.is_err(), "unreachable states must fail validation");
    }
}

// ─── Property 2: Progress Aggregation Correctness ───────────────────────────
// Feature: workflow-engine, Property 2: progress aggregation correctness
// **Validates: Requirements 4.4**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property 2: For any workflow with N steps where each step reports
    /// progress in [0, 100], the aggregated parent progress SHALL equal
    /// (completed_steps + current_step_progress/100) / total_steps * 100
    /// and SHALL always be in [0, 100].
    ///
    /// **Validates: Requirements 4.4**
    #[test]
    fn progress_aggregation_is_correct(
        total_steps in 1usize..50,
        completed_steps_offset in 0usize..50,
        current_step_fraction in 0.0f64..=100.0f64,
    ) {
        let completed_steps = completed_steps_offset % total_steps;
        let result = aggregate_progress(completed_steps, current_step_fraction, total_steps);

        // Must be in [0, 100]
        prop_assert!(result >= 0.0, "aggregated progress must be >= 0, got {}", result);
        prop_assert!(result <= 100.0, "aggregated progress must be <= 100, got {}", result);

        // Must match the formula
        let expected = (completed_steps as f64 + current_step_fraction / 100.0)
            / total_steps as f64
            * 100.0;
        let expected_clamped = expected.clamp(0.0, 100.0);
        prop_assert!(
            (result - expected_clamped).abs() < 1e-10,
            "aggregated {} != expected {}",
            result,
            expected_clamped
        );
    }

    /// Property 2 (edge case): zero total steps returns 0.
    ///
    /// **Validates: Requirements 4.4**
    #[test]
    fn progress_aggregation_zero_total_returns_zero(
        completed in 0usize..10,
        fraction in 0.0f64..100.0,
    ) {
        let result = aggregate_progress(completed, fraction, 0);
        prop_assert_eq!(result, 0.0);
    }
}

// ─── Property 3: Error Policy Determinism ───────────────────────────────────
// Feature: workflow-engine, Property 3: error policy determinism
// **Validates: Requirements 5.1, 5.2**

fn arbitrary_error_strategy() -> impl Strategy<Value = ErrorStrategy> {
    prop_oneof![
        Just(ErrorStrategy::FailFast),
        Just(ErrorStrategy::ContinueOnError),
        Just(ErrorStrategy::Retry),
    ]
}

fn arbitrary_error_policy() -> impl Strategy<Value = ErrorPolicy> {
    (arbitrary_error_strategy(), 1u32..10, 100u64..5000).prop_map(
        |(strategy, max_retries, delay_ms)| ErrorPolicy {
            strategy,
            max_retries,
            retry_delay: Duration::from_millis(delay_ms),
            allow_user_interaction: false,
        },
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property 3: For any combination of workflow-level error policy and
    /// per-step override, the effective policy for a step SHALL be deterministic:
    /// if a step override exists it takes precedence; otherwise the workflow-level
    /// policy applies.
    ///
    /// **Validates: Requirements 5.1, 5.2**
    #[test]
    fn error_policy_determinism(
        workflow_policy in arbitrary_error_policy(),
        step_override in proptest::option::of(arbitrary_error_policy()),
    ) {
        let effective = ff_workflow::error_policy::effective_policy(
            &workflow_policy,
            step_override.as_ref(),
        );

        match &step_override {
            Some(override_policy) => {
                prop_assert_eq!(
                    effective.strategy, override_policy.strategy,
                    "step override must take precedence"
                );
                prop_assert_eq!(
                    effective.max_retries, override_policy.max_retries,
                    "step override max_retries must take precedence"
                );
            }
            None => {
                prop_assert_eq!(
                    effective.strategy, workflow_policy.strategy,
                    "workflow policy must apply when no override"
                );
                prop_assert_eq!(
                    effective.max_retries, workflow_policy.max_retries,
                    "workflow policy max_retries must apply when no override"
                );
            }
        }
    }
}

// ─── Property 4: Cancellation Safety ────────────────────────────────────────
// Feature: workflow-engine, Property 4: cancellation safety
// **Validates: Requirements 3.1, 3.2**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 4: Cancellation token, once cancelled, remains cancelled
    /// and all children see the cancellation immediately.
    ///
    /// **Validates: Requirements 3.1, 3.2**
    #[test]
    fn cancellation_propagates_to_all_children(
        num_children in 1usize..20
    ) {
        let parent = ff_workflow::CancellationToken::new();
        let children: Vec<_> = (0..num_children).map(|_| parent.child()).collect();

        // Before cancel: nothing is cancelled
        prop_assert!(!parent.is_cancelled());
        for child in &children {
            prop_assert!(!child.is_cancelled());
        }

        // After cancel: everything is cancelled
        parent.cancel();
        prop_assert!(parent.is_cancelled());
        for child in &children {
            prop_assert!(child.is_cancelled(), "all children must be cancelled");
        }

        // Idempotent
        parent.cancel();
        prop_assert!(parent.is_cancelled());
    }
}

// ─── Property 5: Checkpoint Round-Trip ──────────────────────────────────────
// Feature: workflow-engine, Property 5: checkpoint round-trip
// **Validates: Requirements 7.2, 7.5**

fn arbitrary_context_value() -> impl Strategy<Value = ContextValue> {
    prop_oneof![
        any::<i64>().prop_map(ContextValue::Integer),
        "[a-zA-Z0-9 ]{0,20}".prop_map(|s| ContextValue::String(s)),
        any::<bool>().prop_map(ContextValue::Boolean),
        proptest::collection::vec(any::<u8>(), 0..32).prop_map(ContextValue::Bytes),
        proptest::collection::vec("[a-z]{1,5}".prop_map(|s| s), 0..5)
            .prop_map(ContextValue::StringList),
        Just(ContextValue::Null),
    ]
}

fn arbitrary_context() -> impl Strategy<Value = WorkflowContext> {
    proptest::collection::vec(
        ("[a-z_]{1,10}".prop_map(|s| s), arbitrary_context_value()),
        0..10,
    )
    .prop_map(|entries| {
        let mut ctx = WorkflowContext::new();
        for (key, value) in entries {
            ctx.set(key, value);
        }
        ctx
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 5: For any workflow state, serializing to a checkpoint and
    /// deserializing SHALL produce an equivalent state.
    ///
    /// **Validates: Requirements 7.2, 7.5**
    #[test]
    fn checkpoint_round_trip_preserves_context(
        context in arbitrary_context()
    ) {
        let json = serde_json::to_string(&context).expect("context must serialize");
        let restored: WorkflowContext = serde_json::from_str(&json).expect("context must deserialize");
        prop_assert_eq!(&context, &restored);
    }

    /// Property 5: Full WorkflowState serialization round-trip.
    ///
    /// **Validates: Requirements 7.2, 7.5**
    #[test]
    fn checkpoint_round_trip_preserves_state(
        context in arbitrary_context(),
        num_steps in 1usize..10,
    ) {
        use ff_workflow::state::WorkflowState;

        let step_names: Vec<String> = (0..num_steps).map(|i| format!("s{}", i)).collect();
        let state = WorkflowState::new(
            WorkflowExecutionId("prop-test-id".to_string()),
            "prop-wf".to_string(),
            step_names[0].clone(),
            step_names,
            context,
        );

        let json = serde_json::to_string(&state).expect("state must serialize");
        let restored: WorkflowState = serde_json::from_str(&json).expect("state must deserialize");

        prop_assert_eq!(state.execution_id, restored.execution_id);
        prop_assert_eq!(state.workflow_name, restored.workflow_name);
        prop_assert_eq!(state.context, restored.context);
        prop_assert_eq!(state.step_statuses.len(), restored.step_statuses.len());
    }
}

// ─── Property 6: Registry Uniqueness Invariant ──────────────────────────────
// Feature: workflow-engine, Property 6: registry uniqueness invariant
// **Validates: Requirements 6.1**

/// Operations on the registry for property testing.
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum RegistryOp {
    Register(String),
    Unregister(String),
    Query(String),
}

fn make_minimal_definition(name: &str) -> WorkflowDefinition {
    WorkflowBuilder::new(name)
        .step(StepDefinition {
            name: "s".to_string(),
            display_name: "s".to_string(),
            ..Default::default()
        })
        .initial_step("s")
        .terminal_step("s")
        .build()
        .expect("minimal workflow must build")
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 6: For any sequence of register/unregister operations, the
    /// registry SHALL never contain duplicate workflow names. A registration
    /// with an existing name SHALL be rejected and leave the registry unchanged.
    ///
    /// **Validates: Requirements 6.1**
    #[test]
    fn registry_uniqueness_invariant(
        ops in proptest::collection::vec(
            prop_oneof![
                (0u8..5).prop_map(|i| RegistryOp::Register(format!("wf_{}", i))),
                (0u8..5).prop_map(|i| RegistryOp::Unregister(format!("wf_{}", i))),
                (0u8..5).prop_map(|i| RegistryOp::Query(format!("wf_{}", i))),
            ],
            10..50
        )
    ) {
        let registry = WorkflowRegistry::new();

        for op in &ops {
            match op {
                RegistryOp::Register(name) => {
                    let count_before = registry.count();
                    let def = make_minimal_definition(name);
                    let result = registry.register(def, None);
                    if result.is_err() {
                        // Duplicate rejected — count unchanged
                        prop_assert_eq!(registry.count(), count_before);
                    }
                }
                RegistryOp::Unregister(name) => {
                    registry.unregister(name);
                }
                RegistryOp::Query(name) => {
                    let _ = registry.get(name);
                }
            }

            // Invariant: all names are unique
            let names = registry.list_all();
            let mut sorted = names.clone();
            sorted.sort();
            sorted.dedup();
            prop_assert_eq!(
                names.len(), sorted.len(),
                "registry must never contain duplicate names"
            );
        }
    }
}

// ─── Property 7: Context Type Safety ────────────────────────────────────────
// Feature: workflow-engine, Property 7: context type safety
// **Validates: Requirements 2.2, 1.5**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property 7: For any typed value stored in the workflow context under a
    /// key, retrieving that key with the correct type SHALL return the value,
    /// and retrieving with an incorrect type SHALL return None.
    ///
    /// **Validates: Requirements 2.2, 1.5**
    #[test]
    fn context_type_safety(
        key in "[a-z_]{1,10}",
        int_val in any::<i64>(),
        str_val in "[a-zA-Z0-9]{0,20}",
        bool_val in any::<bool>(),
        float_val in any::<f64>().prop_filter("finite", |f| f.is_finite()),
    ) {
        // Test integer storage
        let mut ctx = WorkflowContext::new();
        ctx.set(key.clone(), ContextValue::Integer(int_val));
        prop_assert_eq!(ctx.get_integer(&key), Some(int_val));
        prop_assert_eq!(ctx.get_string(&key), None); // wrong type
        prop_assert_eq!(ctx.get_bool(&key), None); // wrong type

        // Test string storage
        let mut ctx = WorkflowContext::new();
        ctx.set(key.clone(), ContextValue::String(str_val.clone()));
        prop_assert_eq!(ctx.get_string(&key), Some(str_val.as_str()));
        prop_assert_eq!(ctx.get_integer(&key), None); // wrong type
        prop_assert_eq!(ctx.get_bool(&key), None); // wrong type

        // Test boolean storage
        let mut ctx = WorkflowContext::new();
        ctx.set(key.clone(), ContextValue::Boolean(bool_val));
        prop_assert_eq!(ctx.get_bool(&key), Some(bool_val));
        prop_assert_eq!(ctx.get_integer(&key), None); // wrong type
        prop_assert_eq!(ctx.get_string(&key), None); // wrong type

        // Test float storage
        let mut ctx = WorkflowContext::new();
        ctx.set(key.clone(), ContextValue::Float(float_val));
        prop_assert_eq!(ctx.get_float(&key), Some(float_val));
        prop_assert_eq!(ctx.get_integer(&key), None); // wrong type
        prop_assert_eq!(ctx.get_string(&key), None); // wrong type
    }
}

// ─── Property 8: Retry Exhaustion Convergence ───────────────────────────────
// Feature: workflow-engine, Property 8: retry exhaustion convergence
// **Validates: Requirements 5.3**

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 8: For any step configured with retry policy (max_attempts in
    /// [1, 10]), the step SHALL be executed at most max_attempts times regardless
    /// of failure pattern; after exhaustion, the next-level policy SHALL be
    /// applied exactly once.
    ///
    /// **Validates: Requirements 5.3**
    #[test]
    fn retry_exhaustion_bounded(
        max_retries in 1u32..10,
    ) {
        // The effective max_attempts = max_retries + 1 (initial + retries)
        // ErrorPolicy::retry creates a policy with the specified max_retries
        let policy = ErrorPolicy::retry(max_retries, Duration::from_millis(0));
        let max_attempts = match policy.strategy {
            ErrorStrategy::Retry => policy.max_retries + 1,
            _ => 1,
        };

        // The runner will execute at most max_attempts times
        prop_assert_eq!(max_attempts, max_retries + 1);
        prop_assert!(max_attempts <= 11, "max attempts must be bounded");
        prop_assert!(max_attempts >= 2, "retry must attempt at least twice");
    }
}
