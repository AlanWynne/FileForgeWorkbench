//! # ff-compare-merge — Compare and Merge Engine
//!
//! GUI-independent diff engine and merge logic for FileForgeWorkbench.
//! Provides Myers and Patience diff algorithms, inline change detection,
//! diff statistics, navigation, two-way and three-way merge, binary
//! comparison, and unified diff export.
//!
//! All rendering and VFS I/O are handled by the shell layer; this crate
//! owns only the pure computation.

pub mod binary;
pub mod error;
pub mod export;
pub mod merge;
pub mod navigator;
pub mod options;
pub mod result;
pub mod session;

pub use binary::{BinaryComparator, BinaryCompareResult};
pub use error::CompareError;
pub use export::DiffExporter;
pub use merge::{ConflictResolution, MergeConflict, MergeResolver, ThreeWayMerge, ThreeWayRegion};
pub use navigator::DiffNavigator;
pub use navigator::{CompareSession, CompareSource, SessionId};
pub use options::{CompareOptions, DiffAlgorithm, ViewMode, WhitespaceMode};
pub use result::{DiffHunk, DiffResult, DiffStatistics, InlineChange};
