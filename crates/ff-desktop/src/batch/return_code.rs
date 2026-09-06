/// Step-level return code following z/OS MAXCC convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum StepReturnCode {
    #[default]
    Success = 0,
    Warning = 4,
    Error = 8,
    Severe = 12,
    Catastrophic = 16,
}

impl StepReturnCode {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

impl From<i32> for StepReturnCode {
    fn from(n: i32) -> Self {
        match n {
            0 => Self::Success,
            1..=4 => Self::Warning,
            5..=8 => Self::Error,
            9..=12 => Self::Severe,
            _ => Self::Catastrophic,
        }
    }
}

/// Accumulates the maximum StepReturnCode across all commands.
#[derive(Debug, Default)]
pub struct BatchReturnCode(pub StepReturnCode);

impl BatchReturnCode {
    pub fn update(&mut self, step: StepReturnCode) {
        if step > self.0 {
            self.0 = step;
        }
    }

    pub fn as_i32(&self) -> i32 {
        self.0.as_i32()
    }
}

/// Policy controlling whether to stop on error.
#[derive(Debug, Clone, Copy)]
pub enum AbortPolicy {
    /// Continue regardless of errors (default).
    BestEffort,
    /// Stop when any step RC >= threshold.
    AbortOnError(StepReturnCode),
}

#[allow(clippy::derivable_impls)] // AbortPolicy::BestEffort is a unit variant but the enum has a data variant
impl Default for AbortPolicy {
    fn default() -> Self {
        Self::BestEffort
    }
}

impl AbortPolicy {
    pub fn should_abort(&self, step: StepReturnCode) -> bool {
        match self {
            Self::BestEffort => false,
            Self::AbortOnError(threshold) => step >= *threshold,
        }
    }
}

// === Tests ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 5.2
    #[test]
    fn batch_return_code_is_maximum_step_rc() {
        let mut brc = BatchReturnCode::default();
        brc.update(StepReturnCode::Warning);
        brc.update(StepReturnCode::Error);
        brc.update(StepReturnCode::Warning);
        assert_eq!(brc.as_i32(), 8);
    }

    // Validates: Requirement 5.1
    #[test]
    fn all_success_gives_zero() {
        let mut brc = BatchReturnCode::default();
        brc.update(StepReturnCode::Success);
        brc.update(StepReturnCode::Success);
        assert_eq!(brc.as_i32(), 0);
    }

    // Validates: Requirement 5.3
    #[test]
    fn step_return_code_values_match_zos_convention() {
        assert_eq!(StepReturnCode::Success.as_i32(), 0);
        assert_eq!(StepReturnCode::Warning.as_i32(), 4);
        assert_eq!(StepReturnCode::Error.as_i32(), 8);
        assert_eq!(StepReturnCode::Severe.as_i32(), 12);
        assert_eq!(StepReturnCode::Catastrophic.as_i32(), 16);
    }

    // Validates: Requirement 6.1
    #[test]
    fn best_effort_never_aborts() {
        let policy = AbortPolicy::BestEffort;
        assert!(!policy.should_abort(StepReturnCode::Catastrophic));
    }

    // Validates: Requirement 6.2
    #[test]
    fn abort_on_error_triggers_at_threshold() {
        let policy = AbortPolicy::AbortOnError(StepReturnCode::Error);
        assert!(!policy.should_abort(StepReturnCode::Warning));
        assert!(policy.should_abort(StepReturnCode::Error));
        assert!(policy.should_abort(StepReturnCode::Severe));
    }
}
