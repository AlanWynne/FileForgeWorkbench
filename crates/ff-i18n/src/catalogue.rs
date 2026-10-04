//! The [`Catalogue`]: per-locale Fluent bundles plus the English Identity_Base.
//!
//! A `Catalogue` owns the active locale's Fluent bundle and the always-present
//! English Identity_Base bundle (localization Req 2.5). Lookups resolve a key
//! against the active bundle, then the Identity_Base, then fall back to the key
//! itself with a WARN (localization Req 3.2, 3.3).
//!
//! Bundles use the CONCURRENT Fluent variant so a `Catalogue` is `Send + Sync`
//! and can live behind a process-wide shared handle (see `seam.rs`). Fluent's
//! Unicode isolation marks are disabled so interpolated values render as plain
//! text, which keeps the output predictable for non-GUI consumers and tests
//! (localization Req 3.5).

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::FluentResource;

use crate::args::MessageArgs;
use crate::error::I18nError;
use crate::locale::Locale;

/// A loaded set of message bundles for one active locale.
///
/// Always holds the English Identity_Base bundle; the active-locale bundle is
/// absent when the active locale IS English (the base already covers it) or
/// when a load failed and the base is the only usable catalogue.
pub struct Catalogue {
    active_locale: Locale,
    active: Option<FluentBundle<FluentResource>>,
    identity_base: FluentBundle<FluentResource>,
}

// `FluentBundle` does not implement `Debug`, so we cannot derive it. A manual
// impl keeps the public `Debug` guarantee while reporting only the inspectable
// state: the active locale and whether an overlay bundle is present.
impl fmt::Debug for Catalogue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Catalogue")
            .field("active_locale", &self.active_locale)
            .field("has_active_overlay", &self.active.is_some())
            .finish()
    }
}

impl Catalogue {
    /// Build a Catalogue from the English Identity_Base `.ftl` files only.
    ///
    /// This is the always-available baseline: it loads even when no other locale
    /// is installed (localization Req 2.5). The resulting catalogue has English
    /// as both the active locale and the Identity_Base.
    ///
    /// # Errors
    ///
    /// Returns [`I18nError`] when the English directory is missing or its files
    /// cannot be read, parsed, or added to a bundle. The English base is
    /// mandatory, so unlike an overlay locale this failure is surfaced to the
    /// caller rather than silently tolerated.
    pub fn load_identity_base(i18n_root: &Path) -> Result<Self, I18nError> {
        let base_locale = Locale::identity_base();
        let identity_base = build_bundle(i18n_root, &base_locale)?;
        Ok(Self {
            active_locale: base_locale,
            active: None,
            identity_base,
        })
    }

    /// Load `locale` as the active overlay on top of an existing Identity_Base.
    ///
    /// On success the returned Catalogue resolves keys against `locale` first,
    /// then the Identity_Base. When `locale` IS English, no overlay is built and
    /// lookups resolve directly against the base.
    ///
    /// # Errors
    ///
    /// Returns [`I18nError`] when the locale's directory is missing or its files
    /// cannot be read, parsed, or added. Callers typically treat this as
    /// RECOVERABLE: retain the prior catalogue (or the Identity_Base) and log a
    /// WARN rather than failing to render (localization Req 2.4).
    pub fn load_overlay(self, i18n_root: &Path, locale: Locale) -> Result<Self, I18nError> {
        if locale.is_identity_base() {
            return Ok(Self {
                active_locale: locale,
                active: None,
                identity_base: self.identity_base,
            });
        }
        let active = build_bundle(i18n_root, &locale)?;
        Ok(Self {
            active_locale: locale,
            active: Some(active),
            identity_base: self.identity_base,
        })
    }

    /// The locale currently active for lookups.
    pub fn active_locale(&self) -> &Locale {
        &self.active_locale
    }

    /// Resolve a message `key` with no arguments.
    ///
    /// Resolution order: active bundle -> Identity_Base -> the key itself with a
    /// WARN (localization Req 3.2, 3.3). The return is an owned `String`, which
    /// is exactly what egui widgets accept, so a render site swaps a literal for
    /// `t("key")` with no API change (localization Req 3.4).
    pub fn resolve(&self, key: &str) -> String {
        self.resolve_args(key, &MessageArgs::new())
    }

    /// Resolve a message `key` supplying named placeable arguments.
    ///
    /// Same fallback chain as [`resolve`](Self::resolve). An embedded command
    /// verb is passed as an ordinary argument and is never translated
    /// (localization Req 9.1, 9.2).
    pub fn resolve_args(&self, key: &str, args: &MessageArgs) -> String {
        if let Some(bundle) = &self.active {
            if let Some(text) = format_from_bundle(bundle, key, args) {
                return text;
            }
        }
        if let Some(text) = format_from_bundle(&self.identity_base, key, args) {
            return text;
        }
        ff_logging::log_warn!(
            "[i18n] missing message key \"{}\" in active locale \"{}\" and the English Identity_Base -- returning the key as fallback",
            key,
            self.active_locale.tag()
        );
        key.to_string()
    }
}

/// Format a single key from one bundle, returning `None` when the key is absent.
///
/// A per-pattern format error (a malformed placeable, say) is logged as a WARN
/// and the partially-formatted text is still returned, so a format hiccup never
/// yields blank UI.
fn format_from_bundle(
    bundle: &FluentBundle<FluentResource>,
    key: &str,
    args: &MessageArgs,
) -> Option<String> {
    let message = bundle.get_message(key)?;
    let pattern = message.value()?;
    let fluent_args = args.to_fluent_args();
    let args_opt = if args.is_empty() {
        None
    } else {
        Some(&fluent_args)
    };
    let mut errors = Vec::new();
    let value = bundle.format_pattern(pattern, args_opt, &mut errors);
    if !errors.is_empty() {
        ff_logging::log_warn!(
            "[i18n] formatting message key \"{}\" produced {} error(s); returning best-effort text",
            key,
            errors.len()
        );
    }
    Some(value.into_owned())
}

/// Build a concurrent Fluent bundle for `locale` from every `.ftl` file in its
/// `i18n/<locale>/` directory.
///
/// Multiple `.ftl` files for one locale are merged into a single bundle. The
/// bundle disables Unicode isolation so interpolated arguments render as plain
/// text (localization Req 3.5).
fn build_bundle(
    i18n_root: &Path,
    locale: &Locale,
) -> Result<FluentBundle<FluentResource>, I18nError> {
    let dir = locale_dir(i18n_root, locale);
    if !dir.is_dir() {
        return Err(I18nError::CatalogueDirMissing {
            locale: locale.tag(),
            path: dir,
        });
    }
    let mut bundle = FluentBundle::new_concurrent(vec![locale.lang_id().clone()]);
    bundle.set_use_isolating(false);
    let mut loaded_any = false;
    for path in ftl_files(&dir)? {
        let source = fs::read_to_string(&path).map_err(|source| I18nError::FileRead {
            path: path.clone(),
            source,
        })?;
        let resource =
            FluentResource::try_new(source).map_err(|(_, parse_errors)| I18nError::Parse {
                path: path.clone(),
                detail: format!("{} parse error(s)", parse_errors.len()),
            })?;
        bundle
            .add_resource(resource)
            .map_err(|errors| I18nError::BundleBuild {
                locale: locale.tag(),
                detail: format!("{} bundle error(s) from {}", errors.len(), path.display()),
            })?;
        loaded_any = true;
    }
    if !loaded_any {
        return Err(I18nError::CatalogueDirMissing {
            locale: locale.tag(),
            path: dir,
        });
    }
    Ok(bundle)
}

/// The on-disk directory holding a locale's `.ftl` files: `i18n/<locale>/`.
fn locale_dir(i18n_root: &Path, locale: &Locale) -> PathBuf {
    i18n_root.join(locale.tag())
}

/// Collect every `.ftl` file directly inside `dir`, sorted for deterministic
/// load order (so bundle contents do not depend on filesystem iteration order).
fn ftl_files(dir: &Path) -> Result<Vec<PathBuf>, I18nError> {
    let mut files = Vec::new();
    let entries = fs::read_dir(dir).map_err(|source| I18nError::FileRead {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| I18nError::FileRead {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("ftl") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}
