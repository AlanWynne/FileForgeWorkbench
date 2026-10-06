# Implementation Tasks -- Command Environments (CR-CH-053)

Phase-1 scope: the environment framework + FFCMD (existing `resolve_target`, no
rewrite) + FFEDIT (editor command-line verbs migrated off the shared ladder).
FFLINE is named/modeled only (no code). All other FF* environments are
vision-only. Each code task is behaviour-preserving, keeps `cargo test -p
ff-desktop` green, and ships a test per criterion written first (red before
green). Keep both the env entry and the ladder arm reachable until the env entry
is test-proven (B080 per-step rollback discipline).

- [x] 1. Scaffold the environment framework (E0, pure indirection)
  - [x] 1.1 Define the `CommandEnvironment` trait (name + claim-or-fall-through
        dispatch returning an outcome with a return code), reusing the
        `TargetResolver` shape. The claim contract resolves a surface-form alias
        to a CANONICAL verb (per-environment alias table) before dispatch.
        Validates: Requirement 1.1, 6a.1.
    - NOTE: the trait is defined in `shell/environment.rs`; FFEDIT claims via
      the `WorkbenchShell::ffedit_claim` method (its verbs mutate shell-entangled
      state), the documented variance in design.md.
  - [x] 1.2 Make the active environment a KIND-SUPPLIED ATTRIBUTE: the focused
        kind declares its command-environment NAME (reconcile with the CR-NR-090
        Kinds registry as the attribute's home); the handler reads the attribute
        and dispatches to it, defaulting to FFCMD when none -- NO central match
        enumerating kinds. Phase-1 kinds declare editor -> FFEDIT, navigator ->
        FFNAV, else FFCMD. Validates: Requirement 1.3, 1.4, 2.1, 2.2, 2.3, 9.1.
    - NOTE: Option 2 -- derived via `environment_for_kind(BuiltinKind::from_tab_kind(..))`
      in code; the persisted user-configurable `command_environment` attribute on
      `KindConfig` is deferred to a later Kinds-config slice (owner decision).
  - [x] 1.3 Insert the active-env step into the shared ladder path
        (`run_command_ladder`, reached by every `handle_command` caller) at the
        former `try_commands_b2` precedence point, gated by the active env being
        FFEDIT; prove no shell test regresses (946 pass). Validates:
        Requirement 3.1, 3.2, 3.3, 3.4, 4.1.
    - NOTE: placement corrected from front-door-only to the shared ladder path
      because `handle_command` has many direct internal/test callers (design.md
      "E1 placement correction").
  - [x] 1.4 Build the per-environment ALIAS TABLE mechanism (surface form ->
        canonical verb), seeded with the existing English aliases (incl. EXCLUDE/X,
        SHOW/INCLUDE); case-insensitive; reject collisions at load; canonical verb
        is what is recorded/persisted. Per-locale alias data arrives under
        CR-NR-103 (no new mechanism then). Validates: Requirement 6a.1, 6a.2,
        6a.3, 6a.4, 6a.5.

- [x] 2. Migrate the FFEDIT navigation family (E1)
  - [x] 2.1 Move LOCATE/TOP/BOTTOM/UP/DOWN/LEFT/RIGHT/SORT into FFEDIT,
        delegating to `nav_manager` exactly as today (incl. up_by_amount/
        down_by_amount vs `scroll_amount` for bare UP/DOWN); delete those arms from
        `try_commands_b2`. Validates: Requirement 6.1, 6.2, 6.3, 4.2.
  - [x] 2.2 Add tests: typed-path parity (identical observable result to the prior
        ladder arm) for a representative nav verb; FFEDIT active only on an editor
        Context. Validates: Requirement 4.2, 6.4.

- [x] 3. Migrate the FFEDIT exclude/show family (E2)
  - [x] 3.1 Move EXCLUDE/X [ALL], SHOW/INCLUDE [ALL], RESET variants into FFEDIT
        via `exclude_manager` with the same snapshot closure; the ALL/text
        sub-parse moves inside the one entry. Delete the arms from
        `try_commands_b2`. Validates: Requirement 6.1, 6.2, 6.3.

- [x] 4. Migrate the FFEDIT find family (E3)
  - [x] 4.1 Move FIND/RFIND/CHANGE/RCHANGE into FFEDIT via `find_manager`,
        carrying CHANGE's `parse_two_args` quoting and the B062 case-preserved
        search term verbatim. Delete the arms from `try_commands_b2`. Validates:
        Requirement 6.1, 6.2, 6.3, 4.2.
  - [x] 4.2 Add a test asserting CHANGE argument case preservation + verb
        case-insensitivity (B062) via the FFEDIT path. Validates: Requirement 4.2,
        6.3.

- [x] 5. Migrate the FFEDIT profile family (E4)
  - [x] 5.1 Move CAPS/NULLS/STATS/LOCK/PROFILE/HILITE into FFEDIT via
        `edit_profile`; delete the arms from `try_commands_b2`. Validates:
        Requirement 6.1, 6.2, 6.3.

- [x] 6. Migrate the FFEDIT scroll verb (E5)
  - [x] 6.1 Move SCROLL <amt> into FFEDIT via `scroll_amount`; delete the arm from
        `try_commands_b2`. Validates: Requirement 6.1, 6.2, 6.3.

- [ ] 7. Name and model FFLINE + FFNAV (E6, no code)
  - [x] 7.1 Document FFLINE in the Environment_Registry model as a sibling of
        FFEDIT with UNCHANGED prefix-area intake (-> Command_Engine), NOT routed
        through the command-line front door; no line-command behaviour change.
        Validates: Requirement 7.1, 7.2, 7.3.
    - NOTE: FFLINE is modeled in environments-vision.md and the EnvironmentKind
      doc comments; a first-class `FfLine` variant is not needed in phase 1 since
      prefix-area intake does not pass through `active_environment`.
  - [x] 7.2 NAME FFNAV for the file-navigator Contexts (FilesPanel /
        FileExplorerPanel / catalog) in the active-env derivation; navigator verbs
        (incl. its own FIND) stay handled as today -- NOT migrated, no behaviour
        change. Makes the per-environment verb model (FFEDIT.FIND vs FFNAV.FIND)
        concrete. Validates: Requirement 2.1, 2a.1, 2a.2, 7a.1, 7a.2, 7a.3.

- [x] 8. Retire the emptied shell-ladder editor arms (E7)
  - [x] 8.1 Once `try_commands_b2` is emptied by tasks 2-6, delete it and drop its
        call from `run_command_ladder`; confirm the front door still routes every
        editor verb through FFEDIT and every workbench verb through FFCMD.
        Validates: Requirement 3.1, 4.1.
    - NOTE: after the E1-E5 migration the only arm left in `try_commands_b2` was
      THEME (an FFCMD verb, not FFEDIT). E7 folded THEME back inline into
      `run_command_ladder` at the same precedence point and deleted
      `commands_ladder_b2.rs` + its module decl + call. Behaviour-preserving.

- [x] 9. Environment-before-FFCMD ordering + shadowing (E8)
  - [x] 9.1 Move the FFEDIT-claim gate to the START of `run_command_ladder`, BEFORE
        `try_commands_a`, gated by the active env being FFEDIT. `X`/`RETURN`/`EXIT`/
        `QUIT`/`LOGOFF` stay ordinary FFCMD verbs in `try_commands_a`, reached AFTER
        the env claim. On an editor Context FFEDIT claims its verbs (incl. bare `X`
        -> EXCLUDE) first; on a non-editor Context the gate is false. Validates:
        Requirement 2a.4, 3.1, 3.3, 4.1, 5.1, 5.3.
    - IMPL NOTE (differs from the first design draft): `try_exit_family` was NOT
      fully removed from `run_command_prelude`. It had to STAY there (gated),
      because the prelude's later `resolve_pom_option_key` resolves bare `X` to the
      POM `Return` option -- so simply deleting the prelude exit-claim let the POM
      fastpath swallow `X` on a non-editor Context (3 exit tests failed). The
      correct shape: in the prelude, run `try_exit_family` ONLY when `(!editor_env_active || is_equals_prefixed)`. So on an editor Context a BARE verb
      skips the prelude exit-claim and falls to the ladder's FFEDIT gate (bare `X`
      -> EXCLUDE), while `=X` and every non-editor `X` are still claimed by the
      prelude exit-family ahead of the POM fastpath, exactly as before. `make_shell()`
      opens on an editor Context, which is why this surfaced immediately in tests.
  - [x] 9.2 Narrow `=` rule (E8 part): a `=`-prefixed command string is NOT offered
        to the active environment -- the ladder's FFEDIT gate is skipped when `cmd`
        starts with `=`, and the prelude exit-family claims `=X` (so it reaches
        FFCMD's `X` = return/exit) regardless of the focused environment. (The FULL
        `=` unification -- one universal `=` step for `=X`/`=0`/`=0.K`/`=FILES` --
        is E8b; 9.2 only guarantees the env skips `=`-prefixed input.) Validates:
        Requirement 3.2, 3.2a, 5.2.
  - [x] 9.2a RESET / RESET BARE ownership boundary (collision found during E8).
        Moving the FFEDIT gate ahead of `try_commands_b1` exposed a verb collision:
        FFEDIT owns `RESET` (exclusion reset), FFCMD owns `RESET BARE` (profile
        reset). FFEDIT was swallowing `RESET BARE` on an editor Context. FIX:
        `ffedit_claim`'s RESET arm DECLINES when the first argument is `BARE`
        (case-insensitive), so `RESET BARE [..]` falls through to FFCMD's ladder
        arm. This is the ONLY FFCMD verb whose first token collides with an FFEDIT
        verb. Validates: Requirement 2a.4 (ownership), 4.3, 5.1.
  - [x] 9.3 Tests (red before green): full-shell -- bare `X` on an editor Context
        EXCLUDEs (does not close/exit); bare `X` on a non-editor Context exits
        (unchanged); `=X` on an editor Context does return/exit (not EXCLUDE); a
        representative FFEDIT verb still resolves; plus the existing exit/reset_bare
        tests stay green (953 passed, 0 failed scoped). Replaced the provisional
        `prelude_owned_verbs_are_not_shadowable_by_ffedit` with
        `ffedit_shadows_ffcmd_x_but_not_the_other_exit_verbs` +
        `equals_prefixed_command_bypasses_the_active_environment`; kept
        `ffedit_claims_owned_verb_only_when_active_env_is_ffedit`. Validates:
        Requirement 4.1, 5.1, 5.2, 5.3.

- [ ] 9b. Unify `=` handling in ONE universal step (E8b)
  - [ ] 9b.1 Refactor so leading `=` is interpreted in ONE place at the top of the
        shared path (drop Navigation_Stack -> re-dispatch the remainder from the
        POM base), and make the existing POM/chained/`=FILES` fastpaths CONSUMERS
        of the already-stripped remainder rather than each re-interpreting `=`.
        Behaviour-preserving for every existing `=0` / `=0.K` / `=0;E.T` / `=FILES`
        case (covered by the menu-workspace / fastpath tests). Validates:
        Requirement 3.2. (Deferred out of E8 to keep E8 small; do only after 9.1-9.3
        are green.)

- [ ] 10. Macro ADDRESS reconciliation (alias map + RC)
  - [ ] 10.1 Route the macro `ADDRESS <env>` / Lua `address(<env>)` binding through
        the Scripting_Bridge into the named environment via the Alias_Map (TSO ->
        FFCMD, ISREDIT -> FFEDIT); addressed-env == active-env is a no-op wrapper;
        the dispatch outcome's return code maps to macro `RC`. Reconciles
        lua-macro-engine Req 11.11-11.15 (no parallel mechanism). Validates:
        Requirement 8.1, 8.2, 8.3.

- [ ] 11. Future-context template + environment-authoring boundary
  - [ ] 11.1 Confirm a new Context gets its environment by declaring the env NAME
        as a kind attribute (data), with NO handler/front-door change; the
        attribute is reconfigurable + RESET-BARE recoverable. Validates:
        Requirement 9.1, 9.2.
  - [ ] 11.2 Enforce the authoring boundary: built-in environments (FFCMD/FFEDIT)
        are code-only and not replaceable via configuration; a NEW executable
        environment is a PLUGIN capability (plugin API + permission model); per-verb
        customization is via a shadowing layered environment (Req 5), not wholesale
        replacement. Phase-1 scope builds no env beyond framework + FFCMD + FFEDIT
        (FFLINE/FFNAV named-only). Validates: Requirement 9.3, 9.4, 9.5, 9.6.

- [ ] 12. Deferred interactive address prefix (non-task; recorded)
  - NOT built in phase 1 (Requirement 8.4). The `parse_address_prefix` seam is
    reserved in the router; no interactive-prefix code or test is written now.

- [ ] 13. Editor-buffer SAVE verb + dirty-aware END/CANCEL/RETURN (E9) -- Requirement 10
  > SCOPE (owner-flagged findings): only SAVE is a clean reusable op
  > (`tab_manager::save_active_tab`). UNDO is keyboard-only inline (Ctrl+Z in
  > `editor_panel/input.rs`) -> claiming it is an EXTRACTION refactor; REDO does
  > NOT exist -> a NEW feature needing its own gate. Both DEFERRED (Req 10.6).
  - [x] 13.1 Add SAVE as an FFEDIT identity alias + a `ffedit_claim` arm that is
        DIRTY-AWARE and STAYS in the editor: clean (`!is_modified`) -> no-op (no
        write); dirty -> `tab_manager::save_active_tab` (write + clear flag + save
        point), write-fail -> stay + error. Non-editor SAVE unclaimed by FFEDIT.
        TDD red-before-green (clean no-op; dirty writes + clears flag; non-editor
        unresolved). Validates: Requirement 10.1, 10.5. (Small refinement over
        today's unconditional save: the clean no-op guard.)
    - DONE 2026-10-03 (scoped 955/0, clippy/fmt clean): `AliasTable::ffedit_english`
      gains `("SAVE","SAVE")`; `ffedit_claim` "SAVE" arm -> `ffedit_save()` in
      dispatch_ffedit.rs (clean no-op; dirty save_active_tab; Err=stay+error).
      Tests: `editor_save_on_clean_buffer_is_noop`,
      `editor_save_on_dirty_untitled_errors_and_stays`.
  - [ ] 13.2 Dirty-aware END / CANCEL / RETURN in FFEDIT via a shared
        `leave_editor` helper: CLEAN -> delegate to `nav_end` (END/CANCEL) /
        `nav_return` (RETURN); DIRTY END -> `save_active_tab` then leave, save-fail
        -> stay + error; DIRTY CANCEL/RETURN -> open a confirm dialog ("changes
        will be lost") via the established modal pattern (new `Option<LeaveEditorConfirm>`
        state + render arm + Cancel-default focus), confirm -> discard + leave,
        cancel -> stay. Validates: Requirement 10.2, 10.3, 10.7.
        - NOTE: this is NEW behaviour (dirty END saves; dirty CANCEL/RETURN
          confirm), not behaviour-preserving. The `-Y`/`-N` + interactive-flag
          wiring is command-framework Req 16 (Phase confirmable-commands); 13.2
          implements the INTERACTIVE dialog path and is Req-16-ready.
        - HELD for owner review (not built unsupervised): unlike 13.1 (SAVE, safe
          + behaviour-preserving), 13.2 introduces a NEW modal confirm dialog +
          changes END/RETURN behaviour in the editor + shadows navigation verbs.
          New visible UX + new behaviour is best signed off by the owner, and the
          discard-confirm's -Y/-N path depends on Req 16 infra (CC.1-CC.7) not yet
          built. Design is fully gated (Req 10.2/10.3/10.4/10.7); implement on the
          owner's return (or alongside the confirmable-command slice).
  - [ ] 13.3 Tests (red before green): per table row -- clean END/CANCEL up one
        level, clean RETURN to top; dirty END saves then leaves; dirty END save-fail
        stays + errors; dirty CANCEL/RETURN open the confirm, confirm discards +
        leaves, cancel stays; non-editor END/RETURN unchanged. Validates:
        Requirement 10.2, 10.3, 10.5, 10.7.
  - [ ] 13.4 UNDO/REDO carve-out (Req 10.6): record the deferral; do NOT build REDO
        (new feature) or refactor UNDO unsupervised. Owner decision pending.

- [ ] 14. Per-editing-Context SAVE (E10 part 1) -- Requirement 11.1/11.2
  - [ ] 14.1 Give each editing Context (Theme / Menus / Keys / Kinds / Config) a
        command-line SAVE that invokes the SAME save ACTION as its existing Save
        button (command parity), via the `pending_action` pattern (claim SAVE ->
        stash the Context's save action -> shell `apply_<panel>_action`). No global
        SAVE, no FFCMD SAVE. Validates: Requirement 11.1, 11.2, 2a.4, 2a.5.
  - [ ] 14.2 Tests: typed SAVE on each editing Context persists the working copy
        identically to its button path (one path, not two). Validates: Requirement
        11.1, 11.2.

- [ ] 15. Command chaining (E10 part 2) -- Requirement 11.3-11.8
  - [ ] 15.1 Add a QUOTE-AWARE `;` splitter and dispatch each segment, in order,
        through the ONE existing front door (sequential re-dispatch; no second
        dispatcher). Re-derive the active environment per segment; `=` scopes to its
        own segment; continue-on-error (surface each segment status). Reconcile with
        the existing menu-only `try_chained_fastpath` so a chain is split ONCE at the
        top (menu fastpath becomes a per-segment consumer). Validates: Requirement
        11.3, 11.4, 11.5, 11.6, 11.7, 11.8.
  - [ ] 15.2 Tests: `SAVE; X ALL; FIND ALL 'a;b'; =0` on an editor Context runs
        each segment in order (SAVE buffer, EXCLUDE ALL, FIND for the literal `a;b`
        -- the quoted `;` not split, then `=0` returns to POM); a failing segment
        does not abort the chain; a `;` inside quotes is not a separator. Validates:
        Requirement 11.3, 11.4, 11.5, 11.6, 11.7.
- [ ] 16. FFEDIT CUA editing verbs + keyboard->command (CR-CH-054) -- Requirement 12 (+ command-framework Req 17)
  > Layered. Slice 1 = keyboard->command dispatch wiring + verbs whose ops exist
  > (SAVE done; UNDO extract). Slice 2 = CUT/PASTE/SELECT-ALL/COPY-unify + REDO
  > (new feature) + ff-clipboard + the FFLINE `C`/`CC` and `A`/`B` accessors.
  - [x] 16.0 Ctrl+S point-fix: `shell/update.rs` Ctrl+S dispatches `SAVE` through
        the front door -> FFEDIT dirty-aware SAVE (not direct `save_active_tab`),
        closing the key/verb divergence. Test `ctrl_s_on_clean_editor_is_noop_via_ffedit_save`.
        DONE 2026-10-03 (scoped 956/0, clippy/fmt clean). Validates: cmd-framework Req 17.4.
  - [ ] 16.1 (slice 1) Keyboard->command dispatch: a bound chord resolves to its
        command string and dispatches through `dispatch_command_string`; bind the
        Reserved_CUA_Set (Ctrl+Z/Y/C/X/V/A/S -> UNDO/REDO/COPY/CUT/PASTE/SELECT ALL/
        SAVE) as reserved (not overridable); non-reserved Ctrl/Alt via keys editor/
        TOML. Retire the editor Ctrl+Z / editor Ctrl+C / shell Ctrl+Shift+P/F inline
        handlers in favour of the dispatch path. Validates: cmd-framework Req 17.1,
        17.2, 17.3, 17.4, 17.6, 17.7.
  - [ ] 16.2 (slice 1) UNDO verb: extract the inline Ctrl+Z undo logic from
        `editor_panel/input.rs` into an FFEDIT UNDO verb so the key and the verb
        share one path. Validates: Requirement 12.1, 12.8 (UNDO half).
  - [ ] 16.3 (slice 2) COPY/CUT verbs with source precedence: Cursor_Context
        selection first; else pending `C`/`CC` line block (to clipboard); else
        no-op. CUT also deletes (dirty + undo). Clear `C`/`CC` markers after.
        Clipboard via `ff-clipboard`. Validates: Requirement 12.1, 12.2, 12.3, 12.6.
  - [ ] 16.4 (slice 2) PASTE verb with destination precedence: cursor-in-editor
        (insert at cursor) first; else pending `A`/`B` marker (insert as whole
        lines after/before, line-granular); cursor wins tie-break; clear `A`/`B`
        after, retain clipboard. Validates: Requirement 12.4, 12.5, 12.6.
  - [ ] 16.5 (slice 2) SELECT ALL verb (whole-buffer Cursor_Context selection) +
        REDO (NEW feature: add a redo stack + logic, coordinate with
        undo-redo-transactions; Ctrl+Y -> REDO). Validates: Requirement 12.7, 12.8
        (REDO half).
  - [ ] 16.6 (slice 2) FFLINE accessors: expose the pending `C`/`CC` source block
        and `A`/`B` destination marker from the line-command layer for the FFEDIT
        verbs to read. Validates: Requirement 12.5.
  - [ ] 16.7 Tests (red before green): keyboard chord == typed verb (identical
        result); reserved keys work in every Context + not overridable; non-reserved
        configurable; COPY/CUT source precedence (cursor / `C`/`CC` / none); CUT
        deletes+dirty+undo+clears markers; PASTE destination precedence (cursor /
        `A`/`B` / none), `A`/`B` line-granular + tie-break + marker clear + clipboard
        retained; SELECT ALL; UNDO via verb; REDO. egui TextEdit native editing
        unaffected. Validates: Requirement 12 (all) + cmd-framework Req 17 (all).

## Phase-1 maturation (CR-CH-053 DESIGN-BRIEF): built registry + address-by-name + owning-environment binding + FS-CE family

Owner-approved DESIGN-BRIEF Phase-1 core change. Vertical-slice order keeps FFWB
building and NATIVE behaviour byte-identical between tasks. Each code-bearing task
writes its test FIRST (red before green); GUI/behaviour criteria ship full-shell
`egui_kittest` tests per `testing.md` / `workspace-conformance.md`. All new/changed
source files stay under the 400-line rule; new CEs wire through the registry + the
one command seam, never a new dispatcher (`wiring-standard.md`). `[ ]` only.

- [ ] 17. Build the Environment_Registry + make FFEDIT a real object (slice a, behaviour-preserving)
  - [ ] 17.1 Replace the closed `EnvironmentKind` enum + hardcoded
        `environment_for_kind` match + `== FfEdit` claim gate with a BUILT
        shell-owned registry of `Box<dyn CommandEnvironment>`; environments register
        into it at startup (FFCMD base implicit = resolve_target; FFEDIT; host FS env
        placeholder). Test FIRST: registry resolves the focused kind's env name,
        defaults to FFCMD for absent/unknown name. Validates: Requirement 13.1,
        13.2, 13.3, 13.4.
  - [ ] 17.2 Make FFEDIT a real `CommandEnvironment` OBJECT (move `ffedit_claim`
        verb bodies behind the trait, still delegating to the existing managers +
        dirty-aware SAVE path); register it in the registry. Test FIRST: every FFEDIT
        verb's observable result is unchanged via the object (parity with the prior
        method path). Validates: Requirement 13.5, 13.6, 4.2, 6.3.
  - [ ] 17.3 Prove the registry is not a second dispatcher: active-env derivation +
        claim gate read FROM the registry on the one shared ladder path; no change to
        `CommandTarget` or the per-tab Navigation_Stack. Full-shell test: with only
        FFEDIT + FFCMD + host FS env registered, a representative FFEDIT verb and a
        representative FFCMD verb resolve exactly as before. Validates: Requirement
        13.6, 13.7, 1.4, 3.3.

- [ ] 18. Add the address-by-name entry point dispatch_to_environment (slice b, additive)
  - [ ] 18.1 Add `dispatch_to_environment(name, raw)` to the registry (REXX ADDRESS
        applied internally): route a raw command to the named env regardless of the
        active one; addressed-env == active-env is a no-op wrapper; the outcome
        carries a return code. Test FIRST: addressing the active env equals not
        addressing; addressing a second registered env routes to it. Validates:
        Requirement 14.1, 14.2.
  - [ ] 18.2 Classify FFEDIT verbs into IN-BUFFER (handled directly) vs
        STORE-AFFECTING (addressed). Test FIRST: an in-buffer verb (e.g. LOCATE /
        EXCLUDE) is never addressed to another env; the store verb SAVE is routed via
        `dispatch_to_environment`. Validates: Requirement 14.3, 14.7, 14.8.

- [ ] 19. TabState owning-environment field + capture at open (slice c, default host FS, behaviour-preserving)
  - [ ] 19.1 Add `owning_environment` to `TabState`; capture it at open from the
        originating catalog/provider (the `CatalogType` discarded today), threaded
        through `file.open` via a new `CommandParams` entry; default =
        Host_FS_Environment when no origin is supplied. Test FIRST: a plain host-path
        open binds to the host FS env; an open carrying an origin binds to that env.
        Validates: Requirement 15.1, 15.2, 15.3.
  - [ ] 19.2 Persist/restore the Owning_Environment via the EXISTING
        `WorkspaceDescriptor` model (recapture from origin on reopen); no new
        persistence format. Test FIRST: a tab reopened from its descriptor recaptures
        the same Owning_Environment. Validates: Requirement 15.5.
  - [ ] 19.3 FFEDIT reads the tab's Owning_Environment to choose the SAVE target.
        Test FIRST (full-shell): on a host-bound tab FFEDIT targets the host FS env;
        on a non-host-bound tab FFEDIT addresses that env. Validates: Requirement
        15.4, 15.6.

- [ ] 20. Redirect FFEDIT SAVE to address the owning environment (slice d, MODIFIES Req 10.1 -- native == today)
  - [ ] 20.1 Move the local-FS byte-write SAVE logic (`tab_manager::save_active_tab`)
        INTO the host FS CE's SAVE; FFEDIT SAVE now ADDRESSes the Owning_Environment
        via `dispatch_to_environment` instead of writing the store directly.
        PRESERVE the Req 10.1 dirty-awareness contract exactly (clean no-op; dirty
        write + stay + clear flag + save point; write-fail stay + error; not
        Confirmable). Test FIRST (red before green): native SAVE is on-disk-identical
        and dirty/save-point-identical to today (byte-for-byte). Validates:
        Requirement 14.4, 14.5, 14.6, 10.1 (routing change, behaviour preserved).
  - [ ] 20.2 Full-shell `egui_kittest` test: edit a native file, type SAVE, assert
        the host FS env performed the write and the dirty flag/save point cleared,
        identical to the pre-change path. Validates: Requirement 14.5, 14.6.

- [ ] 21. Host-fs decider + light ntfs + posix CEs (slice e, additive)
  - [ ] 21.1 Add `ff-ce-host-fs` (decider: detect host at startup, resolve the
        native ROLE to the concrete FS CE) + `ff-ce-ntfs` + `ff-ce-posix` (LIGHT --
        OS-backed controls/attrs; byte-write SAVE == today; cheap FS defaults:
        case-insensitive vs case-sensitive). Each `impl CommandEnvironment` + register
        into the registry. Add the crates to `ff-desktop/Cargo.toml`. Test FIRST: the
        decider resolves Windows -> ntfs and Linux/macOS -> posix; the resolved env is
        the default Owning_Environment. Validates: Requirement 16.1, 16.2, 16.3,
        16.4, 16.5.
  - [ ] 21.2 Record (test or doc-assert) that deep NTFS/POSIX/APFS semantic
        emulation + cross-emulation are DEFERRED, and that a new executable FS CE is a
        plugin capability (Req 9.4) -- built-in CEs code-only, not
        configuration-replaceable (Req 9.3). Validates: Requirement 16.6, 16.7.

- [ ] 22. Live ProviderRegistry registration prerequisite (slice f, additive shell wiring)
  - [ ] 22.1 Register an `ff-vfs` `ProviderRegistry` LIVE in the shell at startup
        (the built/tested stack is not registered live today); it is the seam a
        plugin-provided `VfsProvider` (e.g. the mainframe VFS provider) registers
        into. Test FIRST: host-path file access is unchanged with the registry live;
        a registered provider is resolvable at runtime. Validates: Requirement 17.1,
        17.2, 17.3.
  - [ ] 22.2 Document the dependency: the non-host parts of Req 14-16 depend on this
        live registration; the host-FS default path does not. (No code beyond 22.1;
        this records the dependency so later non-host phases reference it.)
        Validates: Requirement 17.4.
