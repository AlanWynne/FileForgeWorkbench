//! # ff-governance-tests — Architectural Compliance Test Infrastructure
//!
//! This crate provides no runtime functionality. It exists solely to host
//! architectural fitness tests that verify the Dataset Ownership Model
//! governance rules (ADR-001) at CI time.
//!
//! ## What It Tests
//!
//! - **Dependency direction**: Prohibited dependencies do not appear in Cargo.toml files
//! - **Ownership boundaries**: Crates do not import forbidden modules
//! - **Trait-based coupling**: Dependent crates compile with mock trait implementations
//!
//! ## Usage
//!
//! ```bash
//! cargo test -p ff-governance-tests
//! ```

/// Helper utilities for parsing Cargo.toml files and checking dependencies.
pub mod compliance;
