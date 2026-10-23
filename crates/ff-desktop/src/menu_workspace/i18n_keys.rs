//! Localisation key derivation for built-in menu title and option descriptions
//! (CR-NR-103, localization Req 7.1, 7.2, 7.3, 7.4, 4.2).
//!
//! Built-in menus are CODE-ONLY TOML (menu_workspace/defaults.rs) and are NEVER
//! written to disk in a translated form. Instead the render path localises the
//! menu TITLE and each option DESCRIPTION out of the PARSED MenuFile at render
//! time, keyed by menu-name + option-command, through the `ff_i18n::t()` seam.
//! The option `command` VALUE is only an INPUT to the key -- it is never itself
//! localised (Req 7.3, 4.2). A missing key falls back to the parsed English
//! prose (Req 7.4), so a menu always renders even with no catalogue installed.
//!
//! Key convention (the slug lowercases the input, collapses runs of
//! non-alphanumeric chars to a single `-`, and trims leading/trailing `-`):
//!   - Title:       `menu-<menu-slug>-title`               e.g. `menu-pom-title`
//!   - Description: `menu-<menu-slug>-opt-<command-slug>-desc`, e.g.
//!     `menu-settings-opt-reset-bare-desc`

use super::MenuOption;

/// Resolve `key` through the i18n seam, falling back to `fallback` on a miss.
///
/// The seam returns the KEY itself when the key is absent in every locale (and
/// when no catalogue is installed yet), so an unchanged result equal to `key`
/// is the miss sentinel. On a miss the parsed English `fallback` is returned, so
/// the menu always renders (localization Req 7.4, 3.2).
pub(crate) fn t_or(key: &str, fallback: &str) -> String {
    let resolved = ff_i18n::t(key);
    if resolved == key {
        fallback.to_string()
    } else {
        resolved
    }
}

/// Lowercase `input`, collapse every run of non-alphanumeric characters to a
/// single `-`, and trim leading/trailing `-`. Pure and allocation-light.
///
/// Examples: `"Reset bare"` -> `"reset-bare"`; `"Reset  bare!"` -> `"reset-bare"`;
/// `"POM"` -> `"pom"`.
fn slug(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut pending_dash = false;
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.push(ch.to_ascii_lowercase());
        } else {
            // Any non-alphanumeric char starts/extends a single-dash run.
            pending_dash = true;
        }
    }
    out
}

/// The catalogue key for a menu's TITLE: `menu-<menu-slug>-title`.
///
/// Validates: localization Requirement 7.1
pub(crate) fn menu_title_key(menu_name: &str) -> String {
    format!("menu-{}-title", slug(menu_name))
}

/// The catalogue key for one option's DESCRIPTION:
/// `menu-<menu-slug>-opt-<command-slug>-desc`. The `command` value is only an
/// input to the key; it is never itself localised (Req 7.3, 4.2).
///
/// Validates: localization Requirement 7.1
pub(crate) fn menu_option_desc_key(menu_name: &str, command: &str) -> String {
    format!("menu-{}-opt-{}-desc", slug(menu_name), slug(command))
}

/// The localised one-line description for `option` in menu `menu_name`: the
/// catalogue value when the key is present, else the parsed English prose
/// (`option.description`). This is the single display helper both render sites
/// call, so no localisation logic lives in render.rs (Req 7.3, 7.4).
///
/// Validates: localization Requirement 7.1, 7.4
pub(crate) fn localised_option_description(menu_name: &str, option: &MenuOption) -> String {
    t_or(
        &menu_option_desc_key(menu_name, &option.command),
        &option.description,
    )
}

/// The DISPLAYED command-column text for `option` under the active locale
/// (CR-NR-109 Task 11.5, localization Req 11.7, 11.8): the alias SURFACE for the
/// option's ROOT command in the active locale when one exists, else the raw root
/// command string.
///
/// `display_surface_source` is the result of looking the option's ROOT command
/// up in the active environment's active-locale `AliasTable` via
/// `display_surface_for_canonical` (computed SHELL-side, where `&self` and the
/// alias store are available, so this menu-crate helper stays free of shell
/// coupling). It is `Some(surface)` when the environment's table has a row for
/// that root -- where the row carries a localisation key the surface is already
/// the `t(key)` resolution; where it does not, the surface is the raw root, so
/// either way a present source is the right display text. It is `None` when the
/// root is not in the table at all, in which case the raw `option.command` (the
/// ROOT) is shown (Req 11.8 fallback). A blank source also falls back, so a
/// mis-seeded empty surface never blanks the column.
///
/// The returned text is DISPLAY-ONLY: selecting the option still carries the
/// ROOT `option.command` through the 11.6 resolver (Req 11.9), so nothing about
/// dispatch or persistence changes.
///
/// Validates: localization Requirement 11.7, 11.8
pub(crate) fn localised_option_command(
    option: &MenuOption,
    display_surface_source: Option<&str>,
) -> String {
    match display_surface_source {
        Some(surface) if !surface.trim().is_empty() => surface.to_string(),
        _ => option.command.clone(),
    }
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::menu_workspace::MenuOption;

    fn opt(command: &str, description: &str) -> MenuOption {
        MenuOption {
            key: "1".to_string(),
            command: command.to_string(),
            description: description.to_string(),
            enabled: true,
            group: None,
            show_in_menu_bar: true,
            target: None,
        }
    }

    // Validates: Requirement 7.1 -- the title key is menu-<slug>-title.
    #[test]
    fn menu_title_key_builds_menu_slug_title() {
        assert_eq!(menu_title_key("pom"), "menu-pom-title");
        assert_eq!(menu_title_key("Settings"), "menu-settings-title");
    }

    // Validates: Requirement 7.1 -- the description key is
    // menu-<menu-slug>-opt-<command-slug>-desc, command slugged not localised.
    #[test]
    fn menu_option_desc_key_builds_menu_opt_command_desc() {
        assert_eq!(
            menu_option_desc_key("settings", "Reset bare"),
            "menu-settings-opt-reset-bare-desc"
        );
        assert_eq!(
            menu_option_desc_key("pom", "Catalogs"),
            "menu-pom-opt-catalogs-desc"
        );
    }

    // Validates: Requirement 7.1 -- the slug collapses non-alphanumeric runs to
    // a single '-' and trims, so punctuation/repeat-spaces do not leak into keys.
    #[test]
    fn slug_collapses_non_alphanumeric_runs_to_single_dash() {
        assert_eq!(
            menu_option_desc_key("settings", "Reset  bare!"),
            "menu-settings-opt-reset-bare-desc"
        );
        assert_eq!(
            menu_option_desc_key("  pom  ", " Files / Folders "),
            "menu-pom-opt-files-folders-desc"
        );
    }

    // Validates: Requirement 7.4, 3.2 -- t_or returns the parsed English
    // fallback when the key is absent (the seam returns the key = miss sentinel).
    #[test]
    fn t_or_falls_back_to_english_when_key_absent() {
        // A key that cannot exist in any catalogue resolves to itself (miss),
        // so t_or must return the supplied fallback prose unchanged.
        let fallback = "Browse files in a tree view";
        assert_eq!(
            t_or("menu-nonexistent-opt-zzz-desc", fallback),
            fallback.to_string()
        );
    }

    // Validates: Requirement 7.3, 4.2 -- the option command VALUE is only an
    // input to the key; localised_option_description never localises the command
    // itself, and on a miss returns the parsed English description unchanged.
    #[test]
    fn localised_description_falls_back_to_parsed_english_on_miss() {
        let o = opt("Reset bare", "Reset to barebones -- archive config");
        // No catalogue installed in this unit test -> seam returns the key =>
        // t_or falls back to the parsed English description.
        assert_eq!(
            localised_option_description("settings", &o),
            "Reset to barebones -- archive config".to_string()
        );
    }

    // Validates: localization Requirement 11.7, 11.8 -- the command-column helper
    // shows the active-locale alias SURFACE for the option's root when the store
    // supplied one (e.g. af SOEK for a FIND option), and falls back to the raw
    // ROOT command (e.g. CATALOGS) when no alias/surface exists. The option
    // `command` (the ROOT) is unchanged either way -- the surface is display-only.
    #[test]
    fn localised_option_command_resolves_alias_surface_else_root() {
        // A FIND option whose active-locale table resolved to the display surface
        // SOEK (the store looked up display_surface_for_canonical("FIND") and
        // found a row keyed verb-find = SOEK) -> the column shows SOEK.
        let find = opt("FIND", "Find text");
        assert_eq!(
            localised_option_command(&find, Some("SOEK")),
            "SOEK".to_string(),
            "a localised alias surface must be shown in the command column (Req 11.7)"
        );

        // A CATALOGS option with no alias/surface (None) -> the raw ROOT is shown.
        let catalogs = opt("CATALOGS", "Catalog Explorer");
        assert_eq!(
            localised_option_command(&catalogs, None),
            "CATALOGS".to_string(),
            "with no alias/key the raw root command is shown (Req 11.8 fallback)"
        );

        // A blank surface also falls back to the root (never blank the column).
        assert_eq!(
            localised_option_command(&catalogs, Some("   ")),
            "CATALOGS".to_string()
        );
    }
}
