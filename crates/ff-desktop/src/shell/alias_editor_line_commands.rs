//! Alias Editor line-command apply pass + `Command ===>` primary-command parse
//! (command-configurator Req 5.14, 5.16, 5.22, CR-CH-068). Split out of
//! `shell/alias_editor.rs` to keep each file under the 400-line rule.

use super::user_aliases::AliasEnvironment;
use super::WorkbenchShell;
use crate::alias_editor_panel::AliasRowEdit;

impl WorkbenchShell {
    /// Apply the pending Act line commands (I / R / D) as a single deterministic
    /// batch (Req 5.16, design 9.10.4).
    ///
    /// The letters are snapshotted with their BACKING `rows` index, then processed
    /// by DESCENDING index so inserts / repeats / deletes never corrupt one
    /// another's indices:
    ///   D -> remove row i
    ///   I -> insert a blank `AliasRowEdit` AFTER row i
    ///   R -> insert a field-copy clone of row i AFTER it (its Act cleared)
    /// An unrecognised Act letter is ignored; every consumed Act field is cleared.
    ///
    /// Validates: command-configurator Requirement 5.16, 5.22
    pub(super) fn apply_pending_line_commands(&mut self) {
        let rows = &mut self.alias_editor_panel.rows;
        // 1. Snapshot (backing index, letter) for any non-empty Act field.
        let mut pending: Vec<(usize, char)> = Vec::new();
        for (i, row) in rows.iter().enumerate() {
            if let Some(letter) = row.act.trim().chars().next() {
                pending.push((i, letter.to_ascii_uppercase()));
            }
        }
        // Clear EVERY Act field (consumed, recognised or not -- ISPF tolerates and
        // clears stray line commands).
        for row in rows.iter_mut() {
            row.act.clear();
        }
        // 2. Process by DESCENDING index so earlier rows' indices stay valid.
        pending.sort_by_key(|a| std::cmp::Reverse(a.0));
        for (i, letter) in pending {
            if i >= rows.len() {
                continue;
            }
            match letter {
                'D' => {
                    rows.remove(i);
                }
                'I' => {
                    rows.insert(i + 1, AliasRowEdit::default());
                }
                'R' => {
                    let mut clone = rows[i].clone();
                    clone.act.clear();
                    rows.insert(i + 1, clone);
                }
                _ => {
                    // Unrecognised letter: already cleared above; ignore.
                }
            }
        }
    }

    /// Interpret a `Command ===>` primary command (Req 5.14, 5.22). Pending line
    /// commands are ALWAYS applied FIRST (ISPF primary-vs-line ordering), then the
    /// verb (case-insensitive) is dispatched:
    ///   FILTER <ce> <root> -> set the FILTER row (CE and/or Root)
    ///   RESET         -> clear the FILTER row
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
    use crate::alias_editor_panel::AliasEditorAction;

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

    // Validates: command-configurator Requirement 5.16 -- R inserts a FIELD-COPY
    // of the source row after it (its Act cleared).
    #[test]
    fn repeat_duplicates_row_values_into_new_row() {
        let mut shell = make_shell();
        let mut r = data_row("ECOB", "EDIT", AliasEnvironment::Ffcmd);
        r.command_template = "FFWB.COBOL.MASTER(&1)".to_string();
        r.act = "R".to_string();
        shell.alias_editor_panel.rows = vec![r];
        shell.apply_pending_line_commands();
        assert_eq!(shell.alias_editor_panel.rows.len(), 2);
        assert_eq!(shell.alias_editor_panel.rows[1].surface, "ECOB");
        assert_eq!(shell.alias_editor_panel.rows[1].canonical, "EDIT");
        assert_eq!(
            shell.alias_editor_panel.rows[1].command_template,
            "FFWB.COBOL.MASTER(&1)"
        );
        assert!(shell.alias_editor_panel.rows[1].act.is_empty());
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
        shell.dir_overrides.user_aliases = Some(dir.path().join("aliases").join("user.toml"));
        // Focus the Alias Editor so the FFCMD SAVE reaches its seam (Req 20a.10).
        shell.dispatch_command_string("ALIASES");
        let mut r = data_row("GG", "GSEARCH", AliasEnvironment::Ffcmd);
        r.act = "I".to_string();
        shell.alias_editor_panel.rows = vec![r];
        // Route through the action entry point (as the render would).
        shell.apply_alias_editor_action(AliasEditorAction::CommandLine("SAVE".to_string()));
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
