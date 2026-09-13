# Analysis Record: asa-report-preview (W5.7)

- **Wave**: 5 (emulators, tools, connectors)
- **Backing crate**: `ff-asa` (ASA/ANSI carriage-control interpretation + print-preview
  rendering: page breaks, overstrike bold/underline, green-bar/blue-bar simulation,
  line-printer emulation, PDF/text export)
- **Spec files**: requirements.md (409 lines, 12 requirements), tasks.md
  (160 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 5 pass

---

## 1. Split candidacy

NOT a split candidate. 12 reqs, 409 lines. One cohesive concern (ASA preview rendering).
Well-decomposed into 15 files (config, control, detection, export_text, merge,
navigation, page_index, panel, preview, printer, shading, strip, types, ...); NO file
over 350. No split.

---

## 2. Cross-unit consistency -- CLEAN (wired + clean-seam panel)

### WIRED (NOT an orphan)

`ff-asa` is referenced 173x outside the crate (ff_asa / PREVIEW / AsaPreview) -- the
PREVIEW command (Req 3) + the preview panel (Req 6) are integrated into the shell. NOT
an orphan (contrast idcams W5.2 / the Wave-4 orphans).

### Clean-seam panel (data model, shell renders) -- GUI-independent

Despite Req 6 (Print Preview Panel), `ff-asa` has NO egui dependency (sole dep
thiserror; `use egui` = 0). `panel.rs` / `preview.rs` are a pure page-layout + band-
shading DATA MODEL that the SHELL renders with egui -- the same clean-seam / injection
pattern as the clean crates elsewhere (ff-asa produces the paginated/shaded preview
model; the caller draws it). Consistent with GUI-independence; keeps ff-asa testable
without a UI. Positive.

### Line-printer emulation

`printer.rs` models IBM 1403 / 3800 / 4245 + custom profiles (132 columns x 60/66
lines) -- the page-dimension source of truth (Req 8). ASA strip/restore on edit (Req 7)
coordinates with the document model (produces stripped text; the editor owns the buffer).
Overstrike merging (Req 5) + shading (Req 9) are ASA-specific, sole-owned. Export
(Req 11) to PDF/text. No duplication with other units.

### Public types and ownership

- ASA control interpretation, page index, preview/panel model, printer profiles,
  overstrike merge, band shading, strip/restore -- sole-owned by `ff-asa`. No overlap.

### Naming drift (PA-DOC-007)

Spec calls it `ff-asa-report-preview`; the crate dir is `ff-asa`. Same crate-name-drift
pattern as the others (ff-hex/ff-select/.../ff-external-mod). Recorded PA-DOC-007 (add to
the naming-reconciliation set). Docs only.

### Cross-reference integrity

Cross-refs (command-framework PREVIEW, document-model strip/restore, layout panel)
resolve as sub-projects and are wired.

---

## 3. Completeness

Tracking: all 160 sub-tasks `[x]`. Implementation present across all 12 reqs (control
chars, auto-detect, PREVIEW, page breaks, overstrike merge, panel, strip/restore,
printer emulation, shading, navigation, PDF/text export, config) and WIRED (173 refs).
Genuinely complete (contrast database-tool W5.4 skeleton). No PA-INCOMPLETE.

### TCR gap (PA-TCR-030)

TCR.md has 1 row for ff-asa across 12 reqs / ~90 criteria -- thin, despite 160 tasks +
in-file tests. Recorded PA-TCR-030.

---

## 4. Logging audit

Scan of `crates/ff-asa/src` (recursive):

- `ff_logging` / `log_*!`: 0; `ff-logging` dep: ABSENT (not dead).
- `println!` / `eprintln!`: 0; `std::fs`: 0.

ASA detection / parse / render is a low-stakes, deterministic, pure-transform pipeline
(input text -> preview model); errors surface via AsaError (e.g. page-not-found on
navigate). Zero-log is largely defensible here (no I/O, no concurrency, no external
process). Optional LOW dev-logging on auto-detect decisions + export outcomes only.
Recorded PA-LOG-045 (LOW): optional dev-logging on ASA auto-detect + export; low
priority (unlike the emulator/tool logging gaps, this is a pure transform).

---

## 5. Task revision proposals

- **PA-STD-061 (ASCII, runtime strings)**: 8 non-comment non-ASCII -- em-dashes (U+2014)
  and MULTIPLICATION SIGNS (U+00D7) in runtime display strings: error.rs:11 (navigate
  message), printer.rs profile descriptions ("132 columns U+00D7 60 lines") +
  dimensions_annotation (format string using U+00D7). Replace the multiplication sign
  with ASCII `x` and the em-dash with `--`. REFACTOR, no gate. (Note: these are DISPLAY
  strings, not legitimate control characters.)
- **PA-DOC-007 (naming)**: reconcile spec `ff-asa-report-preview` to crate dir `ff-asa`
  (naming-reconciliation set). Docs only.
- **PA-TCR-030**: enumerate per-requirement TCR rows (1 for 12 reqs). No code.
- **PA-LOG-045 (LOW)**: optional ASA auto-detect + export dev-logging.

No PA-STD size item (no file over 350). No PA-INCOMPLETE (complete + wired). No orphan.
No raw-fs.

---

## Summary

asa-report-preview (`ff-asa`) is a cohesive, complete, WIRED ASA carriage-control preview
subsystem (control chars, auto-detect, PREVIEW command, page breaks, overstrike merge,
paginated panel, strip/restore, IBM 1403/3800/4245 printer emulation, green-bar shading,
PDF/text export; 160/160, 15 files, no cap issue). It is well-integrated (173 external
refs -- PREVIEW command + panel in the shell, NOT an orphan) and follows the clean-seam
pattern: despite Req 6's preview panel it has NO egui dep -- `panel.rs` is a pure page-
layout + shading DATA MODEL the shell renders, keeping ff-asa GUI-independent + testable.
Findings are minor: PA-STD-061 (8 runtime-string violations -- multiplication signs
(U+00D7) and em-dashes in printer-profile / navigate display text, should be `x` / `--`), PA-DOC-007
(name drift ff-asa-report-preview vs ff-asa), PA-TCR-030 (thin, 1/12), and PA-LOG-045
(LOW -- optional dev-logging on a pure transform pipeline; zero-log is largely defensible
here). A clean, genuinely-complete unit -- another Wave-5 positive after toolchain + batch.
