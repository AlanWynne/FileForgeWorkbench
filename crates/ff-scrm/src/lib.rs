//! # ff-scrm
//!
//! The Screen Collection and Replay Manager engine for FileForgeWorkbench
//! (CR-NR-098). This crate owns the collection/capture data model, persistence,
//! the replay state machine, capture/masking rules, and exporters. It has NO
//! egui dependency; the desktop shell provides the thin UI wiring.
//!
//! Validates: screen-snapshot-scrm Requirement 7-10, 12, 13, 14, 15, 17, 18.

mod error;
mod evidence;
mod export;
mod model;
mod pdf;
mod pdf_protected;
mod persist;
mod replay;
mod rules;

pub use error::{Result, ScrmError};
pub use evidence::{content_hash, EvidencePackage, EvidenceStatus};
pub use export::{export_html, export_markdown, export_text, Masking};
pub use model::{ScreenCapture, ScreenCollection};
pub use pdf::export_pdf;
pub use pdf_protected::{export_pdf_protected, ProtectionOptions};
pub use persist::{journal_append, journal_recover_latest, load_archive, save_archive};
pub use replay::{elapsed_between_secs, ReplaySession};
pub use rules::{CaptureContext, CaptureRule, CaptureRuleSet, Condition, MaskingRules};
