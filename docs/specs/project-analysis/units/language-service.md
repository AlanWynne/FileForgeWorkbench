# Analysis Record: language-service (W5.9)

- **Wave**: 5 (emulators, tools, connectors)
- **Backing crate**: `ff-language-service` (foundational language layer: definition
  loading from TOML, extension + content-based detection, multi-line lexer state,
  keyword/comment/string syntax, embedded languages, plugin-extensible registration,
  query API -- feeds syntax-highlighting)
- **Spec files**: requirements.md (217 lines, 10 requirements), tasks.md
  (120 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 5 pass

---

## 1. Split candidacy + cap (PA-STD-063)

10 reqs / 217 lines -- not a crate-split candidate. But two files at/near the cap:
`definition.rs` = 464 (over the 400 cap), `query.rs` = 378 (near). Recorded PA-STD-063
(MEDIUM -- cap): split definition.rs by concern (definition model / TOML load / keyword+
comment+string sections / embedded-language handling). REFACTOR.

---

## 2. Cross-unit consistency -- PA-CONFLICT-018 (FIFTH orphan: foundation crate its stated consumer ignores)

The spec's entire premise: "the foundational layer ... that the syntax-highlighting
engine CONSUMES to tokenize and colour source code." But:

- `ff-language-service` is referenced by NO crate (workspace Cargo.toml mentions = 1,
  its own; 0 `ff_language_service` refs anywhere else). ORPHAN.
- `ff-syntax-highlighting` does NOT depend on it (0) and instead does its OWN language
  detection/definition INLINE (5 refs to detect / LanguageDef / by_extension /
  content-based in its own src).
- The shell (ff-desktop) has 0 language_service refs either.

So the stated consumer reimplements the exact capability the foundation crate provides.
This is the FIFTH orphan crate (w/ ff-file-tree PA-CONFLICT-011, ff-idle-processing
PA-CONFLICT-012, ff-large-file-performance PA-CONFLICT-013, ff-external-mod
PA-CONFLICT-014; idcams PA-CONFLICT-016 is a related unwired-engine). Sharpest parallel:
PA-CONFLICT-012, where syntax-highlighting ALSO reimplements idle-styling inline while
ff-idle-processing sits orphaned -- syntax-highlighting now ignores TWO foundation crates
built for it (idle + language-service).

Recorded PA-CONFLICT-018 (owner-gated, MEDIUM-HIGH): rewire ff-syntax-highlighting onto
ff-language-service (consume its LanguageDefinition + detection + lexer-state API; delete
the inline detection) -- PREFERRED, it is the designed architecture; OR, if syntax-
highlighting's inline detection is the intended one, delete/absorb the orphan crate +
reconcile the spec. Pairs with the Wave-4 orphan cluster (PA-W4.1) and PA-CONFLICT-012
(same consumer). Code + owner decision.

### raw-fs TOML loading (note)

query.rs:320/328 load language-definition TOML via raw std::fs (read_dir +
read_to_string) despite an `ff-config` dep. Language-definition TOMLs are config-like;
routing through ff-config/VFS would be more consistent, but this is LOWER-stakes than the
JES job-DB raw-fs (PA-CONFLICT-015) -- these are read-only definition files. Folded into
PA-CONFLICT-018's rewire (when consumed, standardise the load path); not a separate hard
finding.

### Public types and ownership

- LanguageDefinition, extension + content detection, lexer-state persistence, keyword/
  comment/string syntax, embedded languages, query API -- sole-owned by
  `ff-language-service` but ORPHANED (PA-CONFLICT-018); DUPLICATED inline in
  ff-syntax-highlighting.

### Cross-reference integrity

The spec's syntax-highlighting cross-reference is NOT honoured in code (0 dep) --
PA-CONFLICT-018.

---

## 3. Completeness

Tracking: all 120 sub-tasks `[x]`. The crate is complete + tested AS A CRATE (definition
loading, detection, lexer state, query API). FUNCTIONALLY complete -- but orphaned +
duplicated (PA-CONFLICT-018), so the intended single-source-of-truth for language
detection is not realized. No PA-INCOMPLETE for the crate; the gap is architectural
(unwired + duplicated).

### TCR gap (PA-TCR-031) -- TOTAL ABSENCE

TCR.md has 0 rows for ff-language-service across 10 reqs, despite the crate's tests.
Recorded PA-TCR-031. (Consistent with the orphan crates' total-TCR-absence pattern --
idle PA-TCR-024, large-file PA-TCR-025, external-mod PA-TCR-026.)

---

## 4. Logging audit

- `ff_logging` / `log_*!`: 0; `ff-logging` Cargo dep: PRESENT -- 0 uses. DEAD.
- `std::fs`: 2 (TOML definition load, query.rs).

Language detection is a modest logging site: which definition loaded from where, detection
outcome (extension vs content-based), TOML parse failures (a bad language TOML should log,
not silently skip). ff-logging is a DEAD dep. Recorded PA-LOG-047 (LOW-MEDIUM): resolve the
dead dep + add dev-logging on definition load / detection outcome / TOML parse errors under
the `dev-logging` gate. (Most valuable once wired, PA-CONFLICT-018.)

---

## 5. Task revision proposals

- **PA-CONFLICT-018 (owner-gated, MEDIUM-HIGH)**: FIFTH orphan -- rewire
  ff-syntax-highlighting onto ff-language-service (delete its inline detection) [PREFERRED],
  OR delete/absorb the orphan + reconcile spec. Same consumer as PA-CONFLICT-012 (idle).
  Standardise the TOML-load path (ff-config/VFS) when wired. Code + owner decision.
- **PA-STD-063 (MEDIUM -- cap)**: split definition.rs (464) [query.rs 378 near].
  REFACTOR.
- **PA-LOG-047 (LOW-MEDIUM)**: resolve dead ff-logging + definition-load / detection /
  TOML-parse dev-logging.
- **PA-TCR-031**: add TCR rows (0 for 10 reqs). No code.
- **PA-STD-064 (ASCII)**: 1 non-comment non-ASCII (error.rs:44 -- likely em-dash in an
  error string). Replace with `--`. REFACTOR, no gate.

---

## Summary

language-service (`ff-language-service`) is a complete, tested foundational language layer
(TOML definition loading, extension + content-based detection, multi-line lexer state,
keyword/comment/string syntax, embedded languages, plugin registration, query API;
120/120). The headline finding is PA-CONFLICT-018 (MEDIUM-HIGH): it is the FIFTH orphan
crate -- referenced by NO crate -- and, worse, its STATED consumer `ff-syntax-highlighting`
does NOT depend on it (0) and instead reimplements language detection/definition INLINE.
This is the sharpest orphan case yet because the spec explicitly names the consumer; and
it is the SECOND foundation crate syntax-highlighting ignores (it also reimplements
idle-styling inline while ff-idle-processing sits orphaned, PA-CONFLICT-012). Rewire
syntax-highlighting onto ff-language-service (delete the inline detection) -- the designed
architecture -- or delete/absorb the orphan. Minor/hygiene: PA-STD-063 (definition.rs 464
over cap), dead ff-logging (PA-LOG-047), total TCR absence (PA-TCR-031), one error-string
non-ASCII (PA-STD-064), and raw-fs TOML loading to standardise on rewire. The crate itself
is sound; the gap is architectural (unwired + duplicated by its own consumer).
