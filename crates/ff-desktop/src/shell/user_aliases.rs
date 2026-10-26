//! User_Locale_Override_Store data layer (CR-CH-072, command-configurator Req 6;
//! supersedes the CR-NR-113 flat `user.toml` user-store scope of Req 5.11).
//!
//! CR-CH-072 collapses the former two-store / three-overlay alias model to ONE
//! locale-aware store per environment: the shipped read-only SEED
//! (`i18n/<locale>/aliases/<ENV>.toml`) OVERLAID by a USER per-locale override
//! (`<User_Data_Dir>/FileForgeWorkbench/aliases/<locale>/<ENV>.toml`, editable).
//! This module owns ONLY the DATA concerns of that USER override store:
//!   - [`AliasEnvironment`] -- the per-row target Command Environment selector,
//!     used by the Alias Editor to group rows into their `<ENV>.toml` file.
//!   - [`UserAliasRowToml`] -- one override row `(surface, canonical,
//!     command_template?)`. CR-CH-072 (localization Req 13.1) REMOVES the former
//!     `localisation_key` command-surface field: the surface IS the display text.
//!     The per-row `environment` is NOT persisted inside the file -- it is
//!     derived from the `<ENV>.toml` file NAME (localization Req 13.5) -- but is
//!     carried on the in-memory row so the editor can group rows per file.
//!   - [`parse_user_alias_toml`] -- read ONE environment override file's rows.
//!   - [`serialise_user_aliases`] -- the rows -> TOML serialiser for ONE file.
//!   - path resolvers for the per-locale override base / file.
//!
//! The resolver LOAD path (layering these rows onto `LocaleAliasStore` as the
//! single user overlay) lives in `alias_reader.rs`; the editor Context + save
//! flow live in `shell/alias_editor.rs`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Which Command Environment's `AliasTable` a user override row loads into
/// (localization Req 13.5, command-configurator Req 6.2). Also selects the
/// `<ENV>.toml` override FILE the Alias Editor groups the row into on save.
/// Defaults to FFCMD (the workbench shorthand case). FFCMD is the environment
/// for an editor-verb root such as EDIT (file-open is a workbench operation);
/// FFEDIT is chosen for an in-buffer editor verb such as FIND / CHANGE.
/// Serialises to the lowercase stable spelling `ffcmd` / `ffedit`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AliasEnvironment {
    /// The FFCMD base environment (the default).
    #[default]
    Ffcmd,
    /// The FFEDIT editor environment.
    Ffedit,
}

impl AliasEnvironment {
    /// The stable environment NAME this row routes into, matching the
    /// `environment_registry` constants (`FFCMD`/`FFEDIT`).
    pub fn env_name(self) -> &'static str {
        match self {
            AliasEnvironment::Ffcmd => super::environment_registry::FFCMD_NAME,
            AliasEnvironment::Ffedit => super::environment_registry::FFEDIT_NAME,
        }
    }

    /// Map a stable environment NAME (`FFCMD`/`FFEDIT`) back to the selector.
    /// Unknown names default to `Ffcmd` (the override base), so a stray file name
    /// never drops a row.
    pub fn from_env_name(name: &str) -> Self {
        if name.eq_ignore_ascii_case(super::environment_registry::FFEDIT_NAME) {
            AliasEnvironment::Ffedit
        } else {
            AliasEnvironment::Ffcmd
        }
    }

    /// The two selectable environments, for the per-row editor combo (Req 6.2).
    pub const ALL: [AliasEnvironment; 2] = [AliasEnvironment::Ffcmd, AliasEnvironment::Ffedit];

    /// A short label for the editor combo (`ffcmd` / `ffedit`).
    pub fn label(self) -> &'static str {
        match self {
            AliasEnvironment::Ffcmd => "ffcmd",
            AliasEnvironment::Ffedit => "ffedit",
        }
    }
}

/// One `[[alias]]` row in a User_Locale_Override_Store `<ENV>.toml` file
/// (CR-CH-072, localization Req 13.1, 13.5). The file shape mirrors the shipped
/// seed (`AliasRowToml` in `alias_reader.rs`): `surface`, `canonical`, and the
/// optional CR-NR-112 `command_template`. There is NO `localisation_key` (the
/// surface IS the display text) and NO per-row `environment` key (the ENV is the
/// file name); `environment` is an in-memory grouping field only, skipped on
/// serialise.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct UserAliasRowToml {
    /// The alias name (the typeable In_Row_Surface, e.g. `ECOB`). Required.
    pub surface: String,
    /// The root (canonical) command (e.g. `EDIT`). Required.
    pub canonical: String,
    /// The target Command Environment the row loads into (default `ffcmd`).
    /// NOT persisted in the file: it is the `<ENV>.toml` file name. Carried on
    /// the in-memory row so the editor can group rows per override file.
    #[serde(skip)]
    pub environment: AliasEnvironment,
    /// CR-NR-112 optional parameter template (e.g. `FFWB.COBOL.MASTER(&1)`).
    /// Empty -> a pure verb alias (omitted on serialise).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub command_template: String,
}

/// The top-level `[[alias]]` array wrapper for one override file.
#[derive(Debug, Default, Deserialize, Serialize)]
struct UserAliasFileToml {
    #[serde(default)]
    alias: Vec<UserAliasRowToml>,
}

/// Resolve the User_Locale_Override_Store BASE directory (localization Req 13.5):
/// `<User_Data_Dir>/FileForgeWorkbench/aliases/`, the same user-data base the
/// macros/commands dirs use (`dirs::data_dir()/FileForgeWorkbench`). The
/// `FFWB_USER_ALIASES_DIR` environment variable overrides it verbatim -- a
/// test-isolation seam (mirroring `FFWB_HISTORY_PATH`) so tests never read or
/// write the developer's real override store. Returns `None` when no base can be
/// resolved.
///
/// Validates: localization Requirement 13.5
pub fn user_alias_override_base() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("FFWB_USER_ALIASES_DIR") {
        return Some(PathBuf::from(p));
    }
    dirs::data_dir().map(|base| base.join("FileForgeWorkbench").join("aliases"))
}

/// The User_Locale_Override_Store FILE for a locale + environment under `base`:
/// `<base>/<locale>/<ENV>.toml` (localization Req 13.5). The ENV is encoded in
/// the file name so a row needs no per-row environment key.
pub fn user_alias_override_file(base: &Path, locale: &str, env_name: &str) -> PathBuf {
    base.join(locale).join(format!("{env_name}.toml"))
}

/// Parse ONE environment override file's TOML text into rows (Req 13.5). An
/// absent file is handled by the caller (empty override); here a malformed file
/// is a parse error. The parsed rows default `environment` to `Ffcmd`; the
/// caller sets it from the file name.
///
/// Validates: command-configurator Requirement 6.1, 6.3
pub fn parse_user_alias_toml(text: &str) -> Result<Vec<UserAliasRowToml>, String> {
    let parsed: UserAliasFileToml = toml::from_str(text).map_err(|e| e.to_string())?;
    Ok(parsed.alias)
}

/// Serialise `rows` to ONE override file's `[[alias]]` TOML text (Req 6.3). Emits
/// `surface`, `canonical`, and `command_template` ONLY when non-empty, so a pure
/// verb alias round-trips to the minimal `surface`/`canonical` shape. The
/// `environment` is NOT emitted (it is the file name). Field VALUES are DATA (any
/// valid UTF-8); structural keys ASCII.
///
/// Validates: command-configurator Requirement 6.3
pub fn serialise_user_aliases(rows: &[UserAliasRowToml]) -> Result<String, String> {
    let file = UserAliasFileToml {
        alias: rows.to_vec(),
    };
    toml::to_string_pretty(&file).map_err(|e| e.to_string())
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: command-configurator Requirement 6.1 -- an absent/empty file
    // parses to an empty row set (the caller treats an absent file as empty).
    #[test]
    fn empty_text_parses_to_no_rows() {
        let rows = parse_user_alias_toml("").expect("empty parses");
        assert!(rows.is_empty(), "empty override file has no rows");
    }

    // Validates: localization Requirement 13.5 -- the per-locale override file
    // path is <base>/<locale>/<ENV>.toml, ENV encoded in the file name.
    #[test]
    fn override_file_path_is_base_locale_env_toml() {
        let base = Path::new("/data/aliases");
        let p = user_alias_override_file(base, "af", "FFCMD");
        assert!(
            p.ends_with("af/FFCMD.toml") || p.ends_with("af\\FFCMD.toml"),
            "got {p:?}"
        );
    }

    // Validates: command-configurator Requirement 6.3 -- CR-CH-072: a row has NO
    // localisation_key field; the in-row surface IS the display text, parsed as
    // surface/canonical/command_template only.
    #[test]
    fn row_parses_surface_canonical_template_without_l18n() {
        // A stale localisation_key in an old file is tolerated and ignored.
        let rows = parse_user_alias_toml(
            r#"
[[alias]]
surface = "ECOB"
canonical = "EDIT"
localisation_key = "verb-edit"
command_template = "FFWB.COBOL.MASTER(&1)"
"#,
        )
        .expect("parses");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].surface, "ECOB");
        assert_eq!(rows[0].canonical, "EDIT");
        assert_eq!(rows[0].command_template, "FFWB.COBOL.MASTER(&1)");
        // environment defaults to Ffcmd (the caller sets it from the file name).
        assert_eq!(rows[0].environment, AliasEnvironment::Ffcmd);
    }

    // Validates: command-configurator Requirement 6.3 -- a pure verb row
    // serialises to the minimal shape: NO command_template, NO environment, NO
    // localisation_key keys emitted.
    #[test]
    fn pure_row_serialises_to_minimal_shape() {
        let rows = vec![UserAliasRowToml {
            surface: "GG".to_string(),
            canonical: "GSEARCH".to_string(),
            environment: AliasEnvironment::Ffcmd,
            command_template: String::new(),
        }];
        let text = serialise_user_aliases(&rows).expect("serialises");
        assert!(text.contains("surface = \"GG\""), "surface emitted: {text}");
        assert!(
            text.contains("canonical = \"GSEARCH\""),
            "canonical emitted: {text}"
        );
        assert!(
            !text.contains("command_template"),
            "an empty template is omitted: {text}"
        );
        assert!(
            !text.contains("environment"),
            "the environment is the file name, never emitted: {text}"
        );
        assert!(
            !text.contains("localisation_key"),
            "CR-CH-072: there is no localisation_key field: {text}"
        );
    }

    // Validates: command-configurator Requirement 6.3 -- a template row
    // round-trips (emitted and read back) through the serialiser/parser.
    #[test]
    fn rows_round_trip_through_serialise_then_parse() {
        let rows = vec![
            UserAliasRowToml {
                surface: "ECOB".to_string(),
                canonical: "EDIT".to_string(),
                environment: AliasEnvironment::Ffcmd,
                command_template: "FFWB.COBOL.MASTER(&1)".to_string(),
            },
            UserAliasRowToml {
                surface: "GG".to_string(),
                canonical: "GSEARCH".to_string(),
                environment: AliasEnvironment::Ffcmd,
                command_template: String::new(),
            },
        ];
        let text = serialise_user_aliases(&rows).expect("serialises");
        let back = parse_user_alias_toml(&text).expect("parses back");
        assert_eq!(
            back, rows,
            "rows round-trip through the override serialiser/parser"
        );
    }

    // Validates: command-configurator Requirement 6.3 -- a field VALUE may be
    // non-ASCII UTF-8 (user data) and round-trips verbatim.
    #[test]
    fn non_ascii_field_value_round_trips() {
        let rows = vec![UserAliasRowToml {
            surface: "CAFE".to_string(),
            canonical: "EDIT".to_string(),
            environment: AliasEnvironment::Ffcmd,
            command_template: "caf\u{00e9}.txt".to_string(),
        }];
        let text = serialise_user_aliases(&rows).expect("serialises");
        let back = parse_user_alias_toml(&text).expect("parses back");
        assert_eq!(back[0].command_template, "caf\u{00e9}.txt");
    }

    // Validates: localization Requirement 13.5 -- from_env_name maps the stable
    // name back to the selector; an unknown name defaults to Ffcmd.
    #[test]
    fn from_env_name_maps_back_to_selector() {
        assert_eq!(
            AliasEnvironment::from_env_name(super::super::environment_registry::FFEDIT_NAME),
            AliasEnvironment::Ffedit
        );
        assert_eq!(
            AliasEnvironment::from_env_name(super::super::environment_registry::FFCMD_NAME),
            AliasEnvironment::Ffcmd
        );
        assert_eq!(
            AliasEnvironment::from_env_name("UNKNOWN"),
            AliasEnvironment::Ffcmd
        );
    }
}
