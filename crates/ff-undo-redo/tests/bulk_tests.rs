//! Property-based tests for bulk transactions.
//! Feature: undo-redo-transactions

use proptest::prelude::*;
use std::collections::HashMap;

use ff_undo_redo::bulk::{BulkScope, BulkTransactionBuilder, TransformRule};
use ff_undo_redo::record_id::LogicalRecordId;

fn make_rule() -> TransformRule {
    TransformRule {
        pattern: "ERROR".to_string(),
        replacement: "WARN".to_string(),
        case_sensitive: true,
        metadata: HashMap::new(),
    }
}

// --- Property 9: Bulk Transaction Memory Efficiency ---
// **Validates: Requirement 7.8**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property 9: RuleTransaction is O(1), IndexTransaction is O(n).
    #[test]
    fn bulk_transaction_memory_efficiency(
        num_affected in 1usize..1000,
    ) {
        // Feature: undo-redo-transactions, Property 9: memory efficiency

        // Rule transaction: constant memory regardless of affected count
        let rule_builder = BulkTransactionBuilder::new(
            "CHANGE ALL", make_rule(), BulkScope::All,
        );
        let rule_txn = rule_builder.commit(None);
        prop_assert!(rule_txn.is_constant_memory(),
            "Rule transaction should be O(1)");
        prop_assert_eq!(rule_txn.affected_count(), 0);

        // Index transaction: memory scales with n
        let mut idx_builder = BulkTransactionBuilder::new(
            "CHANGE VISIBLE", make_rule(), BulkScope::Visible,
        );
        for i in 0..num_affected {
            idx_builder.record_affected(LogicalRecordId(i as u64 + 1));
        }
        let idx_txn = idx_builder.commit(None);
        prop_assert!(!idx_txn.is_constant_memory(),
            "Index transaction should NOT be O(1)");
        prop_assert_eq!(idx_txn.affected_count(), num_affected,
            "Index transaction should store all {} affected records",
            num_affected);
    }
}
