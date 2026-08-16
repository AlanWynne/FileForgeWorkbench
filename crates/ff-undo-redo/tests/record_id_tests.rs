//! Property-based tests for logical record IDs.
//! Feature: undo-redo-transactions

use proptest::prelude::*;
use std::collections::HashSet;

use ff_undo_redo::record_id::{LogicalRecordId, RecordIdMap};

// --- Property 14: Per-Document Isolation ---
// **Validates: Requirement 11.1**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property 14: Operations on one document do not affect another.
    #[test]
    fn per_document_isolation(
        ops_doc1 in 1usize..20,
        ops_doc2 in 1usize..20,
    ) {
        // Feature: undo-redo-transactions, Property 14: per-document isolation
        use ff_undo_redo::{UndoConfig, WorkbenchUndoManager};

        let wbm = WorkbenchUndoManager::new();
        wbm.register_new_document("doc1", UndoConfig::default());
        wbm.register_new_document("doc2", UndoConfig::default());

        // Operate on doc1
        {
            let mgr = wbm.get_document_manager("doc1").unwrap();
            let mut lock = mgr.lock().unwrap();
            for i in 0..ops_doc1 {
                lock.begin_transaction(&format!("d1_{}", i));
                lock.record_insert(i as u64, b"a");
                lock.end_transaction();
            }
        }

        // Operate on doc2
        {
            let mgr = wbm.get_document_manager("doc2").unwrap();
            let mut lock = mgr.lock().unwrap();
            for i in 0..ops_doc2 {
                lock.begin_transaction(&format!("d2_{}", i));
                lock.record_insert(i as u64, b"b");
                lock.end_transaction();
            }
        }

        // Verify isolation
        {
            let mgr1 = wbm.get_document_manager("doc1").unwrap();
            let lock1 = mgr1.lock().unwrap();
            prop_assert_eq!(lock1.undo_depth(), ops_doc1,
                "doc1 should have {} transactions", ops_doc1);
        }
        {
            let mgr2 = wbm.get_document_manager("doc2").unwrap();
            let lock2 = mgr2.lock().unwrap();
            prop_assert_eq!(lock2.undo_depth(), ops_doc2,
                "doc2 should have {} transactions", ops_doc2);
        }
    }
}

// --- Property 15: Logical Record ID Stability ---
// **Validates: Requirements 14.1, 14.2, 14.3, 14.4**
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property 15: IDs unique, never reused, offsets track correctly.
    #[test]
    fn logical_record_id_stability(
        initial_lines in 1u64..50,
        num_inserts in 0usize..20,
        num_retires in 0usize..10,
    ) {
        // Feature: undo-redo-transactions, Property 15: ID stability
        let mut map = RecordIdMap::new(initial_lines);
        let mut all_ids: HashSet<LogicalRecordId> = HashSet::new();

        // Collect initial IDs
        for i in 1..=initial_lines {
            all_ids.insert(LogicalRecordId(i));
        }

        // Assign new IDs
        let mut new_ids = Vec::new();
        for _ in 0..num_inserts {
            let id = map.assign_id();
            // ID must be unique
            prop_assert!(!all_ids.contains(&id),
                "newly assigned ID {:?} should be unique", id);
            all_ids.insert(id);
            new_ids.push(id);
        }

        // Retire some IDs
        let retire_count = num_retires.min(initial_lines as usize);
        let mut retired_ids = Vec::new();
        for i in 0..retire_count {
            let id = LogicalRecordId(i as u64 + 1);
            map.retire_id(id);
            retired_ids.push(id);
        }

        // Verify retired IDs are not reused
        for _ in 0..10 {
            let new_id = map.assign_id();
            prop_assert!(!retired_ids.contains(&new_id),
                "retired ID {:?} should never be reused", new_id);
            all_ids.insert(new_id);
        }

        // Verify offset tracking after update
        if !new_ids.is_empty() {
            let first_new = new_ids[0];
            map.set_offset(first_new, 100);
            prop_assert_eq!(map.offset_for(first_new), Some(100));

            // Shift offsets
            map.update_offsets(50, 10);
            // Offset 100 >= 50, so should shift to 110
            prop_assert_eq!(map.offset_for(first_new), Some(110));
        }
    }
}
