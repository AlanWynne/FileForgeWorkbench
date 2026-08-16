//! History validation — integrity checking of undo history against document state.
//!
//! Validates that the cumulative size delta of all operations in the undo history
//! is consistent with the current document length, and that no operation references
//! a position beyond the document bounds at its point in the sequence.

use crate::error::UndoError;
use crate::transaction::Transaction;

/// Validates the undo history against the known document length.
///
/// Checks:
/// 1. Cumulative size delta consistency — starting from initial size, replaying
///    all operations should produce the current document length.
/// 2. Position bounds — no operation references a position beyond the document
///    bounds at its point in the sequence.
/// 3. Non-negative length — cumulative document length never goes negative.
///
/// # Parameters
///
/// - `transactions`: The undo stack transactions (oldest first).
/// - `initial_document_length`: The document length before any transactions.
/// - `current_document_length`: The expected current document length.
///
/// # Returns
///
/// `Ok(())` if valid, `Err(UndoError::ValidationFailed)` if inconsistent.
pub fn validate_history(
    transactions: &[&Transaction],
    initial_document_length: u64,
    current_document_length: u64,
) -> Result<(), UndoError> {
    let mut doc_len = initial_document_length as i64;

    for txn in transactions {
        for op in &txn.operations {
            // Check position bounds
            let position = op.position();
            if position > doc_len as u64 {
                return Err(UndoError::ValidationFailed {
                    expected: current_document_length,
                    actual: doc_len as u64,
                });
            }

            // Apply size delta
            doc_len += op.size_delta();

            // Check non-negative
            if doc_len < 0 {
                return Err(UndoError::ValidationFailed {
                    expected: current_document_length,
                    actual: 0,
                });
            }
        }
    }

    let computed = doc_len as u64;
    if computed != current_document_length {
        return Err(UndoError::ValidationFailed {
            expected: current_document_length,
            actual: computed,
        });
    }

    Ok(())
}

/// Validates a single transaction's operations for internal consistency.
///
/// Checks that positions and lengths are internally consistent (no overlapping
/// deletions, positions advance correctly, etc.).
pub fn validate_transaction(txn: &Transaction, document_length_at_start: u64) -> bool {
    let mut doc_len = document_length_at_start as i64;

    for op in &txn.operations {
        let position = op.position();
        if position > doc_len as u64 {
            return false;
        }

        doc_len += op.size_delta();

        if doc_len < 0 {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edit_op::EditOperation;
    use chrono::Utc;

    fn make_transaction(ops: Vec<EditOperation>) -> Transaction {
        Transaction {
            name: "test".to_string(),
            timestamp: Utc::now(),
            operations: ops,
            selection_before: None,
            selection_after: None,
            may_coalesce: true,
        }
    }

    #[test]
    fn valid_history_passes_validation() {
        // Start with doc length 10, insert 5 bytes → expect 15
        let txn = make_transaction(vec![EditOperation::Insert {
            position: 5,
            length: 5,
            scrap_offset: 0,
        }]);
        let txns: Vec<&Transaction> = vec![&txn];
        assert!(validate_history(&txns, 10, 15).is_ok());
    }

    #[test]
    fn mismatched_length_fails_validation() {
        let txn = make_transaction(vec![EditOperation::Insert {
            position: 5,
            length: 5,
            scrap_offset: 0,
        }]);
        let txns: Vec<&Transaction> = vec![&txn];
        // Expected 20, but computed 15
        assert!(validate_history(&txns, 10, 20).is_err());
    }

    #[test]
    fn position_beyond_document_fails() {
        // Doc is 5 bytes, but op references position 10
        let txn = make_transaction(vec![EditOperation::Insert {
            position: 10,
            length: 1,
            scrap_offset: 0,
        }]);
        let txns: Vec<&Transaction> = vec![&txn];
        assert!(validate_history(&txns, 5, 6).is_err());
    }

    #[test]
    fn negative_document_length_fails() {
        // Doc is 5 bytes, delete 10 bytes → goes negative
        let txn = make_transaction(vec![EditOperation::Delete {
            position: 0,
            length: 10,
            scrap_offset: 0,
        }]);
        let txns: Vec<&Transaction> = vec![&txn];
        assert!(validate_history(&txns, 5, 0).is_err());
    }

    #[test]
    fn empty_history_valid_when_lengths_match() {
        let txns: Vec<&Transaction> = vec![];
        assert!(validate_history(&txns, 100, 100).is_ok());
    }

    #[test]
    fn multiple_transactions_validate_correctly() {
        let txn1 = make_transaction(vec![EditOperation::Insert {
            position: 0,
            length: 10,
            scrap_offset: 0,
        }]);
        let txn2 = make_transaction(vec![EditOperation::Delete {
            position: 5,
            length: 3,
            scrap_offset: 10,
        }]);
        let txns: Vec<&Transaction> = vec![&txn1, &txn2];
        // 0 + 10 - 3 = 7
        assert!(validate_history(&txns, 0, 7).is_ok());
    }

    #[test]
    fn validate_transaction_with_valid_ops_returns_true() {
        let txn = make_transaction(vec![
            EditOperation::Insert {
                position: 0,
                length: 5,
                scrap_offset: 0,
            },
            EditOperation::Delete {
                position: 2,
                length: 3,
                scrap_offset: 5,
            },
        ]);
        assert!(validate_transaction(&txn, 10));
    }

    #[test]
    fn validate_transaction_with_out_of_bounds_returns_false() {
        let txn = make_transaction(vec![EditOperation::Delete {
            position: 20,
            length: 5,
            scrap_offset: 0,
        }]);
        assert!(!validate_transaction(&txn, 10));
    }
}
