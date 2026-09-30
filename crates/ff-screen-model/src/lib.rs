//! # ff-screen-model
//!
//! The logical screen model for FileForgeWorkbench screen capture (CR-NR-098).
//!
//! This crate is deliberately GUI-independent: it has no egui dependency and no
//! SCRM (collection/replay) logic. It owns three things:
//!
//! 1. [`ScreenModel`] and its elements -- the authoritative, host-agnostic
//!    representation of a screen (title, fields, tables, messages, status bar,
//!    command line, cursor, dimensions).
//! 2. The [`ScreenProvider`] trait -- what a Context implements to yield its
//!    current [`ScreenModel`]. The capture engine consumes ONLY this; it never
//!    inspects egui widgets.
//! 3. The [`SnapshotFormat`] renderers -- pure `ScreenModel -> String`
//!    functions producing plain text, ANSI, Markdown, HTML, or YAML.
//!
//! The design keeps this crate free of egui so the same seam can later back a
//! dynamic plugin without rework (screen-snapshot-scrm Requirement 2.4).
//!
//! Validates: screen-snapshot-scrm Requirement 2.1, 2.4, 3.x, 4.x, 5.x.

mod model;
mod render;

pub use model::{
    Colour, Cursor, Dimensions, Field, FieldAttributes, ScreenModel, StatusBar, Table,
};
pub use render::{render, RenderOptions};

/// The set of snapshot output formats.
///
/// Validates: Requirement 5.5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotFormat {
    /// Plain text with box-drawing framing (or ASCII fallback).
    PlainText,
    /// Plain text plus ANSI SGR colour escape sequences.
    Ansi,
    /// Markdown: screen text in a fenced code block; tables as Markdown tables.
    Markdown,
    /// HTML preserving colours and layout.
    Html,
    /// Structured YAML (the "AI" format).
    Yaml,
}

impl SnapshotFormat {
    /// Parse a command argument (e.g. the `TEXT` in `SNAPSHOT TEXT`) to a format.
    /// `MD` is an alias for `Markdown`; `AI` is an alias for `Yaml`. An empty
    /// argument selects the default ([`SnapshotFormat::PlainText`]).
    ///
    /// Validates: Requirement 4.1-4.6.
    pub fn parse_arg(arg: &str) -> Option<Self> {
        match arg.trim().to_ascii_uppercase().as_str() {
            "" | "TEXT" => Some(Self::PlainText),
            "ANSI" => Some(Self::Ansi),
            "MARKDOWN" | "MD" => Some(Self::Markdown),
            "HTML" => Some(Self::Html),
            "YAML" | "AI" => Some(Self::Yaml),
            _ => None,
        }
    }
}

/// A Context that can yield a logical [`ScreenModel`] for capture.
///
/// The capture engine obtains screen content ONLY through this trait, so it
/// never fights the GUI framework (screen-snapshot-scrm Requirement 2.1, 2.2).
pub trait ScreenProvider {
    /// Return the current logical screen model for this Context.
    fn screen_model(&self) -> ScreenModel;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 4.1-4.6 -- format argument parsing incl. aliases.
    #[test]
    fn parse_arg_maps_verbs_and_aliases() {
        assert_eq!(
            SnapshotFormat::parse_arg(""),
            Some(SnapshotFormat::PlainText)
        );
        assert_eq!(
            SnapshotFormat::parse_arg("text"),
            Some(SnapshotFormat::PlainText)
        );
        assert_eq!(
            SnapshotFormat::parse_arg("ANSI"),
            Some(SnapshotFormat::Ansi)
        );
        assert_eq!(
            SnapshotFormat::parse_arg("md"),
            Some(SnapshotFormat::Markdown)
        );
        assert_eq!(
            SnapshotFormat::parse_arg("Markdown"),
            Some(SnapshotFormat::Markdown)
        );
        assert_eq!(
            SnapshotFormat::parse_arg("html"),
            Some(SnapshotFormat::Html)
        );
        assert_eq!(SnapshotFormat::parse_arg("ai"), Some(SnapshotFormat::Yaml));
        assert_eq!(
            SnapshotFormat::parse_arg("yaml"),
            Some(SnapshotFormat::Yaml)
        );
        assert_eq!(SnapshotFormat::parse_arg("bogus"), None);
    }

    // Validates: Requirement 2.1 -- a ScreenProvider yields a ScreenModel.
    #[test]
    fn screen_provider_yields_model() {
        struct Dummy;
        impl ScreenProvider for Dummy {
            fn screen_model(&self) -> ScreenModel {
                ScreenModel::new("Dummy")
            }
        }
        let d = Dummy;
        assert_eq!(d.screen_model().title, "Dummy");
    }
}
