//! Thin adapter re-exporting the `ff-context-menu` crate so existing
//! `crate::context_menu::*` references continue to resolve unchanged.
//!
//! core-001 strip: the only in-tree consumer of this re-export was the parked
//! Files Panel (Catalog Explorer). `ff-context-menu` remains a core dependency,
//! so the adapter is retained; `allow(unused_imports)` silences the now-unused
//! glob until a retained Context wires a context menu again.
#[allow(unused_imports)]
pub use ff_context_menu::*;
