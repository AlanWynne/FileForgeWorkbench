//! Coalescing engine — merges rapid consecutive edits into single transactions.
//!
//! Consecutive single-character inserts or deletes that are contiguous in the
//! document are merged into one transaction. Boundary events (cursor move,
//! op type change, timeout, save, explicit begin/end) break the coalescing.

use std::time::Instant;

/// Operation types for coalescing boundary detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoalesceOpType {
    /// Single character insert.
    CharInsert,
    /// Single character delete (backspace — position moves backward).
    CharBackspace,
    /// Single character delete (delete key — position stays).
    CharDelete,
}

/// Tracks the current coalescing state for a document session.
///
/// Evaluates whether consecutive single-char operations should merge into the
/// current in-progress transaction or start a new one.
#[derive(Debug)]
pub struct CoalesceState {
    /// Whether coalescing is currently active (accumulating into existing txn).
    active: bool,
    /// The type of the last operation (for type-change detection).
    last_op_type: Option<CoalesceOpType>,
    /// The end position of the last operation (for contiguity detection).
    last_end_position: Option<u64>,
    /// Timestamp of the last operation (for timeout detection).
    last_timestamp: Option<Instant>,
    /// Whether the current in-progress transaction is marked may_coalesce.
    may_coalesce: bool,
    /// Whether we are inside an explicit begin/end group (overrides char rules).
    in_explicit_group: bool,
    /// Configured coalesce timeout in milliseconds.
    timeout_ms: u32,
}

impl CoalesceState {
    /// Creates a new coalescing state with the given timeout.
    pub fn new(timeout_ms: u32) -> Self {
        Self {
            active: false,
            last_op_type: None,
            last_end_position: None,
            last_timestamp: None,
            may_coalesce: true,
            in_explicit_group: false,
            timeout_ms,
        }
    }

    /// Checks whether a new operation should coalesce with the current transaction.
    ///
    /// Returns `true` if the operation should be merged into the current transaction,
    /// `false` if a new transaction boundary should be created.
    pub fn should_coalesce(
        &self,
        op_type: CoalesceOpType,
        position: u64,
        length: u32,
        new_may_coalesce: bool,
    ) -> bool {
        // If inside explicit group, always coalesce
        if self.in_explicit_group {
            return true;
        }

        // If coalescing is not active, can't coalesce
        if !self.active {
            return false;
        }

        // If either side has may_coalesce=false, don't coalesce
        if !self.may_coalesce || !new_may_coalesce {
            return false;
        }

        // Check timeout
        if let Some(last_time) = self.last_timestamp {
            let elapsed = last_time.elapsed().as_millis() as u32;
            if elapsed > self.timeout_ms {
                return false;
            }
        }

        // Must be single-char (1 or 2 bytes for multi-byte chars)
        if length > 2 {
            return false;
        }

        // Operation type must match
        if self.last_op_type != Some(op_type) {
            return false;
        }

        // Check contiguity based on op type
        if let Some(last_end) = self.last_end_position {
            match op_type {
                CoalesceOpType::CharInsert => {
                    // New insert must be immediately after last insert's end
                    position == last_end
                }
                CoalesceOpType::CharBackspace => {
                    // Backspace: new position + length == previous position
                    position + u64::from(length) == last_end
                }
                CoalesceOpType::CharDelete => {
                    // Delete key: same position (chars consumed at fixed point)
                    position == last_end
                }
            }
        } else {
            false
        }
    }

    /// Updates state after an operation is recorded (either new txn or coalesced).
    pub fn record_operation(
        &mut self,
        op_type: CoalesceOpType,
        position: u64,
        length: u32,
        may_coalesce: bool,
    ) {
        self.active = true;
        self.last_op_type = Some(op_type);
        self.may_coalesce = may_coalesce;
        self.last_timestamp = Some(Instant::now());

        // Set the "end position" based on operation type
        match op_type {
            CoalesceOpType::CharInsert => {
                self.last_end_position = Some(position + u64::from(length));
            }
            CoalesceOpType::CharBackspace => {
                // For backspace, next backspace should be at position (one char before)
                self.last_end_position = Some(position);
            }
            CoalesceOpType::CharDelete => {
                // For delete, next delete is at the same position
                self.last_end_position = Some(position);
            }
        }
    }

    /// Breaks the current coalescing window (boundary event occurred).
    pub fn break_coalesce(&mut self) {
        self.active = false;
        self.last_op_type = None;
        self.last_end_position = None;
        self.last_timestamp = None;
    }

    /// Notifies that the coalesce timeout has elapsed.
    pub fn timeout_expired(&mut self) {
        self.break_coalesce();
    }

    /// Enters an explicit transaction group (overrides char-level rules).
    pub fn enter_explicit_group(&mut self) {
        self.in_explicit_group = true;
    }

    /// Exits an explicit transaction group.
    pub fn exit_explicit_group(&mut self) {
        self.in_explicit_group = false;
        self.break_coalesce();
    }

    /// Returns whether coalescing is currently active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Returns whether we are inside an explicit group.
    pub fn in_explicit_group(&self) -> bool {
        self.in_explicit_group
    }

    /// Resets all state (for delete_history).
    pub fn reset(&mut self) {
        self.active = false;
        self.last_op_type = None;
        self.last_end_position = None;
        self.last_timestamp = None;
        self.may_coalesce = true;
        self.in_explicit_group = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn new_state_is_not_active() {
        let state = CoalesceState::new(2000);
        assert!(!state.is_active());
    }

    #[test]
    fn should_not_coalesce_when_inactive() {
        let state = CoalesceState::new(2000);
        assert!(!state.should_coalesce(CoalesceOpType::CharInsert, 0, 1, true));
    }

    #[test]
    fn contiguous_inserts_coalesce() {
        let mut state = CoalesceState::new(2000);
        state.record_operation(CoalesceOpType::CharInsert, 0, 1, true);
        assert!(state.should_coalesce(CoalesceOpType::CharInsert, 1, 1, true));
    }

    #[test]
    fn non_contiguous_inserts_do_not_coalesce() {
        let mut state = CoalesceState::new(2000);
        state.record_operation(CoalesceOpType::CharInsert, 0, 1, true);
        // Gap: next insert at position 5 instead of 1
        assert!(!state.should_coalesce(CoalesceOpType::CharInsert, 5, 1, true));
    }

    #[test]
    fn contiguous_backspace_coalesces() {
        let mut state = CoalesceState::new(2000);
        // Delete char at position 5 (backspace from pos 6)
        state.record_operation(CoalesceOpType::CharBackspace, 5, 1, true);
        // Next backspace should be at position 4
        assert!(state.should_coalesce(CoalesceOpType::CharBackspace, 4, 1, true));
    }

    #[test]
    fn contiguous_delete_key_coalesces() {
        let mut state = CoalesceState::new(2000);
        // Delete key at position 5
        state.record_operation(CoalesceOpType::CharDelete, 5, 1, true);
        // Next delete at same position
        assert!(state.should_coalesce(CoalesceOpType::CharDelete, 5, 1, true));
    }

    #[test]
    fn op_type_change_breaks_coalescing() {
        let mut state = CoalesceState::new(2000);
        state.record_operation(CoalesceOpType::CharInsert, 0, 1, true);
        // Different op type
        assert!(!state.should_coalesce(CoalesceOpType::CharDelete, 1, 1, true));
    }

    #[test]
    fn multi_char_operation_does_not_coalesce() {
        let mut state = CoalesceState::new(2000);
        state.record_operation(CoalesceOpType::CharInsert, 0, 1, true);
        // 5-byte insert (paste, not single char)
        assert!(!state.should_coalesce(CoalesceOpType::CharInsert, 1, 5, true));
    }

    #[test]
    fn may_coalesce_false_breaks_coalescing() {
        let mut state = CoalesceState::new(2000);
        state.record_operation(CoalesceOpType::CharInsert, 0, 1, true);
        // New op has may_coalesce=false
        assert!(!state.should_coalesce(CoalesceOpType::CharInsert, 1, 1, false));
    }

    #[test]
    fn previous_may_coalesce_false_breaks_coalescing() {
        let mut state = CoalesceState::new(2000);
        state.record_operation(CoalesceOpType::CharInsert, 0, 1, false);
        assert!(!state.should_coalesce(CoalesceOpType::CharInsert, 1, 1, true));
    }

    #[test]
    fn break_coalesce_resets_state() {
        let mut state = CoalesceState::new(2000);
        state.record_operation(CoalesceOpType::CharInsert, 0, 1, true);
        state.break_coalesce();
        assert!(!state.is_active());
        assert!(!state.should_coalesce(CoalesceOpType::CharInsert, 1, 1, true));
    }

    #[test]
    fn explicit_group_always_coalesces() {
        let mut state = CoalesceState::new(2000);
        state.enter_explicit_group();
        // Even without prior state, explicit group coalesces
        assert!(state.should_coalesce(CoalesceOpType::CharInsert, 100, 50, true));
    }

    #[test]
    fn timeout_breaks_coalescing() {
        let mut state = CoalesceState::new(50); // 50ms timeout
        state.record_operation(CoalesceOpType::CharInsert, 0, 1, true);
        thread::sleep(Duration::from_millis(60));
        assert!(!state.should_coalesce(CoalesceOpType::CharInsert, 1, 1, true));
    }

    #[test]
    fn two_byte_char_insert_can_coalesce() {
        let mut state = CoalesceState::new(2000);
        state.record_operation(CoalesceOpType::CharInsert, 0, 2, true);
        assert!(state.should_coalesce(CoalesceOpType::CharInsert, 2, 1, true));
    }
}
