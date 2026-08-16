//! # ff-dsalloc — Dataset Allocator for FileForgeWorkbench
//!
//! This crate is the **desktop equivalent of z/OS Dynamic Allocation (DYNALLOC / SVC 99)**.
//! It parses JCL DD statements, resolves dataset names against locally mounted catalogs,
//! performs symbolic parameter substitution, simulates dataset allocation, handles GDG
//! relative generation references, resolves referback chains, validates JCL for common
//! errors, and exposes a `dataset.resolve` command for interactive DSN-to-path tracing.
//!
//! ## Architecture
//!
//! The resolver processes JCL through four ordered stages:
//!
//! ```text
//! ┌───────────┐    ┌─────────────┐    ┌───────────┐    ┌────────────┐
//! │  1. Parse │───▶│ 2. Substitute│───▶│ 3. Resolve│───▶│ 4. Validate│
//! │           │    │             │    │           │    │            │
//! │ JCL text  │    │ Symbol table│    │ Catalog   │    │ Lint rules │
//! │ → Job     │    │ → Expanded  │    │ → Paths   │    │ → Diags    │
//! │   model   │    │   operands  │    │           │    │            │
//! └───────────┘    └─────────────┘    └───────────┘    └────────────┘
//! ```
//!
//! ## Position in Architecture
//!
//! `ff-dsalloc` is a **Wave 13 (Dataset Catalog and Mainframe Emulation)** crate.
//! It depends on `ff-dataset-catalog` for DSN resolution and uses trait-based
//! abstractions for catalog access and language service queries.

// ─── Public Modules ─────────────────────────────────────────────────────────

/// Error types for the dataset allocator.
pub mod error;

/// Configuration model for the resolver.
pub mod config;

/// Lint diagnostic types and codes.
pub mod diagnostic;

/// Dataset name model and validation.
pub mod dsn;

/// DD statement operand models (DISP, DCB, SPACE).
pub mod operands;

/// DD statement model.
pub mod dd_statement;

/// JCL parser (DD, JOB, EXEC, continuation handling).
pub mod parser;

/// Job structure model (JclJob, JclStep).
pub mod job_model;

/// Symbol table and symbolic substitution engine.
pub mod symbols;

/// Catalog resolution bridge (trait + mock).
pub mod catalog_bridge;

/// DISP interpretation and allocation simulation.
pub mod allocation;

/// Concatenation group handling.
pub mod concatenation;

/// Temporary dataset registry.
pub mod temp_registry;

/// Referback resolution.
pub mod referback;

/// GDG relative generation resolver.
pub mod gdg_resolver;

/// Resolution processing pipeline.
pub mod pipeline;

/// JCL validation and lint diagnostic emitter.
pub mod lint;

/// RESOLVE command handler.
pub mod command;

/// Resolution output panel model.
pub mod panel;

/// Public trait interface for the allocator service.
pub mod traits;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use catalog_bridge::{CatalogError, CatalogMatch, CatalogProvider, GdgGeneration, GdgInfo};
pub use config::{ResolveMode, ResolverConfig};
pub use dd_statement::DdStatement;
pub use diagnostic::{DiagnosticCode, DiagnosticSeverity, LintDiagnostic};
pub use dsn::{DatasetName, DsnReference};
pub use error::JclResolverError;
pub use job_model::{ExecTarget, JclJob, JclStep};
pub use operands::{
    DcbAttributes, DispAction, DispParameter, DispStatus, DsOrg, SpaceAllocation, SpaceUnit,
};
pub use panel::ResolutionPanelModel;
pub use pipeline::{DatasetType, ResolutionOutcome, ResolutionResult, SkipReason};
pub use pipeline::{PipelineState, ResolveOutput, ResolveSummary, StageTiming};
pub use symbols::{substitute_symbols, SymbolTable};
pub use traits::AllocatorService;
