# Analysis Record: jcl-resolver (W5.3)

- **Wave**: 5 (emulators, tools, connectors)
- **Backing crate**: NONE (no `ff-jcl` / `ff-jcl-resolver` crate exists)
- **Spec files**: `.gitkeep` ONLY -- no requirements.md, design.md, or tasks.md
- **Analysed**: Wave 5 pass (STUB)

---

## 1. Split candidacy

N/A -- empty stub, no crate, no requirements. Nothing to split.

---

## 2. Cross-unit consistency -- PA-CONFLICT-017 (JCL handling fragmented; stub never owned it)

`jcl-resolver` is a PENDING stub (docs/specs/jcl-resolver/ = `.gitkeep` only). The
project docs already acknowledge it as a placeholder (specs.md "stub -- no requirements
yet"; sub-project-audit.md "acceptable as placeholder"; project-master requirements.md
row 73 "stub placeholder for future JCL resolution and submission pipeline"). That
part is honest.

The problem is that JCL resolution ALREADY EXISTS -- spread across TWO other crates --
and the docs' explanation of where the stub's scope went is INACCURATE:

- `readiness-summary.md` claims (twice) the jcl-resolver requirements were "incorporated
  within / folded into FFW-JES Requirement 11". But FFW-JES (ff-jes) Req 11 is
  "Dataset Catalog Integration" (per the ff-jes requirements, W5.1) -- NOT JCL
  resolution. So the recorded destination is WRONG.
- Actual JCL resolution lives in:
  - `ff-dsalloc` (dataset-allocator): explicitly "the resolver processes JCL through
    four ordered stages" (JCL text -> Symbol table -> Catalog -> Lint rules), with
    symbolic-parameter scopes, JCL003/JCL012 lint codes, a `register_resolve_command`,
    a `[jcl]` config table, and symbolic-substitution error handling. A full JCL
    symbolic-resolution engine.
  - `ff-jes/ffjcl.rs` (FFJCL parser): "parses job definitions into a structured AST,
    validates them" -- the 469-line file flagged in the JES split (W5.1 PA-SPLIT-013).

So there are TWO JCL parsing/resolution implementations (ffjcl AST parser in ff-jes +
the symbolic resolver in ff-dsalloc), and the empty `jcl-resolver` sub-project -- which
by its name would OWN or UNIFY JCL resolution -- was never populated. Recorded
PA-CONFLICT-017 (owner-gated, LOW-MEDIUM): decide the JCL-resolution ownership --
either (a) keep JCL resolution distributed (ff-dsalloc symbolic resolver + ff-jes FFJCL
parser) and DELETE the misleading empty jcl-resolver stub after correcting the docs; or
(b) populate jcl-resolver as the unifying JCL sub-project and migrate the two
implementations under it. Given both implementations are complete and serve different
layers (allocator symbolic substitution vs JES job-definition AST), option (a)
(distribute + delete stub + fix docs) is the lighter path. Owner decision + docs.

---

## 3. Completeness

N/A for the stub itself (no requirements). But note: the readiness-summary's claim that
the scope was "folded into FFW-JES Req 11" is a DOCUMENTATION-ACCURACY defect
(PA-TRACK-005) -- Req 11 is catalog integration, not JCL. The real JCL work is complete
but lives in ff-dsalloc + ff-jes, not where the docs say.

---

## 4. Logging audit

N/A -- no crate.

---

## 5. Task revision proposals

- **PA-CONFLICT-017 (owner-gated, LOW-MEDIUM)**: resolve JCL-resolution ownership --
  (a) distribute (ff-dsalloc + ff-jes) + delete the empty jcl-resolver stub + correct
  the docs [lighter]; or (b) populate jcl-resolver as the unifying sub-project and
  migrate both implementations. Owner decision + docs.
- **PA-TRACK-005 (docs-accuracy)**: correct readiness-summary.md -- the jcl-resolver
  scope was NOT "folded into FFW-JES Requirement 11" (that req is Dataset Catalog
  Integration). Point the note at the real locations (ff-dsalloc symbolic resolver +
  ff-jes/ffjcl.rs FFJCL parser). Docs only.

No code (no crate). No PA-LOG / PA-TCR / PA-STD (nothing to scan) beyond the ff-jes
ffjcl.rs comment-mojibake already captured under PA-STD-056 (W5.1).

---

## Summary

jcl-resolver is an EMPTY STUB (`.gitkeep` only; no crate). The docs correctly flag it
as a placeholder, BUT the analysis surfaces a real cross-unit issue: JCL resolution
already exists, FRAGMENTED across two crates -- `ff-dsalloc` (dataset-allocator: a full
four-stage JCL symbolic resolver with lint codes + `[jcl]` config) and `ff-jes/ffjcl.rs`
(the FFJCL job-definition AST parser) -- while the readiness-summary INACCURATELY claims
the stub's scope was "folded into FFW-JES Requirement 11" (which is actually Dataset
Catalog Integration, not JCL). Recorded PA-CONFLICT-017 (LOW-MEDIUM): decide JCL-
resolution ownership -- distribute + delete the misleading stub + fix docs (lighter), or
populate jcl-resolver as the unifying home -- and PA-TRACK-005: correct the docs to
point at the real JCL locations. Nothing to split/log/cover (no crate). A Wave-6
spec-inventory item.
