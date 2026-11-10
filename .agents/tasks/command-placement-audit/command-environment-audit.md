# Command Environment Placement Audit (CR-CH-053 / CR-CH-067)

Read-only analysis of every command registered in each Command Environment, an
evaluation of whether each is in the correct environment, a dedicated shortlist
of the misplaced ones, and a gated plan of action to relocate them. Grounded in
live source at `c:\workspace\apps\FileForgeWorkbench` on `main` (HEAD `c73f41d`,
which includes CR-CH-067 the root-enumeration API). No code or specs were changed.

---

## 1. Summary answer (read this first)

The owner's instinct is correct: several commands are in the wrong environment,
and the AUTONUM example is the clearest case. The misplaced set and the single
biggest data-level surprise:

1. **AUTONUM / NUM (and their target NUMBER)** are registered on the **FFCMD**
   ladder (`commands_ladder_c.rs`, and listed in `BUILTIN_FFCMD_LADDER_VERB_ROOTS`
   in `command_config/mod.rs`). They are **in-buffer editor behaviours** (toggle
   automatic line renumbering / generate sequence numbers in the open buffer).
   By the CR-CH-053 ownership principle (Req 2a.4, Req 10) and the
   `ff-sequence-numbers` design, these belong to **FFEDIT**. **Verdict: MISPLACED.**

2. **SUBMIT, STATUS, TIME, CREATE, REPLACE, COMPARE** are FFCMD ladder stubs.
   Most are genuinely workbench/dataset verbs (CORRECT for FFCMD), but **SUBMIT**
   and **TIME** are judgment calls discussed below.

3. **A data-level placement bug already shipped**: the Afrikaans POC alias
   `crates/ff-i18n/i18n/af/aliases/FFCMD.toml` maps `SOEK -> FIND` under the
   **FFCMD** environment, but **FIND is an FFEDIT root** (command-semantics
   Req 12.1; `ffedit_english()` owns FIND). This row is only un-rejected because
   the per-locale reader path does **not** run the CR-CH-067 `contains_root`
   check that the user-store path runs. This is exactly the "misplaced commands
   corrupt the alias data" risk the owner anticipated, and it is already present.

4. The owner's separate `=FILES` observation is confirmed: `=FILES` is registered
   as a literal root token (`BUILTIN_WORKSPACE_VERB_ROOTS`) where `=` is the ISPF
   jump prefix and the real root is `FILES`. Noted here; it is an enumeration/root
   bug, not an environment-placement bug, so it is out of primary scope but
   flagged because it pollutes the FFCMD root set the alias anchoring reads.

Recommended sequencing: do the **placement corrections as ONE gated change
request BEFORE** the Afrikaans alias-data work and the `=FILES` fix, because the
corrected (environment, root) sets are the anchor both of those tasks build on.

---

## 2. The environment model (what "correct" means here)

CR-CH-053 (`docs/specs/command-environments/requirements.md`) defines the model
and the semantic test used for every verdict below:

- **FFCMD** -- the workbench / command-line base environment. It **IS**
  `resolve_target` / `ShellTargetResolver` (Req 1.2). Owns workbench verbs: open
  a Workspace/dataset, navigate menus, manage tabs/splits, run tools, cross-file
  SEARCH. Ownership principle (Req 2a.4): "a verb that acts on the workbench ...
  is owned by FFCMD".
- **FFEDIT** -- the in-buffer text-editor environment. Owns verbs that act on the
  **content of the buffer currently open in the editor** (Req 6.1, Req 2a.4,
  Req 10). "A verb that acts on the active editor buffer ... is owned by FFEDIT."
- **FFLINE** -- prefix-area line commands; **named/modeled only** in phase 1
  (Req 7). Intake is the prefix gutter -> `ff-command-semantics` engine, not the
  command line. Not enumerable (`root_commands("FFLINE")` is empty).
- **FFNAV** -- file-navigator environment (FilesPanel / FileExplorerPanel /
  catalog); **named/modeled only** (Req 7a). Its FIND locates a *file*, distinct
  from FFEDIT.FIND. Not a registered claiming environment; `active_name` derives
  it but it degrades to the FFCMD base (`environment_registry.rs::active_env`).
- Others (FFAMS, FFJES, FFSQL, ...) are vision-only (`environments-vision.md`).

The **two real, enumerable** environments are **FFCMD and FFEDIT**
(`EnvironmentRegistry::root_commands` returns non-empty only for these two --
`environment_registry.rs`, Req 20.1/20.5). The placement question is therefore
almost entirely **FFCMD vs FFEDIT**.

### The semantic test applied below
> Does the command act on the WORKBENCH / a target / navigation (**FFCMD**), or
> does it act on the CONTENT of the buffer currently open in the editor
> (**FFEDIT**)?

---

## 3. FFCMD environment -- command catalogue

FFCMD commands come from three code sources that are unioned into the FFCMD root
set by `builtin_ffcmd_roots()` (`crates/ff-desktop/src/shell/alias_reader.rs:131-175`):

- (a) the two code-only built-in menus' option roots
  (`menu_workspace/defaults.rs` `DEFAULT_POM_TOML` / `DEFAULT_SETTINGS_TOML`);
- (b) `BUILTIN_WORKSPACE_VERB_ROOTS` -- verbs classified by `resolve_target`
  (`command_config/mod.rs:builtin_workspace_target_for`);
- (c) `BUILTIN_FFCMD_LADDER_VERB_ROOTS` -- verbs handled by the `handle_command`
  ladder (`commands_ladder_{a,b,c}.rs`).

`EnvironmentRegistry::root_commands("FFCMD")` returns the sorted, de-duplicated
union of all three (`environment_registry.rs`).

### 3a. Menu-option roots (built-in POM + Settings menus)

Source: `crates/ff-desktop/src/menu_workspace/defaults.rs`.

| Command | Purpose | Source location | Placement verdict | Should be / why |
|---------|---------|-----------------|-------------------|-----------------|
| SETTINGS | Open the Settings menu (POM option 0) | defaults.rs `DEFAULT_POM_TOML` (key 0) | CORRECT | Workbench navigation |
| CATALOGS | Open Virtual File Catalogs (POM option 1) | defaults.rs POM key 1 | CORRECT | Workbench navigation |
| FILES | Open the File Explorer tree (POM option 2) | defaults.rs POM key 2 | CORRECT | Workbench navigation |
| HELP | Open context help (POM option 3) | defaults.rs POM key 3; also `commands_ladder_a.rs` HELP arm | CORRECT | Workbench-wide |
| X | Collapse to visual root / close workspace (POM option X) | defaults.rs POM key X; `commands_ladder_a.rs` X arm | CORRECT (FFCMD) | FFCMD close verb; note FFEDIT **also** owns surface `X`=EXCLUDE (Req 5.1 active-wins). Dual ownership is intended, not a bug |
| CONFIG | Browse every configuration key (Settings C) | defaults.rs `DEFAULT_SETTINGS_TOML` key C; classifier `command_config/mod.rs` | CORRECT | Acts on workbench configuration |
| ALIASES | Open the Alias Editor (Settings A) | defaults.rs Settings key A; classifier | CORRECT | Workbench config tool |
| THEME | Open Theme Editor (bare) / apply theme `THEME <name>` | defaults.rs Settings T; classifier (bare) + `commands.rs` (named apply) | CORRECT | Theme is a workbench working copy, not the buffer |
| MENUS | Open the Menus Editor (Settings M) | defaults.rs Settings M; classifier | CORRECT | Workbench config tool |
| KEYS | Open the Keys Editor (Settings K) | defaults.rs Settings K; classifier | CORRECT | Workbench config tool |
| KINDS | Open the Workspace Kinds editor (Settings W) | defaults.rs Settings W; classifier | CORRECT | Workbench config tool |
| RESET BARE | Archive config and reset to barebones (Settings R) | defaults.rs Settings R; `commands_ladder_b1.rs` RESET BARE arm | CORRECT | Workbench recovery |

### 3b. `resolve_target` classifier roots (`BUILTIN_WORKSPACE_VERB_ROOTS`)

Source: `crates/ff-desktop/src/command_config/mod.rs` (list `~167-182`;
classifier `builtin_workspace_target_for` `~206-280`).

| Command | Purpose | Source location | Placement verdict | Should be / why |
|---------|---------|-----------------|-------------------|-----------------|
| FILES | Open File Explorer workspace | mod.rs classifier `is("FILES")` | CORRECT | Workbench navigation |
| =FILES | Same as FILES, literal `=`-prefixed token | mod.rs classifier `is("=FILES")`; list entry `=FILES` | CORRECT env, **MALFORMED ROOT** | `=` is the ISPF jump prefix; the root should be `FILES`. Owner-known `=FILES` bug (see sec. 7). Environment is right (FFCMD), the *token* is wrong |
| FILE CATALOGS | Open the catalogs workspace (two-word verb) | mod.rs classifier `is("FILE") && arg=="CATALOGS"` | CORRECT | Workbench navigation |
| CATALOGS | Open the catalogs workspace | mod.rs classifier `is("CATALOGS")` | CORRECT | Workbench navigation |
| CONFIG | Open config workspace (optional namespace arg) | mod.rs classifier `is("CONFIG")` | CORRECT | Workbench config |
| KEYS | Open Keys editor (optional kind arg) | mod.rs classifier `is("KEYS")` | CORRECT | Workbench config |
| THEME | Open Theme editor (bare) | mod.rs classifier `is("THEME")` | CORRECT | Workbench working copy |
| COMMANDS | Open the Command Configurator | mod.rs classifier `is("COMMANDS")` | CORRECT | Workbench config |
| LOG | Open the Event Log workspace | mod.rs classifier `is("LOG")` | CORRECT | Workbench tool |
| PLUGINS | Open the Plugin Manager | mod.rs classifier `is("PLUGINS")` | CORRECT | Workbench tool |
| MACROS | Open the Macro Library | mod.rs classifier `is("MACROS")` | CORRECT | Workbench tool |
| GSEARCH | Cross-backend global search workspace | mod.rs classifier `is("GSEARCH")` | CORRECT | Cross-file search = FFCMD (command-semantics Req 12.2) |
| SEARCH | Cross-backend global search workspace | mod.rs classifier `is("SEARCH")` | CORRECT | **Explicitly FFCMD** per command-semantics Req 12.2. (GREP is an intended alias surface, Req 12.3; not yet seeded) |
| KINDS | Open the Workspace Kinds editor | mod.rs classifier `is("KINDS")` | CORRECT | Workbench config |
| MENUS | Open the Menus editor | mod.rs classifier `is("MENUS")` | CORRECT | Workbench config |
| ALIASES | Open the Alias editor | mod.rs classifier `is("ALIASES")` | CORRECT | Workbench config |

### 3c. `handle_command` ladder roots (`BUILTIN_FFCMD_LADDER_VERB_ROOTS`)

Source list: `crates/ff-desktop/src/command_config/mod.rs` `~190-224`. Each arm
cited to its ladder file.

| Command | Purpose | Source location | Placement verdict | Should be / why |
|---------|---------|-----------------|-------------------|-----------------|
| EDIT | Open a path as an editable editor tab | `commands_ladder_a.rs` EDIT arm (`verb_arg(cmd,"EDIT")`) | CORRECT (FFCMD) | The ACT of opening a file into the workbench is a workbench op. (Note the discrepancy in sec. 6: the user-alias enum comment treats ECOB->EDIT as *FFEDIT*; the code places EDIT in FFCMD) |
| VIEW | Open a path read-only (deferred full browse) | `commands_ladder_c.rs` VIEW arm | CORRECT | Opens a target into the workbench |
| BROWSE | Open a path read-only | `commands_ladder_c.rs` BROWSE arm | CORRECT | Opens a target into the workbench |
| COMPARE | Open a compare view (stub) | `commands_ladder_c.rs` COMPARE arm | CORRECT | Opens a workbench view over a target |
| CREATE | Create a dataset (stub) | `commands_ladder_c.rs` CREATE arm | CORRECT | Dataset/workbench op |
| REPLACE | Replace a dataset (stub) | `commands_ladder_c.rs` REPLACE arm | CORRECT | Dataset/workbench op |
| EXIT | Exit the application | `commands_ladder_a.rs` `try_exit_family` | CORRECT | App lifecycle |
| QUIT | Alias of EXIT | `commands_ladder_a.rs` `try_exit_family` | CORRECT | App lifecycle |
| LOGOFF | Alias of EXIT (with message) | `commands_ladder_a.rs` `try_exit_family` | CORRECT | App lifecycle |
| START | Create a new Workspace tab | `commands_ladder_a.rs` START arm | CORRECT | Workspace lifecycle |
| MENU | Open a menu by name | `commands_ladder_a.rs` MENU / MENU.OPEN arms | CORRECT | Workbench navigation |
| CLOSE | Close the current tab | `commands_ladder_a.rs` CLOSE arm | CORRECT | Workspace lifecycle |
| DOCK | Re-dock a detached workspace | `commands_ladder_a.rs` DOCK arm | CORRECT | Window/workspace management |
| END | Pop one navigation level (editor-dirty-aware when FFEDIT active) | `commands_ladder_a.rs` END arm | CORRECT (FFCMD/universal) | Navigation; FFEDIT overrides when active (Req 10.2), already modelled |
| RETURN | Collapse navigation stack to visual root | `commands_ladder_a.rs` RETURN arm | CORRECT (FFCMD/universal) | Navigation; FFEDIT overrides when active (Req 10.2) |
| DETACH | Detach current workspace to an OS window | `commands_ladder_c.rs` DETACH arm | CORRECT | Window management |
| SPLIT | Split the workspace area | `commands_ladder_c.rs` SPLIT/SPLIT RIGHT/DOWN arms | CORRECT | Workbench layout |
| UNSPLIT | Collapse a split | `commands_ladder_c.rs` UNSPLIT arm | CORRECT | Workbench layout |
| FOCUS | Move focus to the next split group | `commands_ladder_c.rs` FOCUS arm | CORRECT | Workbench layout |
| SWAP | Switch between tabs/workspaces | `commands_ladder_c.rs` SWAP arm | CORRECT | Workspace switching |
| WORKSPACE | Open/save/close workspace, add/remove roots | `commands_ladder_c.rs` WORKSPACE arm | CORRECT | Workspace lifecycle |
| PFSHOW | Toggle/cycle the PF-key display | `commands_ladder_a.rs` PFSHOW arm | CORRECT | Workbench chrome |
| COMMAND | Move the command line (TOP/BOTTOM/toggle) | `commands_ladder_b1.rs` COMMAND arm | CORRECT | Workbench chrome (per-Kind command-line position) |
| SNAPSHOT | Copy the active Context's logical screen to clipboard | `commands_ladder_b1.rs` SNAPSHOT arm | CORRECT | Workbench-wide screen capture (any Context) |
| CAPTURE | Drive the Screen Collection/Replay session | `commands_ladder_b1.rs` CAPTURE arm | CORRECT | Workbench SCRM session |
| RETRIEVE | Recall command-line history (F12) | `commands_ladder_b1.rs` RETRIEVE arm | CORRECT | Command-line history (workbench command line) |
| **AUTONUM** | Toggle automatic line renumbering (redirects to `NUMBER ON/OFF`) | `commands_ladder_c.rs` AUTONUM arm (`~128-135`) | **MISPLACED** | **FFEDIT** -- auto-renumbering edits the buffer's sequence columns (sec. 5) |
| **NUM** | Alias of NUMBER (redirects to `NUMBER [rest]`) | `commands_ladder_c.rs` NUM arm (`~140-148`) | **MISPLACED** | **FFEDIT** -- same reasoning as AUTONUM/NUMBER |
| SUBMIT | Submit a job to JES (stub) | `commands_ladder_c.rs` SUBMIT arm | AMBIGUOUS | Judgment call (sec. 5): submitting the *open buffer* leans editor/FFJES; submitting a *named dataset* leans FFCMD. Vision env is FFJES |
| TIME | Show current date/time/day | `commands_ladder_c.rs` TIME arm | CORRECT (lean) | Workbench/session utility; not buffer content |
| STATUS | Show JES routing status (stub) | `commands_ladder_c.rs` STATUS arm | CORRECT (lean) | Workbench/subsystem status; vision env FFJES |

Note: **NUMBER** itself (the redirect target of AUTONUM/NUM) has **no ladder arm
and no CommandEngine implementation** on `main`: it falls through
`run_command_ladder` to `cmd_engine.execute_command_line` (`commands.rs:~320`),
which reports it as an unrecognised command (`ff-command-semantics/src/engine.rs`
`ParsedCommand::Command` arm, step 3-10 "we report unrecognised commands"). The
real NUMBER/UNNUM/NUMBER-SHOW/NUMBER-ON/OFF engine lives in the `ff-sequence-numbers`
crate (`docs/specs/sequence-numbers/tasks.md` tasks 7, 8, 10) but is **not wired
into dispatch yet**. So AUTONUM/NUM are currently placeholder redirects to an
unimplemented verb -- which makes relocating them cheap now (sec. 8).

---

## 4. FFEDIT environment -- command catalogue

FFEDIT roots are the canonical verbs of `AliasTable::ffedit_english()`
(`crates/ff-desktop/src/shell/environment.rs:118-172`), enumerated by
`EnvironmentRegistry::root_commands("FFEDIT")`. Each verb's claim body runs via
the registered `FfEditEnvironment` object at the top of `run_command_ladder`
(`commands.rs:~150-182`, active-wins, Req 5.1). All act on the active editor
buffer.

| Command | Purpose | Source location | Placement verdict | Should be / why |
|---------|---------|-----------------|-------------------|-----------------|
| LOCATE | Jump to a line number / label in the buffer | environment.rs ffedit_english; `nav_manager.rs` | CORRECT | Buffer navigation |
| TOP | Scroll to top of buffer | environment.rs ffedit_english | CORRECT | Buffer navigation |
| BOTTOM | Scroll to bottom of buffer | environment.rs ffedit_english | CORRECT | Buffer navigation |
| UP | Scroll up | environment.rs ffedit_english | CORRECT | Buffer navigation |
| DOWN | Scroll down | environment.rs ffedit_english | CORRECT | Buffer navigation |
| LEFT | Scroll left | environment.rs ffedit_english | CORRECT | Buffer navigation |
| RIGHT | Scroll right | environment.rs ffedit_english | CORRECT | Buffer navigation |
| SORT | Sort buffer lines | environment.rs ffedit_english | CORRECT | Operates on buffer content |
| EXCLUDE | Exclude lines from view (surface alias `X`) | environment.rs ffedit_english (`EXCLUDE`, `X`->EXCLUDE) | CORRECT | Buffer view filter |
| SHOW | Show excluded lines (surface alias `INCLUDE`) | environment.rs ffedit_english (`SHOW`, `INCLUDE`->SHOW) | CORRECT | Buffer view filter |
| RESET | Reset exclude/show state | environment.rs ffedit_english | CORRECT | Buffer view state |
| FIND | Search for a string in the current buffer | environment.rs ffedit_english | CORRECT | **Explicitly FFEDIT** (command-semantics Req 12.1) |
| RFIND | Repeat last find | environment.rs ffedit_english | CORRECT | Buffer search |
| CHANGE | Find-and-replace in the buffer | environment.rs ffedit_english | CORRECT | Buffer edit |
| RCHANGE | Repeat last change | environment.rs ffedit_english | CORRECT | Buffer edit |
| CAPS | Toggle caps mode (edit profile) | environment.rs ffedit_english | CORRECT | Edit profile of the buffer |
| NULLS | Toggle nulls mode (edit profile) | environment.rs ffedit_english | CORRECT | Edit profile of the buffer |
| STATS | Toggle stats (edit profile) | environment.rs ffedit_english | CORRECT | Edit profile of the buffer |
| LOCK | Lock the buffer (edit profile) | environment.rs ffedit_english | CORRECT | Edit profile of the buffer |
| PROFILE | Show/toggle the edit profile | environment.rs ffedit_english | CORRECT | Edit profile of the buffer |
| HILITE | Syntax-highlight toggle | environment.rs ffedit_english | CORRECT | Buffer display |
| SCROLL | Set scroll amount | environment.rs ffedit_english | CORRECT | Buffer viewport |
| SAVE | Write the active buffer (dirty-aware) | environment.rs ffedit_english; `FfEditEnvironment` claim; Req 10.1 | CORRECT | Editor-buffer verb (Req 10.1) |

FFEDIT surface aliases (not roots): `X` -> EXCLUDE, `INCLUDE` -> SHOW. These are
correctly surfaces, not roots (`root_commands` deliberately excludes them,
Req 20.3; `environment_registry.rs` test `root_commands_ffedit_excludes_alias_surfaces`).

Not yet present in FFEDIT (deferred, correctly): UNDO/REDO (Req 10.6 -- UNDO is a
keyboard-only inline handler, REDO does not exist); COPY/CUT/PASTE/SELECT ALL
(CR-CH-054 Req 12, specified but not in `ffedit_english()` on `main`). When these
land they belong to FFEDIT -- none are candidates for FFCMD.

---

## 5. MISPLACED COMMANDS (the shortlist, with justification)

### 5.1 AUTONUM -- MISPLACED (FFCMD -> FFEDIT). Owner's example, confirmed.

- **Current placement**: FFCMD. `commands_ladder_c.rs` AUTONUM arm handles
  `AUTONUM ON`/`AUTONUM OFF` and redirects to `NUMBER ON/OFF`; `AUTONUM` is listed
  in `BUILTIN_FFCMD_LADDER_VERB_ROOTS` (`command_config/mod.rs`).
- **What it does**: toggles **automatic line renumbering** -- when ON, inserting a
  line assigns the next sequence number in the buffer's sequence columns
  (`docs/specs/sequence-numbers/requirements.md` Req 6.7; `tasks.md` 10.5/10.6).
- **Semantic test**: it mutates the **content of the open buffer** (the sequence
  columns), and is meaningless with no editor buffer open.
- **Verdict / why FFEDIT**: CR-CH-053 Req 2a.4 ("a verb that acts on the active
  editor buffer ... is owned by FFEDIT") and Req 10 (editor-buffer verb
  ownership). This is a textbook FFEDIT verb. **MISPLACED.**

### 5.2 NUM -- MISPLACED (FFCMD -> FFEDIT).

- **Current placement**: FFCMD (`commands_ladder_c.rs` NUM arm; listed in the
  ladder roots). Redirects to `NUMBER [rest]`.
- **What it does**: alias of NUMBER -- generate/insert sequence numbers into the
  buffer's sequence columns, and NUMBER ON/OFF auto-numbering.
- **Verdict / why FFEDIT**: same reasoning as AUTONUM. NUMBER/UNNUM/NUMBER SHOW
  are all buffer-content operations owned by `ff-sequence-numbers`, bridged to the
  editor buffer via `DocumentAccess`. **MISPLACED.** (Its eventual root NUMBER --
  and sibling UNNUM, NUMBER SHOW -- should likewise be FFEDIT roots when wired.)

### 5.3 Data-level placement bug already shipped: `af` SOEK -> FIND under FFCMD

- `crates/ff-i18n/i18n/af/aliases/FFCMD.toml` declares `SOEK -> FIND` in the
  **FFCMD** catalogue. **FIND is an FFEDIT root** (command-semantics Req 12.1;
  `ffedit_english()`), NOT an FFCMD root (`builtin_ffcmd_roots()` does not contain
  FIND; `environment_registry.rs` test `contains_root_false_for_bogus_or_unknown`
  asserts `!contains_root("FFCMD","FIND")`).
- **Why it is not currently caught**: the per-locale reader uses the 3-field
  `parse_alias_toml` (`alias_reader.rs`) which has **no environment field and no
  `contains_root` check**. The CR-CH-067 `contains_root` validation only runs on
  the **user** store path (`alias_user_overlay.rs::apply_user_rows`,
  `alias_editor.rs::validate_user_alias_rows`). So the shipped mis-placed row
  loads silently and seeds a FIND alias into the **FFCMD** table, where FIND is
  not even a root.
- **Verdict**: MISPLACED DATA. The row should live in an `af/aliases/FFEDIT.toml`
  (SOEK -> FIND under FFEDIT). This is the concrete instance of the owner's
  concern that misplaced commands corrupt the alias/localisation data.

### 5.4 AMBIGUOUS: SUBMIT

- **Current**: FFCMD stub (`commands_ladder_c.rs`).
- **Tradeoff**: ISPF SUBMIT typically submits the dataset/member being edited
  (an editor-context act -> leans FFEDIT or the vision-only FFJES), but it can
  also submit a named dataset from the command line (workbench act -> FFCMD). The
  vision catalogue assigns job submission to **FFJES** (`environments-vision.md`),
  which is not built. **Recommendation**: leave on FFCMD for now (it is a stub),
  and revisit when FFJES is built; do not move it in this pass. Flag as AMBIGUOUS.

### 5.5 AMBIGUOUS (minor): TIME / STATUS

- **TIME** shows date/time/day -- a session utility, not buffer content; leans
  FFCMD. Low stakes. **STATUS** is a JES-routing stub -> ultimately FFJES, FFCMD
  for now. Neither needs moving in this pass; recorded for completeness.

### 5.6 Scanned and found CORRECT (no FFCMD-ish verb mislabelled FFEDIT)

Every verb in `ffedit_english()` acts on the buffer -- none is a workbench verb
hiding in FFEDIT. The only FFEDIT-vs-FFCMD naming subtlety is the **EDIT** root:
`code` places EDIT in FFCMD (file-open = workbench), while the user-alias enum
doc-comment (`user_aliases.rs:26-28`) says "FFEDIT is chosen for an editor-verb
root such as EDIT (ECOB -> EDIT is an FFEDIT alias)". The **code and tests are
authoritative** (`alias_editor.rs` test `validate_accepts_root_valid_in_declared_environment`
accepts `ECOB -> EDIT` under **FFCMD**). The comment is stale/misleading and
should be corrected to say FFCMD. Flagged as a doc nit, not a misplacement.

---

## 6. Where code disagrees with the spec (cross-checks)

- **AUTONUM/NUM**: code places them on the FFCMD ladder; spec (CR-CH-053 Req 2a.4
  / Req 10, and the `ff-sequence-numbers` design) implies FFEDIT. **Disagreement
  -> the relocation this audit recommends.**
- **FIND / SEARCH**: code agrees with spec. FIND is FFEDIT (`ffedit_english`),
  SEARCH/GSEARCH is FFCMD (`builtin_workspace_target_for`), matching
  command-semantics Req 12.1/12.2. GREP (the intended SEARCH alias surface,
  Req 12.3) is **not yet seeded** anywhere -- a gap, not a misplacement.
- **`af` SOEK -> FIND under FFCMD**: data disagrees with spec (FIND is FFEDIT).
  See 5.3.
- **`user_aliases.rs` EDIT comment**: comment disagrees with code/tests (says
  FFEDIT; code uses FFCMD). Doc nit (5.6).
- **EnvironmentRegistry enumeration**: `root_commands` correctly returns non-empty
  only for FFCMD/FFEDIT and empty for FFNAV/FFLINE/HOSTFS/MAINFRAME
  (`environment_registry.rs` tests). Consistent with the spec's phase-1 scope.

---

## 7. The `=FILES` root-token bug (noted, not primary scope)

`=FILES` is registered as a literal root token in `BUILTIN_WORKSPACE_VERB_ROOTS`
(`command_config/mod.rs`) and classified by `builtin_workspace_target_for`
(`is("=FILES")`). `=` is the ISPF jump prefix (the front-door `=` step,
`dispatch.rs::dispatch_command_string`, drops the nav stack to the POM then runs
the remainder). The real root is **FILES**; `=FILES` should resolve as "`=` then
FILES", not as a distinct root. This pollutes the FFCMD root set that the alias
anchoring enumerates (`root_commands("FFCMD")` includes `=FILES`), so it is
adjacent to this audit's concern but is an **enumeration/root-shape bug**, not an
environment-placement bug. **Primary scope here is placement; fix `=FILES`
separately** (its own small change request), ideally after the placement fix so
the root set is corrected once.

---

## 8. PLAN OF ACTION

### 8.1 Is this a behaviour change or a classification change?

Moving AUTONUM/NUM from FFCMD to FFEDIT is **mostly a classification change with a
small behaviour consequence**:

- **Classification**: AUTONUM/NUM/NUMBER move out of `BUILTIN_FFCMD_LADDER_VERB_ROOTS`
  and become FFEDIT roots (added to `ffedit_english()` or a sequence-numbers
  sub-table). This changes `EnvironmentRegistry::root_commands("FFCMD")` (removes
  them) and `root_commands("FFEDIT")` (adds them) -- which is exactly the anchor
  the Afrikaans aliases and the Alias Editor validation read.
- **Behaviour**: today AUTONUM/NUM are reachable in ANY context (FFCMD ladder is
  global). After the move they would be claimed only when an **editor Context is
  active** (active-wins, Req 5.1), and rejected elsewhere. That is an observable
  change (a non-editor context would no longer "handle" AUTONUM). Because NUMBER
  is currently unimplemented (falls through to an engine error), the *effect*
  today is already "nothing useful happens", so the behaviour risk is low -- but
  it is still a behaviour/semantics change.
- **Therefore this NEEDS the requirements gate** (per `workflow.md` and
  `framework-conformance.md`: changing a command's environment is a behaviour /
  semantics change). Do not edit source before the gate is complete.

### 8.2 Blast radius per relocation

For **AUTONUM / NUM (and the eventual NUMBER/UNNUM/NUMBER SHOW)**:

1. **Dispatch**: the AUTONUM/NUM arms in `commands_ladder_c.rs` would move to the
   FFEDIT claim path (`environment_ffedit.rs` / the `FfEditEnvironment` claim),
   claimed only when an editor Context is active.
2. **Root enumeration (CR-CH-067)**: remove AUTONUM/NUM from
   `BUILTIN_FFCMD_LADDER_VERB_ROOTS` (`command_config/mod.rs`); add the canonical
   root(s) to `ffedit_english()` (`environment.rs`). This flips `root_commands`
   for both environments.
3. **Alias fixtures / tests that assume current placement**:
   - `environment_registry.rs` tests enumerate expected FFCMD/FFEDIT roots -- the
     expected sets must be updated (no AUTONUM-specific assertion exists today,
     but the FFCMD "complete root set" test would need review).
   - `command_config/mod.rs` test `builtin_ffcmd_ladder_verb_roots_are_disjoint_from_classifier_and_carry_file_open`
     iterates the ladder list -- unaffected by removing AUTONUM/NUM (they are not
     classifier verbs), but the list contents change.
   - `alias_editor.rs` / `alias_user_overlay.rs`: any user alias whose row is
     `(FFCMD, AUTONUM/NUM)` would newly be **rejected** by `contains_root`; and a
     `(FFEDIT, NUMBER)` alias would newly be **accepted**. No shipped fixture uses
     these today, so the user-data blast radius is nil on `main`.
4. **FIND-vs-SEARCH model**: unaffected (AUTONUM/NUM are not search verbs).
5. **The `af` data fix (5.3)** is independent of AUTONUM but is the same *class*
   of fix and should ride along in the same gate: move SOEK -> FIND from
   `af/aliases/FFCMD.toml` to `af/aliases/FFEDIT.toml`. Blast radius: the POC test
   `af_poc_alias_catalogue_loads_and_resolves` in `alias_reader.rs` asserts the
   row is read from the **FFCMD** file -- that test must move to FFEDIT too.

### 8.3 Sequencing and sizing

Independent vs coupled:

- **(A) AUTONUM/NUM relocation** (FFCMD ladder -> FFEDIT) -- **medium**. Couples
  dispatch + root enumeration + tests. Best done alongside wiring the real
  `ff-sequence-numbers` NUMBER/UNNUM engine, OR as a pure classification move that
  leaves NUMBER unimplemented (smaller, but leaves a dangling redirect). Recommend
  the pure-classification move now (small-medium), wiring the engine later.
- **(B) `af` SOEK->FIND data fix** (FFCMD.toml -> FFEDIT.toml + move its test) --
  **small**. Independent of (A) but same class; bundle it.
- **(C) `user_aliases.rs` EDIT doc-comment correction** -- **trivial** (doc only,
  no gate needed as no behaviour changes). Can ride along or be done anytime.
- **(D) `=FILES` root-token fix** -- **small-medium**, separate change request,
  **after** the placement fix (so the FFCMD root set is corrected once). Out of
  this audit's primary scope.

Coupling: (A) and (B) both change the FFCMD/FFEDIT root sets that the alias
anchoring reads, so they should land **together, before** the Afrikaans
alias-data authoring and the `=FILES` fix.

### 8.4 One gated change request or several?

**Recommend ONE gated change request** covering (A) + (B) + (C): "Correct editor
command placement -- move AUTONUM/NUM to FFEDIT and relocate the mis-placed `af`
FIND alias", because they are one coherent semantic correction to the
(environment, root) model and share the same test/enumeration blast radius.
Keep **(D) `=FILES`** as a **separate** change request (different bug class:
root-token shape, not environment placement).

### 8.5 Order relative to the pending work

Do the placement correction (one gated CR) **BEFORE**:

1. the Afrikaans FFCMD alias-data authoring (owner message index 10) -- so the
   authored aliases anchor to the corrected root sets and `contains_root` does not
   silently misroute or reject them; and
2. the `=FILES` enumeration fix -- so the FFCMD root set is corrected once, not
   twice.

Rationale: placement feeds both. Authoring Afrikaans aliases against today's
(wrong) root sets would bake the misplacement into shipped localisation data --
precisely the corruption the owner wants to avoid.

---

## 9. Interaction with CR-CH-067 and the unmerged CR-CH-068

- **CR-CH-067 (merged, in HEAD `c73f41d`)**: added `EnvironmentRegistry::root_commands`
  / `contains_root` and the reject-invalid-alias-root hooks at load
  (`alias_user_overlay.rs`, skip-with-WARN, Req 20.7) and save
  (`alias_editor.rs::validate_user_alias_rows`, Req 20.8). **Any placement move
  directly changes what these two functions return**, so the relocation MUST
  update `builtin_ffcmd_roots()` inputs and `ffedit_english()` together, and
  re-baseline the `environment_registry.rs` enumeration tests. The validation
  hooks are the safety net that will start rejecting a now-wrong `(FFCMD, AUTONUM)`
  user alias -- intended, but call it out in the CR so it is not read as a
  regression.

- **CR-CH-068 (UNMERGED -- the Alias Editor ISPF redesign)**: a live worktree
  exists at `c:\workspace\apps\FileForgeWorkbench\.worktrees\crch068-alias-editor-ispf`
  (branch `crch068-alias-editor-ispf`; its `target\debug\...\af\aliases\FFCMD.toml`
  shows it carries the same POC data). That branch redesigns the **Alias Editor
  UI** -- the exact surface that reads `root_commands`/`contains_root` to populate
  the CE/Root dropdowns and validate rows. **Sequencing risk**: if the placement
  CR lands on `main` after CR-CH-068 forks, the two will both touch
  `alias_editor.rs` / `environment_registry.rs` / the enumeration tests and will
  conflict. **Recommendation**: land the small placement CR (sec. 8.4) **first on
  `main`**, then rebase CR-CH-068 onto it, so the Alias Editor redesign builds on
  the corrected (environment, root) sets rather than the current mis-placed ones.
  The `AliasEnvironment` enum (`user_aliases.rs`) currently offers only FFCMD /
  FFEDIT, which already matches the two enumerable environments -- no enum change
  is needed for the AUTONUM move (AUTONUM/NUMBER simply become selectable FFEDIT
  roots once added to `ffedit_english()`).

---

## 10. Evidence index (primary file:symbol citations)

- FFCMD root union: `crates/ff-desktop/src/shell/alias_reader.rs:131` `builtin_ffcmd_roots()`.
- `resolve_target` classifier + root list: `crates/ff-desktop/src/command_config/mod.rs`
  `BUILTIN_WORKSPACE_VERB_ROOTS` (`~167`), `BUILTIN_FFCMD_LADDER_VERB_ROOTS` (`~190`),
  `builtin_workspace_target_for` (`~206`).
- FFCMD ladders: `commands_ladder_a.rs` (EXIT family, EDIT, START, MENU, CLOSE,
  DOCK, PFSHOW, END, RETURN, X, HELP), `commands_ladder_b.rs` (COMMAND, SNAPSHOT,
  CAPTURE, RESET BARE, RETRIEVE), `commands_ladder_c.rs` (DETACH, SPLIT, UNSPLIT,
  FOCUS, SWAP, **AUTONUM ~128**, **NUM ~140**, SUBMIT, TIME, STATUS, CREATE,
  REPLACE, BROWSE, VIEW, COMPARE, WORKSPACE).
- Built-in menus: `crates/ff-desktop/src/menu_workspace/defaults.rs`
  `DEFAULT_POM_TOML` / `DEFAULT_SETTINGS_TOML`.
- FFEDIT roots: `crates/ff-desktop/src/shell/environment.rs:118` `ffedit_english()`.
- Active-env derivation + claim gate + enumeration: `crates/ff-desktop/src/shell/environment_registry.rs`
  (`active_name`, `is_ffedit_active`, `root_commands`, `contains_root`).
- Dispatch front door + ladder order: `crates/ff-desktop/src/shell/dispatch.rs`
  `dispatch_command_string`; `crates/ff-desktop/src/shell/commands.rs`
  `run_command_ladder` (FFEDIT active-wins at `~150`, CommandEngine terminal at `~320`).
- Alias load/save validation (CR-CH-067 hooks): `crates/ff-desktop/src/shell/alias_reader/alias_user_overlay.rs`
  `apply_user_rows`; `crates/ff-desktop/src/shell/alias_editor.rs` `validate_user_alias_rows`.
- User alias environment enum: `crates/ff-desktop/src/shell/user_aliases.rs`
  `AliasEnvironment` (FFCMD/FFEDIT; EDIT doc-comment nit at `~26-28`).
- Mis-placed `af` data: `crates/ff-i18n/i18n/af/aliases/FFCMD.toml` (`SOEK -> FIND`).
- Specs: `docs/specs/command-environments/requirements.md` (Req 2a, 6, 10, 20);
  `docs/specs/command-semantics/requirements.md` (Req 12 FIND/SEARCH, Req 13);
  `docs/specs/sequence-numbers/{requirements,tasks}.md` (NUMBER/UNNUM/auto-number
  engine in `ff-sequence-numbers`).
- Unmerged redesign: worktree `.worktrees/crch068-alias-editor-ispf`.

_Content from upstream specs was rephrased for compliance with licensing restrictions._
