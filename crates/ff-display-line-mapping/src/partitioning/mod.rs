//! Partitioning data structures for O(log n) prefix-sum queries.
//!
//! The Fenwick tree (Binary Indexed Tree) provides the core data structure
//! for mapping document lines to display line offsets.

pub mod fenwick_tree;

pub use fenwick_tree::FenwickTree;
