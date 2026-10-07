//! # FFEDIT as a real Command_Environment object (CR-CH-053 Task 17)
//!
//! This file holds [`FfEditEnvironment`], the real registered FFEDIT (editor)
//! [`CommandEnvironment`] object the built `EnvironmentRegistry` (see
//! `environment_registry.rs`) resolves the name `"FFEDIT"` to. It is split out of
//! `environment_registry.rs` purely to keep each file under the 400-line rule
//! (rust-standards.md); the split is behaviour-preserving.
//!
//! The object is zero-sized: all FFEDIT state lives on the shell. Its `claim`
//! body is the former `WorkbenchShell::ffedit_claim`, moved VERBATIM and reaching
//! shell-entangled state (`self.tabs` + the managers) through the passed
//! `&mut WorkbenchShell`, so the observable result is byte-identical (Req 13.5,
//! 13.6, 4.2, 6.3).
//!
//! See docs/specs/command-environments/{requirements,design}.md.

use super::environment::CommandEnvironment;
use super::WorkbenchShell;

/// Whether a CANONICAL FFEDIT verb is STORE-AFFECTING (it reads or writes the
/// backing store) as opposed to IN-BUFFER (it acts only on the open document /
/// editor display state) -- CR-CH-053 Task 18, Req 14.3/14.7.
///
/// Store-affecting verbs are the ones FFEDIT ADDRESSES to the resource's
/// Owning_Environment (Req 14.4) rather than executing itself; in-buffer verbs
/// FFEDIT handles directly and NEVER addresses elsewhere (Req 14.3). Today SAVE
/// is the only store-affecting FFEDIT verb; future CREATE / REPLACE member and
/// save-time record validation join it when built. Every other Req 6.1 verb
/// (LOCATE, FIND, CHANGE-in-buffer, CAPS, SORT, EXCLUDE, SHOW, RESET, the
/// profile/scroll verbs, and the universal NUMBER/UNNUM/BNDS/COLS when wired) is
/// in-buffer.
///
/// This is the OWNERSHIP boundary (Req 14.7): FFEDIT owns in-buffer editing; the
/// Owning_Environment owns store reads/writes and store-dependent validation.
///
/// Task 18 defines and TESTS this classification (Req 14.3); it becomes
/// load-bearing in Task 20, where the gate consults it to redirect store verbs to
/// the tab's Owning_Environment (Req 14.4/14.5). In this additive slice SAVE still
/// executes in FFEDIT (behaviour-preserving), so the only consumers today are the
/// Req-14.3 classification tests -- hence the scoped allow with this justification.
///
/// Validates: command-environments Requirement 14.3, 14.7
#[allow(dead_code)] // consumed by Task 20 (store-verb redirect to Owning_Environment); tested now.
pub(super) fn is_store_affecting_verb(canonical: &str) -> bool {
    matches!(canonical, "SAVE")
}

/// The real registered FFEDIT (editor) [`CommandEnvironment`] object (Req 13.5).
///
/// Zero-sized: all FFEDIT state lives on the shell. It is the object the registry
/// resolves the name `"FFEDIT"` to; its `claim` body is the former
/// `WorkbenchShell::ffedit_claim`, reaching shell-entangled state through the
/// passed `&mut WorkbenchShell` so the observable result is unchanged (Req 13.6,
/// 4.2, 6.3).
pub(super) struct FfEditEnvironment;

impl CommandEnvironment for FfEditEnvironment {
    /// The FFEDIT (editor) Command_Environment claim step (CR-CH-053).
    ///
    /// Called from the shared ladder's active-env step ONLY when the focused
    /// Context is an editor Context. Returns `true` iff FFEDIT owns `raw` and
    /// handled it (the caller then returns); `false` to fall through to FFCMD
    /// (`resolve_target`) and then the ladder.
    ///
    /// The body is the former `WorkbenchShell::ffedit_claim`, moved VERBATIM: the
    /// surface verb is resolved to its CANONICAL verb via the FFEDIT alias table
    /// (Req 6a) before matching; every family (navigation, exclude/show/reset,
    /// find/change, profile, scroll, SAVE) delegates to the SAME shell managers
    /// as before, so the observable result is byte-identical (Req 13.5, 13.6,
    /// 4.2, 6.3). The `RESET BARE` decline is preserved so FFCMD's `RESET BARE`
    /// ladder arm still receives it.
    ///
    /// Validates: command-environments Requirement 6.1, 6.2, 6.2a, 6.3, 6a.1,
    /// 13.5, 13.6, 4.2
    fn claim(&mut self, shell: &mut WorkbenchShell, raw: &str, upper: &str) -> bool {
        use super::environment::AliasTable;
        use super::helpers::{parse_optional_u64, verb_arg};

        // Resolve the surface verb token to its canonical verb (Req 6a). The
        // handler below matches on the CANONICAL verb; the alias table is the
        // single point localized surface forms (CR-NR-103) will plug into.
        let aliases = AliasTable::ffedit_english();
        let verb_token = raw.split_whitespace().next().unwrap_or("");
        let Some(canonical) = aliases.canonical_verb(verb_token) else {
            // Not an FFEDIT verb -- fall through to FFCMD / the ladder.
            return false;
        };

        // NAVIGATION family (E1). Each arm delegates to `nav_manager`, keyed on
        // the canonical verb. Non-nav canonical verbs (exclude/find/profile/
        // scroll) are handled by their arms below.
        match canonical {
            "LOCATE" => {
                let arg = verb_arg(raw, "LOCATE").filter(|a| !a.is_empty());
                let Some(arg) = arg else { return false };
                let status = shell.nav_manager.locate(arg, &mut shell.tabs);
                shell.open_error = if status.is_empty() {
                    None
                } else {
                    Some(status)
                };
                true
            }
            "TOP" => {
                shell.nav_manager.top(&mut shell.tabs);
                shell.open_error = None;
                true
            }
            "BOTTOM" => {
                shell.nav_manager.bottom(&mut shell.tabs);
                shell.open_error = None;
                true
            }
            "UP" => {
                // CR-NR-087: numeric `UP n` overrides SCROLL; bare `UP` uses the
                // active SCROLL amount.
                let Some(arg) = verb_arg(raw, "UP") else {
                    return false;
                };
                match parse_optional_u64(arg) {
                    Some(n) => shell.nav_manager.up(Some(n), &mut shell.tabs),
                    None => shell
                        .nav_manager
                        .up_by_amount(&shell.scroll_amount, &mut shell.tabs),
                }
                shell.open_error = None;
                true
            }
            "DOWN" => {
                let Some(arg) = verb_arg(raw, "DOWN") else {
                    return false;
                };
                match parse_optional_u64(arg) {
                    Some(n) => shell.nav_manager.down(Some(n), &mut shell.tabs),
                    None => shell
                        .nav_manager
                        .down_by_amount(&shell.scroll_amount, &mut shell.tabs),
                }
                shell.open_error = None;
                true
            }
            "LEFT" => {
                let Some(arg) = verb_arg(raw, "LEFT") else {
                    return false;
                };
                shell
                    .nav_manager
                    .left(parse_optional_u64(arg), &mut shell.tabs);
                shell.open_error = None;
                true
            }
            "RIGHT" => {
                let Some(arg) = verb_arg(raw, "RIGHT") else {
                    return false;
                };
                shell
                    .nav_manager
                    .right(parse_optional_u64(arg), &mut shell.tabs);
                shell.open_error = None;
                true
            }
            "SORT" => {
                let Some(rest) = verb_arg(raw, "SORT") else {
                    return false;
                };
                let args: Vec<&str> = rest.split_whitespace().collect();
                let status = shell
                    .nav_manager
                    .sort(&args, &mut shell.tabs, &shell.runtime);
                shell.open_error = if status.is_empty() {
                    None
                } else {
                    Some(status)
                };
                true
            }

            // EXCLUDE / SHOW / RESET family (E2). X -> EXCLUDE and INCLUDE -> SHOW
            // are resolved by the alias table, so the match keys on the CANONICAL
            // verb; the sub-parse re-derives the argument text from `raw` (which
            // carries the surface form the user typed) to preserve EXACT behaviour
            // incl. the `ALL` suffix.
            "EXCLUDE" => {
                shell.ffedit_exclude(raw);
                true
            }
            "SHOW" => {
                shell.ffedit_show(raw);
                true
            }
            "RESET" => {
                // Ownership boundary (CR-CH-053): FFEDIT's RESET owns the editor
                // exclusion-reset forms (bare RESET, RESET ALL, RESET EXCLUDED).
                // `RESET BARE [..]` is a DIFFERENT, FFCMD-owned command (reset a
                // profile to the compiled baseline, configuration-system Req 19),
                // not an editor verb -- FFEDIT DECLINES it so it falls through to
                // FFCMD's `RESET BARE` ladder arm.
                let rest = verb_arg(raw, "RESET").unwrap_or("");
                let first_arg = rest.split_whitespace().next().unwrap_or("");
                if first_arg.eq_ignore_ascii_case("BARE") {
                    return false;
                }
                shell.ffedit_reset(raw);
                true
            }

            // FIND / RFIND / CHANGE / RCHANGE family (E3). CHANGE's two-argument
            // quoting (`parse_two_args`) and the B062 case-preserved term are
            // carried verbatim from the former ladder arms.
            "RFIND" => {
                let status = shell.find_manager.rfind(&mut shell.tabs, &shell.runtime);
                shell.open_error = WorkbenchShell::find_status_to_error(status);
                true
            }
            "RCHANGE" => {
                let status = shell.find_manager.rchange(&mut shell.tabs, &shell.runtime);
                shell.open_error = WorkbenchShell::find_status_to_error(status);
                true
            }
            "FIND" => {
                let Some(term) = verb_arg(raw, "FIND").filter(|a| !a.is_empty()) else {
                    return false;
                };
                let status = shell
                    .find_manager
                    .find(term, &mut shell.tabs, &shell.runtime);
                shell.open_error = WorkbenchShell::find_status_to_error(status);
                true
            }
            "CHANGE" => {
                let Some(rest) = verb_arg(raw, "CHANGE").filter(|a| !a.is_empty()) else {
                    return false;
                };
                if let Some((old, new)) = super::helpers::parse_two_args(rest) {
                    let status =
                        shell
                            .find_manager
                            .change(&old, &new, &mut shell.tabs, &shell.runtime);
                    shell.open_error = WorkbenchShell::find_status_to_error(status);
                } else {
                    shell.open_error =
                        Some("CHANGE requires two arguments: CHANGE 'old' 'new'".to_string());
                }
                true
            }

            // PROFILE family (E4): CAPS / NULLS / STATS / LOCK / PROFILE / HILITE.
            "CAPS" => {
                shell.ffedit_caps(upper);
                true
            }
            "NULLS" => {
                shell.ffedit_nulls(upper);
                true
            }
            "STATS" => {
                shell.ffedit_stats(upper);
                true
            }
            "LOCK" => {
                shell.ffedit_lock(upper);
                true
            }
            "PROFILE" => {
                shell.ffedit_profile(raw);
                true
            }
            "HILITE" => {
                shell.ffedit_hilite(raw);
                true
            }

            // SCROLL field update (E5).
            "SCROLL" => {
                let Some(arg) = verb_arg(raw, "SCROLL").filter(|a| !a.is_empty()) else {
                    return false;
                };
                if let Some(amount) = crate::scroll_amount::ScrollAmount::parse(arg) {
                    shell.scroll_amount = amount;
                    shell.scroll_field_text = shell.scroll_amount.display_string();
                    shell.open_error = None;
                } else {
                    shell.open_error = Some(format!(
                        "SCROLL: '{arg}' is not a valid scroll amount \
                         (PAGE/HALF/CSR/MAX/DATA/n)"
                    ));
                }
                true
            }

            // Editor-buffer SAVE (E9, Req 10.1): the one STORE-AFFECTING FFEDIT
            // verb (`is_store_affecting_verb`). It reaches this arm via the
            // address-by-name seam (`dispatch_to_environment` -> this `claim`),
            // which is how CR-CH-053 Task 18 routes store verbs (Req 14.3/14.8).
            // In this slice SAVE still EXECUTES here (dirty-aware, STAYS in the
            // editor) so behaviour is byte-identical; Task 20 (Req 14.4/14.5/14.6)
            // will redirect it to the tab's Owning_Environment once the owning-env
            // binding (Task 19) exists, moving only the EXECUTOR, not the contract.
            "SAVE" => {
                shell.ffedit_save();
                true
            }

            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_store_affecting_verb;

    /// Validates: command-environments Requirement 14.3 -- SAVE is the one
    /// store-affecting FFEDIT verb today (the verb FFEDIT addresses to the
    /// Owning_Environment rather than executing in-buffer).
    #[test]
    fn save_is_classified_store_affecting() {
        assert!(is_store_affecting_verb("SAVE"));
    }

    /// Validates: command-environments Requirement 14.3 -- the in-buffer verbs
    /// (Req 6.1) are NOT store-affecting, so FFEDIT handles them directly and
    /// never addresses them to another environment.
    #[test]
    fn in_buffer_verbs_are_not_store_affecting() {
        for verb in [
            "LOCATE", "TOP", "BOTTOM", "UP", "DOWN", "LEFT", "RIGHT", "SORT", "EXCLUDE", "SHOW",
            "RESET", "FIND", "RFIND", "CHANGE", "RCHANGE", "CAPS", "NULLS", "STATS", "LOCK",
            "PROFILE", "HILITE", "SCROLL",
        ] {
            assert!(
                !is_store_affecting_verb(verb),
                "{verb} must be classified in-buffer (handled directly, not addressed)"
            );
        }
    }
}
