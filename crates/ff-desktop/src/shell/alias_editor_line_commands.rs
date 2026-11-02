//! Alias Editor line-command apply pass + `Command ===>` primary-command parse
//! (command-configurator Req 5.14, 5.16, 5.22, CR-CH-068). Split out of
//! `shell/alias_editor.rs` to keep each file under the 400-line rule.

use super::user_aliases::AliasEnvironment;
use super::WorkbenchShell;

impl WorkbenchShell {
    /// Apply the pending Act line commands (I / D / R / C + A/B) as a single
    /// deterministic batch (Req 5.16, 7.4, 7.6), now DELEGATED to the shared
    /// egui-free apply pass ([`crate::shell::ispf_table_editor::apply_line_commands`])
    /// over the Alias host. This replaces the former bespoke I/R/D snapshot loop
    /// with the one shared, fully unit-tested structural pass (CR-CH-075).
    ///
    /// The shared pass sources classification from `ff-line-commands`: I inserts a
    /// blank after, D removes, R clones the row after it (a single-row Copy alias,
    /// Req 7.6), C + A/B copies to the target; Move is NOT in the alias set and is
    /// cleared/ignored. This remains a thin shell-side adapter so
    /// existing callers resolve unchanged; it borrows the rows out of the panel so
    /// the host (`&self.alias_editor_panel`) and the rows do not alias.
    ///
    /// Validates: command-configurator Requirement 5.16, 5.22, 7.4
    pub(super) fn apply_pending_line_commands(&mut self) {
        let mut rows = std::mem::take(&mut self.alias_editor_panel.rows);
        let result = crate::shell::ispf_table_editor::apply_line_commands(
            &self.alias_editor_panel,
            &mut rows,
        );
        self.alias_editor_panel.rows = rows;
        // Non-blocking notices (an unresolved copy source, an ignored letter) are
        // surfaced inline so the user sees why a command did not take effect,
        // without aborting the batch (Req 4.4, 4.5).
        if !result.notices.is_empty() {
            self.alias_editor_panel.error = Some(result.notices.join("; "));
        }
    }

    /// Interpret a `Command ===>` primary command (Req 5.14, 5.22). Pending line
    /// commands are ALWAYS applied FIRST (ISPF primary-vs-line ordering), then the
    /// verb (case-insensitive) is dispatched:
    ///   FILTER <ce> <root> -> set the FILTER row (CE and/or Root)
    ///   RESET         -> clear the FILTER row
    ///   SORT [col] [ASC|DESC] -> reorder the WORKING rows (Req 7.5)
    ///   (empty)       -> no-op beyond applying line commands
    ///   (any OTHER verb) -> handed to the SINGLE FFCMD front door
    ///                       (`dispatch_command_string`) so SAVE / CANCEL / END /
    ///                       RETURN / REFRESH reach the SAME editable-Context seam
    ///                       that F3 / a globally-typed END reaches (one dispatch
    ///                       path, Req 20a.10)
    ///
    /// CR-CH-071 (Req 20a.10): the PRIVATE `SAVE` and `CANCEL | END` arms are
    /// RETIRED. They are no longer interpreted here; the fall-through hands the
    /// typed command to `dispatch_command_string`, closing the two-parallel-paths
    /// divergence (the typed `Command ===>` line and F3 / global END now produce
    /// the SAME result). FILTER / RESET stay panel-local (Req 5.15), and the
    /// pending-line-command first-pass still runs FIRST (Req 5.22).
    ///
    /// Validates: command-configurator Requirement 5.14, 5.15, 5.22;
    /// command-environments Requirement 20a.10
    pub(super) fn apply_alias_command_line(&mut self, text: &str) {
        // Req 5.22: apply pending line commands BEFORE interpreting the verb.
        self.apply_pending_line_commands();

        let mut parts = text.split_whitespace();
        let Some(verb) = parts.next() else {
            // Bare Enter: pending line commands already applied; nothing more.
            return;
        };
        let verb_upper = verb.to_ascii_uppercase();
        match verb_upper.as_str() {
            "FILTER" => {
                let ce = parts.next();
                let root = parts.next();
                self.apply_alias_filter_command(ce, root);
            }
            "RESET" => {
                self.alias_editor_panel.filter_ce = None;
                self.alias_editor_panel.filter_root.clear();
            }
            "SORT" => self.apply_alias_sort_command(text),
            _ => {
                // CR-CH-071 (Req 20a.10): SAVE / CANCEL / END / RETURN / REFRESH
                // (and any other FFCMD command) route through the SINGLE front
                // door so the typed line and F3 / global END behave identically.
                // Pass the ORIGINAL text (not the uppercased verb) so arguments
                // and case are preserved for the front door.
                self.dispatch_command_string(text.trim());
            }
        }
    }

    /// Apply a manual `SORT <column> [ASC|DESC]` over the WORKING rows (Req 7.5),
    /// delegating to the shared stable sort ([`crate::shell::ispf_table_editor::apply_sort`]).
    /// With no column named it sorts by the host's first sortable column (Alias),
    /// so a bare `SORT` orders by uppercased surface ascending -- the SAME
    /// `alias_sort_key` ordering the auto-sort-on-save uses (Req 7.5a). A
    /// non-blocking notice (no sortable column / unknown column) is surfaced
    /// inline. Sorting reorders only; it never changes a field value.
    ///
    /// Validates: command-configurator Requirement 7.5
    fn apply_alias_sort_command(&mut self, text: &str) {
        let Some(mut request) = crate::shell::ispf_table_editor::parse_sort_command(text) else {
            return;
        };
        // A bare `SORT` (no column) defaults to the ALIAS (surface) column so the
        // manual order matches the auto-sort-on-save order (both key on surface,
        // Req 7.5a); without this the generic "first sortable column" fallback
        // would pick Command, which is declared before Alias for the Req 7.2
        // header order.
        if request.column.is_none() {
            request.column =
                Some(super::super::alias_editor_panel::alias_surface_column().to_string());
        }
        let mut rows = std::mem::take(&mut self.alias_editor_panel.rows);
        let result = crate::shell::ispf_table_editor::apply_sort(
            &self.alias_editor_panel,
            &mut rows,
            &request,
        );
        self.alias_editor_panel.rows = rows;
        if let Some(notice) = result.notice {
            self.alias_editor_panel.error = Some(notice);
        }
    }

    /// Apply a `FILTER <ce> <root>` command to the FILTER row (Req 5.15). An
    /// unparseable CE leaves the CE filter unchanged-to-None; a missing Root
    /// clears the Root filter. Filtering is DISPLAY-only.
    fn apply_alias_filter_command(&mut self, ce: Option<&str>, root: Option<&str>) {
        let panel = &mut self.alias_editor_panel;
        panel.filter_ce = ce.and_then(parse_filter_ce);
        panel.filter_root = root.map(|r| r.trim().to_string()).unwrap_or_default();
    }
}

/// Parse a filter CE token (`ffcmd` / `ffedit`, case-insensitive). `--` / `*` /
/// `any` clear the CE filter (None).
fn parse_filter_ce(token: &str) -> Option<AliasEnvironment> {
    match token.trim().to_ascii_lowercase().as_str() {
        "ffcmd" => Some(AliasEnvironment::Ffcmd),
        "ffedit" => Some(AliasEnvironment::Ffedit),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests_common::make_shell;
    use super::*;
    use crate::alias_editor_panel::AliasRowEdit;

    fn data_row(surface: &str, canonical: &str, env: AliasEnvironment) -> AliasRowEdit {
        AliasRowEdit {
            surface: surface.to_string(),
            canonical: canonical.to_string(),
            environment: env,
            ..Default::default()
        }
    }

    // Validates: command-configurator Requirement 5.16 -- I inserts a BLANK row
    // immediately after the source row and clears the consumed Act.
    #[test]
    fn insert_adds_blank_row_after_source() {
        let mut shell = make_shell();
        let mut r = data_row("ECOB", "EDIT", AliasEnvironment::Ffcmd);
        r.act = "I".to_string();
        shell.alias_editor_panel.rows = vec![r];
        shell.apply_pending_line_commands();
        assert_eq!(shell.alias_editor_panel.rows.len(), 2);
        assert_eq!(shell.alias_editor_panel.rows[0].surface, "ECOB");
        assert_eq!(shell.alias_editor_panel.rows[1], AliasRowEdit::default());
        assert!(shell.alias_editor_panel.rows[0].act.is_empty());
    }

    // Validates: command-configurator Requirement 7.4 -- C + A copies a
    // FIELD-COPY of the source row AFTER the A target (its Act cleared). C is the
    // primary duplicate command; R (reinstated per Req 7.6) is the single-row
    // clone-after alias of a Copy. The alias set is I / D / R / C + SORT.
    #[test]
    fn copy_after_duplicates_row_values_into_new_row() {
        let mut shell = make_shell();
        let mut src = data_row("ECOB", "EDIT", AliasEnvironment::Ffcmd);
        src.command_template = "FFWB.COBOL.MASTER(&1)".to_string();
        src.act = "C".to_string();
        let mut target = data_row("TGT", "VIEW", AliasEnvironment::Ffcmd);
        target.act = "A".to_string();
        shell.alias_editor_panel.rows = vec![src, target];
        shell.apply_pending_line_commands();
        // [ECOB(C), TGT(A)] -> copy ECOB after TGT: [ECOB, TGT, ECOB-copy].
        assert_eq!(shell.alias_editor_panel.rows.len(), 3);
        assert_eq!(shell.alias_editor_panel.rows[2].surface, "ECOB");
        assert_eq!(shell.alias_editor_panel.rows[2].canonical, "EDIT");
        assert_eq!(
            shell.alias_editor_panel.rows[2].command_template,
            "FFWB.COBOL.MASTER(&1)"
        );
        assert!(shell.alias_editor_panel.rows[2].act.is_empty());
        // The source and target Act cells are consumed/cleared.
        assert!(shell.alias_editor_panel.rows[0].act.is_empty());
        assert!(shell.alias_editor_panel.rows[1].act.is_empty());
    }

    // Validates: command-configurator Requirement 7.4 -- a single `R` clones the
    // marked alias row (field values) and places the clone AFTER it, on a
    // NON-EMPTY store, with both Act cells cleared. Before the fix, R was not in
    // the Alias enabled set so it was cleared and nothing was cloned.
    #[test]
    fn repeat_clones_row_after_on_non_empty_store() {
        let mut shell = make_shell();
        let mut src = data_row("ECOB", "EDIT", AliasEnvironment::Ffcmd);
        src.command_template = "FFWB.COBOL.MASTER(&1)".to_string();
        src.act = "R".to_string();
        let keep = data_row("KEEP", "VIEW", AliasEnvironment::Ffcmd);
        shell.alias_editor_panel.rows = vec![src, keep];
        shell.apply_pending_line_commands();
        assert_eq!(shell.alias_editor_panel.rows.len(), 3);
        // [ECOB(R), KEEP] -> clone ECOB after itself: [ECOB, ECOB-copy, KEEP].
        assert_eq!(shell.alias_editor_panel.rows[0].surface, "ECOB");
        assert_eq!(shell.alias_editor_panel.rows[1].surface, "ECOB");
        assert_eq!(shell.alias_editor_panel.rows[1].canonical, "EDIT");
        assert_eq!(
            shell.alias_editor_panel.rows[1].command_template,
            "FFWB.COBOL.MASTER(&1)"
        );
        assert_eq!(shell.alias_editor_panel.rows[2].surface, "KEEP");
        assert!(shell.alias_editor_panel.rows[0].act.is_empty());
        assert!(shell.alias_editor_panel.rows[1].act.is_empty());
    }

    // Validates: command-configurator Requirement 7.4 -- a single `R` on a
    // single-row (empty-store scratch row) store clones that row after itself.
    #[test]
    fn repeat_clones_row_after_on_single_row_store() {
        let mut shell = make_shell();
        let mut only = data_row("ONLY", "EDIT", AliasEnvironment::Ffcmd);
        only.act = "R".to_string();
        shell.alias_editor_panel.rows = vec![only];
        shell.apply_pending_line_commands();
        assert_eq!(shell.alias_editor_panel.rows.len(), 2);
        assert_eq!(shell.alias_editor_panel.rows[0].surface, "ONLY");
        assert_eq!(shell.alias_editor_panel.rows[1].surface, "ONLY");
        assert!(shell.alias_editor_panel.rows[1].act.is_empty());
    }

    // Validates: command-configurator Requirement 5.16 -- a single `I` on a
    // single-row (empty-store scratch row) store inserts a blank row after it.
    #[test]
    fn insert_adds_blank_row_after_on_single_row_store() {
        let mut shell = make_shell();
        let mut only = data_row("ONLY", "EDIT", AliasEnvironment::Ffcmd);
        only.act = "I".to_string();
        shell.alias_editor_panel.rows = vec![only];
        shell.apply_pending_line_commands();
        assert_eq!(shell.alias_editor_panel.rows.len(), 2);
        assert_eq!(shell.alias_editor_panel.rows[0].surface, "ONLY");
        assert_eq!(shell.alias_editor_panel.rows[1], AliasRowEdit::default());
        assert!(shell.alias_editor_panel.rows[0].act.is_empty());
    }

    // Validates: ispf-table-editor Requirement 4.3 -- a batch mixing an
    // immediate `I` with a copy (C/A) resolves indices against STABLE original
    // rows, so the I lands after its own row and the copy after its target.
    #[test]
    fn batch_insert_plus_copy_after_resolves_indices() {
        let mut shell = make_shell();
        let mut src = data_row("SRC", "EDIT", AliasEnvironment::Ffcmd);
        src.act = "C".to_string();
        let mut target = data_row("TGT", "VIEW", AliasEnvironment::Ffcmd);
        target.act = "A".to_string();
        let mut ins = data_row("INS", "BROWSE", AliasEnvironment::Ffcmd);
        ins.act = "I".to_string();
        shell.alias_editor_panel.rows = vec![src, target, ins];
        shell.apply_pending_line_commands();
        // [SRC(C), TGT(A), INS(I)] -> copy SRC after TGT, blank after INS:
        // [SRC, TGT, SRC-copy, INS, <blank>].
        assert_eq!(shell.alias_editor_panel.rows.len(), 5);
        assert_eq!(shell.alias_editor_panel.rows[0].surface, "SRC");
        assert_eq!(shell.alias_editor_panel.rows[1].surface, "TGT");
        assert_eq!(shell.alias_editor_panel.rows[2].surface, "SRC");
        assert_eq!(shell.alias_editor_panel.rows[3].surface, "INS");
        assert_eq!(shell.alias_editor_panel.rows[4], AliasRowEdit::default());
    }

    // Validates: command-configurator Requirement 5.16 -- D removes the row.
    #[test]
    fn delete_removes_row() {
        let mut shell = make_shell();
        let mut a = data_row("A", "EDIT", AliasEnvironment::Ffcmd);
        a.act = "D".to_string();
        let b = data_row("B", "VIEW", AliasEnvironment::Ffcmd);
        shell.alias_editor_panel.rows = vec![a, b];
        shell.apply_pending_line_commands();
        assert_eq!(shell.alias_editor_panel.rows.len(), 1);
        assert_eq!(shell.alias_editor_panel.rows[0].surface, "B");
    }

    // Validates: command-configurator Requirement 5.16 -- multiple pending
    // commands apply by DESCENDING index with no index corruption: a D at 0 and an
    // I at 2 both land correctly.
    #[test]
    fn multiple_commands_apply_without_index_corruption() {
        let mut shell = make_shell();
        let mut r0 = data_row("A", "EDIT", AliasEnvironment::Ffcmd);
        r0.act = "D".to_string();
        let r1 = data_row("B", "VIEW", AliasEnvironment::Ffcmd);
        let mut r2 = data_row("C", "BROWSE", AliasEnvironment::Ffcmd);
        r2.act = "I".to_string();
        shell.alias_editor_panel.rows = vec![r0, r1, r2];
        shell.apply_pending_line_commands();
        // Start [A(D), B, C(I)] -> delete A, insert blank after C:
        // [B, C, <blank>].
        assert_eq!(shell.alias_editor_panel.rows.len(), 3);
        assert_eq!(shell.alias_editor_panel.rows[0].surface, "B");
        assert_eq!(shell.alias_editor_panel.rows[1].surface, "C");
        assert_eq!(shell.alias_editor_panel.rows[2], AliasRowEdit::default());
    }

    // Validates: command-configurator Requirement 5.16 -- an unrecognised Act
    // letter is cleared and ignored (the row list is unchanged).
    #[test]
    fn unrecognised_act_letter_is_cleared_and_ignored() {
        let mut shell = make_shell();
        let mut r = data_row("A", "EDIT", AliasEnvironment::Ffcmd);
        r.act = "Q".to_string();
        shell.alias_editor_panel.rows = vec![r];
        shell.apply_pending_line_commands();
        assert_eq!(shell.alias_editor_panel.rows.len(), 1);
        assert!(shell.alias_editor_panel.rows[0].act.is_empty());
    }

    // Validates: command-configurator Requirement 5.15 -- FILTER sets the FILTER
    // row (CE + Root) and RESET clears it.
    #[test]
    fn filter_command_sets_and_reset_clears() {
        let mut shell = make_shell();
        shell.apply_alias_command_line("FILTER ffcmd EDIT");
        assert_eq!(
            shell.alias_editor_panel.filter_ce,
            Some(AliasEnvironment::Ffcmd)
        );
        assert_eq!(shell.alias_editor_panel.filter_root, "EDIT");
        shell.apply_alias_command_line("RESET");
        assert_eq!(shell.alias_editor_panel.filter_ce, None);
        assert!(shell.alias_editor_panel.filter_root.is_empty());
    }

    // Validates: command-configurator Requirement 5.22 -- a SAVE on the command
    // line applies the pending line commands FIRST, then saves: an I pending on
    // the (only) row is applied before validation runs.
    //
    // CR-CH-071 (Req 20a.10): SAVE is no longer a private arm -- it routes through
    // the single FFCMD front door, which reaches the editable-Context seam ONLY
    // when the focused Context is the Alias Editor. So this test now OPENS the
    // Alias Editor first (previously it asserted on a POM tab, where the retired
    // private arm ran regardless); the line-commands-first ordering is unchanged.
    #[test]
    fn command_line_applies_line_commands_before_primary_command() {
        let mut shell = make_shell();
        let dir = tempfile::TempDir::new().expect("tempdir");
        // CR-CH-072: the override is a per-locale DIR base, not a single file.
        shell.dir_overrides.user_aliases = Some(dir.path().join("aliases"));
        // Focus the Alias Editor so the FFCMD SAVE reaches its seam (Req 20a.10).
        shell.dispatch_command_string("ALIASES");
        let mut r = data_row("GG", "GSEARCH", AliasEnvironment::Ffcmd);
        r.act = "I".to_string();
        shell.alias_editor_panel.rows = vec![r];
        // CR-CH-077 single-chrome: the shell's command field routes a typed
        // `Command ===>` verb straight into the Alias command-environment handler.
        shell.apply_alias_command_line("SAVE");
        // The I was applied (a blank row was inserted) BEFORE save; the blank row
        // has an empty surface, so the save FAILS validation and surfaces an error
        // -- proving line commands ran first (the pre-I single valid row would have
        // saved cleanly).
        assert!(
            shell.alias_editor_panel.error.is_some(),
            "the inserted blank row makes SAVE fail, proving I applied first"
        );
        assert_eq!(
            shell.alias_editor_panel.rows.len(),
            2,
            "the pending I inserted a blank row before SAVE"
        );
    }
}
