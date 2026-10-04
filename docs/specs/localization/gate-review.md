# CR-NR-103 Localization Gate Review (docs-only requirements gate)

Reviewer: design-review subagent (fresh context). This review gates whether the
drafted CR-NR-103 localization documentation is complete, internally consistent,
and compliant with the house rules. It is a DOCS-ONLY gate: no cargo was run, no
source was built, and no crate source was inspected for correctness. The review
reads only the drafted docs and the authority context docs named in the step
brief.

Verdict: APPROVED (zero HIGH, zero MEDIUM findings).

---

## Scope of what was read

Drafted deliverables:
- `docs/specs/localization/requirements.md`
- `docs/specs/localization/design.md`
- `docs/specs/localization/tasks.md`
- `docs/project-management/project-master/tasks.md` (appended `Phase (localization)` + Summary row)
- `docs/quality/TCR.md` (appended NOT COVERED rows)
- `docs/status/change-log.md` (CR-NR-103 Status flipped to IN PROGRESS)

Authority context:
- `docs/specs/localization/localization-vision.md`
- `docs/specs/command-environments/requirements.md` (Req 6a.1-6a.7)
- `.agents/tasks/localization-i18n-audit/i18n-posture-audit.md`
- `.kiro/steering/` (workflow.md, specs.md, documentation.md, framework-conformance.md)

---

## Check 1 -- EARS, numbering, Introduction, Glossary

PASS. `requirements.md` has an Introduction (with a deliberately-narrow
first-gate scope, source references, and cross-references) and a Glossary. The
Glossary defines every mandated term: Locale, Message_Catalogue,
Catalogue_Lookup_Seam, Locale_Key, Alias_Catalogue, Identity_Base.

Requirements are numbered sequentially Requirement 1 through Requirement 10, and
every acceptance criterion is in EARS form -- either an unconditional
`THE ... SHALL ...` or a conditional `WHEN/WHERE ... THE ... SHALL ...`. Spot
confirmation across all requirements: Req 1.1-1.6 (mix of THE/WHEN/WHERE SHALL),
Req 2.1-2.5, Req 3.1-3.5, Req 4.1-4.4, Req 5.1-5.6, Req 6.1-6.4, Req 7.1-7.4,
Req 8.1-8.3, Req 9.1-9.3, Req 10.1-10.3. Criteria are numbered sequentially
within each requirement. Total = 43 criteria, matching the change-log and TCR.

## Check 2 -- Coverage

PASS. Every mandated coverage item is present:

- `ui.locale` key + hot-reload: Req 1.1-1.6 (default "en", allowed_values,
  CallbackRegistry::on_reload, both-catalogue reload, defence-in-depth fallback).
- Fluent message-catalogue mechanism + per-locale `.ftl` data: Req 2.1-2.5.
- `t("key")` lookup seam threaded through render paths: Req 3.1-3.5 (incl. Req
  3.4 "no egui API change" and Req 3.5 "GUI-free crate").
- English base catalogue always present: Req 2.5, Req 1.3, Identity_Base glossary.
- Verb/text split consistent with Req 6a: Req 4.1-4.4 (localise only text; never
  verbs/identifiers; description-not-command; never text-as-key).
- Per-locale, per-environment alias catalogue into the EXISTING AliasTable with
  no new mechanism: Req 5.1-5.6, each cross-referencing command-environments Req
  6a.1/6a.3/6a.5/6a.6/6a.7.
- Help per-locale directories: Req 6.1-6.4.
- Menu DEFAULT_*_TOML title/description localisation via the parsed menu model:
  Req 7.1-7.4 (incl. code-only built-in menus, CR-CH-021).
- Plugin UI + CR-CH-053 plugin command environments through the same seam:
  Req 8.1-8.3.
- Argument-bearing / verb-embedding messages via Fluent placeables with the
  embedded verb kept as a non-translated argument: Req 9.1-9.3.
- Additive / no-framework-change statement: Req 10.1-10.3, plus the Introduction
  ("changes NO core framework mechanism") and the design "Framework-conformance
  statement".

## Check 3 -- Decisions encoded (not left open)

PASS.

- Fluent is CHOSEN in `design.md` ("Technology stack (locked once approved)" and
  the "Fluent over gettext (decision, with the rejected tradeoff recorded)"
  section). gettext-style `.po`/flat-map is recorded as the REJECTED alternative
  with its tradeoff, and the text explicitly states "This is recorded as
  considered-and-rejected, not an open question." No open-question framing.
- `ui.locale` String default "en" with allowed_values pinned to shipped locales
  and a reload callback that reloads BOTH the message catalogue AND the
  per-locale/per-environment alias catalogues: design "The `ui.locale` key wiring
  in `ff-config`", steps 1-3, matching Req 1.1-1.5.
- Narrow first-gate scope (mechanism + key + English base + seam + alias loader,
  not a full extraction): requirements Introduction "First-gate scope
  (deliberately NARROW)" and "Out of Scope (this gate)".
- Ship English only first: Introduction + Out of Scope ("It ships ONLY the `en`
  catalogue"); other-language content deferred to the capstone.
- Design references the real audit paths: `ff-config/src/keys.rs`,
  `init/schema.rs`, `callback.rs` / the reload mechanism, `config_handle/mod.rs`
  (re-entrancy constraint), `ff-help/src/content_loader.rs`,
  `ff-desktop/src/shell/environment.rs`, and `menu_workspace/defaults.rs`. All of
  these match the audit's cited paths.
- Explicit "changes no framework mechanism" conformance statement: design
  "Framework-conformance statement" + Req 10.1.

## Check 4 -- Tasks

PASS.

- `tasks.md` uses ONLY `[ ]` markers (no `[x]`, no other symbols). Verified by
  reading every task line; the one `NOTE:` line under Task 10 is a sub-bullet,
  not a checkbox.
- Every task line has a descriptive title.
- Tasks cross-reference criteria ("Satisfies Req X.Y").
- Ordering: Phase 1 begins with the mechanism (Task 1 `ff-i18n` crate + seam),
  config key (Task 2), English base (Task 3), and the alias loader (Task 4) --
  matching "mechanism + key + English base + seam + alias loader" first. Phase 2
  (Tasks 5-9) proceeds through the incremental extraction in the audit's
  recommended order: menu titles (5), help per-locale (6), clean dialog/panel
  crates (7), ff-desktop render literals (8), hard status/error messages +
  plugin seam (9). This matches the audit's 7-step order (config/seam, menu,
  help, dialog/panel crates, ff-desktop literals, status/error, verb aliases),
  with the alias loader MECHANISM pulled into Phase 1 (Task 4) and alias DATA in
  the capstone -- a sound split consistent with the audit's step 7 ("alias
  catalogues land as DATA once the loader exists").
- Capstone LAST: Phase 3 / Task 10 "Build out the other-language catalogue data"
  is the final task group, explicitly notes it depends on Phase 2 extraction
  being substantially complete, authors fr/de message + per-environment alias +
  help data, and updates `ui.locale` allowed_values (Task 10.4). It is present
  and last.

## Check 5 -- Master + TCR + Changelog

PASS.

- Master `tasks.md`: a new `### Phase (localization) -- CR-NR-103 ...` section
  with tasks L10N.1-L10N.5, all `[ ]` only, and a Summary count row
  (`| [ ] Phase (localization) | CR-NR-103 GATE AUTHORED: ... |`). The phase
  narrative and tasks are consistent with the sub-project requirements/design.
- TCR: one NOT COVERED row per criterion, 43 rows total, under a dedicated
  `### Localization / i18n (CR-NR-103, ...)` section, each using the file's
  existing NOT COVERED symbol. Crate placement is correct: Req 1.x -> `ff-config`
  (6 rows); Req 2.x and 3.x -> `ff-i18n` (10 rows); Req 4.x/5.x/7.x/8.x/9.x/10.x
  -> `ff-desktop` (26 rows); Req 6.x -> `ff-help` (4 rows). 6 + 10 + 26 + 4 = 43.
  This matches the four crate sections named in the brief (ff-i18n / ff-config /
  ff-desktop / ff-help).
- change-log.md: the existing CR-NR-103 entry has its Status flipped from
  PENDING GATE to IN PROGRESS, with the gate-authored detail appended inline. No
  duplicate CR-NR-103 entry exists (a search returns the single entry under
  `## New Requirements`).

## Check 6 -- Plain ASCII

PASS. A regex scan for any non-ASCII byte over `docs/specs/localization/*.md`
returned no matches, so there are no em dashes, en dashes, curly quotes,
ellipses, arrows, or logic/math symbols in the drafted sub-project docs. The
master-tasks and TCR additions reviewed inline use ASCII `->`, `--`, and (in
TCR) the pre-existing status emoji permitted by documentation.md for that file.

## Check 7 -- No source touched (localization change confined to docs)

PASS. The CR-NR-103 localization change is confined to `docs/`:
`docs/specs/localization/` (new), and edits to
`docs/project-management/project-master/tasks.md`, `docs/quality/TCR.md`, and
`docs/status/change-log.md`.

The working tree does contain many modified/new files under `crates/`
(the ff-desktop shell decomposition and the command-environments work), but
these belong to the OTHER in-flight stream in this branch, not to CR-NR-103. A
targeted search confirms NO localization source exists: there is no `ff-i18n`
crate anywhere, no `ui.locale` key, and no Fluent / catalogue-lookup code under
`crates/` (the only `fluent` hit is an unrelated "Fluent builder" doc comment in
`ff-workflow`; the only `format_args!`/`t_args`-adjacent hits are in a logging
macro). The gate added no code, as a docs-only gate requires.

---

## Verified Assumptions

1. The Glossary defines all six mandated terms -- VERIFIED by reading
   `requirements.md` Glossary.
2. All 43 criteria are EARS and sequentially numbered Req 1-10 -- VERIFIED by
   reading every criterion.
3. Fluent is a locked decision with gettext recorded as rejected (not open) --
   VERIFIED in `design.md` "Fluent over gettext" and "Technology stack (locked
   once approved)".
4. `ui.locale` is String, default "en", allowed_values pinned, reload callback
   reloads BOTH catalogues -- VERIFIED in Req 1.1-1.5 and design step 1-3.
5. Design cites the real audit paths (keys.rs, init/schema.rs, callback/reload,
   content_loader.rs, environment.rs, menu_workspace/defaults.rs) -- VERIFIED
   against the audit's cited `path:line` references.
6. Alias loading reuses the EXISTING per-environment AliasTable with no new
   mechanism, per-environment-per-locale, English identity + overlay, no per-row
   locale tag, within-environment collision rejection -- VERIFIED in Req 5.1-5.6
   and cross-checked against command-environments Req 6a.1/6a.3/6a.5/6a.6/6a.7,
   which it faithfully realises without contradiction.
7. tasks.md uses only `[ ]`, titles present, criteria cross-referenced, order
   starts with mechanism+key+base+seam+alias-loader and ends with the capstone --
   VERIFIED by reading tasks.md end to end.
8. Master phase present with `[ ]`-only tasks and an updated Summary row -- VERIFIED.
9. TCR has 43 NOT COVERED rows across ff-config/ff-i18n/ff-desktop/ff-help --
   VERIFIED by count and crate.
10. change-log CR-NR-103 flipped to IN PROGRESS, single entry -- VERIFIED.
11. No non-ASCII in the localization sub-project docs -- VERIFIED by regex scan.
12. No localization source under crates -- VERIFIED by targeted search + absence
    of an ff-i18n crate.
13. requirements realise localization-vision.md (locale key, catalogue, t()/tr!
    seam, per-locale data, per-environment-per-locale aliases, plugin seam,
    additive layer) and do not contradict it -- VERIFIED against the vision's
    "Likely implementation shape" and "Relationship to other work".

## Unverified / Wrong Assumptions

None. All claims checked against the drafted docs and authority context held.
(As a docs-only gate, nothing was executed or compiled; correctness of the
future implementation is out of scope and is covered by the NOT COVERED TCR rows
and the TDD obligation recorded in tasks.md.)

---

## Findings

No HIGH, MEDIUM, or NIT findings. The gate package is complete, internally
consistent, consistent with the vision and command-environments Req 6a, and
compliant with the EARS / plain-ASCII / task-format / TCR / change-log house
rules.
