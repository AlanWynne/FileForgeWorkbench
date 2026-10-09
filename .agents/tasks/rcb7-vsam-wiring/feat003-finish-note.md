# RC.B.7 FEAT-003 -- Retire ff-vsam-services (completion note)

CR-CH-059 Mainframe Dataset Stack Rationalisation, RC.B.7, final slice FEAT-003.
Edits made DIRECTLY on `main`, uncommitted, for the owner's full gate.

## Grep pre-check result (PASS)

Searched the whole workspace for `ff-vsam-services` and `ff_vsam_services` in
*.toml and *.rs BEFORE any change:

- NO shipping crate had `ff-vsam-services` as a Cargo dependency
  (`ff-vsam-services = ...` -> zero matches).
- References were ONLY: the root `Cargo.toml` `[workspace].members` line; the
  crate's own files; governance RULES/metadata and two historical comments in
  ff-governance-tests; and stale doc comments. The concrete VSAM authority
  already lives in ff-dscatalog (vsam_service.rs / backends), so FEAT-001 had
  fully supplanted ff-vsam-services. No real code dependency -> safe to retire;
  no need to STOP.

## Exactly what was removed

1. Deleted the crate directory `crates/ff-vsam-services` (entire tree).
2. Removed its `[workspace].members` entry `"crates/ff-vsam-services",` from the
   root `Cargo.toml`. (Cargo.lock regenerates on the next scoped `cargo check`.)

## Governance rules / tests adjusted

- `crates/ff-governance-tests/src/compliance.rs`: removed the THREE
  ff-vsam-services DependencyRules -- the `ff-vfs -> ff-vsam-services` prohibition
  and the two `ff-vsam-services -> ff-idcams` / `-> ff-dsalloc` prohibitions. The
  VSAM-authority / acyclic-DAG intent remains covered by the existing
  ff-dataset-catalog rules (catalog is the lower-level service); no new rules
  invented (no test gap appeared).
- `crates/ff-governance-tests/tests/architecture_compliance.rs`:
  - removed the `vfs_has_no_domain_dependencies` assertion line for
    ff-vsam-services;
  - removed the whole `vsam_services_has_no_upstream_dependencies` test (it
    targeted the now-removed crate);
  - removed the `"ff-vsam-services"` entry from the
    `prohibited_rules_cover_all_dataset_crates` crate list (it would otherwise
    assert a rule exists for a crate that has none);
  - removed the `("ff-vsam-services", true)` entry from the
    `all_governed_crates_exist` required-crates list (the crate no longer exists).
- `crates/ff-governance-tests/tests/mock_compilation.rs` + its `Cargo.toml`:
  RC.A.2 had already repointed these to ff-dscatalog. No residual
  `ff_vsam_services` IMPORT existed -- only a historical comment, which I reworded
  so no live `ff-vsam-services`/`ff_vsam_services` token remains in any build/code
  file.

## Stale doc comments fixed (comment-only; no trait CODE changed)

- `crates/ff-dataset-catalog/src/lib.rs`: "VSAM record storage ... owned by
  ff-vsam-services" -> "owned by ff-dscatalog".
- `crates/ff-dataset-catalog/src/traits.rs`: "VSAM (sub-types handled by
  ff-vsam-services)" -> "handled by ff-dscatalog".
- `crates/ff-idcams/src/services.rs`: the `VsamService` trait doc "implemented by
  ff-vsam-services" -> "implemented by ff-dscatalog" (also converted an em dash to
  ASCII `--`). The `VsamService` trait CODE itself was NOT touched -- that repoint
  is RC.B.8.

## Scoped command results (via pwsh7 non-interactive wrapper, logs read back)

- `cargo fmt` -- clean (empty log).
- `cargo check -p ff-governance-tests -p ff-dscatalog -p ff-dsalloc -p ff-idcams --all-targets`
  -- Finished, no errors.
- `cargo test -p ff-governance-tests` -- **GREEN (THE KEY CHECK)**:
  architecture_compliance 9 passed / 0 failed; mock_compilation 7 passed / 0
  failed; unit + doc tests 0. Re-ran after the comment edits -- still 9+7 green.
- `cargo check -p ff-desktop --all-targets` -- Finished, no errors (the app
  closure still builds without the removed crate).
- Final grep: NO live `ff-vsam-services` / `ff_vsam_services` references remain in
  any *.toml or *.rs.

## TCR rows flipped (docs/quality/TCR.md)

- dataset-catalog Req 35.3: 🔴 -> ✅ (VSAM wired under DatasetAccess before
  ff-vsam-services removed; cites point_on_ksds_round_trips_keyed_record /
  point_relative_rrds_round_trips_and_rejects_rrn_zero + governance green).
- dataset-ownership-model Req 22.3: ✅ text updated from "deprecated-for-merge /
  crate retirement is RC.B" to RETIRED (RC.B.7 FEAT-003).
- Req 34 (VSAM keyed/relative point ops): 🔴 (was "DEFERRED to RC.B.7, returns
  NotYetWired", citing now-deleted tests) -> ✅ citing the real FEAT-001 tests.
- Req 19.3 (dataset-allocator): 🔴 PARTIAL -> ✅ (Allocated/Verified/Passed all
  carry a handle; Verified/Passed resolve-by-DSN) citing FEAT-002 tests.

## Tasks marked

- dataset-catalog Task 41 [x] (41.1 + 41.2).
- dataset-allocator Task 21 [x] and Task 21.3 [x] (NOTE/sub-text updated to DONE
  RC.B.7).
- docs/project-management/project-master/tasks.md RC.B.7 row [x],
  code-complete-pending-gate note added.
- docs/status/RESUME-dataset-rationalisation.md updated: RC.B.7
  done-pending-gate; FEAT-001/002 completed by a prior wedged-then-recovered run,
  FEAT-003 completed here.

## Residual ff-vsam-services references intentionally left

NONE in build/code (*.toml / *.rs) -- all live references are gone. Any remaining
mentions are in docs/specs prose and status/history narrative (e.g. this note,
RESUME doc history, change-log/report prose), which are acceptable historical
record and were left untouched.

## Scope boundary (RC.B.8 NOT entered)

Did NOT repoint the ff-idcams private CatalogService/VsamService trait CODE, did
NOT change the editor SAVE path, did NOT delete ff-dataset-catalog. The
ff-vsam-services removal did NOT force any ff-idcams code change (it depended only
on its own trait, never on the concrete crate), so no STOP was needed.

## Status

RC.B.7 is **CODE-COMPLETE PENDING** the owner's full `cargo gate --build`. The
full gate was NOT run here (owner's manual step). RC.B.8 was NOT started.
