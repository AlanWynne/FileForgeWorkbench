# Analysis Record: encoding-and-characters

- **Wave**: 0 (foundation -- stateless character/encoding service layer)
- **Backing crate**: `ff-encoding`
- **Spec folder**: `docs/specs/encoding-and-characters/`
- **Analysed**: Wave 0, task W0.19
- **Verdict**: COMPLETE (111/111 tasks `[x]`, TCR PASS). BORDERLINE SPLIT
  CANDIDATE (14 reqs, 316 req lines, separable encoding-I/O vs
  character-classification bands). Zero-log is CORRECT (spec mandates
  statelessness). One 400-cap refactor (convert.rs 536).

---

## 1. Scope summary

`ff-encoding` is the GUI-independent character/encoding service. 14
requirements in three natural bands:

- **Encoding I/O boundary**: Req 1 detection, Req 2 BOM, Req 3 convert-on-load,
  Req 4 convert-on-save, Req 5 UTF-8 validate/repair, Req 11 encoding-family
  classification, Req 14 encoding state/metadata + registry.
- **Character classification**: Req 6 CharClassify (byte table), Req 7 Unicode
  category map (UAX #31), Req 12 word-part navigation (camelCase/snake_case),
  Req 13 configurable word-char sets.
- **Script-specific**: Req 8 DBCS (Shift-JIS/GBK/Big5/Korean), Req 9 grapheme
  clusters (UAX #29), Req 10 Unicode case folding (CaseFolding.txt).

316 req lines, 14 requirements, single backing crate.

## 2. Split candidacy -- BORDERLINE

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 316 lines (near) AND 14 reqs (>12) | YES |
| 3+ distinct responsibilities | encoding-I/O band; char-classification band; script-specific (DBCS/grapheme/case-fold) band | YES (3 bands) |
| 2+ crates | one crate `ff-encoding` | No |
| low-cohesion clusters | the three bands are separable, though all consumed together via EncodingFamily + document-model | Weak-Yes |
| file-size pressure | `convert.rs` 536 non-test over cap; category_map 357, registry 337 near | partial |

3 of 5 criteria (>12 reqs; 3 responsibilities; separable bands).
**BORDERLINE SPLIT CANDIDATE.** Recorded as PA-SPLIT-003 (proposal only).

### Proposed spec split (proposal, owner-gated)

- Keep `encoding-and-characters` = encoding I/O boundary (Req 1-5, 11, 14).
- New `character-classification` spec = Req 6, 7, 12, 13 (CharClassify, Unicode
  category, word-part, word-char sets) -- these serve edit/nav/find, a different
  consumer set than load/save.
- DBCS (Req 8), grapheme (Req 9), case-folding (Req 10) could stay with encoding
  (DBCS) or classification (grapheme/case-fold) respectively.

Crate split NOT recommended now: the bands share `EncodingFamily` and the
Unicode data tables, and document-model + find-and-replace + edit-operations
consume the crate as one unit. A SPEC split improves traceability for the large
14-req document without churning the crate. No action without owner approval +
own gate.

## 3. Consistency / conflict

- Public types owned here (EncodingFamily, CharClassify, CharacterCategoryMap,
  CaseFolder/ICaseConverter, ConversionResult/ConversionIssue, BOM detector,
  DBCS functions, grapheme boundary functions, encoding registry) -- sole owner
  `ff-encoding`. Added to consistency-matrix.
- Consumer relationships (all correct direction): document-model delegates
  character navigation and stores UTF-8 internally (converts at boundary);
  find-and-replace uses CaseFolder + word classification; edit-operations and
  navigation-commands use word/word-part classification; fileforge-integration
  uses EBCDIC; background-io may run detection in the load pipeline. Matches spec
  Cross-References. No conflict.
- **LineEndMode overlap with document-model (W0.9)**: Req 5.7 here treats
  U+2028/U+2029/U+0085 as valid UTF-8 that MAY be line endings "depending on the
  document's LineEndMode". document-model owns LineEndMode; ff-encoding only
  validates the byte sequences. Consistent -- no duplicate ownership (confirms
  the LineEndMode matrix row from W0.9).
- Unicode-data generation (Req 7.7 category map, Req 10.8 case tables) is
  build-time generated static data. Watch: verify a generation script/build.rs
  exists and is documented (not hand-maintained tables that could drift from the
  Unicode version). Recorded PA-WATCH-005 (low priority).

## 4. Completeness

- Tasks: 111 `[x]`, 0 `[ ]`. Fully tracked complete.
- TCR: `ff-encoding` row PASS (encoding detection, BOM handling, transcoding).
- Req 3.4 conversion-issues log VERIFIED as a returned data structure
  (`ConversionResult.issues: Vec<ConversionIssue>` in convert.rs) -- lossy
  replacements reported to the caller, not a logging obligation. Correct.
- No std::fs/tokio::fs (operates on byte slices; correct for a stateless service).
- Completeness verdict: **COMPLETE.**

## 5. Logging audit

- Zero log call sites; `ff-logging` is NOT a dependency. For THIS crate that is
  CORRECT, not a gap:
  - Req 1.7 explicitly requires the encoding detector to be "stateless and
    side-effect-free" -- logging would be a side effect.
  - Conversion problems surface via `ConversionResult.issues` (Req 3.4) and
    save-encoding errors via `Result` (Req 4.4/4.5) returned to the caller
    (document-model / file-operations), which is the correct place to log.
  - No requirement mandates a log record from this crate (contrast ff-vfs/
    ff-plugin which have explicit WARN/ERROR criteria).
- This is the STRONGEST defensible zero-log case in Wave 0 (spec-mandated
  statelessness). Distinct from PA-LOG-003 (document-model, defensible) and
  PA-LOG-004 (ff-vfs, a real defect).
- No println!/eprintln!. GUI-independence upheld.
- Logging verdict: **adequate (correctly zero-log for a stateless pure service)**.

## 6. Findings logged

- **PA-SPLIT-003** (SPLIT PROPOSAL, borderline): 3/5 split criteria met (14 reqs,
  3 separable bands). Proposal: split the SPEC into `encoding-and-characters`
  (I/O boundary: Req 1-5, 11, 14) + new `character-classification` (Req 6, 7, 12,
  13); DBCS/grapheme/case-fold assigned per band. Crate split NOT recommended.
  Owner approval + own gate required.
- **PA-STD-004** (REFACTOR -- 400-line cap): `convert.rs` is 536 non-test lines,
  over the cap. Split by direction (`convert_load.rs` decode-to-UTF-8 /
  `convert_save.rs` encode-from-UTF-8, sharing a `convert_tables` helper).
  REFACTOR, no gate. (category_map.rs 357 / registry.rs 337 are near but under.)
- **PA-WATCH-005** (low-priority watch): Req 7.7 / 10.8 require Unicode-data
  tables (category map, case folding) generated at build time from the Unicode
  Character Database. Verify a generation script / build.rs exists and records
  the Unicode version, so tables do not silently drift. Low priority.
- **PA-LOG-001 caveat** (project-wide non-ASCII in .rs): ff-encoding contributes
  83 matches, but MANY are legitimate/necessary -- Unicode data-table literals,
  DBCS/case-fold examples, and character-classification test fixtures (this crate
  is inherently about non-ASCII characters). Like document-model, the project-wide
  cleanup MUST exclude genuine Unicode data/test literals here and target only
  prose em-dashes and box-drawing separators. Rolled into PA-LOG-001 with this
  strong caveat.
