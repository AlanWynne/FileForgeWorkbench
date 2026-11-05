//! # Alias Editor per-Kind Workspace_Status_Segment (CR-NR-116, Req 7.25)
//!
//! The Alias Editor contributes its OWN Status_Bar cells after the three
//! shell-common segments (Last | Prof | Zoom, owned by `common_frame.rs`):
//!
//!   - `Aliases: <n>` -- the alias count `n = rows.len()` (the always-present
//!     trailing scratch row is NOT a backing row, so it is excluded).
//!   - `Rows 1-<m> visible` -- the DISPLAYED-row range, `m` being the number of
//!     rows the FILTER currently shows (`Rows 0 visible` when none match).
//!   - `Filter: <ce>/<root>` -- shown ONLY while a filter is active, with `--`
//!     for an unset half and the CE as its canonical UPPERCASE name
//!     (e.g. `Filter: FFCMD/--`).
//!
//! The ONE method [`WorkbenchShell::alias_status_cells`] is read by BOTH the
//! live Status_Bar (via `render_status.rs::workspace_status_cells`) and the
//! snapshot status band (via `common_frame.rs`), so live/snapshot parity is by
//! construction. No cell contains ` | ` (the band cell separator), so the cells
//! stay discrete. The i18n keys follow the `ff-desktop-shell-status-*` family.
//!
//! Validates: command-configurator Requirement 7.25

use super::WorkbenchShell;
use crate::alias_editor_panel::displayed_indices;

impl WorkbenchShell {
    /// The Alias Editor Workspace_Status_Segment cells (Req 7.25). Called by
    /// `workspace_status_cells` for the active Alias editor and by the snapshot
    /// band through the same path, so the live bar and the capture agree.
    pub(super) fn alias_status_cells(&self) -> Vec<String> {
        let p = &self.alias_editor_panel;
        let count = p.rows.len();
        let visible = displayed_indices(&p.rows, p.filter_ce, &p.filter_root).len();

        let mut cells = vec![
            ff_i18n::t_args(
                "ff-desktop-shell-status-alias-count",
                &ff_i18n::MessageArgs::new().set("count", count.to_string()),
            ),
            if visible == 0 {
                ff_i18n::t("ff-desktop-shell-status-alias-rows-none")
            } else {
                ff_i18n::t_args(
                    "ff-desktop-shell-status-alias-rows",
                    &ff_i18n::MessageArgs::new().set("max", visible.to_string()),
                )
            },
        ];

        // The Filter cell appears ONLY while a filter half is set (Req 7.25). The
        // CE is the canonical UPPERCASE name; an unset half renders as `--`.
        let root = p.filter_root.trim();
        if p.filter_ce.is_some() || !root.is_empty() {
            let ce = p
                .filter_ce
                .map(|env| env.env_name().to_string())
                .unwrap_or_else(|| "--".to_string());
            let root = if root.is_empty() {
                "--".to_string()
            } else {
                root.to_string()
            };
            cells.push(ff_i18n::t_args(
                "ff-desktop-shell-status-alias-filter",
                &ff_i18n::MessageArgs::new().set("ce", ce).set("root", root),
            ));
        }
        cells
    }
}

#[cfg(test)]
mod tests {
    use crate::alias_editor_panel::AliasRowEdit;
    use crate::shell::tests_common::make_shell;
    use crate::shell::user_aliases::AliasEnvironment;

    fn row(surface: &str, canonical: &str, env: AliasEnvironment) -> AliasRowEdit {
        AliasRowEdit {
            surface: surface.to_string(),
            canonical: canonical.to_string(),
            environment: env,
            ..Default::default()
        }
    }

    // Validates: command-configurator Requirement 7.25 -- with no filter active,
    // the Alias status cells are the count and the full visible-row range, and
    // there is NO Filter cell. (`make_shell` installs the test catalogue.)
    #[test]
    fn alias_status_cells_count_and_rows_no_filter() {
        let mut shell = make_shell();
        shell.alias_editor_panel.rows = vec![
            row("ECOB", "EDIT", AliasEnvironment::Ffcmd),
            row("VCOB", "VIEW", AliasEnvironment::Ffcmd),
            row("SOEK", "FIND", AliasEnvironment::Ffedit),
        ];
        let cells = shell.alias_status_cells();
        assert_eq!(cells, vec!["Aliases: 3", "Rows 1-3 visible"]);
        assert!(
            cells.iter().all(|c| !c.contains(" | ")),
            "no cell contains the band separator"
        );
    }

    // Validates: command-configurator Requirement 7.25 -- an empty working copy
    // shows `Aliases: 0` and `Rows 0 visible`.
    #[test]
    fn alias_status_cells_empty_is_zero_and_rows_zero() {
        let shell = make_shell();
        let cells = shell.alias_status_cells();
        assert_eq!(cells, vec!["Aliases: 0", "Rows 0 visible"]);
    }

    // Validates: command-configurator Requirement 7.25 -- a CE-only filter shows
    // the restricted visible range and `Filter: FFCMD/--` (CE UPPERCASE, the
    // unset Root half as `--`).
    #[test]
    fn alias_status_cells_ce_filter_shows_uppercase_and_dashes() {
        let mut shell = make_shell();
        shell.alias_editor_panel.rows = vec![
            row("ECOB", "EDIT", AliasEnvironment::Ffcmd),
            row("SOEK", "FIND", AliasEnvironment::Ffedit),
        ];
        shell.alias_editor_panel.filter_ce = Some(AliasEnvironment::Ffcmd);
        let cells = shell.alias_status_cells();
        assert_eq!(
            cells,
            vec!["Aliases: 2", "Rows 1-1 visible", "Filter: FFCMD/--"]
        );
    }

    // Validates: command-configurator Requirement 7.25 -- a Root-only filter
    // shows `Filter: --/SNAPSHOT` (the unset CE half as `--`).
    #[test]
    fn alias_status_cells_root_only_filter_shows_dash_ce() {
        let mut shell = make_shell();
        shell.alias_editor_panel.rows = vec![row("SS", "SNAPSHOT", AliasEnvironment::Ffcmd)];
        shell.alias_editor_panel.filter_root = "SNAPSHOT".to_string();
        let cells = shell.alias_status_cells();
        assert_eq!(
            cells,
            vec!["Aliases: 1", "Rows 1-1 visible", "Filter: --/SNAPSHOT"]
        );
    }
}
