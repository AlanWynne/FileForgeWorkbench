# Design Review -- Extension Integration Specification

Reviewer: design-review subagent (fresh context, no authoring history).
Scope: DOCUMENTATION review only. No code or doc edits were made.

Package reviewed:
- `docs/specs/extension-integration/requirements.md`
- `docs/specs/extension-integration/design.md`
- `docs/specs/extension-integration/tasks.md`
- `.agents/tasks/extension-system-consolidation/spec-index.md` (pointer)
- `docs/project-management/ROADMAP.md` (KEYSTONE PREREQUISITE note)
- Source of truth: `.agents/tasks/extension-system-consolidation/consolidation.md`

Steering checked against: `framework-conformance.md`, `wiring-standard.md`,
`specs.md`, `documentation-charset.md`, `task-format.md`,
`extension-terminology.md`.

---

## Verdict

CHANGES_REQUESTED.

Mechanical count: HIGH = 0, MEDIUM = 2, NIT = 4. Any HIGH or MEDIUM blocks, so
the package needs one revision pass before the owner's final read. The two
MEDIUM findings are both small, well-scoped gaps (an undecided POM-entry
decision that is cross-referenced but never made, and an unstated
precondition/owner-action for the `extensions.disabled` session key). Neither
undermines the architecture, which is sound and strongly framework-conformant.

---

## Findings

### 1. MEDIUM -- Req 8.1 defers the Extension Manager POM entry to a decision that is never made

Requirement 8.1 (requirements.md) says the Extension Manager command opens the
Context "(the renamed `EXTENSIONS` verb; see design D1/D2 for the POM entry
decision)". But neither D1 (rename) nor D2 (Catalogs/Files) decides whether the
Extension Manager itself gets a POM menu option, a menu-bar entry, or is
reachable only by typing `EXTENSIONS`. The consolidation report (section 4)
explicitly flagged this as an open discrepancy to resolve: the `plugin-manager-ui`
spec assumes "POM option 8 / `=8`", but the live `DEFAULT_POM_TOML` has no such
option, and after CR-CH-073 the POM is trimmed to `0` Settings / `3` Help / `X`.
The design (section "The Extension Manager Context" and D1) renames the verb/tag
but is silent on the POM/menu placement. So a reader cannot tell how an end user
reaches the Extension Manager in the bare core.

Where: requirements.md Req 8.1; design.md "The Extension Manager Context", D1,
D2.

Concrete fix: add one explicit decision. Recommended wording (either as a new
D-point or folded into D1):
"The Extension Manager is reachable by typing `EXTENSIONS` (classified by
`resolve_target` as `CustomWorkspace(\"plugin_manager\"/\"extension_manager\")`,
the existing path). It is NOT added to the compiled bare-core `DEFAULT_POM_TOML`;
a POM option for it, if wanted, is contributed the same way any other menu option
is (an in-memory overlay, Req 4) or added to a user `menus/pom.toml`. The stale
`plugin-manager-ui` 'POM option 8 / =8' wording is superseded." Then change Req
8.1 to reference that decision instead of the non-existent "D1/D2 POM entry
decision".

### 2. MEDIUM -- The `extensions.disabled` session-config key is used as if it exists; its provenance and any required owner action are unstated

Req 2.2, 2.3, 2.4 and design "The extension host" treat
`extensions.disabled: Vec<String>` as the persisted disabled set, and the design
notes it is the renamed form of `plugins.disabled` (plugin-manager-ui Req 2).
But nothing in the spec states (a) whether this key already exists in the live
configuration schema or must be ADDED, (b) that adding/renaming a persisted
config key is itself a change the configuration-system owns, or (c) the
read/default behaviour when the key is absent (first run, or a session saved
before the key existed). An implementer cannot tell whether this is a
pre-existing field to read or a new schema addition to design, and a missing-key
default is unspecified error handling.

Where: requirements.md Req 2.2/2.3/2.4; design.md "The extension host", "Config"
row of the technology stack.

Concrete fix: add an acceptance criterion and a design sentence. Suggested AC
(append to Req 2): "WHEN the session configuration has no `extensions.disabled`
key (first run, or a pre-existing session), THE host SHALL treat the disabled set
as empty and activate all discovered extensions." And in design, state plainly
whether `extensions.disabled` is a NEW session-config key introduced by this
contract (if so, note it is an additive, backward-compatible key owned by the
configuration system, not a framework-type change) or the rename of an existing
`plugins.disabled` key, and under which wave (D1) the rename lands.

### 3. NIT -- Req 10.3/Req 11.5 state the runtime-extensible registry framework change is owner-confirmed, while Req 11.8 and task 5.6 still carry a "if the owner declines" fallback

The spec-index and design D5.3 record D5 as owner-confirmed, and Req 10.3b states
both additive framework extension points are "owner-confirmed before
implementation". Yet Req 11.8 and task 5.6 retain a narrower fallback "IF the
owner DECLINES the runtime-extensible registry". This is defensive and clearly
labelled, so it is not contradictory, but a fresh reader may be unsure whether
the decision is settled. 

Where: requirements.md Req 10.3, Req 11.5, Req 11.8; tasks.md task 5.0 vs 5.6.

Concrete fix: add one clause to Req 11.8 (and task 5.6) marking it explicitly
conditional/contingency, e.g. "Req 11.8 is a RECORDED FALLBACK retained only
should the owner reverse the D5.3 confirmation; under the current owner-confirmed
position (Req 11.5) it does not apply." Keeps the fallback documented without
reading as an undecided fork.

### 4. NIT -- Task 5 begins at sub-task 5.0 (a framework-change step) while every other top-level task starts at .1

tasks.md task 5 lists 5.0 then 5.1-5.6. The `[ ]` markers and titles are all
task-format compliant, so this is purely a numbering-style inconsistency; 5.0 is
used to foreground the owner-confirmed FRAMEWORK CHANGE before the dependent
sub-tasks. 

Where: tasks.md task 5.0.

Concrete fix (optional): renumber 5.0 -> 5.1 and shift the rest, OR add a short
parenthetical on 5.0 noting it is sequenced first deliberately because 5.1-5.6
depend on it. Not blocking.

### 5. NIT -- "POM option 8 / =8" stale wording from plugin-manager-ui is not explicitly retired in the consolidating spec

Req 8 says this spec "CONSOLIDATES and supersedes" the integration-contract
concerns of `plugin-architecture` and `plugin-manager-ui`, but the specific stale
claim (plugin-manager-ui Req 1: "POM option 8 / `=8`") is never called out as
retired. Tied to finding 1; worth an explicit "superseded" note so the two source
specs are not later read as still authoritative on the POM entry.

Where: requirements.md Introduction / Req 8.

Concrete fix: in the Introduction's "consolidates and supersedes" paragraph, add:
"In particular, the `plugin-manager-ui` 'POM option 8 / =8' reachability claim is
superseded by Req 8.1 (see finding-1 decision)."

### 6. NIT -- Design D2's "just LANDED its gate (Option B)" is now stale relative to code; reads as in-flight

design.md D2 and the spec-index describe the Catalogs/Files removal as a separate
in-flight effort that "has just LANDED its gate with OPTION B". The code already
reflects the completed state: `DEFAULT_POM_TOML` (CR-CH-073) is trimmed to
`0`/`3`/`X` with Catalogs/Files menu options removed and the `CATALOGS`/`FILES`
commands + Contexts kept typeable -- exactly Option B. The spec's requirements
(Req 9) are consistent with this end state, so this is only a tense/currency nit
in the prose, not a correctness problem.

Where: design.md D2; requirements.md A2.

Concrete fix (optional): change "has just LANDED" to "has LANDED (CR-CH-073: the
bare-core POM is `0` Settings / `3` Help / `X`; `CATALOGS`/`FILES` remain
typeable)" so the design matches the committed code state.

---

## Verified Assumptions

Each claim below was checked against the actual source, not taken on the spec's
word.

1. VERIFIED -- `EnvironmentRegistry::root_commands(env_name)` and
   `contains_root(env_name, root)` are ASSOCIATED functions over compiled
   constants (no instance registry state).
   `crates/ff-desktop/src/shell/environment_registry.rs:234,265`. This grounds
   D5.3's claim that making them runtime-extensible is a load-bearing signature
   change.
2. VERIFIED -- FFCMD roots are `builtin_ffcmd_roots()` =
   `BUILTIN_WORKSPACE_VERB_ROOTS` unioned with `BUILTIN_FFCMD_LADDER_VERB_ROOTS`
   (environment_registry.rs:239; command_config/mod.rs:173,218;
   alias_reader.rs:177,186). Matches design D5.1 exactly.
3. VERIFIED -- `EDIT` is a member of `BUILTIN_FFCMD_LADDER_VERB_ROOTS` and is NOT
   claimed by the classifier (command_config/mod.rs:218-221, and the drift test
   `builtin_ffcmd_ladder_verb_roots_are_disjoint_from_classifier_and_carry_file_open`).
   Confirms D5.5's statement that the `EDIT` re-home is a no-op over the
   built-ins.
4. VERIFIED -- `PLUGINS` is in `BUILTIN_WORKSPACE_VERB_ROOTS` and
   `builtin_workspace_target_for` classifies `PLUGINS` -> `custom("plugin_manager")`
   (command_config/mod.rs:182,349). Confirms Req 8.1's "existing `PLUGINS` ->
   `plugin_manager` path".
5. VERIFIED -- the alias layer rejects a `canonical` that is not a valid root in
   its environment via `validate_user_alias_rows` (shell/alias_editor.rs:264,
   with tests `validate_rejects_root_invalid_in_declared_environment`) and
   `revalidate_root_for_env` (alias_editor_panel/rows.rs:68). Confirms D5.1/Req
   11.2 that an alias cannot create a capability, only target an existing root.
6. VERIFIED -- the classifier/list drift guard
   `builtin_workspace_verb_roots_list_matches_the_classifier` exists
   (command_config/mod.rs:589). Confirms D5.3's "drift guard stays valid" claim
   and the testability note.
7. VERIFIED -- the five `ff-plugin` service traits named in the design/Req 1.2
   (`CommandRegistration`, `PluginConfigAccess`, `PluginVfsAccess`,
   `PluginEventBus`, `CapabilityRegistrar`) exist with those exact names
   (crates/ff-plugin/src/traits.rs:88,99,110,125,138).
8. VERIFIED -- `DEFAULT_POM_TOML` is trimmed to `0` Settings / `3` Help / `X`
   with Catalogs/Files menu options removed and the commands kept typeable
   (CR-CH-073) -- the Option B end state (menu_workspace/defaults.rs). The design
   D2/Req 9 end state matches the committed code.
9. VERIFIED -- the ROADMAP KEYSTONE PREREQUISITE note pointing to
   `docs/specs/extension-integration/` is present
   (docs/project-management/ROADMAP.md:102).
10. VERIFIED -- all three spec files are ASCII-only (no match for `[^\x00-\x7F]`),
    satisfying documentation-charset.md.
11. VERIFIED -- tasks.md uses only `[ ]` markers, every task line has a title,
    and `(Req X)` cross-references are present throughout, satisfying
    task-format.md (the 5.0 numbering is a style nit, finding 4, not a format
    violation).

## Unverified / Wrong Assumptions

1. STALE (not wrong) -- the CONSOLIDATION REPORT (section 4 and section 8) states
   the current baseline POM is `0` Settings, `1` Catalogs, `2` Files, `3` Help,
   `X`. The live code (CR-CH-073, menu_workspace/defaults.rs) has already removed
   Catalogs/Files, leaving `0`/`3`/`X`. This is a staleness in the INPUT report,
   not in the spec under review; the spec's Req 9 and design D2 correctly describe
   the Option B end state. Noted so the owner is not misled by the older report.
2. UNVERIFIED -- the `extensions.disabled` (formerly `plugins.disabled`) session
   config key: I could not confirm from the spec whether it is an existing schema
   field or a new one, and the configuration-system schema was not in scope to
   read exhaustively. This is the substance of finding 2; the spec must state its
   provenance and absent-key default rather than leave it implied.
3. NOT INDEPENDENTLY RE-DERIVED -- the Rust-ABI / `libloading` instability
   tradeoff underpinning D3's "in-tree feature-gated" recommendation is an
   accepted general fact about dynamic Rust plugin loading; it is stated
   correctly and the recommendation follows from it. No code check applies.

---

## Framework-conformance assessment (explicit, per the review brief)

- Command dispatch: PASS. The contract routes every extension command through the
  single `resolve_target` / `dispatch_command_target` front door and explicitly
  rejects a bespoke `if upper == "..."` intercept or a second dispatcher (Req 3.1,
  3.2; design "Command contribution"). The runtime-extensible root registry is
  correctly identified as the "runtime arm of the one classifier", not a parallel
  table.
- Menus: PASS. In-memory overlay merged with code-only `DEFAULT_*_TOML`; no
  default menu written to disk (Req 4.1-4.3; mechanism 4).
- Focus latch / Context: PASS. Single `render_workspace_context` /
  `apply_interior_focus` path; the one additive `TabKind::Extension { id }` arm is
  correctly flagged as an owner-confirmed framework extension point (Req 5.1-5.4,
  10.3a; mechanism 5; workspace-conformance).
- Navigation / Kinds / persistence: PASS. `navigate_to`, Kinds registry,
  `WorkspaceDescriptor` reused as-is (Req 6; mechanisms 3, 6, 7).
- Framework changes flagged: PASS, and done well. BOTH additive extension points
  (identifier-keyed extension-Context arm; runtime-extensible root registry +
  classifier) are explicitly called out as owner-confirmed FRAMEWORK CHANGES with
  blast radius (Req 10.3, Req 11.5, design D5.3), per framework-conformance.md.
  Nothing appears to silently change a core seam.

Decision points D1-D4 (plus the added D5) are all present, clearly stated, and
internally consistent, EXCEPT for the one undecided sub-point captured in finding
1 (the Extension Manager's own POM entry, cross-referenced to D1/D2 but decided in
neither).
