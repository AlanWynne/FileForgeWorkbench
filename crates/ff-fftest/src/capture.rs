//! FFTest screenshot capture and visual regression.
//!
//! Screenshot capture from a live eframe window requires texture readback
//! which is only available inside the egui render loop. This module provides:
//!
//! - [`CaptureStub`] -- a no-op capture backend used in headless/test mode
//! - [`BaselineStore`] -- manages baseline PNG files under `tests/baselines/`
//! - [`compare_pixels`] -- pixel-level diff with configurable tolerance
//! - [`CheckpointOutcome`] -- result of a CHECKPOINT command
//!
//! Full eframe texture readback is deferred to a future phase when the
//! egui offscreen rendering API stabilises.
//!
//! Validates: Requirement 7.5, 8.1, 8.2, 8.4, 8.5 (automated-dialog-testing)

use std::path::PathBuf;

// === CheckpointOutcome ======================================================

/// The result of executing a CHECKPOINT command.
///
/// Validates: Requirement 8.5
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckpointOutcome {
    /// Screenshot matched the baseline within tolerance.
    Pass,
    /// Screenshot differed from the baseline beyond tolerance.
    VisualRegressionFail {
        /// Number of pixels that exceeded the tolerance threshold.
        differing_pixels: usize,
        /// Configured tolerance (max pixels allowed to differ).
        tolerance: usize,
    },
    /// No baseline existed; one was created from the current screenshot.
    BaselineCreated,
    /// Screenshot capture is not available in this execution mode.
    CaptureUnavailable,
}

// === PixelBuffer ============================================================

/// A minimal RGBA pixel buffer used for visual regression comparison.
///
/// In headless/test mode this is populated with zeros (blank frame).
/// In a live eframe session it would be populated via texture readback.
///
/// Validates: Requirement 8.1
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixelBuffer {
    pub width: u32,
    pub height: u32,
    /// Raw RGBA bytes, length == width * height * 4.
    pub data: Vec<u8>,
}

impl PixelBuffer {
    /// Create a blank (all-zero) pixel buffer of the given dimensions.
    pub fn blank(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0u8; (width * height * 4) as usize],
        }
    }

    /// Returns true if this buffer has the same dimensions as `other`.
    pub fn same_dimensions(&self, other: &PixelBuffer) -> bool {
        self.width == other.width && self.height == other.height
    }
}

// === compare_pixels =========================================================

/// Compare two pixel buffers and return the number of pixels that differ by
/// more than `tolerance` intensity units in any channel.
///
/// Returns `None` if the buffers have different dimensions.
///
/// Validates: Requirement 8.2
pub fn compare_pixels(a: &PixelBuffer, b: &PixelBuffer, tolerance: u8) -> Option<usize> {
    if !a.same_dimensions(b) {
        return None;
    }
    let differing = a
        .data
        .chunks(4)
        .zip(b.data.chunks(4))
        .filter(|(pa, pb)| {
            pa.iter()
                .zip(pb.iter())
                .any(|(&x, &y)| x.abs_diff(y) > tolerance)
        })
        .count();
    Some(differing)
}

// === BaselineStore ==========================================================

/// Manages baseline PNG files stored under a configurable root directory.
///
/// The default root is `tests/baselines/` relative to the workspace root.
///
/// Validates: Requirement 8.1, 8.4, 8.5
pub struct BaselineStore {
    root: PathBuf,
}

impl BaselineStore {
    /// Create a store rooted at `root`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Return the path for a named checkpoint baseline PNG.
    pub fn baseline_path(&self, checkpoint_name: &str) -> PathBuf {
        let safe_name =
            checkpoint_name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        self.root.join(format!("{safe_name}.png"))
    }

    /// Returns true if a baseline exists for the given checkpoint name.
    pub fn baseline_exists(&self, checkpoint_name: &str) -> bool {
        self.baseline_path(checkpoint_name).exists()
    }

    /// Write `data` as a raw binary file at the baseline path.
    ///
    /// Creates parent directories if they do not exist.
    /// In production this would write a PNG; here we write raw bytes for
    /// testability without a PNG encoder dependency.
    ///
    /// Validates: Requirement 8.5
    pub fn write_baseline(&self, checkpoint_name: &str, data: &[u8]) -> std::io::Result<()> {
        let path = self.baseline_path(checkpoint_name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, data)
    }

    /// Read the baseline bytes for a checkpoint.
    ///
    /// Returns `None` if the baseline does not exist.
    pub fn read_baseline(&self, checkpoint_name: &str) -> Option<Vec<u8>> {
        let path = self.baseline_path(checkpoint_name);
        std::fs::read(&path).ok()
    }

    /// Delete all baseline files under the store root.
    ///
    /// Used by `--update-baselines` to force re-creation on next run.
    ///
    /// Validates: Requirement 8.4
    pub fn clear_all(&self) -> std::io::Result<()> {
        if !self.root.exists() {
            return Ok(());
        }
        for entry in std::fs::read_dir(&self.root)? {
            let entry = entry?;
            if entry
                .path()
                .extension()
                .map(|e| e == "png")
                .unwrap_or(false)
            {
                std::fs::remove_file(entry.path())?;
            }
        }
        Ok(())
    }
}

// === CaptureStub ============================================================

/// A no-op capture backend for headless and unit-test execution.
///
/// Always returns a blank [`PixelBuffer`]. In a live eframe session this
/// would be replaced by a real texture-readback implementation.
///
/// Validates: Requirement 7.5, 8.1
pub struct CaptureStub {
    width: u32,
    height: u32,
}

impl CaptureStub {
    /// Create a stub that produces blank frames of the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Capture a blank frame (stub implementation).
    pub fn capture(&self) -> PixelBuffer {
        PixelBuffer::blank(self.width, self.height)
    }
}

// === run_checkpoint =========================================================

/// Execute a CHECKPOINT: capture a frame, compare against baseline (or create it).
///
/// - If no baseline exists: writes one and returns [`CheckpointOutcome::BaselineCreated`].
/// - If a baseline exists: compares with `tolerance` and returns Pass or Fail.
///
/// Validates: Requirement 8.2, 8.5
pub fn run_checkpoint(
    checkpoint_name: &str,
    capture: &CaptureStub,
    store: &BaselineStore,
    tolerance: u8,
) -> CheckpointOutcome {
    let frame = capture.capture();

    if !store.baseline_exists(checkpoint_name) {
        // Auto-create baseline on first run (Req 8.5)
        if store.write_baseline(checkpoint_name, &frame.data).is_err() {
            return CheckpointOutcome::CaptureUnavailable;
        }
        return CheckpointOutcome::BaselineCreated;
    }

    let baseline_bytes = match store.read_baseline(checkpoint_name) {
        Some(b) => b,
        None => return CheckpointOutcome::CaptureUnavailable,
    };

    // Reconstruct a PixelBuffer from stored bytes for comparison.
    let baseline = PixelBuffer {
        width: frame.width,
        height: frame.height,
        data: baseline_bytes,
    };

    match compare_pixels(&frame, &baseline, tolerance) {
        Some(0) => CheckpointOutcome::Pass,
        Some(n) => CheckpointOutcome::VisualRegressionFail {
            differing_pixels: n,
            tolerance: tolerance as usize,
        },
        None => CheckpointOutcome::CaptureUnavailable,
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Validates: Requirement 8.1 -- blank pixel buffer has correct size
    #[test]
    fn blank_pixel_buffer_has_correct_byte_count() {
        let buf = PixelBuffer::blank(4, 4);
        assert_eq!(buf.data.len(), 4 * 4 * 4);
        assert!(buf.data.iter().all(|&b| b == 0));
    }

    // Validates: Requirement 8.2 -- identical buffers produce zero differing pixels
    #[test]
    fn compare_identical_buffers_returns_zero() {
        let a = PixelBuffer::blank(2, 2);
        let b = PixelBuffer::blank(2, 2);
        assert_eq!(compare_pixels(&a, &b, 0), Some(0));
    }

    // Validates: Requirement 8.2 -- differing pixels counted correctly
    #[test]
    fn compare_different_buffers_counts_differing_pixels() {
        let a = PixelBuffer::blank(2, 2);
        let mut b = PixelBuffer::blank(2, 2);
        // Make pixel 0 differ by 10 in the red channel
        b.data[0] = 10;
        assert_eq!(compare_pixels(&a, &b, 5), Some(1));
    }

    // Validates: Requirement 8.2 -- tolerance allows small differences
    #[test]
    fn compare_within_tolerance_returns_zero() {
        let a = PixelBuffer::blank(2, 2);
        let mut b = PixelBuffer::blank(2, 2);
        b.data[0] = 3; // differs by 3, tolerance is 5
        assert_eq!(compare_pixels(&a, &b, 5), Some(0));
    }

    // Validates: Requirement 8.2 -- mismatched dimensions return None
    #[test]
    fn compare_different_dimensions_returns_none() {
        let a = PixelBuffer::blank(2, 2);
        let b = PixelBuffer::blank(3, 3);
        assert!(compare_pixels(&a, &b, 0).is_none());
    }

    // Validates: Requirement 8.5 -- baseline auto-created on first run
    #[test]
    fn checkpoint_creates_baseline_on_first_run() {
        let dir = TempDir::new().expect("tempdir");
        let store = BaselineStore::new(dir.path());
        let capture = CaptureStub::new(4, 4);
        let outcome = run_checkpoint("first_run", &capture, &store, 5);
        assert_eq!(outcome, CheckpointOutcome::BaselineCreated);
        assert!(store.baseline_exists("first_run"));
    }

    // Validates: Requirement 8.2 -- identical frame passes comparison
    #[test]
    fn checkpoint_passes_when_frame_matches_baseline() {
        let dir = TempDir::new().expect("tempdir");
        let store = BaselineStore::new(dir.path());
        let capture = CaptureStub::new(4, 4);
        // First run creates baseline
        run_checkpoint("match_test", &capture, &store, 5);
        // Second run compares -- blank vs blank = pass
        let outcome = run_checkpoint("match_test", &capture, &store, 5);
        assert_eq!(outcome, CheckpointOutcome::Pass);
    }

    // Validates: Requirement 8.4 -- clear_all removes baseline files
    #[test]
    fn clear_all_removes_baseline_files() {
        let dir = TempDir::new().expect("tempdir");
        let store = BaselineStore::new(dir.path());
        let capture = CaptureStub::new(2, 2);
        run_checkpoint("to_clear", &capture, &store, 0);
        assert!(store.baseline_exists("to_clear"));
        store.clear_all().expect("clear ok");
        assert!(!store.baseline_exists("to_clear"));
    }

    // Validates: Requirement 8.5 -- baseline_path sanitises unsafe characters
    #[test]
    fn baseline_path_sanitises_unsafe_characters() {
        let store = BaselineStore::new("/tmp/baselines");
        let path = store.baseline_path("a/b:c*d");
        let name = path.file_name().unwrap().to_string_lossy();
        assert!(!name.contains('/'));
        assert!(!name.contains(':'));
        assert!(!name.contains('*'));
    }
}
