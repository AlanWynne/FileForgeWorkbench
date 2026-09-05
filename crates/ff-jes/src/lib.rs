//! # ff-jes — FileForge Workbench Job Entry Subsystem
//!
//! Provides mainframe-style batch job management:
//! - FFJCL job definition parsing and validation
//! - Priority-based job queue with persistence
//! - Configurable initiator pool for concurrent execution
//! - Scheduler with FIFO and priority dispatch strategies
//! - SDSF-style job monitoring with filtering
//! - Retention policy and purge engine
//! - Provider abstraction for future extensibility
//! - Job and Dataset APIs for programmatic access

pub mod config;
pub mod error;
pub mod ffjcl;
pub mod initiator;
pub mod log;
pub mod model;
pub mod provider;
pub mod queue;
pub mod retention;
pub mod scheduler;
pub mod sdsf_action;
pub mod sdsf_commands;
pub mod sdsf_filter;
pub mod sdsf_filter_expr;
pub mod sdsf_help;
pub mod sdsf_log_panels;
pub mod sdsf_overtype;
pub mod sdsf_panel;
pub mod sdsf_set_p2;
pub mod sdsf_system_panels;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use config::JesConfig;
pub use error::JesError;
pub use ffjcl::{parse_ffjcl, validate_definition, FfjclDd, FfjclDefinition, FfjclStep};
pub use initiator::InitiatorPool;
pub use log::{JobLog, LogEntry, LogLevel, StepLog};
pub use model::{
    Disposition, InitiatorId, InitiatorStatus, Job, JobEvent, JobFilter, JobId, JobSortField,
    JobStatus, JobStatusUpdate, StepStatus,
};
pub use provider::{DesktopJesProvider, JobAction, JobProvider, ProviderHealth, ProviderRegistry};
pub use queue::JobQueue;
pub use retention::{RetentionEngine, RetentionPolicy};
pub use scheduler::{Scheduler, SchedulingStrategy};
pub use sdsf_action::{
    parse_set_rownum, ActionChar, CommandLineAction, NpColumnState, NpDispatch, NpEntry,
};
pub use sdsf_commands::{
    default_auth_list, locate, AuthEntry, AuthKind, FindCase, FindState, LocateResult,
    ScrollCommand, ScrollDir, SdsfSetSettings, WhoInfo,
};
pub use sdsf_filter::{ColumnLayout, QueueTab, SdsfColumn, SdsfFilter, SdsfSort, SortDirection};
pub use sdsf_filter_expr::{ActiveFilter, CmpOp, FilterExpr, FilterPredicate};
pub use sdsf_help::{ActionCharHelp, ColumnHelp, CommandHelp, HelpKind, HelpPanel, HelpRegistry};
pub use sdsf_log_panels::{
    LogPanelKind, LogPanelState, LogSegment, SdsfLogEntry, SnapshotDestination, SnapshotResult,
};
pub use sdsf_overtype::{
    CommandLineOvertype, FieldKind, OvertypeField, OvertypeResult, OvertypeRow,
};
pub use sdsf_panel::{
    main_panel_commands, ActivePanel, CommandGroup, MainPanelCommand, ScrollAmount, SdsfPanelState,
};
pub use sdsf_set_p2::{DateFormat, SetP2Command, SetP2Result, SetP2Settings};
pub use sdsf_system_panels::{
    print_output, AddressSpace, BrowseSettings, ColsRuler, DashMetrics, InitPanelState,
    InitiatorEntry, JcPanelState, JobClassEntry, PrintDestination, PrintResult, RecordFormat,
    SpPanelState, SpoolVolume, SysPanelState,
};
