//! Shared types for the background I/O subsystem.
//!
//! Defines [`ChunkSize`], [`LargeFileThreshold`], [`TaskId`], [`TaskState`],
//! [`IoSuccess`], [`IoTaskType`], and related types used throughout the crate.

use std::time::Duration;

use ff_vfs::ResourceUri;

/// A validated chunk size (4 KB – 1 MB). Values outside range are clamped.
///
/// The chunk size determines the granularity of streaming read and write
/// operations. Smaller chunks provide more responsive progress updates but
/// increase system call overhead.
///
/// # Examples
///
/// ```
/// use ff_background_io::ChunkSize;
///
/// let size = ChunkSize::new(128 * 1024); // 128 KB
/// assert_eq!(size.as_bytes(), 128 * 1024);
///
/// let clamped = ChunkSize::new(1); // Below minimum, clamped to 4 KB
/// assert_eq!(clamped.as_bytes(), ChunkSize::MIN);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChunkSize(u32);

impl ChunkSize {
    /// Minimum chunk size: 4 KB.
    pub const MIN: u32 = 4 * 1024;
    /// Maximum chunk size: 1 MB.
    pub const MAX: u32 = 1024 * 1024;
    /// Default chunk size: 64 KB.
    pub const DEFAULT: u32 = 64 * 1024;

    /// Create a ChunkSize, clamping to the valid range [4 KB, 1 MB].
    pub fn new(bytes: u32) -> Self {
        Self(bytes.clamp(Self::MIN, Self::MAX))
    }

    /// Get the size in bytes.
    pub fn as_bytes(&self) -> u32 {
        self.0
    }
}

impl Default for ChunkSize {
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}

/// A validated large-file threshold (10 MB – 4096 MB). Values outside range are clamped.
///
/// Files exceeding this threshold are loaded in streaming-only mode, where
/// the LoadTask never buffers more than 2× chunk_size of data at any time.
///
/// # Examples
///
/// ```
/// use ff_background_io::LargeFileThreshold;
///
/// let threshold = LargeFileThreshold::new(200 * 1024 * 1024); // 200 MB
/// assert_eq!(threshold.as_bytes(), 200 * 1024 * 1024);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LargeFileThreshold(u64);

impl LargeFileThreshold {
    /// Minimum threshold: 10 MB.
    pub const MIN: u64 = 10 * 1024 * 1024;
    /// Maximum threshold: 4096 MB.
    pub const MAX: u64 = 4096 * 1024 * 1024;
    /// Default threshold: 100 MB.
    pub const DEFAULT: u64 = 100 * 1024 * 1024;

    /// Create a LargeFileThreshold, clamping to the valid range [10 MB, 4096 MB].
    pub fn new(bytes: u64) -> Self {
        Self(bytes.clamp(Self::MIN, Self::MAX))
    }

    /// Get the threshold in bytes.
    pub fn as_bytes(&self) -> u64 {
        self.0
    }
}

impl Default for LargeFileThreshold {
    fn default() -> Self {
        Self(Self::DEFAULT)
    }
}

/// Unique identifier for a background I/O task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(u64);

impl TaskId {
    /// Create a new TaskId with the given value.
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw numeric value.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The lifecycle state of an I/O task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TaskState {
    /// Waiting in the queue for a concurrency slot.
    Queued,
    /// Currently executing.
    InProgress,
    /// Completed successfully.
    Complete,
    /// Failed with an error.
    Failed,
    /// Cancelled by the user or system.
    Cancelled,
}

/// Successful completion of an I/O task.
#[derive(Debug, Clone)]
pub struct IoSuccess {
    /// Total bytes transferred.
    pub bytes_transferred: u64,
    /// Total elapsed time.
    pub elapsed: Duration,
    /// Resource URI that was operated on.
    pub uri: ResourceUri,
}

/// Discriminator for load vs save tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoTaskType {
    /// A file load operation.
    Load,
    /// A file save operation.
    Save,
}

/// An entry in the task list for the task manager UI.
#[derive(Debug, Clone)]
pub struct IoTaskEntry {
    /// Unique task identifier.
    pub id: TaskId,
    /// Resource URI for the task.
    pub uri: String,
    /// Whether this is a load or save task.
    pub task_type: IoTaskType,
    /// Current lifecycle state.
    pub state: TaskState,
    /// Latest progress state.
    pub progress: crate::progress::ProgressState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_size_default_is_64kb() {
        let size = ChunkSize::default();
        assert_eq!(size.as_bytes(), 64 * 1024);
    }

    #[test]
    fn chunk_size_clamps_below_minimum() {
        let size = ChunkSize::new(0);
        assert_eq!(size.as_bytes(), ChunkSize::MIN);

        let size = ChunkSize::new(1);
        assert_eq!(size.as_bytes(), ChunkSize::MIN);

        let size = ChunkSize::new(4095);
        assert_eq!(size.as_bytes(), ChunkSize::MIN);
    }

    #[test]
    fn chunk_size_clamps_above_maximum() {
        let size = ChunkSize::new(2_000_000);
        assert_eq!(size.as_bytes(), ChunkSize::MAX);

        let size = ChunkSize::new(u32::MAX);
        assert_eq!(size.as_bytes(), ChunkSize::MAX);
    }

    #[test]
    fn chunk_size_accepts_values_in_range() {
        let size = ChunkSize::new(ChunkSize::MIN);
        assert_eq!(size.as_bytes(), ChunkSize::MIN);

        let size = ChunkSize::new(ChunkSize::MAX);
        assert_eq!(size.as_bytes(), ChunkSize::MAX);

        let size = ChunkSize::new(128 * 1024);
        assert_eq!(size.as_bytes(), 128 * 1024);
    }

    #[test]
    fn large_file_threshold_default_is_100mb() {
        let threshold = LargeFileThreshold::default();
        assert_eq!(threshold.as_bytes(), 100 * 1024 * 1024);
    }

    #[test]
    fn large_file_threshold_clamps_below_minimum() {
        let threshold = LargeFileThreshold::new(0);
        assert_eq!(threshold.as_bytes(), LargeFileThreshold::MIN);

        let threshold = LargeFileThreshold::new(1024);
        assert_eq!(threshold.as_bytes(), LargeFileThreshold::MIN);
    }

    #[test]
    fn large_file_threshold_clamps_above_maximum() {
        let threshold = LargeFileThreshold::new(u64::MAX);
        assert_eq!(threshold.as_bytes(), LargeFileThreshold::MAX);
    }

    #[test]
    fn large_file_threshold_accepts_values_in_range() {
        let threshold = LargeFileThreshold::new(LargeFileThreshold::MIN);
        assert_eq!(threshold.as_bytes(), LargeFileThreshold::MIN);

        let threshold = LargeFileThreshold::new(LargeFileThreshold::MAX);
        assert_eq!(threshold.as_bytes(), LargeFileThreshold::MAX);

        let threshold = LargeFileThreshold::new(200 * 1024 * 1024);
        assert_eq!(threshold.as_bytes(), 200 * 1024 * 1024);
    }

    #[test]
    fn task_id_display_shows_numeric_value() {
        let id = TaskId::new(42);
        assert_eq!(id.to_string(), "42");
        assert_eq!(id.as_u64(), 42);
    }

    #[test]
    fn task_state_variants_are_distinct() {
        assert_ne!(TaskState::Queued, TaskState::InProgress);
        assert_ne!(TaskState::InProgress, TaskState::Complete);
        assert_ne!(TaskState::Complete, TaskState::Failed);
        assert_ne!(TaskState::Failed, TaskState::Cancelled);
    }

    #[test]
    fn io_task_type_variants_are_distinct() {
        assert_ne!(IoTaskType::Load, IoTaskType::Save);
    }
}
