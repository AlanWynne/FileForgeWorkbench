//! # CR-NR-113 User_Alias_Store overlay for [`LocaleAliasStore`].
//!
//! Split out of `alias_reader.rs` so that file stays within the 400-line source
//! limit (rust-standards.md / rust-ffwb-layout.md). This is a pure REFACTOR: the
//! two methods are MOVED verbatim, with NO behaviour change. As a child module
//! of `alias_reader` it still reaches the parent's private `LocaleAliasStore`
//! field and the `builtin_ffcmd_roots` shadow-guard set.

use std::path::Path;

use super::{builtin_ffcmd_roots, LocaleAliasStore};

impl LocaleAliasStore {
    /// Apply the User_Alias_Store (CR-NR-113) as a THIRD overlay on top of the
    /// identity base + per-locale overlay (Req 5.7). Reads `path` via the NEW
    /// [`super::super::user_aliases::parse_user_alias_toml`] (NOT the 3-field
    /// `parse_alias_toml`, which drops `environment`), then routes each row into
    /// its target environment's `AliasTable`. An absent file is a no-op (empty
    /// store); a malformed file logs a WARN and is skipped whole.
    ///
    /// Per-row discipline (Req 5.4, 5.7):
    ///   - NO-SHADOW: a surface equal to a reserved built-in root
    ///     ([`builtin_ffcmd_roots`]) is skipped with a WARN (a NEW CR-NR-113
    ///     pre-pass invariant, NOT command-framework Req 8.10).
    ///   - CROSS-LAYER: applying one row at a time means the overlay seam's own
    ///     within-environment discipline yields SAME-root = no-op and
    ///     DIFFERENT-root = the row is rejected (the per-locale/base surface
    ///     wins) with a WARN; the remaining rows still load.
    ///
    /// Validates: command-configurator Requirement 5.6, 5.7
    pub(crate) fn apply_user_overlay_from_file(&mut self, path: &Path) {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(_) => return, // absent -> empty user store (Req 5.1)
        };
        let rows = match super::super::user_aliases::parse_user_alias_toml(&text) {
            Ok(rows) => rows,
            Err(e) => {
                ff_logging::log_warn!(
                    "[aliases] user alias store {} failed to parse ({}) -- skipped",
                    path.display(),
                    e
                );
                return;
            }
        };
        self.apply_user_rows(&rows);
    }

    /// Apply parsed User_Alias_Store rows with the CR-NR-113 per-row discipline
    /// (shared by the file path and the save-time in-place refresh). Reserved
    /// built-in roots are the shadow guard set ([`builtin_ffcmd_roots`]).
    ///
    /// Validates: command-configurator Requirement 5.4, 5.7
    pub(crate) fn apply_user_rows(
        &mut self,
        rows: &[super::super::user_aliases::UserAliasRowToml],
    ) {
        let reserved = builtin_ffcmd_roots();
        for row in rows {
            let surface = row.surface.trim().to_uppercase();
            let canonical = row.canonical.trim().to_uppercase();
            if surface.is_empty() || canonical.is_empty() {
                continue; // malformed row: skip (save-time validation rejects these)
            }
            // NO-SHADOW invariant (Req 5.4, 5.7): a user surface equal to a
            // reserved built-in root is skipped with a WARN.
            if reserved.iter().any(|r| r.eq_ignore_ascii_case(&surface)) {
                ff_logging::log_warn!(
                    "[aliases] user alias surface '{}' shadows a built-in root -- skipped",
                    surface
                );
                continue;
            }
            let env_name = row.environment.env_name();
            // REJECT-WITH-WARNING-AT-LOAD (CR-CH-067, Req 20.7): a row whose root
            // is NOT a valid canonical root of its declared environment is skipped
            // with a WARN naming the offending alias / root / environment, and the
            // load CONTINUES. A single invalid row never blocks startup nor drops
            // any OTHER valid row. This runs AFTER the malformed-row and no-shadow
            // guards and BEFORE the row is applied, mirroring their skip-with-WARN
            // discipline; the existing guards stay intact.
            if !super::super::environment_registry::EnvironmentRegistry::contains_root(
                env_name, &canonical,
            ) {
                ff_logging::log_warn!(
                    "[aliases] user alias '{}' names root '{}' which is not a valid root of \
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
            let Some((_, table)) = self.tables.iter_mut().find(|(n, _)| n == env_name) else {
                continue; // unknown environment name: skip
            };
            // Apply ONE row so a cross-layer DIFFERENT-root collision rejects only
            // this row (the per-locale/base surface wins, Req 5.7); a SAME-root
            // row is a harmless no-op via the overlay seam's own discipline.
            if let Err(e) =
                table.apply_locale_overlay_keyed(env_name, &[(surface, canonical, None, template)])
            {
                ff_logging::log_warn!(
                    "[aliases] user alias row rejected ({}) -- the existing surface wins",
                    e
                );
            }
        }
    }
}
