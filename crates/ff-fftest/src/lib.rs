//! # ff-fftest -- FFTest Automated Dialog Testing Framework
//!
//! This crate defines the core types and traits for the FFTest automation
//! subsystem. It has **zero** dependency on egui or any GUI framework.
//!
//! ## Key types
//!
//! - [`AutomationId`] -- a stable dot-path identifier for a UI control
//! - [`ControlState`] -- the observable state of a control at query time
//! - [`AutomationRegistry`] -- trait implemented by the shell to expose control state
//! - [`parser`] -- FFTest script lexer and parser
//! - [`runner`] -- sequential command executor
//! - [`assertions`] -- assertion evaluation engine
//! - [`report`] -- JSON and HTML report generation
//! - [`capture`] -- screenshot capture stub and visual regression
//!
//! Validates: Requirement 2.1, 2.2, 2.3, 2.5, 3.1-3.6, 4.1-4.5,
//!            7.1-7.5, 8.1-8.5 (automated-dialog-testing)

// === Public modules =========================================================

pub mod assertions;
pub mod automation;
pub mod capture;
pub mod parser;
pub mod report;
pub mod runner;

pub use automation::{AutomationId, AutomationRegistry, ControlState};
