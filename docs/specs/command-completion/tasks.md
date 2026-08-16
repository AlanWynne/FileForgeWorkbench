# Implementation Plan: Command Completion (`ff-completion`)

## Overview

This plan covers the complete implementation of the `ff-completion` crate — the auto-complete subsystem for FileForgeWorkbench's primary command field and line-command prefix area. The completion engine provides context-sensitive suggestions sourced from the command registry, VFS, macro engine, and extensible providers. It supports prefix and fuzzy matching, configurable trigger behaviour, intelligent popup positioning, and full keyboard navigation.

This is a **Wave 10 (Extensions and Macros)** sub-project. It depends on `ff-command` (command registry, metadata), `ff-vfs` (file path completion), `ff-lua-macro` (macro name completion), `ff-config` (configuration settings), and `ff-logging` (diagnostics).

---

## Tasks

- [ ] 1. Crate scaffolding and module structure
  - [ ] 1.1 Create `crates/ff-completion/Cargo.toml` with dependencies (ff-command, ff-vfs, ff-config, ff-logging, thiserror, serde, tokio, proptest dev-dep)
  - [ ] 1.2 Create `crates/ff-completion/src/lib.rs` with module declarations and public API re-exports
  - [ ] 1.3 Create module files: `candidate.rs`, `context.rs`, `provider.rs`, `registry.rs`, `engine.rs`, `matching.rs`, `popup.rs`, `navigation.rs`, `config.rs`, `error.rs`
  - [ ] 1.4 Add `ff-completion` to workspace `Cargo.toml` members list
  - Covers: Structural foundation for all requirements

- [ ] 2. CompletionCandidate model
  - [ ] 2.1 Define `CompletionCandidate` struct with fields: label (String), insertion_value (String), category (Option<String>), description (Option<String>), icon_ref (Option<String>), sort_priority (u32)
  - [ ] 2.2 Define `CandidateKind` enum: CommandName, FilePath, Directory, Keyword, MacroName, LineCommand, Custom
  - [ ] 2.3 Implement `Display`, `Debug`, `Clone`, `PartialEq`, `Eq` for `CompletionCandidate`
  - [ ] 2.4 Implement `Ord` / `PartialOrd` based on sort_priority then label for default ordering
  - [ ] 2.5 Write unit tests for candidate construction, ordering, and kind discrimination
  - Covers: Requirement 1 (AC 1.3), Requirement 2 (all), Requirement 7 (AC 7.2)

- [ ] 3. CompletionContext model
  - [ ] 3.1 Define `CompletionField` enum: PrimaryCommand, PrefixArea
  - [ ] 3.2 Define `CompletionContext` struct with fields: field (CompletionField), typed_text (String), cursor_position (usize), parsed_command_id (Option<String>), argument_index (Option<usize>)
  - [ ] 3.3 Implement `CompletionContext::is_command_position()` — true when cursor is in the first token
  - [ ] 3.4 Implement `CompletionContext::is_argument_position()` — true when cursor is after command name
  - [ ] 3.5 Implement builder pattern for test construction
  - [ ] 3.6 Write unit tests for context classification (command vs argument position, primary vs prefix field)
  - Covers: Requirement 1 (AC 1.1), Requirement 2 (AC 2.1), Requirement 7 (AC 7.1)

- [ ] 4. CompletionProvider trait and ProviderRegistry
  - [ ] 4.1 Define `CompletionProvider` trait with method `fn provide_candidates(&self, context: &CompletionContext) -> Result<Vec<CompletionCandidate>, CompletionError>` and async variant
  - [ ] 4.2 Define `ProviderRegistration` struct: provider (Box<dyn CompletionProvider>), target_commands (Vec<String> or wildcard), argument_patterns (Vec<String>)
  - [ ] 4.3 Implement `ProviderRegistry` struct with thread-safe storage (`RwLock<Vec<ProviderRegistration>>`)
  - [ ] 4.4 Implement `register_provider(registration: ProviderRegistration) -> Result<ProviderId, CompletionError>`
  - [ ] 4.5 Implement `deregister_provider(id: ProviderId)` for plugin cleanup
  - [ ] 4.6 Implement `find_providers(context: &CompletionContext) -> Vec<&dyn CompletionProvider>` returning applicable providers for a context
  - [ ] 4.7 Write unit tests for registration, deregistration, and context-based provider lookup
  - Covers: Requirement 10 (AC 10.1, 10.2, 10.3, 10.6)

- [ ] 5. Matching algorithms — prefix and fuzzy
  - [ ] 5.1 Implement `prefix_match(query: &str, candidate: &str, case_sensitive: bool) -> bool` — returns true if candidate starts with query
  - [ ] 5.2 Implement `fuzzy_match(query: &str, candidate: &str, case_sensitive: bool) -> Option<FuzzyMatchResult>` — returns matched positions and score if all query chars appear in order
  - [ ] 5.3 Define `FuzzyMatchResult` struct: matched_positions (Vec<usize>), score (u32), contiguity_bonus (u32)
  - [ ] 5.4 Implement fuzzy scoring: higher for consecutive matches, higher for match at word start, higher for shorter candidates
  - [ ] 5.5 Implement `MatchingMode` enum: Prefix, Fuzzy — dispatches to appropriate algorithm
  - [ ] 5.6 Write unit tests for prefix matching (case variations, empty strings, exact match)
  - [ ] 5.7 Write unit tests for fuzzy matching (subsequence, scoring, non-match, edge cases)
  - Covers: Requirement 1 (AC 1.2), Requirement 6 (AC 6.1, 6.2, 6.4, 6.6)

- [ ] 6. CompletionList model and filtering
  - [ ] 6.1 Define `CompletionList` struct: all_candidates (Vec<CompletionCandidate>), filtered (Vec<usize>), matching_mode (MatchingMode), case_sensitive (bool)
  - [ ] 6.2 Implement `CompletionList::filter(query: &str)` — re-filters all_candidates against query, updating filtered indices
  - [ ] 6.3 Implement ranking: exact prefix matches first, then shorter names, then frequency-weighted (sort_priority field)
  - [ ] 6.4 Implement fuzzy ranking: contiguity_bonus first, then start-of-word bonus, then candidate length
  - [ ] 6.5 Implement `CompletionList::is_empty()` — true if filtered list has zero items
  - [ ] 6.6 Implement `CompletionList::get(index: usize) -> Option<&CompletionCandidate>` for indexed access into filtered view
  - [ ] 6.7 Implement de-duplication by insertion_value when merging from multiple providers
  - [ ] 6.8 Write unit tests for filtering, ranking, de-duplication, and empty-list detection
  - Covers: Requirement 1 (AC 1.4, 1.6, 1.7), Requirement 2 (AC 2.7), Requirement 6 (AC 6.4)

- [ ] 7. CompletionEngine — core orchestrator
  - [ ] 7.1 Define `CompletionEngine` struct holding: ProviderRegistry, CompletionConfig, current active CompletionSession (Option)
  - [ ] 7.2 Define `CompletionSession` struct: context (CompletionContext), list (CompletionList), anchor_position (usize), is_active (bool)
  - [ ] 7.3 Implement `trigger(context: CompletionContext) -> Result<CompletionSession, CompletionError>` — creates session, invokes providers, filters
  - [ ] 7.4 Implement provider invocation — gather candidates from all applicable providers, merge, de-duplicate, rank
  - [ ] 7.5 Implement provider error isolation — catch panics/errors per provider, log WARN, continue with remaining providers
  - [ ] 7.6 Implement `update_filter(new_text: &str)` — re-filter active session dynamically as user types
  - [ ] 7.7 Implement auto-hide when filtered list becomes empty
  - [ ] 7.8 Implement `accept(index: usize) -> AcceptResult` — returns insertion text and cursor adjustment
  - [ ] 7.9 Implement `dismiss()` — close active session without accepting
  - [ ] 7.10 Write unit tests for trigger, filter update, accept, dismiss, and provider error isolation
  - Covers: Requirement 1 (AC 1.5, 1.6, 1.7), Requirement 2 (AC 2.7, 2.8), Requirement 5 (AC 5.1, 5.4), Requirement 10 (AC 10.4, 10.5)

- [ ] 8. Built-in provider — Command Name completion
  - [ ] 8.1 Implement `CommandNameProvider` struct implementing `CompletionProvider`
  - [ ] 8.2 Query `CommandRegistry::list_all()` for all registered commands on trigger
  - [ ] 8.3 Build `CompletionCandidate` from `CommandMetadata`: label=command_name, insertion_value=canonical_name, category=metadata.category, description=metadata.display_name
  - [ ] 8.4 Apply frequency weighting from CommandHistory (if available) to sort_priority
  - [ ] 8.5 Set insertion behaviour: replace prefix with canonical uppercase command name, append trailing space
  - [ ] 8.6 Write unit tests with mock CommandRegistry returning known command sets
  - Covers: Requirement 1 (AC 1.1, 1.2, 1.3, 1.4, 1.5)

- [ ] 9. Built-in provider — File Path completion
  - [ ] 9.1 Implement `FilePathProvider` struct implementing `CompletionProvider`
  - [ ] 9.2 Parse typed text as path prefix — detect bare paths vs Resource_URI (`vfs://provider/path`) format
  - [ ] 9.3 Implement async VFS directory listing query via VFS abstraction layer
  - [ ] 9.4 Build candidates: label=filename, insertion_value=full_path, kind=FilePath|Directory, description=parent_path
  - [ ] 9.5 Mark directory candidates with trailing separator to indicate further completion available
  - [ ] 9.6 Ensure VFS query is non-blocking (async/await, no UI thread blocking)
  - [ ] 9.7 Write unit tests with mock VFS provider returning known directory structures
  - Covers: Requirement 2 (AC 2.2, 2.3)

- [ ] 10. Built-in provider — Keyword/Modifier completion
  - [ ] 10.1 Implement `KeywordProvider` struct implementing `CompletionProvider`
  - [ ] 10.2 Define static keyword sets for known commands (FIND modifiers: CHARS/PREFIX/SUFFIX/WORD, scope modifiers: VISIBLE/EXCLUDED/ALL, etc.)
  - [ ] 10.3 Implement argument schema lookup — determine which keyword set applies at the current argument position
  - [ ] 10.4 Build candidates from the applicable keyword set
  - [ ] 10.5 Write unit tests for keyword resolution at various argument positions
  - Covers: Requirement 2 (AC 2.4)

- [ ] 11. Built-in provider — Macro Name completion
  - [ ] 11.1 Implement `MacroNameProvider` struct implementing `CompletionProvider`
  - [ ] 11.2 Query Lua macro engine for all registered macro names
  - [ ] 11.3 Build candidates: label=macro_name (no extension), insertion_value=macro_name, description=macro_file_path and metadata description
  - [ ] 11.4 Implement cache invalidation — refresh macro list when engine emits add/remove/reload notifications
  - [ ] 11.5 Return empty list (no popup) when no macros are registered
  - [ ] 11.6 Write unit tests with mock macro engine returning known macro sets and empty sets
  - Covers: Requirement 8 (AC 8.1, 8.2, 8.3, 8.4, 8.5)

- [ ] 12. Built-in provider — Line Command completion
  - [ ] 12.1 Implement `LineCommandProvider` struct implementing `CompletionProvider`
  - [ ] 12.2 Define the complete line command kind set (C, CC, M, MM, D, DD, R, RR, X, XX, I, A, B, O, W, S, T, TT, U, UU, >, >>, <, <<, ), )), (, (( ) with descriptions
  - [ ] 12.3 Build candidates: label=command_kind, insertion_value=kind, description=action_description
  - [ ] 12.4 Implement numeric count preservation — when accepting, preserve any numeric suffix already typed
  - [ ] 12.5 Only activate when `completion.line_command_completion` is true
  - [ ] 12.6 Write unit tests for line command filtering, acceptance with numeric suffix, and config disable
  - Covers: Requirement 7 (AC 7.1, 7.2, 7.3, 7.4, 7.5, 7.6)

- [ ] 13. Popup positioning model
  - [ ] 13.1 Define `PopupAnchor` struct: x (f32), y (f32), field_height (f32)
  - [ ] 13.2 Define `PopupBounds` struct: x (f32), y (f32), width (f32), height (f32)
  - [ ] 13.3 Define `ViewportBounds` struct representing the application window dimensions
  - [ ] 13.4 Implement `compute_popup_position(anchor: PopupAnchor, item_count: usize, config: &PopupConfig, viewport: ViewportBounds) -> PopupBounds`
  - [ ] 13.5 Implement default placement below command field (top edge adjacent to bottom of field)
  - [ ] 13.6 Implement flip-above logic when below placement extends beyond viewport bottom
  - [ ] 13.7 Implement best-fit fallback when both above and below extend beyond viewport — choose direction with more space, clip with scrolling
  - [ ] 13.8 Implement width calculation: at least longest visible label width, bounded by `popup_max_width`, truncate with ellipsis
  - [ ] 13.9 Implement height calculation: up to `popup_max_items` rows, scroll if more candidates
  - [ ] 13.10 Implement reposition on viewport resize (recompute from anchor)
  - [ ] 13.11 Ensure popup never overlaps command field text
  - [ ] 13.12 Write unit tests for below placement, flip-above, best-fit, width bounds, and resize recomputation
  - Covers: Requirement 3 (AC 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8)

- [ ] 14. Selection and navigation state
  - [ ] 14.1 Define `SelectionState` struct: selected_index (usize), page_size (usize), total_items (usize), wrap_enabled (bool)
  - [ ] 14.2 Implement `move_down()` — advance index by 1, wrap from last to first if wrap_enabled, else clamp
  - [ ] 14.3 Implement `move_up()` — retreat index by 1, wrap from first to last if wrap_enabled, else clamp
  - [ ] 14.4 Implement `page_down()` — advance index by page_size items, clamp at end
  - [ ] 14.5 Implement `page_up()` — retreat index by page_size items, clamp at start
  - [ ] 14.6 Implement `selected_candidate() -> Option<&CompletionCandidate>` — retrieve the currently highlighted item
  - [ ] 14.7 Implement `reset(new_total: usize)` — reset selection to index 0 when list is re-filtered
  - [ ] 14.8 Write unit tests for navigation (wrap/no-wrap), page movement, boundary clamping, and reset
  - Covers: Requirement 4 (AC 4.1, 4.2, 4.8)

- [ ] 15. Keyboard interaction — accept, dismiss, and special chars
  - [ ] 15.1 Define `CompletionAction` enum: Accept, Dismiss, MoveDown, MoveUp, PageDown, PageUp, StopChar(char), FillUpChar(char), Continue
  - [ ] 15.2 Implement `resolve_key_event(key: KeyEvent, config: &CompletionConfig, session: &CompletionSession) -> CompletionAction`
  - [ ] 15.3 Implement Tab key → Accept currently highlighted candidate (replace prefix, dismiss popup)
  - [ ] 15.4 Implement Enter key → Accept candidate AND submit command if cursor at end with no further args expected
  - [ ] 15.5 Implement Escape key → Dismiss popup without modification
  - [ ] 15.6 Implement Stop_Char detection — dismiss popup on configurable characters (default: space, semicolon)
  - [ ] 15.7 Implement Fill_Up_Char detection — accept candidate then insert the fill-up char
  - [ ] 15.8 Implement `choose_single` behaviour — auto-accept lone match without showing popup
  - [ ] 15.9 Write unit tests for all key actions, stop chars, fill-up chars, and choose_single
  - Covers: Requirement 4 (AC 4.3, 4.4, 4.5, 4.6, 4.7, 4.9, 4.10)

- [ ] 16. Dismiss behaviour logic
  - [ ] 16.1 Implement dismiss on Escape key press
  - [ ] 16.2 Implement dismiss on field focus loss
  - [ ] 16.3 Implement dismiss when cursor retreats past anchor position (`cancel_at_start_pos` config)
  - [ ] 16.4 Implement dismiss when filter produces zero matches (`auto_hide` config)
  - [ ] 16.5 Implement dismiss on command submission (Enter to execute)
  - [ ] 16.6 Implement dismiss-and-retrigger when context changes (different argument position)
  - [ ] 16.7 Write unit tests for each dismiss condition and configuration toggles
  - Covers: Requirement 5 (AC 5.1, 5.2, 5.3, 5.4, 5.5, 5.6)

- [ ] 17. Insertion logic
  - [ ] 17.1 Define `AcceptResult` struct: inserted_text (String), cursor_offset (usize), prefix_start (usize), prefix_end (usize)
  - [ ] 17.2 Implement prefix replacement — replace only the prefix portion used for filtering, preserve text after cursor
  - [ ] 17.3 Implement `drop_rest_of_word` mode — when enabled, remove text after cursor up to next word boundary before insertion
  - [ ] 17.4 Implement command name insertion: replace with canonical uppercase form, append trailing space
  - [ ] 17.5 Implement file path insertion: replace typed path prefix with selected path
  - [ ] 17.6 Implement line command insertion: replace prefix area content, preserve numeric count
  - [ ] 17.7 Write unit tests for insertion at various cursor positions, with and without trailing text, and drop_rest_of_word
  - Covers: Requirement 1 (AC 1.5), Requirement 4 (AC 4.10), Requirement 7 (AC 7.4), Requirement 9 (AC 9.1 — `drop_rest_of_word`)

- [ ] 18. Trigger behaviour and activation control
  - [ ] 18.1 Define `TriggerMode` enum: Manual, Automatic, Both
  - [ ] 18.2 Implement manual trigger — activate only on explicit shortcut (Ctrl+Space / configurable)
  - [ ] 18.3 Implement automatic trigger — activate after `auto_trigger_chars` consecutive typed characters
  - [ ] 18.4 Implement `Both` mode — automatic threshold AND manual shortcut both active
  - [ ] 18.5 Implement trigger shortcut registration as Command_ID `"completion.trigger"` in Shortcut_Registry
  - [ ] 18.6 Write unit tests for each trigger mode, threshold counting, and manual override
  - Covers: Requirement 9 (AC 9.2, 9.3, 9.4, 9.7)

- [ ] 19. Configuration integration
  - [ ] 19.1 Define `CompletionConfig` struct with all configurable fields from Requirement 9 (trigger_mode, auto_trigger_chars, matching_mode, case_sensitive, popup_max_items, popup_max_width, auto_hide, cancel_at_start_pos, choose_single, wrap_navigation, stop_chars, fill_up_chars, line_command_completion, drop_rest_of_word)
  - [ ] 19.2 Implement `CompletionConfig::from_config_system(config: &ConfigurationSystem)` loading from `completion.*` namespace
  - [ ] 19.3 Implement validation with defaults — invalid/out-of-range values fall back to defaults with WARN log
  - [ ] 19.4 Implement range clamping: popup_max_items [3, 50], popup_max_width [100, 1000], auto_trigger_chars [1, 10]
  - [ ] 19.5 Implement matching_mode validation — accept only "prefix" or "fuzzy", fallback to "prefix"
  - [ ] 19.6 Implement hot-reload listener — re-read config on Configuration_System change notification
  - [ ] 19.7 Write unit tests for default loading, validation fallbacks, clamping, and hot-reload
  - Covers: Requirement 9 (AC 9.1, 9.5, 9.6), Requirement 6 (AC 6.5)

- [ ] 20. Fuzzy match highlight positions
  - [ ] 20.1 Implement `highlight_positions(query: &str, candidate_label: &str) -> Vec<usize>` returning character indices to highlight
  - [ ] 20.2 Integrate highlight data into `CompletionCandidate` display metadata when fuzzy mode is active
  - [ ] 20.3 Write unit tests for highlight position correctness with various fuzzy matches
  - Covers: Requirement 6 (AC 6.3)

- [ ] 21. Command registration for built-in providers
  - [ ] 21.1 Register `CommandNameProvider` as a built-in provider on engine initialization
  - [ ] 21.2 Register `FilePathProvider` as a built-in provider for commands expecting file path arguments
  - [ ] 21.3 Register `KeywordProvider` as a built-in provider for commands with known keyword arguments
  - [ ] 21.4 Register `MacroNameProvider` as a built-in provider for macro invocation contexts
  - [ ] 21.5 Register `LineCommandProvider` as a built-in provider for prefix-area contexts
  - [ ] 21.6 Verify all built-in providers use the same `CompletionProvider` trait as plugin providers
  - [ ] 21.7 Write integration test verifying all built-in providers are registered and discoverable
  - Covers: Requirement 10 (AC 10.6)

- [ ] 22. Error types
  - [ ] 22.1 Define `CompletionError` enum with variants: ProviderFailed { provider_id, source }, NoProviders, ConfigInvalid { key, value, fallback }, VfsQueryFailed(source), SessionNotActive, InvalidContext
  - [ ] 22.2 Implement `Display` and `thiserror::Error` derives with descriptive messages
  - [ ] 22.3 Write unit tests for error display output
  - Covers: All requirements (error paths)

- [ ] 23. Thread safety and async validation
  - [ ] 23.1 Write test verifying `CompletionEngine`, `ProviderRegistry` implement `Send + Sync`
  - [ ] 23.2 Write async test — concurrent provider invocation does not deadlock
  - [ ] 23.3 Write test — VFS file path provider query is non-blocking (returns future, does not block caller)
  - [ ] 23.4 Write test — provider panic isolation (panicking provider does not crash engine)
  - Covers: Design Principle 4 (Non-blocking), Requirement 10 (AC 10.4, 10.5)

- [ ] 24. Property-based tests
  - [ ] 24.1 Write PBT: Prefix match correctness property
  - [ ] 24.2 Write PBT: Fuzzy match subsequence property
  - [ ] 24.3 Write PBT: Fuzzy scoring monotonicity property
  - [ ] 24.4 Write PBT: Navigation wrap-around property
  - [ ] 24.5 Write PBT: CompletionList filter idempotence property
  - [ ] 24.6 Write PBT: Popup positioning within viewport property
  - [ ] 24.7 Write PBT: Configuration clamping property
  - [ ] 24.8 Write PBT: De-duplication invariant property
  - [ ] 24.9 Write PBT: Insertion preserves trailing text property
  - Covers: All requirements (property-based validation)

- [ ] 25. Integration tests
  - [ ] 25.1 Write integration test: full command name completion flow (trigger → filter → accept → verify insertion)
  - [ ] 25.2 Write integration test: argument completion with keyword provider
  - [ ] 25.3 Write integration test: line command completion in prefix area
  - [ ] 25.4 Write integration test: dynamic re-filter as user types additional characters
  - [ ] 25.5 Write integration test: dismiss behaviours (escape, focus loss, empty matches)
  - [ ] 25.6 Write integration test: plugin provider registration and candidate merging
  - [ ] 25.7 Write integration test: config hot-reload updates engine behaviour
  - Covers: All requirements (end-to-end validation)

---

## Property-Based Test Definitions

### Property 1: Prefix Match Correctness

**Validates: Requirement 1.2, Requirement 6.2**

- **Statement:** For any query string `q` and candidate string `c`, `prefix_match(q, c, false)` returns true if and only if `c.to_lowercase().starts_with(q.to_lowercase())`.
- **Strategy:** Generate:
  - Queries: strings of length 0–20, alphanumeric + dots + underscores
  - Candidates: strings of length 0–50, alphanumeric + dots + underscores + mixed case
- **Invariant:** `prefix_match(q, c, false) ⟺ c.to_lowercase().starts_with(&q.to_lowercase())`

### Property 2: Fuzzy Match Subsequence

**Validates: Requirement 6.1**

- **Statement:** For any query `q` and candidate `c`, `fuzzy_match(q, c, false)` returns Some if and only if all characters of `q` (case-insensitive) appear in `c` in the same order (not necessarily consecutively).
- **Strategy:** Generate:
  - Valid cases: take a candidate string, select a random subsequence of its characters as the query
  - Invalid cases: append a character not in the candidate to the query
- **Invariant:** `fuzzy_match(q, c, false).is_some() ⟺ q is a case-insensitive subsequence of c`

### Property 3: Fuzzy Scoring Monotonicity

**Validates: Requirement 6.4**

- **Statement:** For any candidate `c` and two queries `q1`, `q2` where `q1` is a prefix of `q2`, if both match `c`, then `fuzzy_match(q2, c).score >= fuzzy_match(q1, c).score` (longer matching queries produce equal or higher scores due to more contiguous character coverage).
- **Strategy:** Generate:
  - Candidates: strings of length 5–30
  - q1: random subsequence of c (length 1–5)
  - q2: q1 extended with one additional character from c (maintaining subsequence order)
- **Invariant:** If both match, `score(q2, c) >= score(q1, c)`

### Property 4: Navigation Wrap-Around

**Validates: Requirement 4.1, 4.2**

- **Statement:** For a `SelectionState` with `total_items = N` and `wrap_enabled = true`, calling `move_down()` from index `N-1` yields index `0`, and calling `move_up()` from index `0` yields index `N-1`. With `wrap_enabled = false`, these calls clamp at the boundary.
- **Strategy:** Generate:
  - total_items: integer in [1, 100]
  - starting_index: integer in [0, total_items - 1]
  - wrap_enabled: boolean
  - operation sequence: 1–50 random move_up/move_down operations
- **Invariant:** After all operations, `selected_index ∈ [0, total_items - 1]` AND wrap semantics are respected

### Property 5: CompletionList Filter Idempotence

**Validates: Requirement 1.6**

- **Statement:** For any `CompletionList` and query `q`, calling `filter(q)` twice with the same query produces the same filtered result.
- **Strategy:** Generate:
  - Candidate lists: 1–50 candidates with random labels
  - Query: random string of length 0–10
- **Invariant:** `list.filter(q); let r1 = list.filtered.clone(); list.filter(q); r1 == list.filtered`

### Property 6: Popup Positioning Within Viewport

**Validates: Requirement 3.2, 3.3, 3.4, 3.5**

- **Statement:** For any valid `PopupAnchor`, item count, and `ViewportBounds`, the computed `PopupBounds` shall always be fully contained within the viewport (no coordinate extends beyond viewport edges), and shall not overlap the command field anchor area.
- **Strategy:** Generate:
  - Viewport: width [200, 2000], height [200, 2000]
  - Anchor: x in [0, viewport.width], y in [20, viewport.height - 20], field_height in [16, 40]
  - Item count: integer in [1, 50]
  - popup_max_items: integer in [3, 50]
  - popup_max_width: integer in [100, 1000]
- **Invariant:** `popup.x >= 0 && popup.x + popup.width <= viewport.width && popup.y >= 0 && popup.y + popup.height <= viewport.height && !overlaps_field(popup, anchor)`

### Property 7: Configuration Clamping

**Validates: Requirement 9.1, 9.5**

- **Statement:** For any input configuration values, the effective `CompletionConfig` shall clamp `popup_max_items` to [3, 50], `popup_max_width` to [100, 1000], and `auto_trigger_chars` to [1, 10]. Invalid `matching_mode` strings shall fall back to `"prefix"`. Invalid `trigger_mode` strings shall fall back to `"both"`.
- **Strategy:** Generate:
  - popup_max_items: i64 in [-100, 200]
  - popup_max_width: i64 in [-100, 5000]
  - auto_trigger_chars: i64 in [-10, 100]
  - matching_mode: random strings including valid ("prefix", "fuzzy") and invalid values
  - trigger_mode: random strings including valid and invalid values
- **Invariant:** Effective config fields are always within valid ranges; invalid enums map to their defaults

### Property 8: De-Duplication Invariant

**Validates: Requirement 2.7**

- **Statement:** When merging candidates from multiple providers, the resulting `CompletionList` shall contain at most one candidate per unique `insertion_value`. If duplicates exist, the candidate with the higher `sort_priority` is retained.
- **Strategy:** Generate:
  - Provider count: 2–5
  - Candidates per provider: 1–20
  - Insertion values drawn from a pool of 5–30 unique values (creating deliberate overlaps)
- **Invariant:** `list.all_candidates.iter().map(|c| &c.insertion_value).collect::<HashSet<_>>().len() == list.all_candidates.len()`

### Property 9: Insertion Preserves Trailing Text

**Validates: Requirement 4.10**

- **Statement:** When a candidate is accepted, text in the command field after the cursor position shall be preserved unchanged (unless `drop_rest_of_word` is enabled). The insertion replaces only the prefix [anchor..cursor].
- **Strategy:** Generate:
  - Full field text: random string of length 5–50
  - Anchor position: integer in [0, len/2]
  - Cursor position: integer in [anchor, len * 3/4]
  - Candidate insertion_value: random string of length 1–20
  - drop_rest_of_word: boolean
- **Invariant (when drop_rest_of_word=false):** `result_text[cursor_after_insert..] == original_text[original_cursor..]`

---

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "label": "Scaffolding", "tasks": ["1"] },
    { "id": 1, "label": "Core Models", "tasks": ["2", "3", "22"], "dependsOn": [0] },
    { "id": 2, "label": "Matching and Provider Infrastructure", "tasks": ["4", "5", "6"], "dependsOn": [1] },
    { "id": 3, "label": "Engine Core", "tasks": ["7", "13", "14"], "dependsOn": [2] },
    { "id": 4, "label": "Keyboard and Trigger", "tasks": ["15", "16", "17", "18"], "dependsOn": [3] },
    { "id": 5, "label": "Built-in Providers", "tasks": ["8", "9", "10", "11", "12"], "dependsOn": [2] },
    { "id": 6, "label": "Configuration and Highlights", "tasks": ["19", "20", "21"], "dependsOn": [4, 5] },
    { "id": 7, "label": "Validation and PBT", "tasks": ["23", "24"], "dependsOn": [6] },
    { "id": 8, "label": "Integration Tests", "tasks": ["25"], "dependsOn": [7] }
  ]
}
```

---

## Notes

- This is a Wave 10 (Extensions and Macros) crate depending on `ff-command` (Wave 2), `ff-vfs` (Wave 3), `ff-config` (Wave 2), `ff-lua-macro` (Wave 10), and `ff-logging` (Wave 0)
- The `ff-lua-macro` dependency is soft — macro name completion gracefully returns an empty list if the macro engine is unavailable or not yet integrated
- The `CompletionProvider` trait is designed for both sync and async usage; file path completion (VFS queries) is the primary async provider, while command names and keywords are synchronous
- The popup positioning model (Task 13) is GUI-independent in its logic — it computes coordinates but does not perform rendering. The egui rendering layer will consume `PopupBounds` to draw the popup widget
- Property-based tests use the `proptest` crate with a minimum of 100 iterations per property
- Thread safety relies on `std::sync::RwLock` and `std::sync::Arc` for the ProviderRegistry and engine state
- The `CommandNameProvider` (Task 8) queries the `CommandRegistry` from `ff-command` — during development, a mock registry can be used until upstream integration is complete
- The line command candidate set (Task 12) is derived from the `line-commands` crate specification; the full set is hardcoded as a constant during initial implementation and will be dynamically sourced once the line-commands crate is available
- Configuration integration (Task 19) uses the `ff-config` crate's `completion.*` namespace — during testing, a mock configuration source provides values
- Plugin provider registration and deregistration (Task 4) follows the same lifecycle pattern as plugin architecture: register during `initialize`, deregister on unload

---

## Acceptance Criteria Coverage Matrix

| Requirement | Criteria | Covered by Task(s) |
|-------------|----------|---------------------|
| Req 1: Command Name Completion | AC 1.1 | Tasks 3, 8 |
| Req 1: Command Name Completion | AC 1.2 | Tasks 5, 8 |
| Req 1: Command Name Completion | AC 1.3 | Tasks 2, 8 |
| Req 1: Command Name Completion | AC 1.4 | Tasks 6, 8 |
| Req 1: Command Name Completion | AC 1.5 | Tasks 7, 8, 17 |
| Req 1: Command Name Completion | AC 1.6 | Tasks 6, 7 |
| Req 1: Command Name Completion | AC 1.7 | Tasks 6, 7 |
| Req 2: Argument Completion | AC 2.1 | Tasks 3, 7 |
| Req 2: Argument Completion | AC 2.2 | Task 9 |
| Req 2: Argument Completion | AC 2.3 | Task 9 |
| Req 2: Argument Completion | AC 2.4 | Task 10 |
| Req 2: Argument Completion | AC 2.5 | Task 11 |
| Req 2: Argument Completion | AC 2.6 | Task 4 |
| Req 2: Argument Completion | AC 2.7 | Tasks 6, 7 |
| Req 2: Argument Completion | AC 2.8 | Task 7 |
| Req 3: Popup Positioning | AC 3.1 | Task 13 |
| Req 3: Popup Positioning | AC 3.2 | Task 13 |
| Req 3: Popup Positioning | AC 3.3 | Task 13 |
| Req 3: Popup Positioning | AC 3.4 | Task 13 |
| Req 3: Popup Positioning | AC 3.5 | Task 13 |
| Req 3: Popup Positioning | AC 3.6 | Task 13 |
| Req 3: Popup Positioning | AC 3.7 | Task 13 |
| Req 3: Popup Positioning | AC 3.8 | Task 13 |
| Req 4: Selection and Navigation | AC 4.1 | Task 14 |
| Req 4: Selection and Navigation | AC 4.2 | Task 14 |
| Req 4: Selection and Navigation | AC 4.3 | Task 15 |
| Req 4: Selection and Navigation | AC 4.4 | Task 15 |
| Req 4: Selection and Navigation | AC 4.5 | Task 15 |
| Req 4: Selection and Navigation | AC 4.6 | Task 15 |
| Req 4: Selection and Navigation | AC 4.7 | Task 15 |
| Req 4: Selection and Navigation | AC 4.8 | Task 14 |
| Req 4: Selection and Navigation | AC 4.9 | Task 15 |
| Req 4: Selection and Navigation | AC 4.10 | Tasks 15, 17 |
| Req 5: Dismiss Behaviour | AC 5.1 | Task 16 |
| Req 5: Dismiss Behaviour | AC 5.2 | Task 16 |
| Req 5: Dismiss Behaviour | AC 5.3 | Task 16 |
| Req 5: Dismiss Behaviour | AC 5.4 | Task 16 |
| Req 5: Dismiss Behaviour | AC 5.5 | Task 16 |
| Req 5: Dismiss Behaviour | AC 5.6 | Task 16 |
| Req 6: Fuzzy Matching | AC 6.1 | Task 5 |
| Req 6: Fuzzy Matching | AC 6.2 | Task 5 |
| Req 6: Fuzzy Matching | AC 6.3 | Task 20 |
| Req 6: Fuzzy Matching | AC 6.4 | Tasks 5, 6 |
| Req 6: Fuzzy Matching | AC 6.5 | Task 19 |
| Req 6: Fuzzy Matching | AC 6.6 | Task 5 |
| Req 7: Line Command Completion | AC 7.1 | Task 12 |
| Req 7: Line Command Completion | AC 7.2 | Tasks 2, 12 |
| Req 7: Line Command Completion | AC 7.3 | Task 12 |
| Req 7: Line Command Completion | AC 7.4 | Tasks 12, 17 |
| Req 7: Line Command Completion | AC 7.5 | Task 12 |
| Req 7: Line Command Completion | AC 7.6 | Task 12 |
| Req 8: Macro Name Completion | AC 8.1 | Task 11 |
| Req 8: Macro Name Completion | AC 8.2 | Task 11 |
| Req 8: Macro Name Completion | AC 8.3 | Task 11 |
| Req 8: Macro Name Completion | AC 8.4 | Task 11 |
| Req 8: Macro Name Completion | AC 8.5 | Task 11 |
| Req 9: Configurable Trigger | AC 9.1 | Task 19 |
| Req 9: Configurable Trigger | AC 9.2 | Task 18 |
| Req 9: Configurable Trigger | AC 9.3 | Task 18 |
| Req 9: Configurable Trigger | AC 9.4 | Task 18 |
| Req 9: Configurable Trigger | AC 9.5 | Task 19 |
| Req 9: Configurable Trigger | AC 9.6 | Task 19 |
| Req 9: Configurable Trigger | AC 9.7 | Task 18 |
| Req 10: Provider Extensibility | AC 10.1 | Task 4 |
| Req 10: Provider Extensibility | AC 10.2 | Task 4 |
| Req 10: Provider Extensibility | AC 10.3 | Task 4 |
| Req 10: Provider Extensibility | AC 10.4 | Tasks 7, 23 |
| Req 10: Provider Extensibility | AC 10.5 | Tasks 7, 23 |
| Req 10: Provider Extensibility | AC 10.6 | Task 21 |
