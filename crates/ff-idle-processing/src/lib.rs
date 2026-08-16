//! # ff-idle-processing — Cooperative Idle-Time Background Work Scheduler
//!
//! This crate provides a **GUI-independent background work coordinator** that
//! grants time slices to registered work sources when no user input is active.
//! It enables computationally intensive operations — syntax re-highlighting,
//! word-wrap height calculation, fold-level computation, and search index
//! building — to proceed incrementally without blocking user interactions.
//!
//! ## Design
//!
//! The scheduler operates on a cooperative time-slicing model:
//! - Registered work sources receive bounded time budgets during idle periods
//! - Any user input immediately cancels the current idle work
//! - Work sources are dispatched in priority order (lower value = higher priority)
//! - Round-robin among equal-priority sources prevents starvation
//!
//! ## Example
//!
//! ```rust
//! use ff_idle_processing::{IdleScheduler, IdleConfig, WorkPriority, WorkStatus, WorkProgress};
//! use ff_idle_processing::traits::{IdleWorkSource, IdleNotifier};
//! use ff_idle_processing::test_support::ManualIdleNotifier;
//! use ff_idle_processing::context::IdleWorkContext;
//!
//! struct CountingSource { count: u64, target: u64 }
//!
//! impl IdleWorkSource for CountingSource {
//!     fn perform_work(&mut self, ctx: &mut IdleWorkContext) -> WorkStatus {
//!         if ctx.is_cancelled() { return WorkStatus::Interrupted; }
//!         self.count += 1;
//!         if self.count >= self.target { WorkStatus::Complete } else { WorkStatus::MoreWork }
//!     }
//!     fn priority(&self) -> WorkPriority { WorkPriority::SYNTAX_HIGHLIGHT }
//!     fn name(&self) -> &str { "counter" }
//!     fn progress(&self) -> WorkProgress { WorkProgress::new(self.target) }
//! }
//! ```

pub mod config;
pub mod context;
pub mod error;
pub mod priority;
pub mod progress;
pub mod scheduler;
pub mod test_support;
pub mod traits;

pub use config::IdleConfig;
pub use context::IdleWorkContext;
pub use error::IdleProcessingError;
pub use priority::WorkPriority;
pub use progress::{WorkProgress, WorkStatus};
pub use scheduler::IdleScheduler;
pub use traits::{IdleNotifier, IdleWorkSource};
