//! # CR-CH-072 User_Locale_Override_Store overlay for [`LocaleAliasStore`].
//!
//! Split out of `alias_reader.rs` so that file stays within the 400-line source
//! limit (rust-standards.md / rust-ffwb-layout.md). As a child module of
//! `alias_reader` it still reaches the parent's private `LocaleAliasStore` field
//! and the `builtin_ffcmd_roots` shadow-guard set.
//!
//! CR-CH-072 REPLACES the former CR-NR-113 flat, locale-INDEPENDENT `user.toml`
//! third overlay with a per-LOCALE override: for the active locale the user
//! override lives at `<base>/<locale>/<ENV>.toml` and is applied as the SINGLE
//! user overlay on top of the shipped seed overlay. The ENV comes from the file
//! name, so an override row carries no per-row environment key.

use std::path::Path;

use super::{builtin_ffcmd_roots, LocaleAliasStore};

impl LocaleAliasStore {
    /// Apply the User_Locale_Override_Store for `locale` from override base
    /// `base` (localization Req 13.5): for each environment the store holds, read
    /// `<base>/<locale>/<ENV>.toml` and overlay its rows on top of the shipped
    /// seed overlay. An absent file is a no-op (no user override for that ENV); a
    /// malformed file logs a WARN and is skipped whole, leaving the seed intact.
    ///
    /// The per-row discipline mirrors the previous user overlay:
    ///   - NO-SHADOW: a surface equal to a reserved built-in root
    ///     ([`builtin_ffcmd_roots`]) is skipped with a WARN.
    ///   - ROOT-VALIDITY (CR-CH-067): a root that is not a valid canonical root of
    ///     the ENV is skipped with a WARN; the load continues.
    ///   - CROSS-LAYER: applying one row at a time means a DIFFERENT-root clash
    ///     with the seed is rejected (seed wins) with a WARN, while a SAME-root
    ///     surface customisation REPLACES the seed row (the user override WINS,
    ///     Req 13.5).
    ///
    /// Validates: localization Requirement 13.5, 13.6; command-configurator Req 6.1
    pub(crate) fn apply_user_overlay_from_base(&mut self, base: &Path, locale: &str) {
        let env_names: Vec<String> = self.table_name_list();
        for env_name in env_names {
            let path =
                super::super::user_aliases::user_alias_override_file(base, locale, &env_name);
            let text = match std::fs::read_to_string(&path) {
                Ok(text) => text,
                Err(_) => continue, // absent -> no user override for this ENV
            };
            let rows = match super::super::user_aliases::parse_user_alias_toml(&text) {
                Ok(rows) => rows,
                Err(e) => {
                    ff_logging::log_warn!(
                        "[aliases] user override {} failed to parse ({}) -- skipped",
                        path.display(),
                        e
                    );
                    continue;
                }
            };
            self.apply_user_rows_for_env(&env_name, &rows);
        }
    }

    /// Apply parsed User_Locale_Override_Store rows for ONE environment with the
    /// CR-CH-072 per-row discipline (shared by the file path and the save-time
    /// in-place refresh). Reserved built-in roots are the shadow guard set
    /// ([`builtin_ffcmd_roots`]). The user override WINS on a shared surface (the
    /// overlay seam's same-root REPLACE), so a seeded surface is customisable.
    ///
    /// Validates: localization Requirement 13.5, 13.6; command-configurator Req 6.6
    pub(crate) fn apply_user_rows_for_env(
        &mut self,
        env_name: &str,
        rows: &[super::super::user_aliases::UserAliasRowToml],
    ) {
        let reserved = builtin_ffcmd_roots();
        for row in rows {
            let surface = row.surface.trim().to_uppercase();
            let canonical = row.canonical.trim().to_uppercase();
            if surface.is_empty() || canonical.is_empty() {
                continue; // malformed row: skip (save-time validation rejects these)
            }
            // NO-SHADOW invariant: a user surface equal to a reserved built-in
            // root is skipped with a WARN.
            if reserved.iter().any(|r| r.eq_ignore_ascii_case(&surface)) {
                ff_logging::log_warn!(
                    "[aliases] user override surface '{}' shadows a built-in root -- skipped",
                    surface
                );
                continue;
            }
            // ROOT-VALIDITY (CR-CH-067, Req 20.7): a row whose root is NOT a valid
            // canonical root of its environment is skipped with a WARN; the load
            // CONTINUES so one invalid row never drops the others.
            if !super::super::environment_registry::EnvironmentRegistry::contains_root(
                env_name, &canonical,
            ) {
                ff_logging::log_warn!(
                    "[aliases] user override '{}' names root '{}' which is not a valid root of \
                     environment '{}' -- skipped",
                    surface,
                    canonical,
                    env_name
                );
                continue;
            }
            let template = if row.command_template.trim().is_empty() {
                None
            } else {
                Some(row.command_template.trim().to_string())
            };
            let Some(table) = self.table_for_mut(env_name) else {
                continue; // unknown environment name: skip
            };
            // Apply ONE row: a same-root surface REPLACES the seed row (user wins,
            // Req 13.5); a DIFFERENT-root clash is rejected (seed wins) with a
            // WARN; the overlay seam carries the discipline.
            if let Err(e) =
                table.apply_locale_overlay_keyed(env_name, &[(surface, canonical, template)])
            {
                ff_logging::log_warn!(
                    "[aliases] user override row rejected ({}) -- the shipped seed surface wins",
                    e
                );
            }
        }
    }
}
