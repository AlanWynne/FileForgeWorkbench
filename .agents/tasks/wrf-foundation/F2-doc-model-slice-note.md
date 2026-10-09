# CR-CH-058 F2 -- document-model slice (Window_Band + Total_Records authority)

## Scope delivered

The document-model-LOCAL half of F2 ONLY. This is the `WindowBand` primitive in
`ff-document-model` plus the `Total_Records`-as-authority distinction. It
deliberately does NOT touch the shell/viewport/scrollbar/zoom wiring, because the
other chat is actively editing `ff-desktop/shell/render_*.rs` (CR-CH-059) and
end-to-end F2 would collide there. The shell slice waits until F1 merges to main
and that render work settles.

## Files

- `crates/ff-document-model/src/window_band.rs` (new): `RecordRange { start, end }`
  (`[start, end)` record-number range) + `WindowBand { total_records, page_size,
  current_page, band }`.
  - `new(total_records, page_size)` -- positions at page 0, computes the band.
  - `total_records()` / `resident_range()` / `resident_len()` -- the Req 12.5
    distinction: `total_records()` is the whole-file authority (scrollbar sizes on
    THIS), `resident_len()` is the <=3-page band span. They are NEVER fused.
  - `compute_band(page)` -- 3-page band (prev + current + next), clamped to
    `[0, total_records)`; edge pages yield a 2-page band, no over-read (Req 12.4).
  - `scroll_to_record(top)` -> bool -- hysteresis: returns false (no I/O) when the
    target page is already fully resident, true + shifts otherwise (Req 12.7).
  - `jump_to_record(target)` -> RecordRange -- single-shift index-resolved jump;
    recomputes the band arithmetically around the target page, touches no
    intermediate records (Req 12.8).
  - `evictable(previous, pinned)` -> Vec<RecordNumber> -- (previously resident)
    MINUS (new band) MINUS (pinned); dirty pinned records are never evicted
    (Req 12.7).
  - `set_total_records(total)` -- refine estimate->exact or post-edit count;
    re-clamps the current page and recomputes the band (Req 12.5).
- `crates/ff-document-model/src/window_band_tests.rs` (new): 15 unit tests, each
  `// Validates: Requirement 12.x`.
- `crates/ff-document-model/src/lib.rs`: `pub mod window_band;` +
  `pub use window_band::{RecordRange, WindowBand};`.

## Criteria status (document-model rows)

- Req 12.4 (3-page band, residency decoupled) -- PASS
- Req 12.5 (Total_Records authority, never the resident window -- THE scrollbar
  -collapse bug fix) -- PASS
- Req 12.6 (zoom never loads) -- still 🔴, F2 SHELL slice (render/zoom wiring),
  no document-model behaviour to assert; deferred.
- Req 12.7 (hysteresis load/evict + dirty pinned) -- PASS (overscan tuning is an
  F5 budget concern)
- Req 12.8 (down/up N index-resolved jump, no intermediate reads) -- PASS

tasks.md: 20.5 -> [x] (document-model slice done); 20.6 -> still [ ] (Total_Records
half done, zoom-never-loads shell half outstanding).

## Verification (SCOPED, Kiro-run)

- `cargo test --manifest-path <worktree>/crates/ff-document-model/Cargo.toml --lib`
  = **201 passed, 0 failed** (186 F1 + 15 new WindowBand).
- `cargo clippy --manifest-path <...> --all-targets -- -D warnings` = clean.
- `cargo fmt` = clean.

Run against the `wrf-foundation` worktree manifest explicitly, because the shell
wrapper's `cd <main>` prefix otherwise resolves the package to the main checkout
(which does not contain the F1/F2 modules). Logs in `tools/logs/f2-*.txt`.

## Hand-off

Scoped checks clean -> owner runs the full `cargo gate --build` manually outside
Kiro. The F2 SHELL slice (egui scrollbar sized on Total_Records + zoom-render-only
over resident records, Req 12.6 + the actual scrollbar/viewport consumer of
`WindowBand`) is intentionally NOT in this commit; it is sequenced after F1 merges
to main and the other chat's CR-CH-059 render work settles.

## MERGE-HOLD LIFTED (coordination note, 2026-10-09)

The two conditions this slice (and the F1 merge) were waiting on are BOTH now met:

1. The owner's in-progress dataset/volume work is COMMITTED + PUSHED to `main`:
   CR-CH-059 RC.A (`216c3cd`) through RC.B.8 (`b9921f3`) are all on `origin/main`,
   each full-gate-clean. (RC.B.5/B.6 landed within RC.A's commit; RC.B.7 `02ce339`
   and RC.B.8 `b9921f3` followed.)
2. The CR-CH-059 shell render work (RC.B.7/B.8) that this note said F2's shell
   slice must wait for has therefore SETTLED on `main`.

STATE of `.worktrees/wrf-foundation` (branch `feature/windowed-record-foundation`,
HEAD `74a0414`): clean tree, 9 ahead / 2 behind `main`; the 2 "behind" commits are
exactly RC.B.7 + RC.B.8. Merge-base is RC.A (`216c3cd`) -- the branch was already
built on RC.A and recorded its intended "Option-A rebase onto CR-CH-059" in commit
`7f3f5e0`.

RECOMMENDED NEXT (for the CR-CH-058 session that OWNS this branch -- NOT to be done
by the dataset/orchestrator session, to avoid rewriting this branch's history out
from under an active session): rebase `feature/windowed-record-foundation` onto the
current `main` tip (`b9921f3`), resolve any conflicts on the shared SAVE path
(`tab_manager.rs::save_active_tab_via_backend`, `shell/dispatch_ffedit.rs`) and
`ff-dscatalog`, run the scoped document-model checks + the owner's full
`cargo gate --build`, then fast-forward F1 (and the F2 doc slice) to `main`.

COORDINATION with the record-aware SAVE work: a shared framework-change gate for
the record-aware `ff-vfs::BackendEnvironment` store contract (serving BOTH CR-CH-058
and CR-CH-059 RC.B.8 Part 2) has been launched (see
`.agents/tasks/backendenv-record-contract-gate/gate-summary.md` once written, and
`.agents/tasks/crch058-rcb8-overlap/report.md` section E/F). The CR-CH-058 SAVE walk
should consume THAT contract rather than reshaping the SAVE seam independently.
