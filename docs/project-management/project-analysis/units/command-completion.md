# Analysis Record: command-completion (W3.2)

- **Wave**: 3 (Shell, commands, menus, session)
- **Backing crate**: `ff-completion` (spec says `ff-command-completion`; actual dir
  is `ff-completion` -- naming drift)
- **Spec files**: requirements.md (317 lines, 10 requirements), tasks.md
  (191 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 3 pass

---

## 1. Split candidacy

NOT a split candidate.

- Requirement volume: 10 reqs, 317 lines. Below thresholds. (no firm signal)
- Responsibilities: ONE cohesive concern -- auto-complete for the command field /
  line-command prefix (candidate generation, matching, ranking, selection, popup
  positioning, triggers). Well-decomposed into 18 files across matching/ (prefix,
  fuzzy) and provider/ (command_name, file_path, keyword, line_command, macro_name).
- Crates: single crate; GUI-independent core (Req principle 1).
- Cohesion: high.

No split.

### Source file size violation (PA-STD-033)

`engine.rs` = 478 non-test lines, over the 400 cap (the Completion_Engine core:
candidate generation + filtering + ranking + selection state). Split by concern
(candidate aggregation vs filter/rank vs selection-state management). REFACTOR,
no gate. Recorded PA-STD-033. (Only file over cap.)

---

## 2. Cross-unit consistency -- CLEAN (well-architected consumer)

command-completion is a model CONSUMER: it reads from many upstream sources but
duplicates nothing, via a clean provider-trait design.

- `CompletionProvider` trait (provider/mod.rs) is the injection seam. Built-in
  providers (command_name, file_path, keyword, line_command, macro_name) implement
  it; the concrete sources are INJECTED:
  - command_name provider sources from the command-framework `CommandRegistry`
    (comment: "the actual CommandRegistry" injected) -- Req 1, principle 2.
  - file_path provider takes injected entries (`with_entries`), NO raw fs (0 fs
    calls) -- VFS async listing is caller-supplied per Req principle 3 /
    FFW-ARCH-001. CLEAN (no FFW-ARCH-001 violation).
  - macro_name from lua-macro-engine; line_command from line-commands; keyword
    from config/language.
- All consumption is read-only via the provider trait -- no duplicated registry,
  no duplicated command/line-command/macro lists. Correct single-owner-many-readers.
- Config keys `completion.*` (trigger_mode, auto_trigger_chars, matching_mode,
  max items, popup dims) -- sole reader; ff-config. No key collision.
- Deps: ff-command + ff-config + ff-logging. GUI-independent core; the popup
  rendering/positioning (Req 3) coordinates with the egui shell but the engine is
  headless-testable (Req principle 1). No egui dep in the crate. Clean layering.
- Req 10 (Provider Extensibility): plugins register custom providers via the same
  trait -- consistent with plugin-architecture. Confirm the plugin registration
  path at plugin-manager-ui (Wave 4).

### Cross-reference integrity

All 6 declared cross-refs (command-framework, command-semantics,
virtual-file-system, lua-macro-engine, configuration-system, line-commands)
resolve and are correctly consumed (injection). No dangling refs, no duplication.
This is the cleanest cross-unit story in Waves 2-3 so far.

---

## 3. Completeness

Tracking: all 191 sub-tasks `[x]`. Implementation present across all 10 reqs
(name completion, argument completion, popup positioning, navigation/selection,
dismiss, fuzzy matching, line-command completion, macro completion, trigger config,
provider extensibility). Tests in-file. No false-positive pattern. No
PA-INCOMPLETE.

### TCR gap (PA-TCR-014)

TCR.md has 1 row for the crate against 10 reqs / ~90 criteria. Thin (contrast the
sibling command-semantics' 39). Recorded PA-TCR-014.

---

## 4. Logging audit

Scan of `crates/ff-completion/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: PRESENT -- 0 uses. DEAD.
- `println!` / `eprintln!`: 0
- `std::fs` / `tokio::fs`: 0

THREE mandated logs, unimplemented:
- Req 6.5 / 9.5: invalid `completion.*` config value -> fall back + (WARN expected).
- Req 10.5: a CompletionProvider that fails/panics -> the Completion_Engine SHALL
  catch the failure (and log it) -- provider isolation. This one matters: a failing
  provider should be logged so a broken plugin provider is diagnosable.

Recorded PA-LOG-019 (LOW-MED): wire ff-logging (dead dep) for the config-coercion
WARN (6.5/9.5) and the provider-failure catch (10.5); dev-logging on completion
trigger/candidate-generation is a nice-to-have. The provider-failure log (10.5) is
the substantive one -- it aids debugging plugin providers (CR-NR-058 territory).

---

## 5. Task revision proposals

- **PA-STD-033 (REFACTOR)**: split `engine.rs` (478 non-test) by concern. No gate.
- **PA-LOG-019 (LOW-MED)**: wire ff-logging for the Req 10.5 provider-failure catch
  (substantive) + Req 6.5/9.5 config-coercion WARN + dev-logging on trigger.
- **PA-STD-034 (ASCII, comment-only)**: 60 non-ASCII bytes, ALL in comments
  (box-drawing banners + em/en dashes; 0 in runtime strings -- clean like
  structure-catalog). documentation.md violation. Replace with `--` / `// === ===`.
  REFACTOR, no gate.
- **PA-TCR-014**: enumerate per-requirement TCR rows (1 row for 10 reqs). No code.
- **PA-DOC (naming)**: spec crate `ff-command-completion`; actual `ff-completion`.
  Fold into the naming reconciliation set.

No requirement CHANGE proposed; the spec is internally consistent and complete.

---

## Summary

command-completion (`ff-completion`, spec says `ff-command-completion`) is a clean,
well-architected auto-complete subsystem and the best cross-unit story in Waves
2-3: a `CompletionProvider` trait injects all upstream sources (command-framework
registry, VFS paths, macro names, line commands, keywords) with ZERO duplication
and 0 fs (VFS listing caller-supplied, no FFW-ARCH-001 issue). Not a split
candidate. Findings are all mechanical: `engine.rs` over the 400 cap (PA-STD-033),
dead ff-logging dep with three mandated logs unimplemented -- notably the Req 10.5
provider-failure catch (PA-LOG-019), comment-only ASCII (PA-STD-034), thin TCR
(PA-TCR-014), and the ff-completion/ff-command-completion naming drift.
