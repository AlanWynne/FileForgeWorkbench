# Analysis Record: bootstrap-scripts (W5.20 / row 78)

- **Wave**: 5 (emulators, tools, connectors)
- **Backing artifacts**: platform bootstrap SCRIPTS (not a crate) --
  `bootstrap/bootstrap-windows.ps1` (179 lines), `bootstrap-linux.sh` (116),
  `bootstrap-macos.sh` (119), `bootstrap/README.md` (89)
- **Spec files**: requirements.md (144 lines, 5 requirements), tasks.md
  (37 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 5 pass

---

## 1. Split candidacy

N/A -- shell/PowerShell scripts, not a Rust crate. The 400-line cap + crate-split criteria
do not apply. The three scripts are each ~120-180 lines (fine). No split.

---

## 2. Cross-unit consistency -- CLEAN + LOW-RISK (no findings)

Purpose: let a new contributor download + set up FileForge Workbench (Rust toolchain) on
Windows/Linux/macOS. 5 reqs cleanly map to the deliverables: Req 1-3 = the three platform
scripts, Req 4 = README, Req 5 = cross-cutting constraints.

- **Safe download practice**: the scripts download the Rust toolchain via the OFFICIAL
  rustup HTTPS endpoint (`https://sh.rustup.rs`; Windows uses Invoke-WebRequest with a
  WebClient fallback), installing into a USER-LEVEL location (RUSTUP_HOME). This is the
  standard, trusted rustup bootstrap method -- NOT an obfuscated download-and-exec, no
  credential handling, no piping remote content blindly to a shell. No supply-chain
  finding (rustup's sh.rustup.rs is a redirector without a stable published checksum;
  HTTPS to the official host is the accepted approach).
- **Post-install verification present**: each script has a "Verify" section confirming
  rustc/cargo work after install (Req 5 cross-cutting).
- **Logging discipline**: the scripts log to `bootstrap/logs/` (Write-Log etc.; 80
  log-related lines) -- CONSISTENT with the tooling steering's stdout-capture mandate.
  Positive.
- **ASCII-clean**: 0 non-ASCII bytes in the scripts.

No cross-unit conflict, no duplication (the only toolchain bootstrap), no orphan concept
(scripts). Complements compiler-toolchain-integration (W5.5, the IN-APP toolchain
detection/install) -- bootstrap is the PRE-app contributor setup; toolchain-integration is
the runtime detection. Distinct, complementary roles (like the local-emulation vs remote
boundary W5.18). No overlap.

---

## 3. Completeness

Tracking: all 37 sub-tasks `[x]`. Three platform scripts + README present + verified.
Genuinely complete. No PA-INCOMPLETE.

### TCR (28 rows) -- good

TCR.md has 28 rows for bootstrap across 5 reqs -- good coverage (scripts validated against
the acceptance criteria). Positive; no PA-TCR gap.

---

## 4. Logging audit

N/A (not a Rust crate / no ff-logging). The scripts DO log to bootstrap/logs/ per the
tooling steering (Write-Log, tee-style capture) -- appropriate + present.

---

## 5. Task revision proposals

NONE of substance. This is a CLEAN, complete, low-risk unit. Optional future hardening
(NOT a finding, contributor convenience only): the scripts could pin/verify a rustup
version, but downloading the official rustup over HTTPS is standard + acceptable.

No PA-CONFLICT / PA-STD / PA-LOG / PA-TCR / PA-SPLIT / PA-INCOMPLETE. No code change
proposed.

---

## Summary

bootstrap-scripts is a CLEAN, complete, low-risk unit (three platform bootstrap scripts +
README; 5 reqs, 37/37, 28 TCR rows) -- one of the cleanest units in the analysis. The
scripts download the Rust toolchain via the OFFICIAL rustup HTTPS endpoint into a
user-level location (safe, standard practice -- not download-and-exec), verify the install
afterward, log to bootstrap/logs/ (consistent with the tooling steering), and are
ASCII-clean. They complement compiler-toolchain-integration (pre-app contributor setup vs
in-app toolchain detection -- distinct roles, no overlap). No findings: no conflict, no
duplication, no orphan, no incompleteness, no cap issue (shell scripts), no ASCII/logging
gap. A genuine positive to close the Wave-5 unit analysis.
