//! Error types for the localization layer.
//!
//! Load-time errors are RECOVERABLE: the loader retains the previously loaded
//! catalogue (or the English Identity_Base on first load) and logs a WARN,
//! rather than panicking or rendering blank text (localization Req 2.4). This
//! enum names the distinct recoverable failure modes so a caller can log the
//! precise cause.

use std::path::PathBuf;

use thiserror::Error;

/// A recoverable failure that can occur while loading a locale's catalogue.
///
/// None of these variants is fatal: the localization layer always falls back to
/// a usable catalogue (the prior one or the English Identity_Base) so the UI
/// never renders blank text (localization Req 2.4, 2.5).
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum I18nError {
    /// No catalogue directory was found for the requested locale.
    #[error("i18n: no catalogue directory for locale \"{locale}\" at {path}")]
    CatalogueDirMissing {
        /// The requested locale identifier (e.g. `en`, `fr`).
        locale: String,
        /// The directory that was expected to contain the locale's `.ftl` files.
        path: PathBuf,
    },

    /// A `.ftl` file could not be read from disk.
    #[error("i18n: failed to read catalogue file {path}: {source}")]
    FileRead {
        /// The `.ftl` file that could not be read.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },

    /// A `.ftl` file was read but failed to parse as a Fluent resource.
    ///
    /// The parse diagnostics are flattened into a single human-readable string
    /// so the error type stays simple and does not leak Fluent's internal
    /// parser types.
    #[error("i18n: failed to parse catalogue file {path}: {detail}")]
    Parse {
        /// The `.ftl` file that failed to parse.
        path: PathBuf,
        /// A flattened description of the parse diagnostics.
        detail: String,
    },

    /// A parsed resource could not be added to a Fluent bundle (e.g. a
    /// duplicate message identifier across files for the same locale).
    #[error("i18n: failed to build bundle for locale \"{locale}\": {detail}")]
    BundleBuild {
        /// The locale whose bundle could not be built.
        locale: String,
        /// A flattened description of the bundle-build diagnostics.
        detail: String,
    },
}
