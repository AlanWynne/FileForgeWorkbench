# Implementation Plan: Function Keys and Command History (`ff-keys`)

## Overview

This plan covers the complete implementation of the `ff-keys` crate — configurable function key bindings (F1–F24), the Key Label Bar display model, the RETRIEVE command, and the bounded deduplicated Command History ring with cross-session TOML persistence. The crate owns the Key_Map resolution logic (global vs. profile full-replacement model), function key dispatch through the command framework, the Retrieve_Pointer cycling mechanism, and the History_Store persistence layer.

This is a **Wave 9 (Desktop Integration)** sub-project. It depends on `ff-command` (command framework), `ff-config` (configuration system), `ff-session` (startup-and-session — User_Data_Dir, startup/exit sequence hooks), and `ff-logging` (logging subsystem).

---

## Tasks

- [ ] 1. Crate scaffolding and core types
  - [ ] 1.1 Create `crates/ff-keys/Cargo.toml` with dependencies (ff-command, ff-config, ff-logging, thiserror, serde, toml, proptest dev-dep)
  - [ ] 1.2 Create `crates/ff-keys/src/lib.rs` with module declarations and public API re-exports
  - [ ] 1.3 Create module files: `key_map.rs`, `key_map_resolver.rs`, `function_key.rs`, `key_label_bar.rs`, `command_history.rs`, `history_store.rs`, `retrieve.rs`, `config_keys.rs`, `error.rs`
  - [ ] 1.4 Add `ff-keys` to workspace `Cargo.toml` members list
  - [ ] 1.5 Define `KeysError` enum with variants: InvalidFunctionKey, KeyMapLoadFailed, KeyMapEntryInvalid, HistoryStoreLoadFailed, HistoryStoreCorrupt, HistoryStoreWriteFailed, CommandNotRegistered, RetrieveEmptyHistory, RetrieveEndOfHistory, ConfigInvalid
  - [ ] 1.6 Implement `Display` and `thiserror::Error` derives with descriptive messages for all error variants
  - Covers: Structural foundation for all requirements

- [ ] 2. Key map data model
  - [ ] 2.1 Define `FunctionKey` enum with variants F1–F24, implementing `Display`, `FromStr`, `Serialize`, `Deserialize`
  - [ ] 2.2 Implement `FunctionKey::from_str` parsing — accept "F1"–"F24" case-insensitively, reject all other input
  - [ ] 2.3 Define `KeyBinding` struct with fields: command (String), label (Option<String>)
  - [ ] 2.4 Define `KeyMap` struct wrapping `HashMap<FunctionKey, KeyBinding>` with constructor, lookup, and iteration methods
  - [ ] 2.5 Implement `KeyMap::from_toml_table` — parse a TOML section into a KeyMap, rejecting invalid key identifiers with warnings
  - [ ] 2.6 Implement `KeyMap::get(key: FunctionKey) -> Option<&KeyBinding>` for single-key lookup
  - [ ] 2.7 Implement `KeyMap::is_empty()` and `KeyMap::len()` convenience methods
  - [ ] 2.8 Write unit tests for FunctionKey parsing (valid/invalid), KeyMap construction, TOML parsing with valid/invalid entries
  - Covers: Requirement 1 (AC 1.3, 1.5), Requirement 11 (AC 11.1, 11.2)

- [ ] 3. Key map resolver — global and profile resolution
  - [ ] 3.1 Define `KeyMapResolver` struct with fields: global_key_map (KeyMap), active_profile_key_map (Option<KeyMap>), active_language_profile (Option<String>)
  - [ ] 3.2 Implement `KeyMapResolver::load_global_key_map` — read `[global_key_map]` from effective configuration at startup, apply empty map if absent
  - [ ] 3.3 Implement `KeyMapResolver::load_profile_key_map(profile: &str)` — read `[key_map]` section from language profile TOML file, return None if section absent
  - [ ] 3.4 Implement `KeyMapResolver::active_key_map() -> &KeyMap` — return Profile_Key_Map if active, otherwise Global_Key_Map
  - [ ] 3.5 Implement full-replacement semantics — when Profile_Key_Map is active, Global_Key_Map is entirely inactive; keys not in Profile_Key_Map are unassigned
  - [ ] 3.6 Implement `KeyMapResolver::on_profile_changed(profile: Option<&str>)` — recompute active key map when active language profile changes
  - [ ] 3.7 Implement hot-reload listener — subscribe to configuration-system change notifications for `[global_key_map]` and language profile `[key_map]` sections
  - [ ] 3.8 Implement fallback on profile removal — when `[key_map]` section is removed from profile TOML, revert to Global_Key_Map without restart
  - [ ] 3.9 Write unit tests for: global-only resolution, profile override, full-replacement (no inheritance), profile removal fallback, empty global map, profile switch
  - Covers: Requirement 1 (AC 1.1, 1.2, 1.4), Requirement 2 (AC 2.1–2.6)

- [ ] 4. Function key binding and execution
  - [ ] 4.1 Implement `FunctionKeyDispatcher` struct owning a reference to KeyMapResolver and command framework dispatch trait
  - [ ] 4.2 Implement `FunctionKeyDispatcher::on_key_press(key: FunctionKey)` — look up binding in active key map, dispatch command string through command framework if assigned
  - [ ] 4.3 Implement no-op behaviour when pressed key has no assignment — produce no action, no error
  - [ ] 4.4 Implement full command syntax support — pass complete command string (with arguments and modifiers) to command framework dispatcher
  - [ ] 4.5 Implement macro invocation support — detect and pass `MACRO <name>` syntax through command framework
  - [ ] 4.6 Implement history integration — after successful dispatch, add command to Command_History unless it is an Excluded_Command
  - [ ] 4.7 Implement Excluded_Command bypass — do NOT add UNDO, REDO, RETRIEVE to history when dispatched via function key
  - [ ] 4.8 Write unit tests for: assigned key dispatch, unassigned key no-op, full syntax passthrough, macro syntax, history addition, excluded command bypass
  - Covers: Requirement 3 (AC 3.1–3.6)

- [ ] 5. Key Label Bar model
  - [ ] 5.1 Define `KeyLabelEntry` struct with fields: key (FunctionKey), label (String), is_assigned (bool)
  - [ ] 5.2 Define `KeyLabelBarModel` struct providing the display data for the Key Label Bar UI
  - [ ] 5.3 Implement `KeyLabelBarModel::from_key_map(map: &KeyMap) -> Self` — derive labels from active key map
  - [ ] 5.4 Implement label derivation logic — use explicit label if configured, else use first token of command string
  - [ ] 5.5 Implement blank slot handling — unassigned keys produce blank/omitted entries
  - [ ] 5.6 Implement `KeyLabelBarModel::update(&mut self, map: &KeyMap)` — refresh label data when active key map changes (profile switch, hot-reload, tab change)
  - [ ] 5.7 Implement change notification — emit a signal/callback when label bar data changes so UI can re-render in same frame
  - [ ] 5.8 Write unit tests for: label derivation from command first-token, explicit label override, blank slots for unassigned keys, update on key map change
  - Covers: Requirement 4 (AC 4.1–4.6)

- [ ] 6. Command History ring
  - [ ] 6.1 Define `CommandHistory` struct with fields: entries (VecDeque<String>), max_entries (usize), excluded_commands (HashSet<String>)
  - [ ] 6.2 Implement `CommandHistory::new(max_entries: usize, excluded_commands: HashSet<String>)` constructor
  - [ ] 6.3 Implement `CommandHistory::add(command: &str)` — insert at front with deduplication and capacity enforcement
  - [ ] 6.4 Implement deduplication — case-insensitive on command name (first token), case-preserving on arguments; promote existing duplicate to front
  - [ ] 6.5 Implement capacity enforcement — evict oldest entry (tail) when adding would exceed max_entries
  - [ ] 6.6 Implement exclusion check — reject commands in the Excluded_Command set regardless of invocation source
  - [ ] 6.7 Implement `CommandHistory::get(index: usize) -> Option<&str>` — index 0 = most recent
  - [ ] 6.8 Implement `CommandHistory::len()` and `CommandHistory::is_empty()` convenience methods
  - [ ] 6.9 Implement `CommandHistory::entries() -> impl Iterator<Item = &str>` — iterate most-recent-first
  - [ ] 6.10 Implement `CommandHistory::trim_to(new_max: usize)` — trim oldest entries when max_entries is reduced via hot-reload
  - [ ] 6.11 Implement default Excluded_Command set: RETRIEVE, UNDO, REDO
  - [ ] 6.12 Implement configurable exclusion — merge user-configured `history_excluded_commands` with defaults
  - [ ] 6.13 Write unit tests for: add/dedup, capacity eviction, exclusion, case-insensitive dedup, case-preserving args, trim, ordering
  - Covers: Requirement 7 (AC 7.1–7.3), Requirement 8 (AC 8.1–8.4), Requirement 9 (AC 9.1–9.4)

- [ ] 7. RETRIEVE command and Retrieve Pointer
  - [ ] 7.1 Define `RetrieveState` struct with fields: pointer (Option<usize>), cycle_active (bool)
  - [ ] 7.2 Implement `RetrieveState::new()` — initialise with pointer at initial (no retrieval) position
  - [ ] 7.3 Implement `RetrieveState::retrieve(history: &CommandHistory) -> RetrieveResult` — advance pointer backward, return entry at pointer position
  - [ ] 7.4 Implement initial retrieval — when pointer is at initial position, set to index 0 (most recent) and return that entry
  - [ ] 7.5 Implement successive retrieval — advance pointer one step older on each call without intervening non-RETRIEVE command
  - [ ] 7.6 Implement end-of-history detection — when pointer reaches oldest entry, return status message and do not modify command field
  - [ ] 7.7 Implement empty history detection — when history is empty, return status message and do not modify command field
  - [ ] 7.8 Implement `RetrieveState::reset()` — reset pointer to initial position when any non-RETRIEVE command is submitted
  - [ ] 7.9 Implement `RetrieveState::set_position(index: usize)` — set pointer to specific entry (for History_Dropdown selection)
  - [ ] 7.10 Define `RetrieveResult` enum with variants: Entry(String), EndOfHistory, EmptyHistory
  - [ ] 7.11 Write unit tests for: initial retrieve, successive retrieves, end-of-history, empty history, reset on non-RETRIEVE, set_position from dropdown
  - Covers: Requirement 5 (AC 5.1–5.7), Requirement 10 (AC 10.4)

- [ ] 8. TOML persistence — History Store
  - [ ] 8.1 Define `HistoryStore` struct encapsulating file path and I/O operations for Command_History persistence
  - [ ] 8.2 Implement `HistoryStore::load(path: &Path) -> Result<Vec<String>, KeysError>` — read and parse History_Store TOML file
  - [ ] 8.3 Implement TOML format — `[[entries]]` array-of-tables or `entries = [...]` array-of-strings in most-recent-first order
  - [ ] 8.4 Implement graceful load on missing file — return empty Vec without error
  - [ ] 8.5 Implement graceful load on corrupt/unparseable file — log WARN with file path and parse error, return empty Vec
  - [ ] 8.6 Implement `HistoryStore::save(path: &Path, entries: &[String]) -> Result<(), KeysError>` — serialize entries to TOML and write atomically (temp + rename)
  - [ ] 8.7 Implement configurable file path — resolve `history_file` config key relative to User_Data_Dir, apply default path when not configured
  - [ ] 8.8 Implement startup loading — integrate with startup-and-session startup sequence to load history during initialisation
  - [ ] 8.9 Implement exit-time save — integrate with startup-and-session exit sequence to persist history on normal shutdown
  - [ ] 8.10 Write unit tests for: TOML round-trip, missing file graceful load, corrupt file warning and empty result, atomic save, path resolution
  - Covers: Requirement 6 (AC 6.1–6.7), Requirement 11 (AC 11.4)

- [ ] 9. Profile support and configuration schema
  - [ ] 9.1 Define `KeysConfig` struct with all configuration fields: max_history_entries (usize), history_file (Option<String>), history_excluded_commands (Vec<String>)
  - [ ] 9.2 Implement `Default` for `KeysConfig` — max_history_entries=200, history_file=None (use default in User_Data_Dir), excluded_commands=empty (defaults always applied)
  - [ ] 9.3 Implement configuration key registration for `max_history_entries`, `history_file`, `history_excluded_commands` under the appropriate TOML namespace
  - [ ] 9.4 Implement validation for `max_history_entries` — reject zero or negative values, apply default of 200 with WARN log
  - [ ] 9.5 Implement `[global_key_map]` schema validation — each key is F1–F24, each value is string or table with `command` (required) and `label` (optional)
  - [ ] 9.6 Implement `[key_map]` schema in language profile files — same schema as `[global_key_map]`
  - [ ] 9.7 Implement invalid value-type handling — emit descriptive warning identifying field name and expected type, apply default
  - [ ] 9.8 Implement hot-reload for all configuration keys — `[global_key_map]` changes take effect immediately; `max_history_entries` changes trim on next addition
  - [ ] 9.9 Write unit tests for: default values, validation (zero/negative max), schema parsing (string shorthand vs table), invalid type warning, hot-reload trim
  - Covers: Requirement 9 (AC 9.1–9.4), Requirement 11 (AC 11.1–11.7)

- [ ] 10. Conflict detection and key map validation
  - [ ] 10.1 Implement duplicate key detection within a single key map — if same FunctionKey appears twice in TOML, last-wins with WARN log
  - [ ] 10.2 Implement command existence validation — optionally verify assigned command_id is registered in command framework, emit WARN if not found (non-blocking)
  - [ ] 10.3 Implement label length validation — warn if explicit label exceeds display width threshold (configurable, default 8 chars)
  - [ ] 10.4 Implement profile key map diagnostic — log INFO when profile key map activates listing unassigned keys count
  - [ ] 10.5 Write unit tests for: duplicate key warning, unregistered command warning, label length warning, profile activation diagnostics
  - Covers: Requirement 1 (AC 1.5), Requirement 2 (AC 2.5), Requirement 11 (AC 11.6)

- [ ] 11. Command registration — RETRIEVE command
  - [ ] 11.1 Implement RETRIEVE as a registered command in the command framework with command_id "RETRIEVE"
  - [ ] 11.2 Implement RETRIEVE command handler — invoke RetrieveState::retrieve, populate Primary_Command_Field with result
  - [ ] 11.3 Implement RETRIEVE exclusion from history — ensure RETRIEVE is in the Excluded_Command set
  - [ ] 11.4 Implement status message output — emit appropriate status messages for EndOfHistory and EmptyHistory results
  - [ ] 11.5 Implement non-RETRIEVE command hook — subscribe to command execution events to call RetrieveState::reset on any non-RETRIEVE command submission
  - [ ] 11.6 Write unit tests for: RETRIEVE registration, handler invocation, exclusion from history, status messages, pointer reset on other commands
  - Covers: Requirement 5 (AC 5.1–5.7), Requirement 8 (AC 8.1–8.2)

- [ ] 12. History Dropdown model
  - [ ] 12.1 Define `HistoryDropdownModel` struct providing display data for the History_Dropdown UI control
  - [ ] 12.2 Implement `HistoryDropdownModel::entries() -> &[String]` — expose Command_History in most-recent-first order
  - [ ] 12.3 Implement `HistoryDropdownModel::select(index: usize)` — populate command field with selected entry and update Retrieve_Pointer position
  - [ ] 12.4 Implement `HistoryDropdownModel::is_empty() -> bool` — for empty state indicator logic
  - [ ] 12.5 Implement highlight navigation model — track highlighted index for up/down arrow keyboard navigation
  - [ ] 12.6 Write unit tests for: entries ordering, select populates field and sets pointer, empty state, highlight navigation
  - Covers: Requirement 10 (AC 10.1–10.6)

- [ ] 13. Property-based tests
  - [ ] 13.1 Write PBT: Key map resolution full-replacement invariant
  - [ ] 13.2 Write PBT: Command History deduplication and ordering property
  - [ ] 13.3 Write PBT: Command History bounded capacity invariant
  - [ ] 13.4 Write PBT: Retrieve Pointer cycling correctness property
  - [ ] 13.5 Write PBT: History Store TOML round-trip fidelity
  - [ ] 13.6 Write PBT: Key Label Bar derivation consistency property
  - [ ] 13.7 Write PBT: Excluded Command never enters history property
  - [ ] 13.8 Write PBT: Function key dispatch idempotency property
  - [ ] 13.9 Write PBT: Configuration hot-reload convergence property
  - [ ] 13.10 Write PBT: Deduplication case-sensitivity correctness property
  - Covers: All requirements (property-based validation)

- [ ] 14. Integration tests
  - [ ] 14.1 Write integration test: global key map load and function key dispatch end-to-end
  - [ ] 14.2 Write integration test: profile key map override fully replaces global map
  - [ ] 14.3 Write integration test: RETRIEVE cycles through history and resets on command submission
  - [ ] 14.4 Write integration test: History Store persistence across simulated startup/shutdown cycle
  - [ ] 14.5 Write integration test: corrupt History Store file triggers graceful degradation to empty history
  - [ ] 14.6 Write integration test: hot-reload of global_key_map updates Key Label Bar and function key bindings
  - [ ] 14.7 Write integration test: max_history_entries enforcement and hot-reload trim
  - [ ] 14.8 Write integration test: function key dispatches excluded command without history recording
  - [ ] 14.9 Write integration test: History Dropdown selection updates Retrieve_Pointer position
  - [ ] 14.10 Write integration test: profile switch triggers key map recomputation and label bar update
  - Covers: Cross-requirement interaction validation

---

## Property-Based Test Definitions

### Property 1: Key Map Resolution Full-Replacement Invariant

**Validates: Requirements 1.2, 2.2, 2.5**

- **Statement:** When a Profile_Key_Map is active, the resolved key map contains ONLY entries from the Profile_Key_Map. No Global_Key_Map entry is ever visible through the resolver. Keys not defined in the Profile_Key_Map are unassigned regardless of their Global_Key_Map binding.
- **Strategy:** Generate:
  - Global_Key_Map: random subset of F1–F24 with random command bindings (1–24 entries)
  - Profile_Key_Map: different random subset of F1–F24 with random command bindings (0–24 entries)
  - Query key: random FunctionKey from F1–F24
- **Invariant:** When profile map is active: `resolver.active_key_map().get(key) == profile_map.get(key)` for ALL keys F1–F24. No key resolves to a global binding.

### Property 2: Command History Deduplication and Ordering

**Validates: Requirements 7.1, 7.2, 7.3**

- **Statement:** After any sequence of add operations, Command_History contains no duplicate entries (per the case-insensitive-name, case-preserving-args rule), and the most recently added non-duplicate command is always at index 0.
- **Strategy:** Generate:
  - Command pool: 5–30 commands with varying case on first token and identical/different args
  - Add sequence: 20–200 random selections from the pool
- **Invariant:** After every add: (1) no two entries have the same normalised form (case-insensitive first token + exact args), (2) the last-added command (or its promoted form) is at index 0.

### Property 3: Command History Bounded Capacity Invariant

**Validates: Requirements 9.1, 9.3**

- **Statement:** For any sequence of add operations with a configured max_history_entries of N, the history length never exceeds N.
- **Strategy:** Generate:
  - max_entries: integer in [1, 500]
  - Command pool: 10–100 unique commands
  - Add sequence: 50–500 random add operations
- **Invariant:** `history.len() <= max_entries` after every operation. When len == max_entries and a new unique command is added, the oldest entry is evicted.

### Property 4: Retrieve Pointer Cycling Correctness

**Validates: Requirements 5.2, 5.3, 5.4, 5.5**

- **Statement:** For a Command_History of length N, successive RETRIEVE calls cycle through entries from index 0 (most recent) to index N-1 (oldest). The (N+1)th RETRIEVE produces EndOfHistory. Any non-RETRIEVE command resets the pointer so the next RETRIEVE starts at index 0 again.
- **Strategy:** Generate:
  - History contents: 1–50 unique command strings
  - Operation sequence: interleaved RETRIEVE calls and non-RETRIEVE command submissions (10–100 ops)
- **Invariant:** (1) k-th consecutive RETRIEVE returns `history.get(k-1)` for k in 1..=N. (2) (N+1)-th RETRIEVE returns EndOfHistory. (3) After any non-RETRIEVE submission, next RETRIEVE returns `history.get(0)`.

### Property 5: History Store TOML Round-Trip Fidelity

**Validates: Requirements 6.1, 6.7**

- **Statement:** For any valid Command_History state, serializing to the History_Store TOML format and deserializing back produces an identical ordered list of entries.
- **Strategy:** Generate:
  - Entry count: 0–200
  - Entry content: random ASCII command strings (command name + 0–3 arguments), including strings with special TOML characters (single quotes, double quotes, backslashes)
- **Invariant:** `deserialize(serialize(entries)) == entries` for all generated entry lists. Order is preserved (most-recent-first).

### Property 6: Key Label Bar Derivation Consistency

**Validates: Requirements 4.2, 4.4, 4.5**

- **Statement:** For any KeyMap, the derived Key_Label_Bar entries always reflect the current active map: assigned keys show either the explicit label or the first token of the command; unassigned keys are blank. The label bar and key map are never out of sync.
- **Strategy:** Generate:
  - KeyMap: random subset of F1–F24 with random command strings, 50% chance of explicit label on each entry
  - Query: iterate all 24 function keys
- **Invariant:** For each key: if key is in map AND has explicit label → bar shows explicit label. If key is in map AND no explicit label → bar shows first token of command. If key is not in map → bar shows blank/omitted.

### Property 7: Excluded Command Never Enters History

**Validates: Requirements 8.1, 8.2, 8.4**

- **Statement:** For any sequence of command submissions (typed, function key, macro), no command in the Excluded_Command set ever appears in Command_History regardless of how it was invoked or how many times it was submitted.
- **Strategy:** Generate:
  - Excluded set: RETRIEVE, UNDO, REDO + 0–3 user-configured additional exclusions
  - Submission sequence: 50–300 commands, mix of excluded and non-excluded, from typed/function-key/macro sources
- **Invariant:** After all submissions: `history.entries().all(|e| !excluded_set.contains(normalise(e)))`. History only contains non-excluded commands.

### Property 8: Function Key Dispatch Idempotency

**Validates: Requirements 3.1, 3.2**

- **Statement:** Pressing the same function key N times (with no intervening key map change) dispatches the same command string N times. Pressing an unassigned key any number of times produces exactly zero dispatches.
- **Strategy:** Generate:
  - KeyMap: random bindings for random subset of keys
  - Press sequence: 10–50 key presses (mix of assigned and unassigned keys)
- **Invariant:** For assigned key K pressed consecutively: each press dispatches `key_map.get(K).command`. For unassigned key U: dispatch count for U is always 0.

### Property 9: Configuration Hot-Reload Convergence

**Validates: Requirements 2.4, 11.7**

- **Statement:** After a configuration change event (global_key_map modification, profile key_map removal, max_history_entries change), the system converges to a consistent state within a single processing cycle. The active key map, label bar, and history capacity all reflect the new configuration.
- **Strategy:** Generate:
  - Initial config: random global map, optional profile map, random max_history_entries
  - Change event: one of {modify global map, remove profile key_map, change max_history_entries}
  - Post-change queries: resolve key, read label bar, check history capacity
- **Invariant:** After reload: resolver returns entries from new map only. Label bar matches new map. History capacity respects new max (existing entries trimmed if over new limit).

### Property 10: Deduplication Case-Sensitivity Correctness

**Validates: Requirement 7.2**

- **Statement:** Two commands are considered duplicates if and only if their first tokens match case-insensitively AND their remaining arguments match exactly (case-sensitive). Commands with same first token but different argument casing are NOT duplicates and both remain in history.
- **Strategy:** Generate:
  - Command pairs: same first token in varying case + arguments that are identical or differ only in case
  - Add both commands to history in sequence
- **Invariant:** If first_tokens match case-insensitively AND args are identical → history contains only the later entry (promoted). If first_tokens match but args differ in case → both entries exist in history.

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Key Map Model", "tasks": ["2"], "dependsOn": [0] },
    { "id": 2, "label": "Key Map Resolution", "tasks": ["3"], "dependsOn": [1] },
    { "id": 3, "label": "Function Key Dispatch", "tasks": ["4"], "dependsOn": [2] },
    { "id": 4, "label": "Key Label Bar", "tasks": ["5"], "dependsOn": [2] },
    { "id": 5, "label": "Command History Ring", "tasks": ["6"], "dependsOn": [0] },
    { "id": 6, "label": "RETRIEVE Command", "tasks": ["7"], "dependsOn": [5] },
    { "id": 7, "label": "History Persistence", "tasks": ["8"], "dependsOn": [5] },
    { "id": 8, "label": "Configuration and Profiles", "tasks": ["9", "10"], "dependsOn": [1, 5, 7] },
    { "id": 9, "label": "Command Registration", "tasks": ["11"], "dependsOn": [3, 6] },
    { "id": 10, "label": "History Dropdown", "tasks": ["12"], "dependsOn": [5, 6] },
    { "id": 11, "label": "Property-Based Tests", "tasks": ["13"], "dependsOn": [2, 3, 4, 5, 6, 7, 8, 9, 10] },
    { "id": 12, "label": "Integration Tests", "tasks": ["14"], "dependsOn": [8, 9, 10, 11] }
  ]
}
```

---

## Notes

- This is a Wave 9 (Desktop Integration) crate depending on `ff-command` (Wave 2), `ff-config` (Wave 2), `ff-session` (Wave 8), and `ff-logging` (Wave 0)
- The full-replacement key map model is a deliberate ISPF-faithful design choice — profile maps do NOT inherit from the global map; unmentioned keys become unassigned
- Key_Map_Resolver logic is GUI-independent (FFW-ARCH-001) — it provides data models consumed by the GUI shell but has no framework dependency
- Key_Label_Bar is a data model only in this crate; the rendering lives in the GUI shell (`menu-and-statusbar` UI layer)
- History_Store uses TOML format consistent with the configuration-system's choice; the file lives in User_Data_Dir alongside `session.toml`
- The `configuration-system` crate handles TOML parsing for key maps within configuration files; `ff-keys` only parses the History_Store file directly
- RETRIEVE is registered in the command framework like any other primary command — its dispatch follows the same pipeline
- The History_Dropdown is a UI model only — the actual dropdown widget rendering belongs to the `menu-and-statusbar` GUI shell
- Deduplication uses a split comparison: `command_name.to_ascii_uppercase()` for the first token, exact byte comparison for remainder
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- The `history_excluded_commands` config key is additive — user entries are merged with the hardcoded defaults (RETRIEVE, UNDO, REDO), never replacing them
- Hot-reload of `max_history_entries` trims existing entries on next add, not immediately on config change — this avoids surprising data loss during configuration experimentation
- Atomic file writes (temp + rename) for History_Store prevent data corruption on crash during save

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Global Default Key Map | AC 1.1 | Task 3 |
| Req 1: Global Default Key Map | AC 1.2 | Task 3 |
| Req 1: Global Default Key Map | AC 1.3 | Task 2 |
| Req 1: Global Default Key Map | AC 1.4 | Task 3 |
| Req 1: Global Default Key Map | AC 1.5 | Tasks 2, 10 |
| Req 2: Profile-Specific Key Map | AC 2.1 | Task 3 |
| Req 2: Profile-Specific Key Map | AC 2.2 | Task 3 |
| Req 2: Profile-Specific Key Map | AC 2.3 | Tasks 3, 9 |
| Req 2: Profile-Specific Key Map | AC 2.4 | Task 3 |
| Req 2: Profile-Specific Key Map | AC 2.5 | Task 3 |
| Req 2: Profile-Specific Key Map | AC 2.6 | Tasks 3, 5 |
| Req 3: Function Key Execution | AC 3.1 | Task 4 |
| Req 3: Function Key Execution | AC 3.2 | Task 4 |
| Req 3: Function Key Execution | AC 3.3 | Task 4 |
| Req 3: Function Key Execution | AC 3.4 | Task 4 |
| Req 3: Function Key Execution | AC 3.5 | Task 4 |
| Req 3: Function Key Execution | AC 3.6 | Task 4 |
| Req 4: Key Label Bar Display | AC 4.1 | Task 5 |
| Req 4: Key Label Bar Display | AC 4.2 | Task 5 |
| Req 4: Key Label Bar Display | AC 4.3 | Task 5 |
| Req 4: Key Label Bar Display | AC 4.4 | Task 5 |
| Req 4: Key Label Bar Display | AC 4.5 | Task 5 |
| Req 4: Key Label Bar Display | AC 4.6 | Tasks 3, 5 |
| Req 5: RETRIEVE Command | AC 5.1 | Tasks 7, 11 |
| Req 5: RETRIEVE Command | AC 5.2 | Task 7 |
| Req 5: RETRIEVE Command | AC 5.3 | Task 7 |
| Req 5: RETRIEVE Command | AC 5.4 | Task 7 |
| Req 5: RETRIEVE Command | AC 5.5 | Tasks 7, 11 |
| Req 5: RETRIEVE Command | AC 5.6 | Tasks 7, 11 |
| Req 5: RETRIEVE Command | AC 5.7 | Task 7 |
| Req 6: History Storage and Persistence | AC 6.1 | Task 8 |
| Req 6: History Storage and Persistence | AC 6.2 | Task 8 |
| Req 6: History Storage and Persistence | AC 6.3 | Task 8 |
| Req 6: History Storage and Persistence | AC 6.4 | Tasks 8, 9 |
| Req 6: History Storage and Persistence | AC 6.5 | Task 8 |
| Req 6: History Storage and Persistence | AC 6.6 | Task 8 |
| Req 6: History Storage and Persistence | AC 6.7 | Task 8 |
| Req 7: Command History Deduplication | AC 7.1 | Task 6 |
| Req 7: Command History Deduplication | AC 7.2 | Task 6 |
| Req 7: Command History Deduplication | AC 7.3 | Task 6 |
| Req 8: Command History Exclusion Rules | AC 8.1 | Task 6 |
| Req 8: Command History Exclusion Rules | AC 8.2 | Tasks 6, 11 |
| Req 8: Command History Exclusion Rules | AC 8.3 | Tasks 6, 9 |
| Req 8: Command History Exclusion Rules | AC 8.4 | Tasks 4, 6, 11 |
| Req 9: Configurable History Capacity | AC 9.1 | Task 9 |
| Req 9: Configurable History Capacity | AC 9.2 | Tasks 6, 9 |
| Req 9: Configurable History Capacity | AC 9.3 | Task 6 |
| Req 9: Configurable History Capacity | AC 9.4 | Task 9 |
| Req 10: History Dropdown | AC 10.1 | Task 12 |
| Req 10: History Dropdown | AC 10.2 | Task 12 |
| Req 10: History Dropdown | AC 10.3 | Task 12 |
| Req 10: History Dropdown | AC 10.4 | Tasks 7, 12 |
| Req 10: History Dropdown | AC 10.5 | Task 12 |
| Req 10: History Dropdown | AC 10.6 | Task 12 |
| Req 11: Configuration Schema | AC 11.1 | Tasks 2, 9 |
| Req 11: Configuration Schema | AC 11.2 | Tasks 3, 9 |
| Req 11: Configuration Schema | AC 11.3 | Task 9 |
| Req 11: Configuration Schema | AC 11.4 | Tasks 8, 9 |
| Req 11: Configuration Schema | AC 11.5 | Tasks 6, 9 |
| Req 11: Configuration Schema | AC 11.6 | Tasks 9, 10 |
| Req 11: Configuration Schema | AC 11.7 | Tasks 3, 9 |

---

## Phase AM — Per-Context Key Maps, PFSHOW, 24-Key Bar, Hotspots, END/RETURN, LIST+RETRIEVE

### New Requirements (Req 12–19)

- [ ] 15. PFSHOW command
  - [ ] 15.1 Register `keys.pfshow` command in the command framework; handle `PFSHOW ON`, `PFSHOW OFF`, `PFSHOW` (toggle) arguments
  - [ ] 15.2 Add `key_bar_visible: bool` field to session state; persist and restore across launches
  - [ ] 15.3 Wire PFSHOW handler to show/hide Key_Label_Bar in `ff-desktop` shell render loop
  - [ ] 15.4 Write unit tests: PFSHOW ON shows bar, PFSHOW OFF hides bar, PFSHOW toggles, idempotent ON/OFF
  - Covers: Requirement 12 (AC 12.1–12.7)

- [ ] 16. Two-row Key Label Bar layout
  - [ ] 16.1 Update `KeyLabelBarModel` to produce two ordered rows: F1–F12 (row 0) and F13–F24 (row 1)
  - [ ] 16.2 Update `KeyLabelBarModel::from_key_map` to always include all 24 slots (blank label for unassigned keys)
  - [ ] 16.3 Update `ff-desktop` Key_Label_Bar render to iterate two rows, rendering each slot as "Fn Label"
  - [ ] 16.4 Write unit tests: two rows produced, unassigned slots present with blank label, assigned slots show correct label
  - Covers: Requirement 13 (AC 13.1–13.5)

- [ ] 17. Per-context key map
  - [ ] 17.1 Add `context_key_maps: HashMap<String, KeyMap>` to `KeyMapResolver`
  - [ ] 17.2 Implement `KeyMapResolver::set_context(context_name: &str)` — activates the Context_Key_Map for the named context or falls back to Global_Key_Map
  - [ ] 17.3 Define context name constants: `"pom"`, `"editor"`, `"settings"`, `"files"`, `"hex"`, `"toolchain"`
  - [ ] 17.4 Wire context activation into `ff-desktop` tab-switch logic: on active tab change, call `set_context` with the tab's context name
  - [ ] 17.5 Parse `[context_key_maps.<name>]` sections from workbench configuration into `KeyMapResolver`
  - [ ] 17.6 Write unit tests: context map overrides global, unknown context falls back to global, tab switch triggers context change, full-replacement semantics
  - Covers: Requirement 14 (AC 14.1–14.7)

- [ ] 18. Built-in default 24-key assignment set
  - [ ] 18.1 Define `KeyMap::default_global() -> KeyMap` returning the built-in default map (F1=HELP/Help, F3=END/End, F7=UP MAX/Up, F8=DOWN MAX/Down, F12=RETRIEVE/Retrieve)
  - [ ] 18.2 Use `KeyMap::default_global()` as the fallback when no `[global_key_map]` section is present in configuration
  - [ ] 18.3 Write unit tests: default map contains exactly the 5 specified assignments, remaining 19 slots are unassigned
  - Covers: Requirement 15 (AC 15.1–15.4)

- [ ] 19. Key Label Bar hotspots
  - [ ] 19.1 Add `on_slot_clicked(key: FunctionKey)` method to `KeyLabelBarModel` or expose slot click events via the shell
  - [ ] 19.2 Wire slot click in `ff-desktop` Key_Label_Bar render: each slot rendered as a clickable `egui::Button` or response area
  - [ ] 19.3 On slot click, dispatch the assigned command through `FunctionKeyDispatcher::dispatch(key)`
  - [ ] 19.4 Add tooltip rendering: on hover over an assigned slot, show the full command string
  - [ ] 19.5 Write unit tests: click on assigned slot dispatches command, click on blank slot is no-op, tooltip text equals full command string
  - Covers: Requirement 16 (AC 16.1–16.5)

- [ ] 20. END and RETURN navigation commands
  - [ ] 20.1 Register `nav.end` command; implement handler: close current tab, navigate to previous tab or POM; if on POM, exit
  - [ ] 20.2 Register `nav.return` command; implement handler: navigate to POM tab; if already on POM, exit
  - [ ] 20.3 Add `END` and `RETURN` to the Excluded_Command set so they are never recorded in Command_History
  - [ ] 20.4 Track "previous active tab" in `ff-desktop` shell state so END can return to it
  - [ ] 20.5 Write unit tests: END from editor navigates to previous tab, END from POM exits, RETURN from any tab navigates to POM, RETURN from POM exits, neither recorded in history
  - Covers: Requirement 17 (AC 17.1–17.7)

- [ ] 21. Contextual help "not available yet" fallback
  - [ ] 21.1 In `ff-help` Context_Detector: after resolving Topic_Key, check Help_Topic_Registry; if topic absent, emit status message "Help not available yet for: <context>. Press F1 again or type HELP for the Help Index."
  - [ ] 21.2 Display the fallback message in the status bar (not the full Help_Panel)
  - [ ] 21.3 Write unit tests: missing topic produces fallback message, existing topic opens Help_Panel normally
  - Covers: Requirement 18 (AC 18.1–18.3)

- [ ] 22. LIST + RETRIEVE history browser
  - [ ] 22.1 In `RetrieveHandler::retrieve()`: detect when Primary_Command_Field contains `LIST` (case-insensitive) and return a new `RetrieveResult::ShowList` variant containing all history entries
  - [ ] 22.2 Add `RetrieveResult::ShowList { entries: Vec<String> }` variant
  - [ ] 22.3 In `ff-desktop` shell: when `RetrieveResult::ShowList` is received, render a modal history-list overlay anchored to the command field
  - [ ] 22.4 Implement list selection: clicking or Enter on an entry populates the command field without executing; Escape clears the field and closes the list
  - [ ] 22.5 Ensure `LIST` is not added to Command_History when used as the RETRIEVE trigger
  - [ ] 22.6 Write unit tests: LIST+RETRIEVE returns ShowList with all history entries in order, empty history shows empty-state, selection populates field, LIST not recorded in history
  - Covers: Requirement 19 (AC 19.1–19.7)

---

## Phase AN — Key Configuration Dialog (Req 20)

- [ ] 23. Add `KeyModifier` enum and `ModifiedKey` struct to `ff-keys`
  - [ ] 23.1 Define `KeyModifier` enum with variants `None`, `Shift`, `Ctrl`, `Alt` in `function_key.rs`
  - [ ] 23.2 Define `ModifiedKey { key: FunctionKey, modifier: KeyModifier }` struct with `PartialOrd`, `Ord`, `Hash`, `Serialize`, `Deserialize`
  - [ ] 23.3 Implement `ModifiedKey::plain(key)` constructor and `ModifiedKey::ALL` constant (96 entries)
  - [ ] 23.4 Implement TOML key name parsing: `F1`–`F24` → `None`, `SF1`–`SF24` → `Shift`, `CF1`–`CF24` → `Ctrl`, `AF1`–`AF24` → `Alt`
  - [ ] 23.5 Implement `Display` for `ModifiedKey` producing the canonical TOML key name (e.g., `SF3`, `CF12`)
  - [ ] 23.6 Write unit tests: parse all 96 key names round-trip, reject invalid prefixes, `Display` matches parse
  - Covers: Requirement 20.11, 20.12

- [ ] 24. Add `description` field to `KeyBinding` and update `KeyMap` to use `ModifiedKey`
  - [ ] 24.1 Add `description: Option<String>` field to `KeyBinding`; update `new()`, `with_label()`, add `with_description()` and `with_label_and_description()` constructors
  - [ ] 24.2 Update `KeyMap` internal `HashMap` key from `FunctionKey` to `ModifiedKey`
  - [ ] 24.3 Update `KeyMap::get()` to accept `ModifiedKey`; add `get_plain(key: FunctionKey)` convenience method
  - [ ] 24.4 Update `KeyMap::from_toml_table()` to parse all four modifier prefixes and the `description` field
  - [ ] 24.5 Update `KeyMap::default_global()` to use `ModifiedKey::plain(...)` keys (no behaviour change)
  - [ ] 24.6 Update `KeyLabelBarModel` to use `get_plain()` (label bar shows only plain bindings — no change to label bar behaviour)
  - [ ] 24.7 Update all existing tests that construct `KeyMap` entries to use `ModifiedKey::plain(...)` or the updated API
  - [ ] 24.8 Write new unit tests: modifier bindings stored and retrieved independently, description field round-trips through TOML, plain binding unaffected by modifier binding on same key
  - Covers: Requirement 20.3, 20.9, 20.11, 20.12

- [ ] 25. Update `KeyMapResolver` for `ModifiedKey`
  - [ ] 25.1 Update `active_key_map().get(modified_key)` call sites in `ff-desktop` shell to pass `ModifiedKey`
  - [ ] 25.2 Confirm `KeyMapResolver` itself needs no structural change (it holds `KeyMap` which now uses `ModifiedKey` internally)
  - [ ] 25.3 Write unit tests: context map with modifier binding resolves correctly; modifier binding does not affect plain binding resolution
  - Covers: Requirement 20.10, 20.12

- [ ] 26. Modifier key dispatch in `ff-desktop` shell
  - [ ] 26.1 In `shell.rs` `update()` loop, read `egui::Modifiers` alongside function key events
  - [ ] 26.2 Construct `ModifiedKey { key, modifier }` from the pressed key + active modifiers
  - [ ] 26.3 Look up `ModifiedKey` in `resolver.active_key_map()` and dispatch if assigned
  - [ ] 26.4 Write unit tests: Shift+F3 dispatches Shift binding, plain F3 dispatches plain binding, unassigned modifier is no-op
  - Covers: Requirement 20.10

- [ ] 27. Create `key_config_dialog.rs` in `ff-desktop`
  - [ ] 27.1 Define `KeyConfigDialog` struct with fields: `open`, `active_scope`, `staged_global`, `staged_contexts`, `original_global`, `original_contexts`
  - [ ] 27.2 Define `ScopeTab` enum: `Default` and `Context(String)`
  - [ ] 27.3 Implement `KeyConfigDialog::new(resolver: &KeyMapResolver)` — clones current global and all context maps as staged and original copies
  - [ ] 27.4 Implement `render()` method: scope selector tabs (Default + one per context name), scrollable grid per tab
  - [ ] 27.5 Implement grid: 10-column `egui::Grid` with rows F1–F24; each row shows Key (read-only), Command, Label (read-only derived), Description, Shift Cmd, Shift Desc, Ctrl Cmd, Ctrl Desc, Alt Cmd, Alt Desc
  - [ ] 27.6 Implement Save: serialise staged maps to TOML and write via `config_handle.set_user_value`; close dialog
  - [ ] 27.7 Implement Cancel: discard staged maps; close dialog
  - [ ] 27.8 Implement Reset to Defaults per tab: restore Default tab to `KeyMap::default_global()`; clear context tab to empty map
  - [ ] 27.9 Write unit tests: `new()` pre-populates from resolver, staged edits do not affect originals, save produces correct TOML key names for all four modifier variants, cancel leaves resolver unchanged, reset restores defaults
  - Covers: Requirement 20.2, 20.3, 20.5, 20.6, 20.7, 20.8, 20.14, 20.15

- [ ] 28. Wire `KEYS` command and menu item in `ff-desktop` shell
  - [ ] 28.1 Add `KEYS` as a recognised shell-level command intercept in `handle_command()`; set `key_config_dialog.open = true`
  - [ ] 28.2 Add `Edit > Key Assignments…` menu item wired to open the dialog
  - [ ] 28.3 Call `key_config_dialog.render(ui, &resolver, &config_handle)` in the shell `update()` loop when `open == true`
  - [ ] 28.4 Write unit tests: `KEYS` command recognised as shell intercept, dialog `open` flag set on command
  - Covers: Requirement 20.1

- [ ] 29. Validation in Key Configuration Dialog
  - [ ] 29.1 On focus-loss from a Command field: if text is non-empty and non-whitespace, accept; if empty or whitespace, treat as unassigned (clear the binding in staged map)
  - [ ] 29.2 Display a subtle inline indicator (e.g., greyed-out placeholder text "unassigned") for empty command fields
  - [ ] 29.3 Write unit tests: empty command string clears binding in staged map, non-empty string updates staged map, whitespace-only treated as empty
  - Covers: Requirement 20.4

- [ ] 30. Property-based tests for `ModifiedKey` and extended `KeyMap`
  - [ ] 30.1 PBT: All 96 `ModifiedKey` TOML name strings parse back to the original `ModifiedKey` (round-trip)
  - [ ] 30.2 PBT: Modifier bindings never interfere with plain bindings — for any `KeyMap`, `get_plain(F)` always returns the `None`-modifier entry regardless of what Shift/Ctrl/Alt entries exist for the same key
  - [ ] 30.3 PBT: `KeyMap::from_toml_table` with mixed modifier entries produces exactly the expected set of `ModifiedKey` entries with no cross-contamination
  - Covers: Requirement 20.9, 20.11, 20.12

---

## Phase AQ -- Key Map TOML Persistence (Req 20.8)

- [x] 31. Implement save_to_config() in KeyConfigDialog
  - [x] 31.1 Add to_config_table(source) to ScopeRows -- converts staged rows to ConfigValue::Table using canonical TOML key names (F3, SF3, CF3, AF3)
  - [x] 31.2 Add save_to_config(config: &ConfigHandle) to KeyConfigDialog -- writes global_key_map and context_key_maps.<name> via config_handle.set_user_value
  - [x] 31.3 Update render_if_open and render signatures to accept &ConfigHandle; wire Save button to call save_to_config
  - [x] 31.4 Update shell.rs call site to pass &self.config_handle
  - [x] 31.5 Write unit tests: save_produces_correct_config_values_for_global_scope, save_produces_correct_config_key_for_context_scope, empty_context_scope_produces_empty_table
  - Covers: Requirement 20.8

---

## Phase AR -- [context_key_maps] TOML Config Parsing (Req 14.7)

- [x] 32. Parse [context_key_maps] from workbench config into KeyMapResolver at startup
  - [x] 32.1 Add load_context_maps_from_config(config, resolver) helper in ff-desktop shell.rs
  - [x] 32.2 Add config_value_to_toml_value() converter (handles non_exhaustive ConfigValue)
  - [x] 32.3 Call load_context_maps_from_config in WorkbenchShell::new() after building resolver
  - [x] 32.4 Write unit test: context_key_maps_parsed_from_config_value_table (editor + pom contexts, full-replacement, unknown-context fallback)
  - [x] 32.5 Write unit test: context_key_maps_invalid_key_skipped (F99 produces warning, F3 loaded)
  - Covers: Requirement 14.7
