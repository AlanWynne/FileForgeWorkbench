# Gate Review -- CR-NR-108 (load libraries / RECFM=U / program search)

Docs-only requirements-gate review. Reviewed fresh against the real current
types in `crates/ff-dscatalog/src/dataset.rs` and the existing numbering in
`docs/specs/dataset-catalog/requirements.md`. No build or test was run (nothing
to run for a docs gate).

## Verdict

APPROVED. Zero HIGH and zero MEDIUM findings. Only NITs below.

## Scope confirmation

The CR-NR-108 gate authored only the six required docs files. A grep of
`crates/**/*.rs` for `CR-NR-108 | LibraryKind | resolve_program | loadlib |
LoadLibrary | MemberContentType` returned NO matches: no load-library / RECFM=U /
program-search code was written. `git status` does show modified/untracked `.rs`
files (`dataset_access/impl_catalog.rs`, `tests_crch061.rs`,
`mainframe_backend.rs`, command-environments spec edits), but these are the
PARALLEL CR-CH-061 "catalog-backed DatasetAccess" work stream (their own
`.agents/tasks/crch061-live-handles/` folder), unrelated to this gate. Item 8 is
therefore satisfied for CR-NR-108: this gate is documentation only.

## Checklist results

1. EARS phrasing -- PASS. Every new criterion (37.1-37.7, 38.1-38.8, 39.1-39.7)
   uses `WHEN ... THE ... SHALL ...` or a bare `THE ... SHALL ...` state rule.
   Spot examples: 37.2 "WHEN `AllocParams::validate` is called ... THE validation
   SHALL succeed"; 38.1 "THE `ff-dscatalog` crate SHALL define a ... enum"; 39.1
   "THE `ff-dscatalog` crate SHALL expose a Program_Search ...".

2. Numbering -- PASS. Real last existing requirement is Requirement 36; new reqs
   continue 37, 38, 39. Real last task is Task 43; new tasks continue 44, 45, 46.
   Last change-log id is CR-NR-107; new id is CR-NR-108 (next unprefixed, mainline
   stream, no duplicate). All sequential and correct.

3. No contradiction with existing criteria -- PASS. The one real interaction is
   the existing Requirement 7 AC 10 LRECL rule ("LRECL must be > 0 and <= 32760;
   BLKSIZE must be >= LRECL"). Req 37 does NOT delete or silently contradict it:
   37.2 and 37.3 explicitly narrow that rule to non-U formats and state it
   "SHALL continue to apply for RECFM F/FB/V/VB", and the Req 37 framework note
   records the in-place amendment-in-scope. This is a clean, documented narrowing,
   consistent with the real `AllocParams::validate` which today rejects `lrecl ==
   0` unconditionally (the implementation will relax it for U). Design.md section
   1 restates the same amendment.

4. Additive, builds ON existing types -- PASS. Verified against `dataset.rs`:
   `Recfm` already carries the `U` variant, so Req 37 adds NO variant and only
   relaxes `validate`/`with_defaults` branches for the existing `Recfm::U`. Req 38
   adds two NEW `#[non_exhaustive]` enums (`LibraryKind`, `MemberContentType`) and
   OPTIONAL fields on `AllocParams`/`DatasetRecord` (default `None` = record-data),
   leaving `Dsorg`, `Recfm`, and `PartitionedSubtype` unreshaped. Req 39 adds a
   read-only `resolve_program` primitive plus additive result types. No
   `ff-vfs::BackendEnvironment` reshape; the change-log and design both state the
   store contract is untouched. No breaking reshape is baked in -- NOT a blocking
   finding.

5. Ownership (ADR-001) -- PASS. All new types and the resolution primitive are
   explicitly placed in `ff-dscatalog`. The JCL concatenation / `EXEC PGM=` /
   compile / link work is deferred and cross-referenced to `jes-emulator` and
   `compiler-toolchain-integration` (new Cross-References rows + design "deferred"
   section + tasks coverage table "future / DEFERRED" rows). Allocator / JES
   boundaries are referenced, not duplicated. Req 39.4 keeps JCL concatenation out
   of ff-dscatalog (caller assembles the ordered list).

6. Plain ASCII -- PASS for all new content. A line-ranged scan of every new
   region (requirements 896+, design 2285+, tasks 746+, master 3526+, TCR
   3984-4009, change-log 25-32) found the ONLY non-ASCII characters to be the
   red-circle status emoji in the TCR rows, which documentation.md explicitly
   allows in `docs/quality/TCR.md` tables. The pre-existing `<=`/`>=`/en-dash/`->`
   glyphs flagged by the author sit in OLD content (e.g. Req 7 AC 10 line 232,
   Glossary 1-8 ranges) that CR-NR-108 did not author; see NIT 1.

7. TCR + master Summary consistency -- PASS. TCR adds exactly 22 NOT COVERED
   (red-circle) rows, all in the `ff-dscatalog` crate column, one per new
   criterion (37.1-37.7 = 7, 38.1-38.8 = 8, 39.1-39.7 = 7 = 22). Master tasks add
   Phase (loadlib-model) with LLM.1-LLM.3 mapped to dataset-catalog Tasks 44-46
   and a per-phase `| Status | Count |` row (matching the existing CR-CH-060/061
   per-phase pattern; the file has no maintained grand-total, so none is bumped).
   The tasks.md "Acceptance Criteria Coverage" table correctly maps every
   criterion to a task (Req 37 -> 44.1-44.3, Req 38 -> 45.1-45.5, Req 39 ->
   46.1-46.5). Cross-checked: all 22 criteria are covered by at least one task.

8. No code written by this gate -- PASS (see Scope confirmation above).

## Verified Assumptions

- `Recfm` enum in `dataset.rs` already has the `U` variant -- so Req 37 needs no
  new variant. CONFIRMED by reading `dataset.rs` lines defining `enum Recfm { F,
  FB, V, VB, U }`.
- `AllocParams::validate` today rejects `lrecl == 0` unconditionally (`if lrecl
  == 0 || lrecl > 32760`). CONFIRMED -- so the "relax for U" design is accurate
  about the real starting point.
- `AllocParams::with_defaults` injects `Recfm::FB` / `lrecl = 80` / `blksize =
  27920` for `Dsorg::PS | PO`. CONFIRMED -- Req 37.5/37.6's "do not inject
  FB/80 for U" is a real behaviour change on a real branch.
- `AllocParams` and `DatasetRecord` both exist with the fields design.md says it
  will extend (`recfm: Option<Recfm>`, `lrecl: Option<u32>`, `subtype:
  Option<PartitionedSubtype>`). CONFIRMED.
- Last existing Requirement = 36, last Task = 43, last CR-NR = 107. CONFIRMED by
  reading the file ends and change-log.
- No `.rs` file references any CR-NR-108 concept. CONFIRMED by grep.

## Unverified / Wrong Assumptions

- None material. The design's claim that member-level content-type can reuse
  "existing member metadata (Requirement 8)" is a forward reference to member
  operations whose storage detail is not fully pinned here; design.md itself
  flags this as "may be deferred to the member-operations implementation", so it
  is an acknowledged open implementation detail, not a wrong assumption. Captured
  as NIT 2.

## Findings

NIT 1 (requirements.md Req 7 AC 10, line ~232). The existing LRECL criterion that
Req 37 narrows still carries the prohibited Unicode `<=` / `>=` glyphs
(pre-existing, not authored by this gate). Req 37 correctly narrows it by note
rather than editing it. Concrete fix (at implementation time, author decision 1):
when AC 10 is edited in place to add "for RECFM F/FB/V/VB", also replace the two
Unicode glyphs with ASCII `<=` / `>=`. Not blocking -- the glyphs predate this CR
and the narrowing is unambiguous as written.

NIT 2 (design.md section 2 / Req 38.3). "per-member content type where member-level
metadata exists" leaves the per-member storage path semi-specified. Concrete fix:
when Requirement 8 member storage is implemented, record whether per-member
content-type is stored or derived-only; design.md already states the dataset-level
`LibraryKind`-derived default governs in the interim, which is sufficient for the
gate.

NIT 3 (design.md section 3, `resolve_program` signature). The primitive takes
`catalog: &impl CatalogQuery` but `CatalogQuery` is named as "existing catalog
query surface" without a definition reference. Concrete fix: at implementation,
confirm the concrete trait/type name (or introduce the minimal query trait) and
cite it; the proposed shape is clearly marked "final signatures in
implementation", so this is a NIT, not a gap.
