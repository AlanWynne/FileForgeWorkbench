//! # Parameterised alias template substitution (CR-NR-112, command-semantics
//! Requirement 14, command-framework Requirement 19).
//!
//! This is the ONE piece of genuinely new logic CR-NR-112 adds: a pure,
//! deterministic function that expands an alias's parameter TEMPLATE against the
//! alias invocation's argument string, producing the fully resolved ROOT command
//! STRING. It has no I/O and no shell state, so it is unit-tested in isolation.
//!
//! The expansion is PURE TEXT substitution with NO command-specific
//! special-casing (Req 14.8): the result is handed back to the existing pre-pass,
//! which lets the unchanged single front door (`resolve_target` -> the EDIT
//! ladder arm) consume it exactly as a typed command.
//!
//! ## Grammar (Req 14.2, 14.3, 14.10)
//!
//! - `&N` (N a positive decimal integer, 1-based) substitutes the Nth
//!   whitespace-delimited token of the argument string. The digit run is maximal,
//!   so `&12` is token 12, not token 1 followed by a literal `2`.
//! - `&*` substitutes the argument tokens AFTER the highest referenced `&N`
//!   (joined by single spaces). When the template references no `&N`, `&*`
//!   substitutes ALL tokens. This "after the highest referenced positional" rule
//!   means a template mixing `&1` and `&*` does NOT repeat tokens.
//! - A referenced `&N` with no corresponding token substitutes the EMPTY string
//!   and raises NO alias-layer error (Req 14.4).
//! - Any other character (including a bare `&` not followed by a digit or `*`) is
//!   literal text copied verbatim (Req 14.2).
//! - Tokenisation is whitespace splitting only; quoted multi-word positional
//!   arguments are out of scope (a `&N` is one whitespace token, Req 14.10).
//!
//! ## Surplus argument append (Req 14.5)
//!
//! When the argument string carries MORE tokens than the highest referenced `&N`
//! AND the template contains NO `&*`, the tokens after the highest referenced
//! `&N` are APPENDED to the expansion (space-separated, preceded by a single
//! space), reusing the verb-only residual-argument pass-through so no token is
//! silently discarded. RECORDED LIMITATION: the append keys off the highest
//! REFERENCED index, so a template referencing a non-contiguous subset (e.g. only
//! `&2`, skipping `&1`) will NOT re-append the lower unreferenced tokens.
//!
//! ## Single-pass + forward-compat guards (Req 14.7, 19.5)
//!
//! Expansion is SINGLE-PASS in this version: the returned string is NOT re-fed
//! through alias expansion by the caller. A future chaining capability, if gated
//! separately, would bound recursion with an explicit maximum expansion DEPTH and
//! a VISITED-ALIAS set; both are recorded here as the additive path:
//!
//! - [`MAX_EXPANSION_DEPTH`] -- the hard depth cap a future chaining loop would
//!   enforce (1 = today's single pass).
//! - A visited-alias set (surface names already expanded on this invocation)
//!   would be threaded alongside the depth counter to detect cycles.
//!
//! Neither guard is WIRED in v1 (there is no loop to bound); enabling chaining is
//! an additive change to the caller, not a new dispatch path.

/// The maximum alias-expansion depth. v1 is strictly single-pass, so this is `1`:
/// one alias level is expanded per invocation and the result is never re-fed.
/// Recorded as the forward-compat knob a future chaining CR would raise (together
/// with a visited-alias set); it is NOT consulted by any loop in v1.
// Documented forward-compat knob only; it is deliberately not consulted by any
// loop in v1 (single-pass), so it is dead in BOTH the bin and test builds.
#[allow(dead_code)]
pub(crate) const MAX_EXPANSION_DEPTH: usize = 1;

/// Expand a parameter template against an alias invocation's argument string,
/// returning the fully resolved ROOT command STRING.
///
/// `root` is the alias's canonical verb (e.g. `EDIT`), `template` is the
/// parameter template (e.g. `FFWB.COBOL.MASTER(&1)`), and `arg` is the trimmed
/// remainder of the typed line after the alias verb (whitespace-tokenised for
/// `&N`/`&*`). The result is `root`, a single space, the substituted template,
/// and -- when `arg` has surplus tokens and the template has no `&*` -- a
/// space-separated append of the surplus tokens (Req 14.2, 14.5).
///
/// Pure and deterministic: no I/O, no shell state. Single-pass (Req 14.7); the
/// caller does NOT re-feed the result through alias expansion.
///
/// Validates: command-semantics Requirement 14.2, 14.3, 14.4, 14.5, 14.10
pub(crate) fn expand_template(root: &str, template: &str, arg: &str) -> String {
    let tokens: Vec<&str> = arg.split_whitespace().collect();
    let highest_ref = highest_referenced_index(template);
    let has_rest = template_has_rest_placeholder(template);

    let substituted = substitute(template, &tokens, highest_ref);

    let mut out = String::with_capacity(root.len() + 1 + substituted.len());
    out.push_str(root);
    out.push(' ');
    out.push_str(&substituted);

    // Surplus-append (Req 14.5): tokens after the highest referenced &N, only
    // when the template carries no &* (which would already have consumed them).
    if !has_rest && tokens.len() > highest_ref {
        let surplus = tokens[highest_ref..].join(" ");
        if !surplus.is_empty() {
            out.push(' ');
            out.push_str(&surplus);
        }
    }

    out
}

/// The highest positional index `N` referenced by any `&N` in `template`, or `0`
/// when the template references no `&N`. `&*` and bare `&` do not count.
fn highest_referenced_index(template: &str) -> usize {
    let mut highest = 0usize;
    let bytes = template.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'&' {
            let (idx, consumed) = parse_positional(&bytes[i + 1..]);
            if let Some(n) = idx {
                highest = highest.max(n);
            }
            i += 1 + consumed;
        } else {
            i += 1;
        }
    }
    highest
}

/// Whether `template` contains a `&*` rest placeholder.
fn template_has_rest_placeholder(template: &str) -> bool {
    let bytes = template.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'&' {
            if bytes.get(i + 1) == Some(&b'*') {
                return true;
            }
            i += 1;
        } else {
            i += 1;
        }
    }
    false
}

/// Scan the template left-to-right, copying literal text and substituting each
/// `&N` / `&*` placeholder. A bare `&` not followed by a digit or `*` is copied
/// verbatim (Req 14.2).
fn substitute(template: &str, tokens: &[&str], highest_ref: usize) -> String {
    let mut out = String::with_capacity(template.len());
    let bytes = template.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'&' {
            // &* -> tokens after the highest referenced &N (all tokens when no &N).
            if bytes.get(i + 1) == Some(&b'*') {
                let start = highest_ref; // 0 when no &N referenced
                let rest = tokens.get(start..).unwrap_or(&[]).join(" ");
                out.push_str(&rest);
                i += 2;
                continue;
            }
            // &N -> the Nth token (1-based), empty when absent.
            let (idx, consumed) = parse_positional(&bytes[i + 1..]);
            if let Some(n) = idx {
                if let Some(tok) = n.checked_sub(1).and_then(|k| tokens.get(k)) {
                    out.push_str(tok);
                }
                i += 1 + consumed;
                continue;
            }
            // Bare `&` (not &N, not &*): literal.
            out.push('&');
            i += 1;
        } else {
            // Copy one UTF-8 character starting at byte i verbatim.
            let ch_len = utf8_char_len(bytes[i]);
            let end = (i + ch_len).min(bytes.len());
            out.push_str(&template[i..end]);
            i = end;
        }
    }
    out
}

/// Parse a maximal run of ASCII decimal digits at the start of `after_amp`,
/// returning `(Some(index), digits_consumed)` or `(None, 0)` when the first byte
/// is not a digit. The index is the 1-based positional the `&N` refers to.
fn parse_positional(after_amp: &[u8]) -> (Option<usize>, usize) {
    let mut consumed = 0;
    let mut value: usize = 0;
    while consumed < after_amp.len() && after_amp[consumed].is_ascii_digit() {
        value = value
            .saturating_mul(10)
            .saturating_add((after_amp[consumed] - b'0') as usize);
        consumed += 1;
    }
    if consumed == 0 {
        (None, 0)
    } else {
        (Some(value), consumed)
    }
}

/// The byte length of the UTF-8 character whose leading byte is `b`.
fn utf8_char_len(b: u8) -> usize {
    if b < 0x80 {
        1
    } else if b >> 5 == 0b110 {
        2
    } else if b >> 4 == 0b1110 {
        3
    } else if b >> 3 == 0b11110 {
        4
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Validates: command-semantics Requirement 14.2, 14.9 -- the worked example:
    /// root EDIT, template FFWB.COBOL.MASTER(&1), arg "helloW" expands to
    /// EDIT FFWB.COBOL.MASTER(helloW); literal template text is preserved verbatim.
    #[test]
    fn worked_example_single_positional() {
        assert_eq!(
            expand_template("EDIT", "FFWB.COBOL.MASTER(&1)", "helloW"),
            "EDIT FFWB.COBOL.MASTER(helloW)"
        );
    }

    /// Validates: command-semantics Requirement 14.4 -- an unsupplied &N
    /// substitutes the EMPTY string (no alias error), so `ecob` with no arg yields
    /// EDIT FFWB.COBOL.MASTER().
    #[test]
    fn unsupplied_positional_substitutes_empty() {
        assert_eq!(
            expand_template("EDIT", "FFWB.COBOL.MASTER(&1)", ""),
            "EDIT FFWB.COBOL.MASTER()"
        );
    }

    /// Validates: command-semantics Requirement 14.5 -- surplus tokens after the
    /// highest referenced &N are APPENDED (space-separated) when the template has
    /// no &*, so `ecob a b` -> EDIT FFWB.COBOL.MASTER(a) b.
    #[test]
    fn surplus_tokens_are_appended_when_no_rest_placeholder() {
        assert_eq!(
            expand_template("EDIT", "FFWB.COBOL.MASTER(&1)", "a b"),
            "EDIT FFWB.COBOL.MASTER(a) b"
        );
    }

    /// Validates: command-semantics Requirement 14.3 -- `&1` takes the first
    /// token and `&*` takes the tokens AFTER the highest referenced &N, with NO
    /// repetition: template "&1 TO &*" with [a,b,c] -> "a TO b c".
    #[test]
    fn rest_placeholder_takes_tokens_after_highest_positional_no_repeat() {
        assert_eq!(expand_template("CMD", "&1 TO &*", "a b c"), "CMD a TO b c");
    }

    /// Validates: command-semantics Requirement 14.3 -- when the template
    /// references no &N, `&*` substitutes ALL tokens.
    #[test]
    fn rest_placeholder_without_positional_takes_all_tokens() {
        assert_eq!(
            expand_template("CMD", "PREFIX &*", "a b c"),
            "CMD PREFIX a b c"
        );
    }

    /// Validates: command-semantics Requirement 14.3 -- a two-digit positional
    /// `&12` refers to token 12 (maximal digit run), not token 1 then literal 2.
    #[test]
    fn two_digit_positional_is_token_twelve() {
        // Exactly 12 tokens: &12 is the maximal-digit-run positional (token 12),
        // and there is no surplus token beyond the highest referenced index, so
        // the two-digit parsing is isolated from the surplus-append of Req 14.5
        // (covered separately).
        let arg = "t1 t2 t3 t4 t5 t6 t7 t8 t9 t10 t11 t12";
        assert_eq!(expand_template("CMD", "<&12>", arg), "CMD <t12>");
    }

    /// Validates: command-semantics Requirement 14.2 -- a bare `&` not followed by
    /// a digit or `*` is literal text, and a template with no placeholder is
    /// copied verbatim (surplus tokens still append since there is no &*).
    #[test]
    fn bare_ampersand_and_no_placeholder_are_literal() {
        assert_eq!(expand_template("CMD", "A&B", ""), "CMD A&B");
        assert_eq!(expand_template("CMD", "LITERAL", ""), "CMD LITERAL");
    }

    /// Validates: command-semantics Requirement 14.10 -- tokenisation is
    /// whitespace splitting only (multiple spaces collapse; a &N is one token).
    #[test]
    fn tokenisation_is_whitespace_only() {
        assert_eq!(
            expand_template("CMD", "(&1)(&2)", "  one   two  "),
            "CMD (one)(two)"
        );
    }

    /// Validates: command-semantics Requirement 14.5 -- the recorded limitation:
    /// a non-contiguous template (references only &2, skips &1) does NOT re-append
    /// the lower unreferenced token &1; the append keys off the highest REFERENCED
    /// index (2), so only tokens after index 2 are appended (here none).
    #[test]
    fn non_contiguous_template_does_not_reappend_lower_unreferenced_token() {
        // arg = "a b": &2 -> "b"; highest referenced = 2; no tokens after index 2.
        assert_eq!(expand_template("CMD", "[&2]", "a b"), "CMD [b]");
        // arg = "a b c": highest referenced = 2, so "c" (token 3) is appended.
        assert_eq!(expand_template("CMD", "[&2]", "a b c"), "CMD [b] c");
    }
}
