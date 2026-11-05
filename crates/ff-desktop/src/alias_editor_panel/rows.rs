//! Per-row and filter-row render helpers for the Alias Editor ISPF layout
//! (Req 5.13, 5.15, 5.18, design 9.10.2 / 9.10.5). Split out of `render.rs` to
//! keep each file under the 400-line rule.

use super::state::AliasRowEdit;
use crate::shell::environment_registry::EnvironmentRegistry;
use crate::shell::user_aliases::AliasEnvironment;

/// Stable salted egui id of a per-row control, keyed by the row's BACKING index
/// so ids never collide across rows (design 9.10.8 part 2).
pub(super) fn row_control_id(prefix: &'static str, backing: usize) -> egui::Id {
    egui::Id::new((prefix, backing))
}

/// Render a CE dropdown bound to `env`, returning whether the selection changed.
/// The CE list is `AliasEnvironment::ALL` (FFCMD / FFEDIT) for v1 (Req 5.18,
/// design 9.10.5). `salt` makes the combo id stable and unique.
///
/// CR-NR-116 (ce-case-investigation decision a/e): the DISPLAY uses the
/// canonical UPPERCASE `env_name()` (FFCMD / FFEDIT), NOT the lowercase
/// `label()`. Storage, file names, and the dropdown constraints are unchanged;
/// only the shown text is uppercased (this supersedes the lowercase `[ffcmd v]`
/// literal in design fixture 9.11.1).
pub(super) fn render_ce_combo(
    ui: &mut egui::Ui,
    salt: impl std::hash::Hash,
    env: &mut AliasEnvironment,
) -> egui::Response {
    let before = *env;
    let combo = egui::ComboBox::from_id_salt(salt)
        .selected_text(env.env_name())
        .width(84.0)
        .show_ui(ui, |ui| {
            for choice in AliasEnvironment::ALL {
                ui.selectable_value(env, choice, choice.env_name());
            }
        });
    let mut resp = combo.response;
    if *env != before {
        resp.mark_changed();
    }
    resp
}

/// Render a Root dropdown bound to `canonical`, sourced from
/// `root_commands(env)`. A blank selection is offered so a row can start empty.
/// `salt` makes the combo id stable and unique (Req 5.18, design 9.10.5).
pub(super) fn render_root_combo(
    ui: &mut egui::Ui,
    salt: impl std::hash::Hash,
    env: AliasEnvironment,
    canonical: &mut String,
) -> egui::Id {
    let roots = EnvironmentRegistry::root_commands(env.env_name());
    let shown = if canonical.is_empty() {
        "--".to_string()
    } else {
        canonical.clone()
    };
    let combo = egui::ComboBox::from_id_salt(salt)
        .selected_text(shown)
        .width(96.0)
        .show_ui(ui, |ui| {
            ui.selectable_value(canonical, String::new(), "--");
            for root in roots {
                ui.selectable_value(canonical, root.clone(), root);
            }
        });
    combo.response.id
}

/// Re-validate a row's `canonical` against its (possibly just-changed) CE: if the
/// root is no longer valid in the new environment, clear it (design 9.10.5). A
/// blank root is left blank.
pub(super) fn revalidate_root_for_env(row: &mut AliasRowEdit) {
    if row.canonical.is_empty() {
        return;
    }
    if !EnvironmentRegistry::contains_root(row.environment.env_name(), &row.canonical) {
        row.canonical.clear();
    }
}

/// Render one DATA row (Act / CE / Root / ALIAS / Parameters / Description) at
/// backing index `backing`. CR-CH-072 (Req 6.2): the L18N column is removed.
/// CR-CH-075 (Req 7.3) adds the Description cell. All edits bind directly to the
/// mutable row (B054 pattern).
///
/// Returns `true` WHEN Enter was pressed with the Act field focused, so the
/// surface can apply the pending line-command batch RIGHT THERE in the Act cell
/// -- the ISPF behaviour where the user types I / D / R / C in a row and presses
/// Enter without tabbing to the `Command ===>` line (command-configurator
/// Req 7.6; ispf-table-editor Req 4.6). The stable Act egui id is PRESERVED so
/// the full-shell focus tests are unaffected.
///
/// Validates: command-configurator Requirement 5.13, 5.18, 5.21, 6.2, 7.3, 7.6;
/// ispf-table-editor Requirement 11.3 (the fresh same-frame cell ids)
pub(super) fn render_data_row(
    ui: &mut egui::Ui,
    backing: usize,
    row: &mut AliasRowEdit,
) -> crate::shell::ispf_table_editor::render::RowCells {
    ui.horizontal(|ui| {
        // Act: the ISPF line-command field (I / R / D). One focusable control.
        let act_id = row_control_id("alias_act", backing);
        let act_resp = ui.add(
            egui::TextEdit::singleline(&mut row.act)
                .id(act_id)
                .desired_width(28.0)
                .hint_text("_"),
        );
        let act_enter = act_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        // CE dropdown; on change, re-validate the root against the new env. Its
        // id is captured FRESH from the response (B056: a guessed combo id does
        // not round-trip through focus).
        let ce_resp = render_ce_combo(ui, ("alias_ce", backing), &mut row.environment);
        let ce_id = ce_resp.id;
        if ce_resp.changed() {
            revalidate_root_for_env(row);
        }
        // Root dropdown sourced from root_commands(CE); fresh id from the combo.
        let root_id = render_root_combo(
            ui,
            ("alias_root", backing),
            row.environment,
            &mut row.canonical,
        );
        // CR-CH-072 (Req 6.2): the L18N (localisation-key) column is REMOVED --
        // the surface IS the authoritative localised text.
        // ALIAS (surface).
        let surface_id = row_control_id("alias_surface", backing);
        let surface_resp = ui.add(
            egui::TextEdit::singleline(&mut row.surface)
                .id(surface_id)
                .desired_width(120.0)
                .hint_text("name"),
        );
        if surface_resp.lost_focus() {
            let upper = row.surface.trim().to_uppercase();
            if upper != row.surface {
                row.surface = upper;
            }
        }
        // Parameters (command_template).
        let template_id = row_control_id("alias_template", backing);
        ui.add(
            egui::TextEdit::singleline(&mut row.command_template)
                .id(template_id)
                .desired_width(260.0)
                .hint_text("(&1)"),
        );
        // CR-CH-075 (Req 7.3): Description -- an optional human-readable note.
        let description_id = row_control_id("alias_description", backing);
        ui.add(
            egui::TextEdit::singleline(&mut row.description)
                .id(description_id)
                .desired_width(180.0)
                .hint_text("note"),
        );
        crate::shell::ispf_table_editor::render::RowCells {
            act_enter,
            // Visual (Tab) order: Act, CE, Root, Alias, Parameters, Description.
            cell_ids: vec![
                act_id,
                ce_id,
                root_id,
                surface_id,
                template_id,
                description_id,
            ],
        }
    })
    .inner
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: command-configurator Requirement 5.18 -- a row's Root options
    // equal root_commands(env_name()) for its CE.
    #[test]
    fn root_options_match_root_commands_for_ce() {
        let ffcmd = EnvironmentRegistry::root_commands(AliasEnvironment::Ffcmd.env_name());
        assert!(ffcmd.iter().any(|r| r == "EDIT"));
        let ffedit = EnvironmentRegistry::root_commands(AliasEnvironment::Ffedit.env_name());
        assert!(ffedit.iter().any(|r| r == "FIND"));
        // The two environments expose different root sets (the cascade source).
        assert_ne!(ffcmd, ffedit);
    }

    // Validates: command-configurator Requirement 5.18 -- changing a row's CE and
    // re-validating clears a root that is no longer valid in the new environment.
    #[test]
    fn changing_ce_clears_now_invalid_root() {
        let mut row = AliasRowEdit {
            environment: AliasEnvironment::Ffedit,
            canonical: "FIND".to_string(),
            ..Default::default()
        };
        // FIND is a valid FFEDIT root -> kept.
        revalidate_root_for_env(&mut row);
        assert_eq!(row.canonical, "FIND");
        // Switch to FFCMD, where FIND is NOT a root -> cleared.
        row.environment = AliasEnvironment::Ffcmd;
        revalidate_root_for_env(&mut row);
        assert!(row.canonical.is_empty(), "a now-invalid root is cleared");
    }

    // Validates: command-configurator Requirement 5.18 -- a (CE, Root) pair chosen
    // from the dropdowns (Root sourced from root_commands) satisfies contains_root
    // by construction.
    #[test]
    fn dropdown_pair_satisfies_contains_root() {
        for env in AliasEnvironment::ALL {
            for root in EnvironmentRegistry::root_commands(env.env_name()) {
                assert!(
                    EnvironmentRegistry::contains_root(env.env_name(), &root),
                    "a dropdown-offered root must be valid in its environment"
                );
            }
        }
    }

    // Validates: command-configurator Requirement 5.18 -- a blank root is left
    // blank by revalidation (an empty row stays insertable).
    #[test]
    fn blank_root_is_left_blank() {
        let mut row = AliasRowEdit::default();
        revalidate_root_for_env(&mut row);
        assert!(row.canonical.is_empty());
    }
}
