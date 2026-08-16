//! # ff-workflow — State-Machine-Based Workflow Execution Engine
//!
//! The `ff-workflow` crate provides a declarative framework for defining,
//! executing, and managing multi-step operations in the FileForgeWorkbench
//! platform. It supports:
//!
//! - **Workflow definitions** as directed graphs of states and transitions
//! - **Step execution** (sequential, parallel, conditional) with shared context
//! - **Progress reporting** with determinate/indeterminate modes and throttling
//! - **Cooperative cancellation** with timeout enforcement
//! - **Error policies** (fail-fast, continue-on-error, retry with backoff)
//! - **Compensating actions** for rollback on failure or cancellation
//! - **Workflow registry** for discovery by name, category, and capability
//! - **Checkpoint persistence** for long-running workflow resumption
//!
//! # Architecture
//!
//! ```text
//! Invocation (Command Framework / Plugin / Startup)
//!     → WorkflowRegistry (lookup by name)
//!         → WorkflowRunner (drives state machine)
//!             → WorkflowStep implementations (execute logic)
//!                 → WorkflowContext (shared typed state)
//!                 → ProgressReporter (throttled events → Event Bus)
//!                 → CancellationToken (cooperative signal)
//! ```
//!
//! # Dependencies
//!
//! - `ff-logging` for diagnostic output
//! - `tokio` for async step execution
//! - `serde` for checkpoint serialization
//! - `thiserror` for structured error types

pub mod builtin;
pub mod cancellation;
pub mod checkpoint;
pub mod context;
pub mod definition;
pub mod error;
pub mod error_policy;
pub mod progress;
pub mod registry;
pub mod runner;
pub mod state;
pub mod step;

// Public API re-exports for convenience
pub use cancellation::CancellationToken;
pub use checkpoint::{Checkpoint, CheckpointManager};
pub use context::{ContextValue, ContextValueType, WorkflowContext};
pub use definition::{
    ContextKeyDeclaration, ParameterDeclaration, StepDefinition, StepKind, Transition,
    TransitionPredicate, WorkflowBuilder, WorkflowDefinition,
};
pub use error::{RollbackStatus, WorkflowError, WorkflowErrorReport};
pub use error_policy::{ErrorPolicy, ErrorStrategy, UserErrorAction};
pub use progress::{
    aggregate_progress, ProgressEvent, ProgressMode, ProgressReporter, WorkflowExecutionId,
};
pub use registry::{WorkflowMetadata, WorkflowRegistry};
pub use runner::{WorkflowHandle, WorkflowResult, WorkflowRunner};
pub use state::{StepStatus, WorkflowPhase, WorkflowState};
pub use step::{CompensatingAction, NoOpEventDispatcher, WorkflowEventDispatcher, WorkflowStep};
