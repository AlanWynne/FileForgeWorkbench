//! Work progress and status types.

/// Result returned by `IdleWorkSource::perform_work`.
///
/// Indicates the outcome of a single time slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkStatus {
    /// More work remains; the source should be serviced again on the next idle callback.
    MoreWork,
    /// All work is complete; the source transitions to dormant state.
    Complete,
    /// The work was interrupted by a cancellation signal. Progress has been saved.
    Interrupted,
}

/// Progress information for a single work source.
///
/// # Examples
///
/// ```
/// use ff_idle_processing::WorkProgress;
/// let p = WorkProgress::new(100);
/// assert_eq!(p.completed_units, 0);
/// assert_eq!(p.total_units, 100);
/// assert!(!p.is_complete);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkProgress {
    /// Amount of work completed (e.g., lines styled, lines measured).
    pub completed_units: u64,
    /// Total scope of work (e.g., total document lines).
    pub total_units: u64,
    /// Whether all work is finished.
    pub is_complete: bool,
}

impl WorkProgress {
    /// Create a new progress value indicating no work done.
    pub fn new(total_units: u64) -> Self {
        Self {
            completed_units: 0,
            total_units,
            is_complete: false,
        }
    }

    /// Create a completed progress value.
    pub fn completed(total_units: u64) -> Self {
        Self {
            completed_units: total_units,
            total_units,
            is_complete: true,
        }
    }

    /// Fraction complete in [0.0, 1.0].
    pub fn fraction(&self) -> f64 {
        if self.total_units == 0 {
            1.0
        } else {
            self.completed_units as f64 / self.total_units as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_progress_starts_at_zero() {
        // Validates: Requirement 6 AC 1
        let p = WorkProgress::new(500);
        assert_eq!(p.completed_units, 0);
        assert_eq!(p.total_units, 500);
        assert!(!p.is_complete);
    }

    #[test]
    fn completed_progress_is_done() {
        let p = WorkProgress::completed(100);
        assert_eq!(p.completed_units, 100);
        assert!(p.is_complete);
    }

    #[test]
    fn fraction_zero_total_returns_one() {
        let p = WorkProgress::new(0);
        assert_eq!(p.fraction(), 1.0);
    }

    #[test]
    fn fraction_half_complete() {
        let p = WorkProgress {
            completed_units: 50,
            total_units: 100,
            is_complete: false,
        };
        assert!((p.fraction() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn work_status_variants_distinct() {
        assert_ne!(WorkStatus::MoreWork, WorkStatus::Complete);
        assert_ne!(WorkStatus::Complete, WorkStatus::Interrupted);
        assert_ne!(WorkStatus::MoreWork, WorkStatus::Interrupted);
    }
}
