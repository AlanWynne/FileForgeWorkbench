# Extension Integration Tasks

Implementation task list for the extension integration contract
(`requirements.md` + `design.md`). These tasks are GATED: none may start until
the owner confirms the package at the gate (decision points D1-D4). All tasks
follow TDD (`.kiro/steering/testing.md`): write the failing test first, then the
minimum implementation. Agents compile-verify (scoped `cargo check` / `clippy` /
`fmt`); the owner runs the full gate.

Task numbering satisfies the requirement it implements; the `(Req X)` tags
cross-reference `requirements.md`.

- [ ] 1. Extension host skeleton and live registry (Req 1)
  - [ ] 1.1 Create `crates/ff-desktop/src/extension_host/` module (`mod.rs`) with an `ExtensionHost` struct that owns the `ff-plugin` registry; hold it on the shell inside a grouped sub-struct (Req 1.1)
  - [ ] 1.2 Construct the registry at startup; on construction failure, continue as bare core, log ERROR, do not crash (Req 1.5)
  - [ ] 1.3 Wire `ExtensionHost::shutdown()` into the workbench shutdown path (reverse-order deactivate + shutdown, 5s budget) (Req 1.4)

- [ ] 2. Production platform services (Req 1.2, 1.3)
  - [ ] 2.1 Implement `CommandRegistration` delegating to the runtime-extensible root registry (Req 3, Req 11)
  - [ ] 2.2 Implement `PluginConfigAccess` namespace-scoped to `[extensions.{name}]` (Req 1.3)
  - [ ] 2.3 Implement `PluginVfsAccess` over the `BackendEnvironment` SAVE seam (Req 7.2)
  - [ ] 2.4 Implement `PluginEventBus` over the shared `NotificationQueue` (Req 1.3)
  - [ ] 2.5 Implement `CapabilityRegistrar` bridged to `apply.rs` (Req 1.2)

- [ ] 3. Discovery and startup load (D3 in-tree registry) (Req 2.1, 2.2)
  - [ ] 3.1 Implement `discovery.rs` enumerating the in-tree feature-gated extension registry behind a pluggable discovery seam (Req 2.1)
  - [ ] 3.2 Load each extension through discover -> load -> validate -> initialize -> activate; honour the persisted `extensions.disabled` set (Req 2.1, 2.2)

- [ ] 4. Capability apply / revoke engine (Req 2.3, 2.4, 2.6)
  - [ ] 4.1 Implement `apply_capability` / `revoke_capability` in `apply.rs` plus the per-extension contribution index (Req 2.3, 2.6)
  - [ ] 4.2 Enable/disable from the host drives `activate`/`deactivate` + apply/revoke, persisting the disabled set (Req 2.3, 2.4)
  - [ ] 4.3 Isolate enable/disable failures (catch, set Failed, keep running) (Req 2.5)

- [ ] 5. Command contribution seam -- runtime-extensible root registry (Req 3, Req 11)
  - [ ] 5.0 FRAMEWORK CHANGE (owner-confirmed, Req 11.5) -- sequenced FIRST deliberately because sub-tasks 5.1-5.6 depend on this framework change: make `EnvironmentRegistry::root_commands`/`contains_root` and the `builtin_workspace_target_for`/`resolve_target` classifier consult a runtime set of extension-registered roots in addition to the compiled built-ins; keep built-ins authoritative/first; preserve the existing drift guard (Req 11.5, Req 10.3b)
  - [ ] 5.1 `apply_capability` registers a contributed ROOT (classification + owning env) into the runtime registry; `revoke_capability` removes it (Req 11.1, 11.7)
  - [ ] 5.2 Apply the first-wins duplicate rule + WARN log for root collisions; built-ins win over extension roots (Req 3.3, 3.5, Req 11.5)
  - [ ] 5.3 Revoke a contributed root on disable; root classifies/enumerates iff its extension is active; Alias Editor dropdown + alias validation reflect it (Req 3.4, Req 11.2, 11.7)
  - [ ] 5.4 Install/remove extension-shipped alias rows (surface -> env + root + template) in the live alias store, aliases removed before the root; record/persist the ROOT, not the surface; coexist with user aliases (Req 11.2, 11.3)
  - [ ] 5.5 Entry-root vs in-Context-environment-verb split: entry root -> `CustomWorkspace`/`Function` + `navigate_to`; in-buffer verbs owned by the Context's CE via the existing `active_name` derivation; no runtime-extensible CE-object registry assumed for in-scope work (Req 11.4, 11.6)
  - [ ] 5.6 Narrower fallback if the owner declines the framework change: restrict extension roots to existing compiled built-ins + aliases over them (Req 11.8) -- CONDITIONAL/contingency only, retained should the owner reverse the D5.3 confirmation; does NOT apply under the current owner-confirmed position (Req 11.5)

- [ ] 6. Menu and menu-bar contribution seam (Req 4)
  - [ ] 6.1 Implement the in-memory menu overlay merged with `DEFAULT_*_TOML`; never write a default menu to disk (Req 4.1, 4.2)
  - [ ] 6.2 POM/menu-bar options appear only while the extension is active; bare-core `DEFAULT_POM_TOML` unchanged (Req 4.3, 4.4)
  - [ ] 6.3 Toolbar/affordance invokes `dispatch_bound_command`, not the handler directly (Req 4.5)

- [ ] 7. Workspace Context contribution seam -- the one core generalization (Req 5, Req 10.3)
  - [ ] 7.1 Add the single `TabKind::Extension { id }` variant and ONE `render_body.rs` arm dispatching all extension Contexts via `render_workspace_context` (Req 5.1, 5.3)
  - [ ] 7.2 Host map from id to extension Context factory; owned-panel swap + stashed-action apply (Req 5.1, 5.2)
  - [ ] 7.3 On disable, remove the factory and close/transform open tabs of that Context (Req 5.5)
  - [ ] 7.4 Full-shell `egui_kittest` first-Tab test: first Tab lands on the extension Context's reported first interior control (Req 5.4)

- [ ] 8. Workspace Kind contribution seam (Req 6)
  - [ ] 8.1 Register an extension Kind in the Kinds registry (title, tag, menu bar, key list, profile) (Req 6.1, 6.2)
  - [ ] 8.2 Persist/restore an extension Context via `WorkspaceDescriptor` (Req 6.3)
  - [ ] 8.3 Restore-time "unavailable" notice when the contributing extension is absent/disabled (Req 6.4)
  - [ ] 8.4 Remove the Kind from the registry on disable (Req 6.5)

- [ ] 9. Viewer and VFS extension points (Req 7)
  - [ ] 9.1 Register `Viewers` capabilities through the existing `ff-viewers::plugin_bridge`; auto-deregister on shutdown (Req 7.1)
  - [ ] 9.2 Express the `Providers` file-I/O point over the current `BackendEnvironment` SAVE seam; flag any required contract reshape as an owner-confirmed framework change (Req 7.2, 7.3)
  - [ ] 9.3 Remove viewer/provider registrations on disable (Req 7.4)

- [ ] 10. Extension Manager Context wired to the live registry (Req 8)
  - [ ] 10.1 Point the (renamed) Extension Manager panel at the live `ExtensionHost` registry; read each frame (Req 8.1, 8.2)
  - [ ] 10.2 Enable/Disable actions via the `apply_<feature>_action` pending-action pattern -> `ExtensionHost::enable`/`disable` (Req 8.3)
  - [ ] 10.3 Failed-state reason display and the detail view (author, licence, homepage, capabilities, commands, config keys + Settings link) (Req 8.4, 8.5)
  - [ ] 10.4 Session persist/restore reloading live registry state (Req 8.6)

- [ ] 11. Catalogs/Files return-as-extension reference (Req 9)
  - [ ] 11.1 Prove the contract with a reference Catalogs/Files extension (or a thin stand-in) that re-introduces the POM options, verbs, Contexts, and Kinds through the seams above (Req 9.2)
  - [ ] 11.2 Verify bare-core absence and disabled-state revocation leave the workbench in the bare state (Req 9.1, 9.3)
  - [ ] 11.3 Coordinate with (do NOT duplicate) the separate bare-core POM removal effort (Req 9.4)

- [ ] 12. Framework-conformance verification (Req 10, Req 11)
  - [ ] 12.1 Confirm no parallel dispatcher, no second nav stack, one focus latch, code-only menus across all new code (Req 10.1)
  - [ ] 12.2 Record BOTH additive framework extension points (identifier-keyed extension-Context arm; runtime-extensible root registry + classifier) as owner-confirmed (Req 10.3a, 10.3b)
  - [ ] 12.3 End-to-end test the canonical EDIT -> Editor Context walk-through (design D5.5): `EDIT` root registered -> `resolve_target` classifies -> `navigate_to` opens the Context -> FFEDIT owns in-buffer verbs; the `ECOB` alias resolves via CR-NR-112 substitution; `EDIT`/`ECOB` absent when disabled (Req 11.1, 11.3, 11.4, 11.7)

- [ ] 13. Rename staging (D1 waves) -- gated on owner acceptance of D1
  - [ ] 13.1 Wave A: rename spec folders `plugin-architecture` -> `extension-architecture`, `plugin-manager-ui` -> `extension-manager-ui`; fold their content into / cross-reference this spec
  - [ ] 13.2 Wave B: `PLUGINS` verb/POM -> `EXTENSIONS`; "Plugin Manager" -> "Extension Manager"; `[PLUGINS]` -> `[EXTENSIONS]` (optional hidden `PLUGINS` alias)
  - [ ] 13.3 Wave C: module/type renames (`extension_manager_panel`, `KindTag::ExtensionManager`) via compiler-assisted rename
  - [ ] 13.4 Wave D: crate renames `ff-plugin` -> `ff-extension`, `ff-mdx-plugin` -> `ff-mdx-extension` and the persisted Kind id, landed on its own
