# Analysis Record: plugin-manager-ui (W4.8)

- **Wave**: 4 (UI, panels, layout)
- **Backing code**: NO dedicated crate -- `ff-desktop/src/plugin_manager_panel.rs`
  (191 non-test). The Plugin Manager Context (a Workspace listing/enabling/
  disabling/configuring installed plugins). Reads the `ff-plugin` PluginRegistry.
- **Spec files**: requirements.md (113 lines, 4 requirements -- `## Requirement`
  headings), tasks.md (24 sub-tasks, **18 done / 6 OPEN**), design.md present
- **Analysed**: Wave 4 pass

---

## 1. Split candidacy

N/A / NOT a split candidate. Small: 4 reqs, 113 lines, one file (191 non-test,
under cap). One cohesive concern (plugin manager panel). No split.

---

## 2. Completeness -- PA-INCOMPLETE-012 (Enable/Disable + config display UNBUILT)

18/24 tasks done, **6 OPEN**. BUILT vs UNBUILT:

BUILT:
- Req 1 (Plugin Manager Context): the panel lists installed plugins (reads
  ff-plugin PluginRegistry -- 92 ff-plugin refs).
- Req 3 (Plugin Details View): basic details render.

UNBUILT (6 open tasks):
- Req 2 (Enable and Disable Plugins), Task 3.1-3.5: the panel doc-comment CLAIMS
  "enable/disable controls" but the actual Disable/Enable BUTTONS + `deactivate()`/
  `activate()` calls are NOT present (grep found only the comment, no button/call
  code). Also: activation-failure error display (3.3), persistence of disabled
  plugin names in session TOML (3.4), tests (3.5). The ff-plugin registry DOES
  expose activate/deactivate/enable/disable (9 methods, W0.13) -- so the API is
  READY; the UI just hasn't wired the buttons.
- Req 3/4, Task 4.3: render the config-keys section (key name / current value /
  edit link) in the details view.

So the plugin LISTING is built but the core ENABLE/DISABLE ACTIONS (Req 2, the
whole point of a plugin manager) + config-key display are UNBUILT. Honest `[ ]`
(the code genuinely lacks the buttons/calls -- NOT a false-positive, contrast the
doc-comment which overstates). Recorded PA-INCOMPLETE-012 (MEDIUM): wire the
Enable/Disable buttons to ff-plugin activate()/deactivate(), add activation-failure
display + disabled-plugin session persistence + config-keys display + tests. The
registry API is ready; this is UI wiring. A plugin manager that cannot
enable/disable is a notable functional gap.

Req 1 + Req 4 (session persistence contract) partially built (listing + persist
scaffold); Req 2 is the substantive gap.

---

## 3. Cross-unit consistency

### ff-plugin registry -- consumed correctly (W0.13)

The panel consumes the `ff-plugin` PluginRegistry (92 refs) for listing + the
enable/disable API (activate/deactivate, 9 methods in ff-plugin). Correct
consumer relationship -- the UI reads/drives the registry; ff-plugin (W0.13) owns
plugin lifecycle. No duplication. The enable/disable WIRING is the unbuilt part
(PA-INCOMPLETE-012).

### Session persistence -- ff-session contract

Req 4 + Task 3.4: disabled plugin names persist in the session TOML (ff-session,
W3.9). Consistent with the WorkspaceDescriptor/session-state model. The persistence
of disabled-plugins is part of the unbuilt Task 3.4.

### Plugin config keys (Task 4.3) -- ff-config namespace

Req 3 config-keys display reads plugin config from the `[plugins.{name}]` namespace
(configuration-system W0.3 + plugin-architecture W0.13 Req 2.7/7.5 -- the plugin
config scoping, confirmed consistent in W0.3/W0.13). The DISPLAY is unbuilt (4.3);
the config model exists. Consistent.

### Public types and ownership

- Plugin manager panel state, list/detail render, (pending) enable/disable actions
  + config display -- sole-owned by the ff-desktop plugin_manager_panel. No
  duplication. 0 fs (reads the registry; persistence via ff-session contract).

### Cross-reference integrity

Cross-refs (plugin-architecture ff-plugin, configuration-system, startup-and-session)
resolve. No dangling refs.

---

## 4. Logging audit

Scan of `ff-desktop/src/plugin_manager_panel.rs`:

- `ff_logging` / `log_*!`: 0
- `println!` / `eprintln!`: 0
- `std::fs`: 0
- non-ASCII: 0 (clean)

The module has no ff-logging dep (UI panel). Once enable/disable is wired
(PA-INCOMPLETE-012), plugin activation FAILURE (Task 3.3) is a natural WARN/ERROR
log site -- but plugin lifecycle logging belongs to ff-plugin (W0.13), which the UI
drives. Recorded PA-LOG-034 (LOW): on activation failure, ensure ff-plugin logs it
(W0.13) + the panel shows the error; add dev-logging on enable/disable actions once
built. Deferred to PA-INCOMPLETE-012.

---

## 5. Task revision proposals

- **PA-INCOMPLETE-012 (MEDIUM)**: wire Req 2 Enable/Disable (Task 3.1-3.5:
  Disable/Enable buttons -> ff-plugin activate()/deactivate(); activation-failure
  display; disabled-plugin session persistence; tests) + Task 4.3 config-keys
  display. The ff-plugin API is ready; this is UI wiring. A plugin manager that
  can't enable/disable is a functional gap. Also correct the panel doc-comment,
  which claims enable/disable controls that don't exist yet.
- **PA-LOG-034 (LOW)**: on activation failure, ff-plugin logs (W0.13) + panel shows
  error; dev-logging on enable/disable once built. Deferred with PA-INCOMPLETE-012.
- **PA-TCR (none)**: 9 TCR rows for 4 reqs -- adequate; Req 2/config rows come with
  PA-INCOMPLETE-012.

No PA-STD (191 non-test under cap, 0 non-ASCII). No conflict. No requirement CHANGE;
Req 2 is correctly-gated pending UI wiring.

---

## Summary

plugin-manager-ui is a small Plugin Manager Context panel in `ff-desktop`
(plugin_manager_panel.rs, 191 non-test, clean -- 0 fs/ASCII, no cap, adequate TCR
9). It correctly consumes the ff-plugin PluginRegistry (W0.13) for listing (92
refs). The finding is PA-INCOMPLETE-012 (MEDIUM): the plugin LISTING + basic
details are built, but the core ENABLE/DISABLE ACTIONS (Req 2 -- the buttons +
ff-plugin activate()/deactivate() calls + activation-failure display + disabled-
plugin session persistence) and the config-keys display (Task 4.3) are UNBUILT (6
honest `[ ]` tasks; the panel doc-comment overstates by claiming enable/disable
controls that don't exist). The ff-plugin API is ready (9 lifecycle methods) -- this
is UI wiring. A plugin manager that cannot enable/disable is a notable functional
gap. No conflict, no split, no size/ASCII issue.
