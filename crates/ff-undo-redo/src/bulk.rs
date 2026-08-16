//! Bulk transaction types — optimised storage for multi-edit operations.
//!
//! Supports two strategies:
//! - [`RuleTransaction`]: O(1) memory, stores transformation rule. Re-scans on undo.
//! - [`IndexTransaction`]: O(n) memory, stores rule + affected record IDs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::record_id::LogicalRecordId;
use crate::selection::SelectionState;

/// Optimised storage for multi-edit operations.
#[derive(Debug, Clone)]
pub enum BulkTransaction {
    /// O(1) memory — stores the transformation rule; re-scans on undo.
    Rule(RuleTransaction),
    /// O(n) memory — stores rule + list of affected record IDs.
    Index(IndexTransaction),
}

impl BulkTransaction {
    /// Returns the name of this bulk transaction.
    pub fn name(&self) -> &str {
        match self {
            Self::Rule(r) => &r.name,
            Self::Index(i) => &i.name,
        }
    }

    /// Returns the timestamp of this bulk transaction.
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::Rule(r) => r.timestamp,
            Self::Index(i) => i.timestamp,
        }
    }

    /// Returns the memory cost category.
    pub fn is_constant_memory(&self) -> bool {
        matches!(self, Self::Rule(_))
    }

    /// Returns the number of affected records (for Index) or 0 (for Rule).
    pub fn affected_count(&self) -> usize {
        match self {
            Self::Rule(_) => 0,
            Self::Index(i) => i.affected_records.len(),
        }
    }
}

/// A rule-based bulk transaction. Scope is deterministic and re-scannable.
#[derive(Debug, Clone)]
pub struct RuleTransaction {
    /// The transformation rule (pattern + replacement).
    pub rule: TransformRule,
    /// Scope description for re-scanning on undo.
    pub scope: BulkScope,
    /// Human-readable name.
    pub name: String,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
    /// Selection state before the operation.
    pub selection_before: Option<SelectionState>,
    /// Selection state after the operation.
    pub selection_after: Option<SelectionState>,
}

/// An index-based bulk transaction. Materialises affected record IDs.
#[derive(Debug, Clone)]
pub struct IndexTransaction {
    /// The transformation rule.
    pub rule: TransformRule,
    /// The logical record IDs of affected records.
    pub affected_records: Vec<LogicalRecordId>,
    /// Human-readable name.
    pub name: String,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
    /// Selection state before the operation.
    pub selection_before: Option<SelectionState>,
    /// Selection state after the operation.
    pub selection_after: Option<SelectionState>,
}

/// The transformation rule applied by a bulk operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformRule {
    /// Pattern to match (regex or literal).
    pub pattern: String,
    /// Replacement string.
    pub replacement: String,
    /// Whether match is case-sensitive.
    pub case_sensitive: bool,
    /// Additional rule metadata.
    pub metadata: HashMap<String, String>,
}

/// Scope of a bulk operation — determines Rule vs Index strategy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BulkScope {
    /// All records — deterministic, use RuleTransaction.
    All,
    /// Explicit line range — deterministic.
    Range {
        /// Start line (inclusive).
        start: u64,
        /// End line (inclusive).
        end: u64,
    },
    /// CC block — deterministic from command context.
    Block {
        /// Block start line.
        start: u64,
        /// Block end line.
        end: u64,
    },
    /// Visible/non-excluded — transient, use IndexTransaction.
    Visible,
    /// Excluded only — transient.
    Excluded,
    /// Tagged records — transient.
    Tagged,
    /// Filtered by criteria — transient.
    Filtered,
}

impl BulkScope {
    /// Returns true if this scope is deterministic (should use RuleTransaction).
    pub fn is_deterministic(&self) -> bool {
        matches!(self, Self::All | Self::Range { .. } | Self::Block { .. })
    }

    /// Returns true if this scope is transient (should use IndexTransaction).
    pub fn is_transient(&self) -> bool {
        !self.is_deterministic()
    }
}

/// Builder for constructing bulk transactions during execution.
#[derive(Debug)]
pub struct BulkTransactionBuilder {
    /// Name of the bulk operation.
    name: String,
    /// The transformation rule.
    rule: TransformRule,
    /// The scope.
    scope: BulkScope,
    /// Collected affected record IDs (for Index transactions).
    affected_records: Vec<LogicalRecordId>,
    /// Selection before.
    selection_before: Option<SelectionState>,
    /// Whether the builder is active.
    active: bool,
}

impl BulkTransactionBuilder {
    /// Creates a new builder for the given rule and scope.
    pub fn new(name: &str, rule: TransformRule, scope: BulkScope) -> Self {
        Self {
            name: name.to_string(),
            rule,
            scope,
            affected_records: Vec::new(),
            selection_before: None,
            active: true,
        }
    }

    /// Records an affected record ID.
    pub fn record_affected(&mut self, record_id: LogicalRecordId) {
        self.affected_records.push(record_id);
    }

    /// Sets the selection state before the operation.
    pub fn set_selection_before(&mut self, state: SelectionState) {
        self.selection_before = Some(state);
    }

    /// Returns whether this builder is active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Commits the bulk transaction, producing the appropriate variant.
    pub fn commit(self, selection_after: Option<SelectionState>) -> BulkTransaction {
        let timestamp = Utc::now();

        if self.scope.is_deterministic() {
            BulkTransaction::Rule(RuleTransaction {
                rule: self.rule,
                scope: self.scope,
                name: self.name,
                timestamp,
                selection_before: self.selection_before,
                selection_after,
            })
        } else {
            BulkTransaction::Index(IndexTransaction {
                rule: self.rule,
                affected_records: self.affected_records,
                name: self.name,
                timestamp,
                selection_before: self.selection_before,
                selection_after,
            })
        }
    }

    /// Returns the affected record count so far.
    pub fn affected_count(&self) -> usize {
        self.affected_records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rule() -> TransformRule {
        TransformRule {
            pattern: "ERROR".to_string(),
            replacement: "WARN".to_string(),
            case_sensitive: true,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn deterministic_scope_produces_rule_transaction() {
        let builder = BulkTransactionBuilder::new("CHANGE ALL", make_rule(), BulkScope::All);
        let txn = builder.commit(None);
        assert!(txn.is_constant_memory());
        assert!(matches!(txn, BulkTransaction::Rule(_)));
    }

    #[test]
    fn transient_scope_produces_index_transaction() {
        let mut builder =
            BulkTransactionBuilder::new("CHANGE VISIBLE", make_rule(), BulkScope::Visible);
        builder.record_affected(LogicalRecordId(1));
        builder.record_affected(LogicalRecordId(5));
        let txn = builder.commit(None);
        assert!(!txn.is_constant_memory());
        assert_eq!(txn.affected_count(), 2);
    }

    #[test]
    fn scope_all_is_deterministic() {
        assert!(BulkScope::All.is_deterministic());
    }

    #[test]
    fn scope_range_is_deterministic() {
        assert!(BulkScope::Range { start: 1, end: 100 }.is_deterministic());
    }

    #[test]
    fn scope_visible_is_transient() {
        assert!(BulkScope::Visible.is_transient());
    }

    #[test]
    fn scope_tagged_is_transient() {
        assert!(BulkScope::Tagged.is_transient());
    }

    #[test]
    fn scope_filtered_is_transient() {
        assert!(BulkScope::Filtered.is_transient());
    }

    #[test]
    fn rule_transaction_memory_is_constant() {
        let txn = BulkTransaction::Rule(RuleTransaction {
            rule: make_rule(),
            scope: BulkScope::All,
            name: "test".to_string(),
            timestamp: Utc::now(),
            selection_before: None,
            selection_after: None,
        });
        assert!(txn.is_constant_memory());
        assert_eq!(txn.affected_count(), 0);
    }

    #[test]
    fn index_transaction_memory_scales_with_records() {
        let txn = BulkTransaction::Index(IndexTransaction {
            rule: make_rule(),
            affected_records: vec![LogicalRecordId(1), LogicalRecordId(2), LogicalRecordId(3)],
            name: "test".to_string(),
            timestamp: Utc::now(),
            selection_before: None,
            selection_after: None,
        });
        assert!(!txn.is_constant_memory());
        assert_eq!(txn.affected_count(), 3);
    }
}
