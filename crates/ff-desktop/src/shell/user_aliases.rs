//! User_Alias_Store data layer (CR-NR-113, command-configurator Requirement 5).
//!
//! A user-scoped, locale-INDEPENDENT alias file, SEPARATE from the shipped
//! per-locale `i18n/<locale>/aliases/` catalogues (localization Req 11). It holds
//! the user's own `[[alias]]` rows -- the surface the Alias Editor Context reads
//! and writes -- under the workbench user data directory
//! (`<User_Data_Dir>/FileForgeWorkbench/aliases/user.toml`, the same base the
//! macros/commands dirs use).
//!
//! This module owns ONLY the DATA concerns (Req 5.1, 5.4, 5.12):
//!   - [`AliasEnvironment`] -- the per-row target Command Environment selector.
//!   - [`UserAliasRowToml`] -- the NEW row type, DISTINCT from the 3-field
//!     `AliasRowToml` in `alias_reader.rs` (which has no `environment` and would
//!     silently drop it); the shipped `i18n/` layout is untouched.
//!   - [`parse_user_alias_toml`] -- read the user file into rows.
//!   - [`serialise_user_aliases`] -- the NEW rows -> TOML serialiser (none
//!     existed; the alias layer was read-only).
//!
//! The resolver LOAD path (layering these rows onto `LocaleAliasStore` as a third
//! overlay) lives in `alias_reader.rs`; the editor Context + save flow live in
//! `shell/alias_editor.rs`.

use serde::{Deserialize, Serialize};

/// Which Command Environment's `AliasTable` a user row loads into (Req 5.1, 5.7).
/// Defaults to FFCMD (the workbench shorthand case). FFCMD is the environment for
/// an editor-verb root such as EDIT: file-open is a workbench operation, so EDIT
/// is an FFCMD root and `ECOB -> EDIT` is an FFCMD alias (CR-CH-069 Req 21.5;
/// the `validate_accepts_root_valid_in_declared_environment` test accepts
/// `ECOB -> EDIT` under FFCMD). FFEDIT is chosen for an in-buffer editor verb
/// such as FIND / CHANGE. Serialises to the lowercase stable spelling
/// `ffcmd` / `ffedit`.
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

    /// The two selectable environments, for the per-row editor combo (Req 5.3).
    pub const ALL: [AliasEnvironment; 2] = [AliasEnvironment::Ffcmd, AliasEnvironment::Ffedit];

    /// A short label for the editor combo (`ffcmd` / `ffedit`).
    pub fn label(self) -> &'static str {
        match self {
            AliasEnvironment::Ffcmd => "ffcmd",
            AliasEnvironment::Ffedit => "ffedit",
        }
    }
}

/// One `[[alias]]` row in the User_Alias_Store (Req 5.1). DISTINCT from the
/// 3-field `AliasRowToml` (per-locale reader) so the `environment` selector and
/// `command_template` survive a round-trip; the shipped `i18n/` row type is
/// unchanged. The `ffcmd` default `environment`, an empty `command_template`, and
/// the always-blank `localisation_key` are OMITTED on serialise so a plain
/// `ffcmd` pure-verb alias round-trips to the minimal `surface`/`canonical` shape.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct UserAliasRowToml {
    /// The alias name (the typeable surface, e.g. `ECOB`). Required.
    pub surface: String,
    /// The root (canonical) command (e.g. `EDIT`). Required.
    pub canonical: String,
    /// The target Command Environment the row loads into (default `ffcmd`).
    #[serde(default, skip_serializing_if = "is_default_env")]
    pub environment: AliasEnvironment,
    /// CR-NR-112 optional parameter template (e.g. `FFWB.COBOL.MASTER(&1)`).
    /// Empty -> a pure verb alias (omitted on serialise).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub command_template: String,
    /// Always blank for a user shorthand (a user alias is language-independent);
    /// retained in the schema so an out-of-band hand edit round-trips.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub localisation_key: String,
}

/// `skip_serializing_if` predicate: the `ffcmd` default environment is omitted.
fn is_default_env(env: &AliasEnvironment) -> bool {
    *env == AliasEnvironment::default()
}

/// The top-level `[[alias]]` array wrapper for the user file.
#[derive(Debug, Default, Deserialize, Serialize)]
struct UserAliasFileToml {
    #[serde(default)]
    alias: Vec<UserAliasRowToml>,
}

/// Resolve the User_Alias_Store file path (Req 5.1):
/// `<User_Data_Dir>/FileForgeWorkbench/aliases/user.toml`, the same user-data
/// base the macros/commands dirs use (`dirs::data_dir()/FileForgeWorkbench`).
/// The `FFWB_USER_ALIASES_PATH` environment variable overrides it verbatim -- a
/// test-isolation seam (mirroring `FFWB_HISTORY_PATH`) so tests never read or
/// write the developer's real user alias store. Returns `None` when no base can
/// be resolved.
///
/// Validates: command-configurator Requirement 5.1
pub fn user_aliases_file() -> Option<std::path::PathBuf> {
    if let Some(p) = std::env::var_os("FFWB_USER_ALIASES_PATH") {
        return Some(std::path::PathBuf::from(p));
    }
    dirs::data_dir().map(|base| {
        base.join("FileForgeWorkbench")
            .join("aliases")
            .join("user.toml")
    })
}

/// Parse the User_Alias_Store TOML text into rows (Req 5.1). An absent file is
/// handled by the caller (empty store); here a malformed file is a parse error.
/// The NEW parser -- NOT the 3-field `parse_alias_toml`, which drops
/// `environment` (Req 5.7 routing would be impossible).
///
/// Validates: command-configurator Requirement 5.1, 5.7
pub fn parse_user_alias_toml(text: &str) -> Result<Vec<UserAliasRowToml>, String> {
    let parsed: UserAliasFileToml = toml::from_str(text).map_err(|e| e.to_string())?;
    Ok(parsed.alias)
}

/// Serialise `rows` to the User_Alias_Store `[[alias]]` TOML text (Req 5.4). The
/// NEW serialiser; none existed (the alias layer was read-only). Emits `surface`,
/// `canonical`, the `environment` ONLY when non-default, and `command_template`
/// ONLY when non-empty, so a plain `ffcmd` pure-verb alias round-trips to the
/// minimal shape. Field VALUES are DATA (any valid UTF-8), structural keys ASCII
/// (Req 5.12).
///
/// Validates: command-configurator Requirement 5.4, 5.12
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

    // Validates: command-configurator Requirement 5.1 -- an absent/empty file
    // parses to an empty row set (the caller treats an absent file as empty).
    #[test]
    fn empty_text_parses_to_no_rows() {
        let rows = parse_user_alias_toml("").expect("empty parses");
        assert!(rows.is_empty(), "empty user store has no rows");
    }

    // Validates: command-configurator Requirement 5.1, 5.7 -- a row WITHOUT an
    // `environment` field defaults to ffcmd (routing default).
    #[test]
    fn row_without_environment_defaults_to_ffcmd() {
        let rows = parse_user_alias_toml(
            r#"
[[alias]]
surface = "ECOB"
canonical = "EDIT"
"#,
        )
        .expect("parses");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].environment, AliasEnvironment::Ffcmd);
        assert_eq!(rows[0].command_template, "");
    }

    // Validates: command-configurator Requirement 5.1 -- an explicit
    // `environment = "ffedit"` row parses to Ffedit.
    #[test]
    fn row_with_ffedit_environment_parses() {
        let rows = parse_user_alias_toml(
            r#"
[[alias]]
surface = "ECOB"
canonical = "EDIT"
environment = "ffedit"
command_template = "FFWB.COBOL.MASTER(&1)"
"#,
        )
        .expect("parses");
        assert_eq!(rows[0].environment, AliasEnvironment::Ffedit);
        assert_eq!(rows[0].command_template, "FFWB.COBOL.MASTER(&1)");
    }

    // Validates: command-configurator Requirement 5.4 -- a pure ffcmd verb row
    // serialises to the minimal shape: NO environment, NO command_template, NO
    // localisation_key keys emitted.
    #[test]
    fn pure_ffcmd_row_serialises_to_minimal_shape() {
        let rows = vec![UserAliasRowToml {
            surface: "GG".to_string(),
            canonical: "GSEARCH".to_string(),
            environment: AliasEnvironment::Ffcmd,
            command_template: String::new(),
            localisation_key: String::new(),
        }];
        let text = serialise_user_aliases(&rows).expect("serialises");
        assert!(text.contains("surface = \"GG\""), "surface emitted: {text}");
        assert!(
            text.contains("canonical = \"GSEARCH\""),
            "canonical emitted: {text}"
        );
        assert!(
            !text.contains("environment"),
            "the ffcmd default environment is omitted: {text}"
        );
        assert!(
            !text.contains("command_template"),
            "an empty template is omitted: {text}"
        );
        assert!(
            !text.contains("localisation_key"),
            "a blank localisation_key is omitted: {text}"
        );
    }

    // Validates: command-configurator Requirement 5.1, 5.4 -- a non-default
    // environment and a template round-trip (emitted and read back) through the
    // NEW parser/serialiser; a pure ffcmd row round-trips to ffcmd with an empty
    // template.
    #[test]
    fn rows_round_trip_through_serialise_then_parse() {
        let rows = vec![
            UserAliasRowToml {
                surface: "ECOB".to_string(),
                canonical: "EDIT".to_string(),
                environment: AliasEnvironment::Ffedit,
                command_template: "FFWB.COBOL.MASTER(&1)".to_string(),
                localisation_key: String::new(),
            },
            UserAliasRowToml {
                surface: "GG".to_string(),
                canonical: "GSEARCH".to_string(),
                environment: AliasEnvironment::Ffcmd,
                command_template: String::new(),
                localisation_key: String::new(),
            },
        ];
        let text = serialise_user_aliases(&rows).expect("serialises");
        let back = parse_user_alias_toml(&text).expect("parses back");
        assert_eq!(
            back, rows,
            "rows round-trip through the NEW parser/serialiser"
        );
    }

    // Validates: command-configurator Requirement 5.12 -- a field VALUE may be
    // non-ASCII UTF-8 (user data) and round-trips verbatim.
    #[test]
    fn non_ascii_field_value_round_trips() {
        let rows = vec![UserAliasRowToml {
            surface: "CAFE".to_string(),
            canonical: "EDIT".to_string(),
            environment: AliasEnvironment::Ffedit,
            command_template: "caf\u{00e9}.txt".to_string(),
            localisation_key: String::new(),
        }];
        let text = serialise_user_aliases(&rows).expect("serialises");
        let back = parse_user_alias_toml(&text).expect("parses back");
        assert_eq!(back[0].command_template, "caf\u{00e9}.txt");
    }
}
