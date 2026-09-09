# Analysis Record: plugin-architecture

- **Wave**: 0 (foundation -- all optional features are plugins using this API)
- **Backing crate**: `ff-plugin`
- **Spec folder**: `docs/specs/plugin-architecture/`
- **Analysed**: Wave 0, task W0.13
- **Verdict**: COMPLETE (169/169 tasks `[x]`, Req 1-7 TCR-PASS). NOT a split
  candidate. Logging is EXEMPLARY (contrast ff-vfs). One 400-cap refactor
  (registry.rs 698 lines).

---

## 1. Scope summary

`ff-plugin` is the trait-based extensibility foundation. 7 requirements:

- Req 1 FileForgePlugin trait (lifecycle: initialize/activate/deactivate/
  shutdown; object-safe; PluginError); Req 2 PluginContext (sole interface to
  platform services, namespace-scoped config, Send+Sync); Req 3 registration +
  loading (discover->load->validate->initialize->activate; dependency graph;
  topological order; hot-reload opt-in); Req 4 capability discovery
  (Capability_Registry, typed query, change events); Req 5 lifecycle management
  (state machine; panic isolation via catch_unwind; cleanup guarantees); Req 6
  versioning/compatibility (PLUGIN_API_VERSION semver checks); Req 7 security/
  sandboxing (API-level boundaries, VFS-only FS, capability-based network,
  no cross-plugin state).

149 req lines, 7 requirements, single backing crate. Small, cohesive.

## 2. Split candidacy

Split criteria (design.md section 5) -- flag if 2+ hold:

| Criterion | Finding | Met? |
|-----------|---------|------|
| >~350 req lines OR >12 Reqs | 149 lines; 7 reqs | No |
| 3+ distinct responsibilities | trait/context/registry/capability/lifecycle/version/security are facets of ONE plugin system | No |
| 2+ crates | one crate `ff-plugin` | No |
| low-cohesion clusters | high cohesion; module split (14 files) is clean per-concern | No |
| file-size pressure | `registry.rs` 698 non-test (over 400 cap); rest under | Yes (1, not a split driver) |

0-1 criteria (only the registry.rs size). **NOT a split candidate.** The crate
is already well-modularised; the only issue is one oversized coordinator file.

## 3. Consistency / conflict

- Public types owned here (FileForgePlugin, PluginContext, PluginError,
  PluginMetadata, Capability, CapabilityRegistry, PluginState, PluginRegistry,
  PLUGIN_API_VERSION, Dependency types) -- sole owner `ff-plugin`. Added to
  consistency-matrix.
- PluginContext is the aggregation point for foundation services -- it delegates
  to ff-logging (PluginLogHandle), ff-command (command registration), ff-config
  (namespace-scoped reads, Req 2.7 / 7.5 = configuration-system Req 8.2/8.3),
  ff-vfs (file access, Req 7.2). All are correct consumer relationships (ff-plugin
  depends on the foundation crates). Verified consistent with:
  - configuration-system Req 8 (plugin namespace scoping) -- same
    `[plugins.{name}]` rule, same NamespaceViolation semantics.
  - logging-subsystem Req 10 (plugin logging handle) -- same PluginLogHandle
    contract, `[plugin:name]` prefix.
  - command-framework Req 1.3 (plugins register during initialize) -- consistent.
  - virtual-file-system Req 7.2 (plugins use VFS, not std::fs) -- consistent.
  No conflict; ff-plugin is the correct aggregator.
- Req 6 PLUGIN_API_VERSION semver contract is owned solely here. No duplication.

## 4. Completeness

- Tasks: 169 `[x]`, 0 `[ ]`. Fully tracked complete.
- TCR: `ff-plugin` row PASS (lib unit tests: plugin lifecycle, registry,
  dependency ordering).
- Panic isolation (Req 5.3) verified: registry uses catch_unwind and logs
  ERROR with the plugin name + panic message (registry.rs:489), transitioning to
  Shutdown -- panic does not propagate. Dependency graph, topological order,
  circular-dependency rejection all present.
- Completeness verdict: **COMPLETE.**

## 5. Logging audit -- EXEMPLARY

- 27 log call sites via `ff_logging::log(LogLevel::X, ...)` (the explicit-level
  form; a valid Log_Call_Site per logging-subsystem Req 12). `ff-logging` is a
  declared dependency and is actually used.
- Every mandated ERROR/WARN path is present at the correct level:
  - Req 3.4 circular dependency -> `LogLevel::Error` (registry.rs:216). PASS.
  - Req 3.7 / 6.3 unmet dep / incompatible API major -> `LogLevel::Error`
    (registry.rs:288). PASS.
  - Req 5.3 lifecycle panic -> `LogLevel::Error` (registry.rs:489). PASS.
  - Req 5.4 lifecycle error (init/activate/deactivate) -> `LogLevel::Warn`
    (registry.rs:366, 425, 481). PASS.
  - Discovery/manifest-read failures -> `LogLevel::Warn`; successful discovery ->
    `LogLevel::Info`. Consistent with the spec's diagnostic intent.
- No `println!`/`eprintln!`. GUI-independence upheld.
- **Reference exemplar**: this crate is the correct template for the ff-vfs
  logging fix (PA-LOG-004) -- same foundation, but ff-plugin declares
  `ff-logging` and logs the mandated levels, whereas ff-vfs does neither.
  Cross-referenced in PA-LOG-004 as the pattern to follow.
- Logging verdict: **exemplary / fully adequate.**

## 6. Findings logged

- **PA-STD-002** (REFACTOR -- 400-line cap): `registry.rs` is 698 non-test lines,
  well over the rust-standards.md 400 cap. Split by concern (e.g.
  `registry_discovery.rs` for scan/discover, `registry_lifecycle.rs` for
  init/activate/deactivate/shutdown transitions, keep `registry.rs` as the thin
  coordinator + state). REFACTOR, no behaviour change, no gate.
- **PA-LOG-REF-001** (positive reference, no action): ff-plugin is the exemplar
  logging pattern for foundation crates; cite it when fixing PA-LOG-004 (ff-vfs).
  No defect here.
- **PA-LOG-001** (project-wide non-ASCII in .rs): ff-plugin contributes 68
  matches (doc-comment prose, separators). Rolled into project-wide PA-LOG-001;
  not fixed here.
