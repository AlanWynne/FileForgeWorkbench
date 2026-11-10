# Gate Summary -- Parameterised Command Aliases (CR-NR-112)

Documentation-only requirements gate for the owner's change request (original
messages index 4-5): extend a command alias from a pure single-verb-to-single-root
rewrite into a mapping of a verb to a ROOT command WITH PARAMETERS and positional
substitution. First iteration -- full doc set authored from scratch, grounded in
`.agents/tasks/parameterised-aliases/findings.md` and re-verified against the live
spec files. NO source code was written.

## Allocations

- Change-request id: **CR-NR-112** (next free; re-verified -- highest existing was
  CR-NR-111, highest CR-CH-065).
- **command-framework Requirement 19** -- parameterised alias expansion runs in the
  existing pre-pass before `resolve_target`; routing unchanged (6 criteria 19.1-19.6).
- **command-semantics Requirement 14** -- the 4-field alias record + the `&1..&N` /
  `&*` substitution grammar, empty-sub rule, extra-arg append, single-pass,
  back-compat, worked example (10 criteria 14.1-14.10).

## Docs produced / edited

| File | Change |
|------|--------|
| `docs/status/change-log.md` | NEW entry CR-NR-112 (matches existing CR entry format). |
| `docs/specs/command-semantics/requirements.md` | NEW Requirement 14 (appended after Req 13). |
| `docs/specs/command-semantics/design.md` | NEW Section: Parameterised Command Aliases (grammar + substitution algorithm + extended record/TOML + testability). |
| `docs/specs/command-semantics/tasks.md` | NEW Phase (parameterised-aliases) Tasks 30-33. |
| `docs/specs/command-framework/requirements.md` | NEW Requirement 19 (appended after Req 18). |
| `docs/specs/command-framework/design.md` | NEW Section: pre-pass placement + routing proof + single-pass guards + menu finding + CR-NR-111 coexistence + explicit "no framework change". |
| `docs/specs/command-framework/tasks.md` | NEW Phase (parameterised-aliases) Tasks 32-35. |
| `docs/quality/TCR.md` | NEW CR-NR-112 coverage section -- 16 NOT-COVERED rows (one per criterion: 10 for Req 14, 6 for Req 19). |
| `docs/project-management/ROADMAP.md` | NEW CR-NR-112 tracking sub-bullet under the alias-layer cluster. |

## Owner decisions encoded (all settled, no change to them)

1. Placeholders: positional `&1..&N` AND `&*` (all remaining args). (Req 14.3)
2. No-arg / extra-arg: pure text substitution. Unsupplied `&N` -> EMPTY string
   (`ecob` -> `EDIT FFWB.COBOL.MASTER()`); surplus args beyond the highest `&N`
   (no `&*`) are APPENDED -- this reuses the verified verb-only residual-argument
   pass-through in `rewrite_verb_to_root`. (Req 14.4, 14.5)
3. Alias->alias chains: SINGLE-PASS in v1. Forward-compatible guards documented --
   a max expansion DEPTH limit (e.g. 10) AND a VISITED-SET of alias surfaces for
   cycle detection; enabling chaining later is ADDITIVE, not a redesign. (Req 14.7,
   command-framework Req 19.5 + design section)
4. Alias record: FOUR fields -- root, alias name, localisation key, parameter
   template -- stored as an additive `#[serde(default)] command_template` on the
   per-locale `i18n/<locale>/aliases/<env>.toml` surface. Absent/empty template ==
   today's pure verb rewrite (back-compat). A language-independent shorthand lives
   in the base/identity catalogue with no localisation key (no new neutral locale).
   (Req 14.1, 14.6; design section)
5. Menu composition (VERIFIED): the "command-line residue is appended" behaviour
   exists ONLY on the function-key seam, NOT on a menu-option click. So typed and
   key-bound parameterised aliases are served by the pre-pass; a parameterised
   alias used as a MENU OPTION'S command is OUT OF SCOPE for argument supply
   (routing the field into menu-option activation is a new mechanism needing
   separate owner confirmation). (Req 19.6)

## Framework-conformance

NO framework change is required. The expansion is a STRING-to-STRING rewrite in
the EXISTING `rewrite_verb_to_root` pre-pass (shell/locale_resolver.rs), strictly
upstream of `resolve_target`, called at the two existing seams (shell/dispatch.rs
typed; shell/commands_menu.rs menu). It adds no second dispatcher, no parallel
resolver, no `CommandTarget`/dispatch/nav/persistence change, and does not move
alias logic inside `resolve_target`. The only type change is an additive
`#[serde(default)]` field on an internal `ff-desktop` struct.

## Coexistence with CR-NR-111 (parallel gate on the same AliasTable)

Confirmed no double-expansion. CR-NR-111 stores the ROOT on a menu option and
derives the localised display SURFACE at render time (display axis); the parameter
TEMPLATE belongs to the ALIAS ENTRY (execution axis). A localised alias surface
that is also a parameter alias resolves surface -> canonical FIRST, THEN expands
the template -- one deterministic lookup. Cross-references localization Req 12,
command-framework Req 18, command-semantics Req 13 without contradicting them.
COORDINATION NOTE: both CR-NR-111 and CR-NR-112 edit `AliasEntry` / `AliasRowToml`;
the two field additions (`command_template` here; any CR-NR-111 mnemonic-source
field there) must be merged carefully at implementation time.

## Open decisions for the owner

1. **`&*` semantics when mixed with `&N`**: `&*` is defined as ALL remaining
   argument tokens (the whole argument string), not "tokens after the highest
   `&N`". Mixing `&1` and `&*` therefore repeats tokens by design (pure text
   substitution). This is documented, not special-cased. Confirm this is the
   intended meaning, or whether `&*` should instead mean "tokens after the highest
   referenced positional".
2. **Quoted multi-word positional args**: out of scope for v1 (a `&N` is one
   whitespace-delimited token, matching the current front-door split). Confirm
   this limitation is acceptable, or raise a follow-on CR if quoted args are wanted.
3. **Menu-option argument supply**: out of scope (item 5 above). Confirm, or raise
   a separate CR to route the command-line field into menu-option activation.

No finding required changing a settled owner decision or a framework mechanism, so
no `send_message` warning was raised. Awaiting the design-review iteration.
