# Analysis Record: encoding-and-characters

- **Wave**: 0 (foundation -- stateless character/encoding service layer)
- **Backing crate**: `ff-encoding`
- **Spec folder**: `docs/specs/encoding-and-characters/`
- **Analysed**: Wave 0, task W0.19 (CR-NR-057 re-baseline)
- **Verdict**: COMPLETE (111/111 tasks `[x]`, TCR PASS). BORDERLINE SPLIT
  CANDIDATE (14 reqs, 316 lines, 3 separable bands). Zero-log CORRECT
  (spec-mandated stateless). One 400-cap refactor (convert.rs 536).
- **CR-NR-057 impact**: NONE (encoding specs untouched). Re-verified from code.

---

## 1. Scope summary

`ff-encoding` is the GUI-independent character/encoding service, three bands:
- **Encoding I/O**: Req 1 detection, Req 2 BOM, Req 3 convert-on-load, Req 4
  convert-on-save, Req 5 UTF-8 validate/repair, Req 11 encoding-family, Req 14
  encoding state/registry.
- **Character classification**: Req 6 CharClassify, Req 7 Unicode category (UAX
  #31), Req 12 word-part nav, Req 13 configurable word-char sets.
- **Script-specific**: Req 8 DBCS, Req 9 grapheme clusters (UAX #29), Req 10
  Unicode case folding.

316 req lines, 14 requirements, single backing crate.

## 2. Split candidacy -- BORDERLINE

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 316 lines (near) AND 14 reqs (>12) | YES |
| 3+ distinct responsibilities | encoding-I/O band; char-classification band; script-specific (DBCS/grapheme/case-fold) band | YES (3) |
| 2+ crates | one crate | No |
| low-cohesion clusters | bands separable but share EncodingFamily + Unicode tables | Weak-Yes |
| file-size pressure | convert.rs 536 over cap; category_map 357 / registry 337 near | partial |

3/5. **BORDERLINE SPLIT CANDIDATE** (PA-SPLIT-003, proposal only).

### Proposed spec split (proposal, owner-gated)

- Keep `encoding-and-characters` = encoding I/O (Req 1-5, 11, 14).
- New `character-classification` = Req 6, 7, 12, 13. DBCS/grapheme/case-fold
  assigned per band. Crate split NOT recommended (bands share EncodingFamily +
  Unicode data). SPEC split improves traceability for the 14-req doc. Owner
  approval + own gate.

## 3. Consistency / conflict

- Public types owned here (EncodingFamily, CharClassify, CharacterCategoryMap,
  CaseFolder/ICaseConverter, ConversionResult/ConversionIssue, BOM detector, DBCS
  functions, grapheme boundary functions, encoding registry) -- sole owner. Added
  to consistency-matrix.
- Consumers (correct direction): document-model (delegates char nav, stores UTF-8,
  converts at boundary); find-and-replace (CaseFolder + word classification);
  edit-operations / navigation-commands (word/word-part classification);
  fileforge-integration (EBCDIC); background-io (detection in load pipeline).
- LineEndMode overlap with document-model: Req 5.7 treats U+2028/2029/0085 as valid
  UTF-8 that MAY be line endings "depending on LineEndMode"; document-model owns
  LineEndMode, ff-encoding only validates bytes. No duplicate ownership (confirmed
  W0.9). No conflict.
- **PA-WATCH-005 (carried)**: Req 7.7 / 10.8 require Unicode-data tables generated
  at build time. Verify a generation script / build.rs exists + records the Unicode
  version. Low priority.

## 4. Completeness

- Tasks: 111 `[x]`, 0 `[ ]`. Fully tracked complete.
- TCR: `ff-encoding` row PASS (detection, BOM, transcoding).
- Req 3.4 conversion-issues log is a returned data structure
  (`ConversionResult.issues: Vec<ConversionIssue>`) -- not a logging obligation.
- No std::fs/tokio::fs (byte-slice service; correct).
- Completeness verdict: **COMPLETE.**

## 5. Logging audit

- Zero log call sites; `ff-logging` is NOT a dependency. CORRECT for THIS crate:
  Req 1.7 explicitly requires the detector to be "stateless and side-effect-free"
  (logging would be a side effect); conversion problems surface via
  `ConversionResult.issues` (Req 3.4) and save-encoding errors via `Result` (Req
  4.4/4.5) to the caller. No requirement mandates a log record. STRONGEST
  defensible zero-log case in Wave 0 (spec-mandated statelessness).
- No println!/eprintln!.
- Logging verdict: **adequate (correctly zero-log for a stateless pure service)**.

## 6. Findings logged

- **PA-SPLIT-003** (SPLIT PROPOSAL, borderline): 3/5 criteria (14 reqs, 3 bands).
  Split SPEC into `encoding-and-characters` (Req 1-5, 11, 14) + new
  `character-classification` (Req 6, 7, 12, 13). Crate split NOT recommended.
  Owner approval + own gate.
- **PA-STD-006** (REFACTOR -- 400-line cap): `convert.rs` is 536 non-test lines,
  over the cap. Split by direction (convert_load / convert_save + convert_tables
  helper). REFACTOR, no gate. (category_map 357 / registry 337 near but under.)
- **PA-WATCH-005** (CONSISTENCY WATCH, low): Req 7.7 / 10.8 Unicode-data tables
  build-time generated -- verify a generation script / build.rs exists + records
  the Unicode version so tables do not drift. Low priority.
- **PA-LOG-001 caveat** (project-wide non-ASCII in .rs): ff-encoding contributes
  many matches, MOSTLY legitimate/necessary -- Unicode data-table literals,
  DBCS/case-fold examples, character-classification test fixtures (this crate is
  inherently about non-ASCII characters). The project-wide cleanup MUST exclude
  genuine Unicode data/test literals here and target only prose em-dashes and
  box-drawing separators. Rolled into PA-LOG-001 with this strong caveat.
