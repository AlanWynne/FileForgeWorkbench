# Analysis Record: plugin-architecture

- **Wave**: 0 (foundation -- all optional features are plugins using this API)
- **Backing crate**: `ff-plugin`
- **Spec folder**: `docs/specs/plugin-architecture/`
- **Analysed**: Wave 0, task W0.13 (CR-NR-057 re-baseline)
- **Verdict**: COMPLETE (169/169 tasks `[x]`, TCR PASS). NOT a split candidate.
  Logging EXEMPLARY. One 400-cap refactor (registry.rs 698).
- **CR-NR-057 impact**: NONE (plugin specs untouched). Re-verified from code.

---

## 1. Scope summary

`ff-plugin` is the trait-based extensibility foundation. 7 requirements:
Req 1 FileForgePlugin trait (lifecycle, object-safe, PluginError); Req 2
PluginContext (sole interface to platform services, namespace-scoped config,
Send+Sync); Req 3 registration + loading (discover->load->validate->initialize->
activate; dependency graph; topological order; hot-reload opt-in); Req 4
capability discovery; Req 5 lifecycle management (state machine; panic isolation
via catch_unwind); Req 6 versioning/compatibility (PLUGIN_API_VERSION semver);
Req 7 security/sandboxing (API-level boundaries, VFS-only FS, no cross-plugin
state).

149 req lines, 7 requirements, single backing crate. Small, cohesive.

## 2. Split candidacy

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 149 lines; 7 reqs | No |
| 3+ distinct responsibilities | trait/context/registry/capability/lifecycle/version/security are facets of ONE plugin system | No |
| 2+ crates | one crate | No |
| low-cohesion clusters | high cohesion; 14-file per-concern split is clean | No |
| file-size pressure | `registry.rs` 698 non-test over cap | Yes (1) |

0-1 criteria (only registry.rs size). **NOT a split candidate.** Fix is the
refactor (PA-STD-004).

## 3. Consistency / conflict

- Public types owned here (FileForgePlugin, PluginContext, PluginError,
  PluginMetadata, Capability, CapabilityRegistry, PluginState, PluginRegistry,
  PLUGIN_API_VERSION) -- sole owner. Added to consistency-matrix.
- PluginContext aggregates foundation services -- delegates to ff-logging
  (PluginLogHandle), ff-command (registration), ff-config (namespace-scoped reads,
  matching configuration-system Req 8 W0.3), ff-vfs (Req 7.2 file access). All
  correct consumer relationships. No conflict.
- `[plugins.{name}]` scoping consistent with configuration-system Req 8;
  PluginLogHandle `[plugin:name]` prefix consistent with logging-subsystem Req 10.
  No conflict.

## 4. Completeness

- Tasks: 169 `[x]`, 0 `[ ]`. Fully tracked complete.
- TCR: `ff-plugin` row PASS (plugin lifecycle, registry, dependency ordering).
- Panic isolation (Req 5.3) via catch_unwind, logs ERROR + transitions to
  Shutdown. Dependency graph / topological order / circular-dependency rejection
  present.
- Completeness verdict: **COMPLETE.**

## 5. Logging audit -- EXEMPLARY

- 24 log call sites via `ff_logging::log(LogLevel::X, ...)`; `ff-logging` declared
  and used. Every mandated ERROR/WARN path present at the correct level:
  - Req 3.4 circular dependency -> Error; Req 3.7/6.3 unmet dep / incompatible API
    -> Error; Req 5.3 lifecycle panic -> Error; Req 5.4 lifecycle error -> Warn
    (8 Error/Warn sites in registry.rs). PASS.
- No `println!`/`eprintln!`. GUI-independence upheld.
- Reference exemplar (with ff-core) for the ff-vfs logging fix (PA-LOG-004).
- Logging verdict: **exemplary / fully adequate.**

## 6. Findings logged

- **PA-STD-004** (REFACTOR -- 400-line cap): `registry.rs` is 698 non-test lines,
  well over the cap. Split by concern (registry_discovery / registry_lifecycle;
  keep registry.rs coordinator + state). REFACTOR, no gate.
- **PA-LOG-REF-001** (positive reference, no action): ff-plugin (with ff-core) is
  the exemplar foundation-crate logging pattern -- cite when fixing PA-LOG-004
  (ff-vfs) and evaluating zero-log crates from PA-LOG-002. No defect.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-plugin contributes 68 matches
  (doc-comment prose, separators). Rolled into project-wide PA-LOG-001.
