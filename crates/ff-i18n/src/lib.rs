//! # ff-i18n -- Localization message-catalogue mechanism and lookup seam
//!
//! `ff-i18n` is the GUI-free foundation of FileForgeWorkbench localization
//! (CR-NR-103). It owns the Fluent message-catalogue load/lookup and exposes the
//! Catalogue_Lookup_Seam (`t` / `t_args`) that render code calls instead of
//! embedding raw string literals.
//!
//! ## What this crate provides
//!
//! - [`Locale`] -- a BCP-47 locale selected by the `ui.locale` config string,
//!   with English (`en`) as the always-present Identity_Base.
//! - [`Catalogue`] -- per-locale Fluent bundles plus the English Identity_Base,
//!   loaded from per-locale `.ftl` DATA files under an `i18n/<locale>/` tree
//!   (localization Req 2.1, 2.2, 2.5).
//! - [`t`] / [`t_args`] -- the lookup seam: resolve a key against the active
//!   locale, fall back to the Identity_Base, then to the key itself with a WARN
//!   (localization Req 3.1, 3.2, 3.3).
//! - [`MessageArgs`] -- named placeable arguments for interpolated and
//!   verb-embedding messages (localization Req 9.1, 9.2).
//! - [`I18nError`] -- the recoverable load-error taxonomy (localization Req 2.4).
//!
//! ## GUI-free by construction
//!
//! This crate depends only on Fluent, `unic-langid`, and the workspace logging
//! facade -- never on egui or any shell/editor crate -- so any crate (and tests)
//! can resolve messages the same way (localization Req 3.5). Lookups return owned
//! `String` values, exactly what egui widgets already accept, so threading the
//! seam into render code is a one-line literal-to-`t("key")` swap with no egui
//! API change (localization Req 3.4).
//!
//! ## Fallback discipline
//!
//! Nothing in this crate panics on a missing key, a missing catalogue, or a
//! `.ftl` parse error. A missing key yields the key string plus a WARN; a load
//! error is surfaced as a recoverable [`I18nError`] so callers retain the prior
//! catalogue (or the Identity_Base) and WARN rather than render blank text
//! (localization Req 2.4, 3.3).

/// Named placeable arguments for argument-bearing lookups.
pub mod args;

/// Per-locale Fluent bundles plus the English Identity_Base.
pub mod catalogue;

/// The recoverable load-error taxonomy.
pub mod error;

/// The [`Locale`] type and the Identity_Base constant.
pub mod locale;

/// The Catalogue_Lookup_Seam: free `t` / `t_args` over a shared catalogue.
pub mod seam;

pub use args::MessageArgs;
pub use catalogue::Catalogue;
pub use error::I18nError;
pub use locale::{Locale, IDENTITY_BASE_TAG};
pub use seam::{is_initialised, set_catalogue, t, t_args};

#[cfg(test)]
mod tests;
