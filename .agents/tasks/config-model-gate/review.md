# Design review (round 2) -- CR-CH-116 write-through config model + ISPF END/RETURN/EXIT gate

Reviewer: design-review subagent (REVISION pass -- re-checking the round-1 findings
against the revised gate, then reviewing what changed).
Target: the requirements + design gate for the owner-confirmed write-through config
model and the ISPF END/RETURN/EXIT command model (CR-CH-116).
Verdict: **APPROVED** (0 HIGH, 0 MEDIUM; 2 NITs carried forward, non-blocking).

## Where the work lives (unchanged from round 1)

The review prompt's absolute paths resolve to the MAIN repo root. As in round 1,
the only CR-CH-116 artefact at the main root is this gate's `.agents` scratch;
`git -C <root> status` shows none of the 16 doc edits (only an unrelated
`docs/specs/theme-and-appearance/requirements.md` modification + the usual
untracked `.agents/` scratch). The real deliverables live in the sibling worktree
`c:/workspace/apps/FileForgeWorkbench/.worktrees/config-model-gate/`, whose
`git status --porcelain` lists exactly the 16 files the summary claims, ALL `.md`:

```
 M .kiro/steering/framework-conformance.md
 M docs/specs/command-configurator/{design,requirements,tasks}.md
 M docs/specs/configuration-system/{design,requirements,tasks}.md
 M docs/specs/menu-workspace/{design,requirements,tasks}.md
 M docs/specs/navigation-commands/{requirements,tasks}.md
 M docs/specs/screen-snapshot-scrm/requirements.md
 M docs/specs/startup-and-session/{requirements,tasks}.md
 M docs/status/change-log.md
```

This review was conducted against the worktree copies. The main-root vs worktree
divergence remains (NIT 2 below) but is not a blocker.

## Docs-only check (PASS)

The worktree `git status --porcelain` lists only `.md` edits. No `.rs` file is
modified. The explicit "flag any .rs change as blocking" gate is satisfied.

---

## Round-1 findings -- re-check (all three scored findings CLOSED)

### Finding 1 (was HIGH) -- startup-and-session 11.9 contradicted new Req 23 -- CLOSED
`docs/specs/startup-and-session/requirements.md` criterion 11.9 (now line 291) is
rewritten to the write-through model, matching the fix I proposed:
> 9. (REVISED by CR-CH-116 -- supersedes the CR-CH-021 "compiled-only, never
> materialised" clause.) THE built-in menus SHALL be WRITTEN to disk on first
> launch (Requirement 23.1-23.2; menu-workspace Requirement 4.1); the compiled
> Recovery_Source is consumed only to perform that first-run write, to rebuild on
> Reset bare, and as a last-ditch fallback ... A fresh install still opens to a
> fully usable Home Context because the first-run write produces the menu files
> before the POM loads.
The companion criterion 11.8 (now criterion 8, line ~285) has its CR-CH-021 tag
qualified to the write-through Recovery_Source model. No "SHALL NOT be
materialised" clause survives in the file. The same-document contradiction is
gone.

### Finding 2 (was HIGH) -- screen-snapshot-scrm 25.5 contradicted revised 22.8 -- CLOSED
`docs/specs/screen-snapshot-scrm/requirements.md` criterion 25.5 (line 987) is
rewritten and now AGREES with 22.8:
> 25.5 (REVISED by CR-CH-116 -- supersedes the CR-CH-021 "never materialised"
> clause.) THE SCRM menu screen SHALL be WRITTEN to `menus/scrm.toml` on first run
> and REBUILT by RESET BARE from the Recovery_Source `DEFAULT_SCRM_MENU_TOML` (via
> the shared Config_Rebuild_Helper, Req 22.8); the application runs from the
> on-disk file. A user `menus/scrm.toml` remains authoritative ...; the compiled
> default is the Recovery_Source only.
22.8 itself (line 763) carries the matching "write-through model supersedes the
CR-CH-021 clause" wording. The residual `CR-CH-021` mentions in the file (lines
700, 726, 965) are historical/background references, not live "never materialise"
directives. The contradiction is gone.

### Finding 3 (was MEDIUM) -- five wrong "command-configurator Requirement 6" cites -- CLOSED
`Select-String` over `docs/specs/*/requirements.md` returns ZERO occurrences of
"command-configurator Requirement 6". The five previously-wrong citations now read
"command-configurator Requirement 8":
- configuration-system/requirements.md line 513 (Reset-bare criterion 17) and
  line 929 (Req 23.3).
- menu-workspace/requirements.md line 197 (criterion 2.1g), line 638 (POM note),
  line 971 (criterion 14.13).
Confirmed the targets are correct: command-configurator Requirement 8 (line 545)
is "FFCMD seed alias `X -> EXIT`, written on first run and rebuilt by Reset bare
(CR-CH-116)"; Requirement 6 (line 299) is the unrelated pre-existing "Alias Editor
edits the per-locale localisation aliases (CR-CH-072)". navigation-commands Req
21.4 (line 684) and startup-and-session (lines 664, 675) also cite Requirement 8
correctly. The mis-direction is gone.

### Finding 4 (was NIT) -- pre-existing non-ASCII -- UNCHANGED (still NIT)
The ASCII scan still flags non-ASCII in touched files (e.g. startup-and-session
lines 27/66/69/97/248: plugin-lifecycle arrows, layered-merge arrows, an en-dash
range). All are PRE-EXISTING lines not authored by CR-CH-116; the round-2 fix
lines (startup 291, scrm 987, and the citation edits) are ASCII-clean. Not
introduced by this change; not blocking.

### Finding 5 (was NIT) -- main-root vs worktree divergence -- UNCHANGED (still NIT)
Still present: deliverables are in the worktree, main root is not in sync. The
gate-summary section 9 acknowledges this and explains the intentional split
(worktree carries the merging docs; `.agents` is shared session scratch). Not
blocking because the complete deliverables exist in the worktree.

## New-issue scan on the revised content (no new HIGH/MEDIUM)

The round-1 fixes touched three criteria plus five citation lines. I re-read each
in context:
- startup 11.9 / criterion 8 now form a consistent pair (first-run write + runtime
  fallback) with no residual "never materialise" clause.
- scrm 25.5 now agrees with 22.8 and preserves user-override precedence.
- The five Requirement-8 citations read coherently in their surrounding criteria
  (configuration-system Req 19.17 line 513 and Req 23.3 line 929 both reference the
  single `X -> EXIT` seed correctly; the menu-workspace citations likewise).
No new contradiction, dangling reference, or scope change was introduced by the
fixes.

---

## Verified assumptions (re-confirmed this pass)

- **Rust recovery defaults kept as recovery source only.** CONFIRMED.
  configuration-system Req 23.1 + Recovery_Source glossary; framework mechanism 4
  rewritten; startup criterion 8/9 now consistent.
- **First run writes complete TOML; Reset bare rebuilds.** CONFIRMED. Req 23.2
  (per-file first-run write), 23.4 (Complete_Config_File), Req 19.16-19.18 (archive
  `aliases/` + rebuild pom/settings/scrm + FFCMD seed).
- **First-run write and Reset-bare rebuild SHARE ONE helper.** CONFIRMED. Req 23.6;
  configuration-system Req 19.17 and command-configurator Req 8.4 both route through
  the shared Config_Rebuild_Helper.
- **END, RETURN, EXIT distinct; PF3 stays END.** CONFIRMED. navigation-commands
  Req 21.1-21.3; framework mechanism 3 note; PF3 not remapped to EXIT.
- **X = EXIT everywhere WITHOUT shadowing FFEDIT X -> EXCLUDE (active-first).**
  CONFIRMED. command-configurator Req 8.5 ("COEXIST with the FFEDIT base alias"),
  Req 8.7 (existing overlay path), navigation-commands Req 21.4/21.5.
- **nav_x retirement orphans no caller.** CONFIRMED as specified (reference search
  appropriately deferred to implementation; `close_workspace_or_exit` retained).
- **Test isolation preserved; inverted ensure_menus_dir tests accounted for.**
  CONFIRMED. Req 23.5 + command-configurator Req 8.3 (all writes through the dir
  seams); task 47.6 inverts the `ensure_menus_dir_does_not_materialise_*` tests.
- **Deferred UPGRADE not implemented (version-stamp room + follow-up only).**
  CONFIRMED. Req 23.9 (optional defaulted `config_version`, inert) and 23.10
  (UPGRADE/merge NOT implemented).
- **Complete-flags rule incl. show_calendar.** CONFIRMED. Req 23.4; Settings
  `show_calendar = false` and POM's flag emitted explicitly.
- **CR id unique; mechanism 4 properly superseded.** CONFIRMED. CR-CH-116 (distinct
  prefix from the parked alias-kind-gate's CR-NR-116; below the current high).
  framework mechanism 4 rewritten to the write-through rule with CR-CH-021
  explicitly RETIRED and the themes code-only rule explicitly NOT affected.
- **Docs-only, no .rs.** CONFIRMED (worktree git status).

## Unverified / wrong assumptions

- **UNVERIFIED (code-level, appropriately deferred).** That `nav_x`'s only caller is
  the `commands_ladder_a.rs` X arm and that `close_workspace_or_exit` stays
  referenced by END/RETURN is asserted from the investigation, not re-checked
  against source (this is a docs gate; the specs correctly defer the reference
  search to implementation). Not scored.
- **NIT (worktree/main divergence).** The prompt paths resolve to the main root,
  where the edits are absent; the deliverables are in the worktree. Documented,
  intentional, non-blocking.

## Verdict

0 HIGH + 0 MEDIUM => **APPROVED**. All three round-1 scored findings (startup 11.9,
scrm 25.5, the five mis-numbered command-configurator citations) are resolved and
internally consistent, and the fixes introduced no new HIGH/MEDIUM issues. The
write-through model, the one-helper design, the ISPF END/RETURN/EXIT model, X =
EXIT with FFEDIT EXCLUDE preserved via active-first resolution, test isolation,
deferred UPGRADE, complete-flags, CR-id uniqueness, and the mechanism-4
supersession are all correctly specified. The two remaining items (pre-existing
non-ASCII; worktree/main-root divergence) are NITs carried forward and do not
block.
