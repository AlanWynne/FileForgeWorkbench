//! # On-disk per-locale, per-environment alias-catalogue reader + the
//! shell-owned `LocaleAliasStore` (CR-NR-109, localization Requirement 11.2,
//! 11.3, 11.5, 11.10).
//!
//! This module is the first PRODUCTION caller of the locale-overlay machinery in
//! `shell/alias_overlay.rs` + `shell/environment.rs`. It parses per-locale,
//! per-environment alias DATA from
//! `i18n/<locale>/aliases/<environment>.toml` into an in-memory
//! [`super::alias_overlay::AliasCatalogue`] (carrying the per-row optional
//! localisation key, Req 11.1), then layers each environment's rows onto that
//! environment's `AliasTable` via the EXISTING
//! `AliasTable::apply_locale_overlay_keyed` seam (no new overlay mechanism,
//! Req 11.5).
//!
//! The data shape mirrors the help/`.ftl` "content is data" posture:
//!
//! ```toml
//! # i18n/<locale>/aliases/<environment>.toml -- one [[alias]] per surface form
//! [[alias]]
//! surface = "SOEK"
//! canonical = "FIND"
//! localisation_key = "verb-find"   # optional
//! ```
//!
//! A within-environment collision is rejected by the overlay's existing
//! discipline; the reader logs a WARN and retains the Identity_Base for THAT
//! environment, while other environments still load (Req 11.10).
//!
//! No new persistence format and no new dispatch path: the ROOT (canonical)
//! command stays the dispatched/persisted identity (Req 11.9). With the default
//! `en` locale and no `i18n/en/aliases/` tree, the store is identity-only and
//! behaviour-neutral (Req 10.2).

use std::path::Path;

use serde::Deserialize;

use super::alias_overlay::{AliasCatalogue, KeyedAliasRow};
use super::environment::AliasTable;
use super::environment_registry::{FFCMD_NAME, FFEDIT_NAME};

/// CR-NR-113 User_Alias_Store overlay methods for [`LocaleAliasStore`], split
/// into a child module so `alias_reader.rs` stays within the 400-line source
/// limit (rust-standards.md). The `impl` block lives there; no behaviour change.
mod alias_user_overlay;

/// One `[[alias]]` row as deserialised from a per-environment TOML file.
#[derive(Debug, Deserialize)]
struct AliasRowToml {
    /// The typeable/displayed surface form (uppercased when loaded).
    surface: String,
    /// The root (canonical) verb this surface resolves to.
    canonical: String,
    /// Optional `ff-i18n` key resolving the displayed surface per locale.
    #[serde(default)]
    localisation_key: Option<String>,
    /// CR-NR-112: optional parameter template (e.g. `FFWB.COBOL.MASTER(&1)`).
    /// Absent/empty keeps today's pure verb-rewrite behaviour (Req 14.6). The
    /// template VALUE is preserved verbatim (it is DATA, not ASCII-scoped).
    #[serde(default)]
    command_template: Option<String>,
}

/// The parsed content of one `aliases/<environment>.toml` file.
#[derive(Debug, Deserialize)]
struct AliasFileToml {
    /// Zero or more alias rows (the `[[alias]]` array).
    #[serde(default)]
    alias: Vec<AliasRowToml>,
}

/// Parse one environment's alias-catalogue TOML text into keyed overlay rows
/// `(surface-UPPERCASED, canonical-UPPERCASED, localisation_key?)`.
///
/// Surfaces and canonicals are uppercased to match the `AliasTable`
/// case-insensitive convention; the localisation key is preserved verbatim.
/// Returns a parse error string on malformed TOML.
///
/// Validates: localization Requirement 11.5
fn parse_alias_toml(text: &str) -> Result<Vec<KeyedAliasRow>, String> {
    let parsed: AliasFileToml = toml::from_str(text).map_err(|e| e.to_string())?;
    Ok(parsed
        .alias
        .into_iter()
        .map(|r| {
            (
                r.surface.trim().to_uppercase(),
                r.canonical.trim().to_uppercase(),
                r.localisation_key
                    .map(|k| k.trim().to_string())
                    .filter(|k| !k.is_empty()),
                // CR-NR-112: the template is preserved VERBATIM (data, not
                // uppercased), with empty mapped to None so a blank/absent field
                // keeps the pure verb-alias path (Req 14.1, 14.6).
                r.command_template
                    .map(|t| t.trim().to_string())
                    .filter(|t| !t.is_empty()),
            )
        })
        .collect())
}

/// Read the per-environment alias files under `i18n/<locale>/aliases/` into a
/// keyed [`AliasCatalogue`]. For each environment NAME in `environments`, read
/// `<aliases_dir>/<name>.toml` when present and add its rows under that name.
///
/// A missing file for an environment is NOT an error -- that environment simply
/// contributes no overlay rows (identity-only, Req 10.2). A malformed file logs
/// a WARN and is skipped, so one bad file cannot block the others (Req 11.10).
///
/// Validates: localization Requirement 11.5
fn read_catalogue_from_dir(aliases_dir: &Path, environments: &[&str]) -> AliasCatalogue {
    let mut catalogue = AliasCatalogue::new();
    for &env in environments {
        let path = aliases_dir.join(format!("{env}.toml"));
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(_) => continue, // absent -> no overlay rows for this environment
        };
        match parse_alias_toml(&text) {
            Ok(rows) if !rows.is_empty() => {
                catalogue = catalogue.with_environment_keyed(env, rows);
            }
            Ok(_) => {}
            Err(e) => {
                ff_logging::log_warn!(
                    "[i18n] alias catalogue {} failed to parse ({}) -- \
                     retaining the Identity_Base for environment {}",
                    path.display(),
                    e,
                    env
                );
            }
        }
    }
    catalogue
}

/// The built-in FFCMD root commands: the distinct `command` values of every
/// option in the compiled Recovery_Baseline POM and Settings menus (CR-NR-109,
/// localization Requirement 11.3). These are the roots the FFCMD alias table is
/// SEEDED with so every built-in menu option is resolvable in its source
/// language. Deduplicated, case-insensitively, in first-seen order.
///
/// Built-in menus are the authoritative source of the FFCMD option roots (they
/// are code-only, never written to disk). A user `menus/*.toml` override's roots
/// are still resolvable because the on-disk overlay layers on top; FEAT-002's
/// resolver also falls back to the root command table, so an unseeded root is
/// never unreachable.
///
/// Validates: localization Requirement 11.3
pub(crate) fn builtin_ffcmd_roots() -> Vec<String> {
    let mut roots: Vec<String> = Vec::new();
    let push_root = |root: String, roots: &mut Vec<String>| {
        if !root.is_empty() && !roots.iter().any(|r| r.eq_ignore_ascii_case(&root)) {
            roots.push(root);
        }
    };
    // The two built-in menus' option roots (code-only Recovery_Baseline menus).
    for menu in [
        crate::menu_workspace::defaults::recovery_pom_menu(),
        crate::menu_workspace::defaults::recovery_settings_menu(),
    ] {
        for option in menu.options {
            push_root(option.command.trim().to_uppercase(), &mut roots);
        }
    }
    // PLUS every built-in CustomWorkspace / navigation verb root the front door
    // classifies (CR-NR-109 Req 11.3): COMMANDS, LOG, PLUGINS, MACROS, SEARCH,
    // GSEARCH and the rest are legitimate FFCMD roots that are NOT menu-option
    // roots, so without this union a bare one resolves to Unknown rather than
    // Root (Req 11.6 branch 1.2.1). The list is co-located with the classifier in
    // `command_config` so the two cannot drift.
    for &verb in crate::command_config::BUILTIN_WORKSPACE_VERB_ROOTS {
        push_root(verb.trim().to_uppercase(), &mut roots);
    }
    roots
}

/// The shell-owned, locale-overlaid per-environment alias store (CR-NR-109,
/// localization Requirement 11.2, 11.3). Holds one `AliasTable` per environment
/// name (at least FFCMD and FFEDIT), each SEEDED with the synthesised identity
/// rows for its root commands (Req 11.3) and then overlaid with the active
/// locale's on-disk alias DATA (Req 11.5).
///
/// This is the store the FEAT-002 unified resolver and the menu-kind render read
/// from. It is held as a GROUPED sub-struct field on `WorkbenchShell` (not a flat
/// field), seeded at construction, and re-populated on a `ui.locale` reload via
/// the existing callback seam (mirroring `help::reload_active_help_registry_for_locale`).
#[derive(Debug, Default)]
pub(crate) struct LocaleAliasStore {
    /// `(environment-name, overlaid AliasTable)` pairs. A small Vec: there are
    /// only a couple of environments.
    tables: Vec<(String, AliasTable)>,
}

impl LocaleAliasStore {
    /// Seed the store with identity-only tables for the given environments
    /// BEFORE any on-disk overlay (Req 11.3): FFEDIT starts from the English
    /// base (`AliasTable::ffedit_english`, whose entries are already identity +
    /// the two genuine English aliases), and FFCMD starts from
    /// `AliasTable::identity_for(ffcmd_roots)` so every built-in menu-option root
    /// command (and any registered root verb) is resolvable in its source
    /// language. Behaviour-neutral at `en` with no `aliases/` tree (Req 10.2).
    ///
    /// Validates: localization Requirement 11.3
    pub(crate) fn seed(ffcmd_roots: &[&str]) -> Self {
        Self {
            tables: vec![
                (FFEDIT_NAME.to_string(), AliasTable::ffedit_english()),
                (
                    FFCMD_NAME.to_string(),
                    AliasTable::identity_for(ffcmd_roots),
                ),
            ],
        }
    }

    /// Re-seed and overlay the store for `locale`, resolving the shipped `i18n/`
    /// root via the same search path as the message catalogue
    /// ([`super::i18n_bootstrap::resolve_i18n_root`]). Equivalent to
    /// [`Self::seed`] followed by [`Self::apply_overlay_from_dir`] against
    /// `<root>/<locale>/aliases/`. A missing root (or missing `aliases/` dir)
    /// leaves the store identity-only (Req 10.2, graceful degradation).
    ///
    /// CR-NR-113 (command-configurator Req 5.6): the re-seed is EXTENDED to ALSO
    /// apply the User_Alias_Store overlay as a THIRD layer, so ONE re-seed
    /// re-applies BOTH the per-locale overlay AND the user store -- a save (which
    /// triggers a reload) never drops the per-locale aliases and the newly saved
    /// user alias goes live without a restart.
    ///
    /// Validates: localization Requirement 11.2, 11.5; command-configurator Req 5.6
    pub(crate) fn seed_and_overlay_for_locale(ffcmd_roots: &[&str], locale: &str) -> Self {
        let mut store = Self::seed(ffcmd_roots);
        if let Some(root) = super::i18n_bootstrap::resolve_i18n_root() {
            let aliases_dir = root.join(locale).join("aliases");
            store.apply_overlay_from_dir(&aliases_dir);
        }
        // CR-NR-113: the user store is the THIRD overlay, on top of the identity
        // base and the per-locale overlay (Req 5.7).
        if let Some(path) = super::user_aliases::user_aliases_file() {
            store.apply_user_overlay_from_file(&path);
        }
        store
    }

    /// Overlay the active locale's on-disk alias DATA from `aliases_dir` onto the
    /// seeded tables. For each environment this reads `<aliases_dir>/<env>.toml`
    /// and loads its keyed rows via `AliasCatalogue::load_into_keyed`. A
    /// within-environment collision logs a WARN and retains that environment's
    /// Identity_Base; other environments still load (Req 11.10).
    ///
    /// Validates: localization Requirement 11.5, 11.10
    pub(crate) fn apply_overlay_from_dir(&mut self, aliases_dir: &Path) {
        let names: Vec<&str> = self.tables.iter().map(|(n, _)| n.as_str()).collect();
        let catalogue = read_catalogue_from_dir(aliases_dir, &names);
        for (name, table) in self.tables.iter_mut() {
            if let Err(e) = catalogue.load_into_keyed(name, table) {
                ff_logging::log_warn!(
                    "[i18n] alias overlay rejected for environment {} ({}) -- \
                     retaining the Identity_Base",
                    name,
                    e
                );
            }
        }
    }

    /// The overlaid `AliasTable` for `environment_name` (case-sensitive stable
    /// name), or `None` when the store holds no table for it. The FEAT-002
    /// resolver reads this to resolve a locale surface to its root command.
    pub(crate) fn table_for(&self, environment_name: &str) -> Option<&AliasTable> {
        self.tables
            .iter()
            .find(|(n, _)| n == environment_name)
            .map(|(_, t)| t)
    }

    /// Test-only: overlay a single `(surface -> canonical)` row (with the
    /// optional `verb-find` localisation key) onto `environment_name`'s table,
    /// so a test can simulate an active-locale alias without writing a TOML file.
    /// Panics on a within-environment collision (a test authoring error).
    #[cfg(test)]
    pub(crate) fn overlay_row_for_test(
        &mut self,
        environment_name: &str,
        surface: &str,
        canonical: &str,
    ) {
        self.overlay_row_for_test_with_template(environment_name, surface, canonical, None);
    }

    /// Test-only: like [`Self::overlay_row_for_test`] but with an explicit
    /// optional parameter template (CR-NR-112), so a test can simulate an
    /// active-locale PARAMETERISED alias without writing a TOML file. Panics on a
    /// within-environment collision (a test authoring error).
    #[cfg(test)]
    pub(crate) fn overlay_row_for_test_with_template(
        &mut self,
        environment_name: &str,
        surface: &str,
        canonical: &str,
        template: Option<&str>,
    ) {
        if let Some((_, table)) = self.tables.iter_mut().find(|(n, _)| n == environment_name) {
            table
                .apply_locale_overlay_keyed(
                    environment_name,
                    &[(
                        surface.to_uppercase(),
                        canonical.to_uppercase(),
                        Some("verb-find".to_string()),
                        template.map(str::to_string),
                    )],
                )
                .expect("test overlay row must not collide");
        }
    }

    /// The FFCMD root commands this store was seeded with (the built-in
    /// menu-option roots plus registered root verbs), read back from the FFCMD
    /// table's identity rows so a reload re-seeds with the SAME root set without
    /// the caller re-deriving it.
    pub(crate) fn ffcmd_roots(&self) -> Vec<String> {
        self.table_for(FFCMD_NAME)
            .map(AliasTable::canonical_verbs_owned)
            .unwrap_or_default()
    }
}

/// The process-global slot holding a CLONE of the shell's alias-store `Arc`, so
/// the `ui.locale` reload closure (registered in `main.rs` off the shell) can
/// re-seed+overlay the SAME store the shell reads from -- mirroring
/// `help.rs::ACTIVE_HELP_REGISTRY`. `None` until the shell publishes it during
/// construction; the reload is then a no-op until published (startup order).
static ACTIVE_ALIAS_STORE: std::sync::Mutex<
    Option<std::sync::Arc<std::sync::Mutex<LocaleAliasStore>>>,
> = std::sync::Mutex::new(None);

/// Publish the shell-owned alias-store `Arc` to the process-global reload slot
/// (CR-NR-109). Called once during shell construction so the `ui.locale` reload
/// closure can reach the live store. Mirrors `help::build_help_registry_for_locale`'s
/// publish step.
pub(crate) fn publish_active_alias_store(
    store: &std::sync::Arc<std::sync::Mutex<LocaleAliasStore>>,
) {
    if let Ok(mut slot) = ACTIVE_ALIAS_STORE.lock() {
        *slot = Some(store.clone());
    }
}

/// Re-seed and overlay the published alias store for `locale` (CR-NR-109,
/// localization Req 11.2). Called from the `ui.locale` hot-reload closure in
/// `main.rs`. Re-reads the store's existing FFCMD root set so the re-seed keeps
/// the same roots, then applies the new locale's on-disk overlay IN PLACE, so
/// every holder of the `Arc` (the shell) observes the swap. A no-op when no store
/// has been published yet (the shell is not built), degrading safely.
///
/// Validates: localization Requirement 11.2, 11.5
pub(crate) fn reload_active_alias_store_for_locale(locale: &str) {
    let store = match ACTIVE_ALIAS_STORE.lock() {
        Ok(slot) => slot.clone(),
        Err(_) => return,
    };
    let Some(store) = store else { return };
    let Ok(mut guard) = store.lock() else { return };
    let roots = guard.ffcmd_roots();
    let root_refs: Vec<&str> = roots.iter().map(String::as_str).collect();
    *guard = LocaleAliasStore::seed_and_overlay_for_locale(&root_refs, locale);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Write `<dir>/aliases/<env>.toml` with `text`, creating the aliases dir.
    fn write_alias_file(dir: &Path, env: &str, text: &str) {
        let aliases = dir.join("aliases");
        fs::create_dir_all(&aliases).expect("create aliases dir");
        fs::write(aliases.join(format!("{env}.toml")), text).expect("write alias toml");
    }

    /// Validates: localization Requirement 11.5 -- the reader parses `[[alias]]`
    /// TOML rows (one keyed, one plain) into the keyed catalogue with the key
    /// preserved; loading them into the ffedit_english base makes the localised
    /// surface resolve to its root canonical.
    #[test]
    fn reader_parses_toml_rows_into_catalogue() {
        let dir = TempDir::new().unwrap();
        write_alias_file(
            dir.path(),
            FFEDIT_NAME,
            r#"
[[alias]]
surface = "SOEK"
canonical = "FIND"
localisation_key = "verb-find"

[[alias]]
surface = "VERANDER"
canonical = "CHANGE"
"#,
        );

        let catalogue =
            read_catalogue_from_dir(&dir.path().join("aliases"), &[FFEDIT_NAME, FFCMD_NAME]);
        let rows = catalogue
            .keyed_rows_for(FFEDIT_NAME)
            .expect("FFEDIT rows present");
        assert_eq!(rows.len(), 2);
        // Keyed row preserved with its localisation key (no template).
        assert!(rows.contains(&(
            "SOEK".to_string(),
            "FIND".to_string(),
            Some("verb-find".to_string()),
            None
        )));
        // Plain row has no key and no template.
        assert!(rows.contains(&("VERANDER".to_string(), "CHANGE".to_string(), None, None)));

        // Loading into the English base makes the localised surfaces resolve to
        // their root canonicals (one source of truth, Req 11.2).
        let mut table = AliasTable::ffedit_english();
        catalogue
            .load_into_keyed(FFEDIT_NAME, &mut table)
            .expect("keyed overlay loads");
        assert_eq!(table.canonical_verb("SOEK"), Some("FIND"));
        assert_eq!(table.canonical_verb("VERANDER"), Some("CHANGE"));
        // The base English verbs remain resolvable (overlay is a layer).
        assert_eq!(table.canonical_verb("FIND"), Some("FIND"));
    }

    /// Validates: command-semantics Requirement 14.6 -- a row WITHOUT a
    /// `command_template` field parses to template `None` (back-compat: the
    /// pre-CR-NR-112 schema deserialises byte-identically).
    #[test]
    fn reader_row_without_template_parses_to_none() {
        let rows = parse_alias_toml(
            r#"
[[alias]]
surface = "SOEK"
canonical = "FIND"
localisation_key = "verb-find"
"#,
        )
        .expect("toml parses");
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0],
            (
                "SOEK".to_string(),
                "FIND".to_string(),
                Some("verb-find".to_string()),
                None
            ),
            "a templateless row deserialises to command_template = None (Req 14.6)"
        );
    }

    /// Validates: command-semantics Requirement 14.1 -- a row WITH a
    /// `command_template` parses to `Some(template)` with surface/canonical
    /// uppercased and the template preserved VERBATIM (case and punctuation).
    #[test]
    fn reader_row_with_template_parses_to_some_verbatim() {
        let rows = parse_alias_toml(
            r#"
[[alias]]
surface = "ecob"
canonical = "edit"
command_template = "FFWB.COBOL.MASTER(&1)"
"#,
        )
        .expect("toml parses");
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0],
            (
                "ECOB".to_string(),
                "EDIT".to_string(),
                None,
                Some("FFWB.COBOL.MASTER(&1)".to_string())
            ),
            "surface/canonical uppercase; template preserved verbatim (Req 14.1)"
        );
    }

    /// Validates: localization Requirement 11.10 -- a surface mapping to two
    /// different roots within one environment is rejected and the base table is
    /// left UNCHANGED (Identity_Base retained); a separate clean environment in
    /// the same read still loads.
    #[test]
    fn reader_rejects_within_environment_collision_and_retains_base() {
        let dir = TempDir::new().unwrap();
        // FFEDIT: SOEK mapped to two different roots -> within-env collision.
        write_alias_file(
            dir.path(),
            FFEDIT_NAME,
            r#"
[[alias]]
surface = "SOEK"
canonical = "FIND"

[[alias]]
surface = "SOEK"
canonical = "CHANGE"
"#,
        );
        // FFCMD: one clean row.
        write_alias_file(
            dir.path(),
            FFCMD_NAME,
            r#"
[[alias]]
surface = "KATALOGUSSE"
canonical = "CATALOGS"
"#,
        );

        let mut store = LocaleAliasStore::seed(&["CATALOGS", "FILES", "SETTINGS"]);
        store.apply_overlay_from_dir(&dir.path().join("aliases"));

        // FFEDIT overlay rejected: the colliding SOEK resolves to NOTHING and the
        // English base is retained unchanged.
        let ffedit = store.table_for(FFEDIT_NAME).expect("FFEDIT table");
        assert_eq!(
            ffedit.canonical_verb("SOEK"),
            None,
            "a rejected overlay leaves no partial surface resident"
        );
        assert_eq!(
            ffedit.canonical_verb("FIND"),
            Some("FIND"),
            "the FFEDIT Identity_Base is retained"
        );

        // The clean FFCMD environment still loads.
        let ffcmd = store.table_for(FFCMD_NAME).expect("FFCMD table");
        assert_eq!(ffcmd.canonical_verb("KATALOGUSSE"), Some("CATALOGS"));
        assert_eq!(ffcmd.canonical_verb("CATALOGS"), Some("CATALOGS"));
    }

    /// Validates: localization Requirement 11.3, 10.2 -- a seeded store with no
    /// on-disk overlay is identity-only: every FFCMD root resolves to itself and
    /// the FFEDIT base verbs resolve as before (behaviour-neutral at en).
    #[test]
    fn seeded_store_is_identity_only_without_overlay() {
        let store = LocaleAliasStore::seed(&["CATALOGS", "FILES", "SETTINGS"]);
        let ffcmd = store.table_for(FFCMD_NAME).expect("FFCMD table");
        assert_eq!(ffcmd.canonical_verb("CATALOGS"), Some("CATALOGS"));
        assert_eq!(ffcmd.canonical_verb("FILES"), Some("FILES"));
        assert_eq!(ffcmd.canonical_verb("SETTINGS"), Some("SETTINGS"));

        let ffedit = store.table_for(FFEDIT_NAME).expect("FFEDIT table");
        assert_eq!(ffedit.canonical_verb("FIND"), Some("FIND"));
        assert_eq!(ffedit.canonical_verb("X"), Some("EXCLUDE"));
    }

    /// Install the SHIPPED Afrikaans catalogue (English Identity_Base + the `af`
    /// overlay) on the process-wide seam, so `ff_i18n::t("verb-find")` resolves
    /// to the Afrikaans surface "SOEK" authored in `af/ff-desktop-verbs.ftl`.
    /// Builds from the real shipped root so the POC is exercised end-to-end.
    fn install_shipped_af_catalogue(root: &Path) {
        let base = ff_i18n::Catalogue::load_identity_base(root).expect("English base loads");
        let af = ff_i18n::Locale::parse("af").expect("af parses");
        let catalogue = base.load_overlay(root, af).expect("af overlay loads");
        ff_i18n::set_catalogue(catalogue);
    }

    /// Validates: localization Requirement 11.1, 5.1, 5.5 -- the SHIPPED
    /// Afrikaans POC alias catalogue (`af/aliases/FFCMD.toml`) loads via the
    /// reader and, overlaid onto an FFCMD table that carries the FIND root,
    /// resolves the localised surface SOEK to the root FIND (11.1/5.1) while the
    /// root FIND still resolves to itself (Identity_Base retained, 5.5). With the
    /// shipped `af` catalogue installed, the overlaid row's display surface is
    /// `t("verb-find")` = "SOEK" (the minimal verb key the POC rows reference).
    #[test]
    fn af_poc_alias_catalogue_loads_and_resolves() {
        let root = super::super::i18n_bootstrap::resolve_i18n_root()
            .expect("shipped i18n/ root must resolve for the af POC");
        let af_aliases = root.join("af").join("aliases");
        assert!(
            af_aliases.join(format!("{FFCMD_NAME}.toml")).exists(),
            "the af POC must ship {FFCMD_NAME}.toml under {}",
            af_aliases.display()
        );

        // Install the shipped af catalogue so t(verb-find) = SOEK end-to-end.
        install_shipped_af_catalogue(&root);

        // Read the shipped af alias catalogue for FFCMD and load it onto an FFCMD
        // table seeded with the FIND root (the POC's SOEK aliases the FIND verb;
        // the menu-kind FFCMD context surfaces it, mirroring the 11.6 full-shell
        // test's FIND menu option).
        let catalogue = read_catalogue_from_dir(&af_aliases, &[FFCMD_NAME]);
        let rows = catalogue
            .keyed_rows_for(FFCMD_NAME)
            .expect("the af FFCMD catalogue carries rows");
        assert!(
            rows.contains(&(
                "SOEK".to_string(),
                "FIND".to_string(),
                Some("verb-find".to_string()),
                None
            )),
            "the POC ships the SOEK -> FIND row keyed verb-find, got {rows:?}"
        );

        let mut table = AliasTable::identity_for(&["FIND", "CATALOGS", "FILES", "SETTINGS"]);
        catalogue
            .load_into_keyed(FFCMD_NAME, &mut table)
            .expect("the af POC overlay loads cleanly");

        // The Afrikaans surface reaches the root (Req 11.1, 5.1).
        assert_eq!(table.canonical_verb("SOEK"), Some("FIND"));
        // The root is retained and still self-resolves (Identity_Base, Req 5.5).
        assert_eq!(table.canonical_verb("FIND"), Some("FIND"));
        // The overlaid row's display surface is the localisation-key resolution
        // under the active af catalogue.
        assert_eq!(
            table.display_surface_for_canonical("FIND"),
            Some("SOEK".to_string()),
            "the overlaid FIND row displays t(verb-find) = SOEK under the af catalogue"
        );
    }

    /// Validates: localization Requirement 11.5 -- a missing per-environment file
    /// is not an error: the environment contributes no overlay and stays
    /// identity-only, while a present file for another environment still loads.
    #[test]
    fn missing_environment_file_leaves_that_environment_identity_only() {
        let dir = TempDir::new().unwrap();
        // Only FFCMD has a file; FFEDIT has none.
        write_alias_file(
            dir.path(),
            FFCMD_NAME,
            r#"
[[alias]]
surface = "KATALOGUSSE"
canonical = "CATALOGS"
"#,
        );
        let mut store = LocaleAliasStore::seed(&["CATALOGS"]);
        store.apply_overlay_from_dir(&dir.path().join("aliases"));

        assert_eq!(
            store
                .table_for(FFCMD_NAME)
                .unwrap()
                .canonical_verb("KATALOGUSSE"),
            Some("CATALOGS")
        );
        // FFEDIT had no file -> still the identity-only English base.
        assert_eq!(
            store.table_for(FFEDIT_NAME).unwrap().canonical_verb("FIND"),
            Some("FIND")
        );
    }
}
