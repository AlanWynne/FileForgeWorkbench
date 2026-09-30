//! Generic PDF 1.7 writer — dependency-free core, optional lopdf encryption.
//!
//! Generalises the hand-written PDF builder from `ff-scrm` so any crate can
//! produce selectable-text PDFs without pulling in a heavy PDF crate.
//!
//! # Features
//! - `protected` — enables `export_pdf_protected` via `lopdf` encryption

pub mod writer;

#[cfg(feature = "protected")]
pub mod protected;

pub use writer::{PdfDocument, PdfPage};

/// Build a PDF from a list of pages, each page being a list of text lines.
/// Returns raw PDF bytes with selectable text (Courier, 9pt, US Letter).
pub fn build_pdf_from_pages(pages: &[Vec<String>]) -> Vec<u8> {
    writer::build_pdf(pages)
}
