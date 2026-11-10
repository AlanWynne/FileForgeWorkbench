# af-locale Hang and English-Fallback -- Root-Cause Findings (CR-NR-103)

Read-only investigation. No source was modified, nothing was built, no commits.

## Summary answer (both symptoms)

There are TWO independent defects, in two different crates. They do NOT share a
single root cause, though both sit on the `ui.locale` -> catalogue path.

- **Symptom 1 (screen hangs on selecting `af`): a self-deadlock on the config
  `RwLock`.** `ConfigHandle::set_user_value` holds the config write lock while it
  invokes the reload callbacks; the registered `ui.locale` callback
  (`main.rs`) re-enters the SAME `ConfigHandle` with a read lock
  (`get_string("ui.locale")`). `std::sync::RwLock` is NOT reentrant, so the UI
  thread blocks forever waiting for a read lock it can never get because it still
  holds the write lock. This happens on EVERY `ui.locale` change (and actually on
  any config value written from the panel while this callback is registered), not
  just `af`.

- **Symptom 2 (restart: persisted `af` but everything still English): the
  runtime `i18n/` root cannot be resolved to the shipped catalogue, so the `af`
  overlay is never loaded.** `resolve_i18n_root()` in
  `shell/i18n_bootstrap.rs` searches four exe-relative / data-dir candidates.
  NONE of them point at the only place the catalogues actually live
  (`crates/ff-i18n/i18n/`) when the PRODUCTION binary runs from `target/<profile>/`:
  the dev fallback uses THREE `..` segments, which is calibrated for the TEST
  binary location (`target/<profile>/deps/`), not the one-level-shallower
  production binary. Nothing copies `i18n/` next to the binary (the `build.rs`
  only compiles a Windows manifest; there is no top-level `i18n/` and none under
  `target/`). Result at startup: either the seam is left uninstalled (every `t()`
  returns its key) OR, in whatever layout the user is actually running, the root
  that resolves contains `en/` but the `af/` overlay load fails/falls back to the
  English Identity_Base. Either way, `af` is never applied.

The two defects also reinforce each other operationally: because symptom 1
deadlocks the live swap, the user never sees `af` applied at runtime, and because
symptom 2 breaks the startup load, the persisted `af` is not applied after
restart either -- so the feature appears completely dead for `af`.

---

## Evidence

### Symptom 1 -- the config RwLock self-deadlock

Chain of custody for the lock, newest frame last:

1. The Config panel commits a value through
   `crates/ff-desktop/src/config_panel/commit.rs:35` (`commit_value`), which calls
   `config.set_user_value(key, value)`.

2. `ConfigHandle::set_user_value`
   (`crates/ff-config/src/config_handle/runtime.rs`, approx. lines 161-200)
   writes the TOML file, then:

   ```rust
   // Trigger reload so the effective store picks up the change.
   let mut system = self.inner.write().unwrap();          // <-- WRITE LOCK HELD
   let _ = system
       .manager
       .reload_file(&user_path, crate::layer::ConfigLayer::User);
   ```

   The write lock on `self.inner` (an `RwLock<ConfigSystem>`) is held across the
   `reload_file` call.

3. `ReloadManager::reload_file`
   (`crates/ff-config/src/reload/ops.rs`, Step 6, approx. line 82) invokes the
   callbacks WHILE that write lock is still held:

   ```rust
   self.callbacks.invoke(&event);
   ```

   (`CallbackRegistry::invoke` in `crates/ff-config/src/callback.rs` correctly
   drops its OWN internal mutex before calling callbacks -- that is not the
   problem. The problem is the outer `ConfigSystem` write lock from frame 2.)

4. The registered `ui.locale` callback is the closure in
   `crates/ff-desktop/src/main.rs` (approx. lines 116-128), registered via
   `ff_config::on_locale_reload` (`crates/ff-config/src/locale_reload.rs`). Its
   FIRST action is:

   ```rust
   shell::i18n_bootstrap::install_catalogue(&config_for_reload);
   ```

5. `install_catalogue`
   (`crates/ff-desktop/src/shell/i18n_bootstrap.rs`, approx. line 70) calls:

   ```rust
   let locale_tag = config
       .get_string(ff_config::keys::ui::LOCALE)   // <-- RE-ENTRANT READ LOCK
       .unwrap_or_else(|_| ff_i18n::IDENTITY_BASE_TAG.to_string());
   ```

6. `ConfigHandle::get_string`
   (`crates/ff-config/src/config_handle/mod.rs:155-159`) does
   `let system = self.inner.read().unwrap();` -- a READ lock on the SAME `RwLock`
   the current thread already holds for WRITE (frame 2).

`std::sync::RwLock` does not permit a thread to acquire a read lock while it
holds the write lock; the attempt blocks permanently. The UI thread is the one
that called `set_user_value` (the panel runs on the egui/update thread), so the
whole UI freezes -- exactly "the screen hangs and nothing happens."

Note the closure would deadlock a SECOND time at
`config_for_reload.get_string(ff_config::keys::ui::LOCALE)` for the help-locale
re-resolution (`main.rs` ~line 123) even if the first call were fixed; both
reads happen under the still-held write lock. The help path itself
(`shell/help.rs:423 reload_active_help_registry_for_locale`) takes the locale as
a plain `&str` argument and is not implicated -- the hang is upstream of it.

Why the file-watcher path does not save it: hot-reload is enabled by default
(`ConfigInitOptions::enable_hot_reload = true`,
`crates/ff-config/src/init/mod.rs`). The watcher would also fire the same
callback on the file change, but from a watcher thread -- however the UI thread
has already deadlocked inside `set_user_value` before returning, so the user sees
the freeze regardless.

### Symptom 2 -- runtime `i18n/` root not resolved to the `af` catalogue

`resolve_i18n_root()`
(`crates/ff-desktop/src/shell/i18n_bootstrap.rs`, approx. lines 37-58) builds
these candidates, first existing wins:

1. `exe_dir/i18n`
2. `exe_dir/../../i18n`
3. `exe_dir/../../../crates/ff-i18n/i18n`
4. `dirs::data_dir()/FileForgeWorkbench/i18n`

The catalogues exist ONLY at `crates/ff-i18n/i18n/` (verified: it contains
`af`, `de`, `en`, `fr`). Observed on disk during this investigation:

- `crates/ff-i18n/i18n/{af,de,en,fr}` exist.
- There is NO top-level `FileForgeWorkbench/i18n/` (`Test-Path` returned False).
- There is NO `i18n/` anywhere under `target/` (recursive search returned empty)
  -- so nothing stages the catalogue next to the binary.
- `crates/ff-desktop/build.rs` only compiles the Windows manifest
  (`winres` + `ffwb.manifest`); it does NOT copy `i18n/`.

Path arithmetic for the dev fallback (candidate 3), which is the only candidate
that could ever reach `crates/ff-i18n/i18n`:

- PRODUCTION binary at `target/debug/ff-desktop.exe` -> `exe_dir = target/debug`.
  Three `..` from `target/debug` climbs to the PARENT of the repo
  (`...\target\debug` -> `target` -> repo root -> `C:\workspace`), then appends
  `crates/ff-i18n/i18n`, yielding `C:\workspace\crates\ff-i18n\i18n` -- which does
  NOT exist. It overshoots the repo root by one level.
- TEST binary at `target/debug/deps/<hash>.exe` -> `exe_dir = target/debug/deps`.
  Three `..` climbs `deps` -> `debug` -> `target` -> repo root, then appends
  `crates/ff-i18n/i18n` -> `...\FileForgeWorkbench\crates\ff-i18n\i18n` -- which
  DOES exist.

This is why the unit tests in `i18n_bootstrap.rs`
(`install_catalogue_for_locale_en_installs_the_seam`) PASS -- the test binary
sits one directory deeper than production, so the same three-`..` path happens to
land correctly. The test's own comment states this explicitly
("target/<profile>/deps -> ../../../crates/ff-i18n/i18n"). Production is one level
shallower, so the fallback misses. The passing test gives false confidence that
the production path resolves.

Consequences at startup (`main.rs` ~line 115 `install_catalogue`, which runs
BEFORE the reload handle is registered, so no lock is held and it does execute):

- If NO candidate resolves, `install_catalogue_for_locale` logs a WARN and
  RETURNS without calling `set_catalogue` (`i18n_bootstrap.rs` ~lines 95-105).
  The seam stays uninstalled and `ff_i18n::t()` returns the raw KEY for every
  lookup (`seam.rs::t_args`, the `None` arm).
- If SOME layout the user runs DOES resolve a root that contains `en/` but the
  `af/` overlay cannot load there, `build_catalogue`
  (`i18n_bootstrap.rs` ~lines 110-135) catches the overlay error and calls
  `load_base(root)` again, installing the ENGLISH Identity_Base. The user then
  sees genuine English words and `af` is silently dropped (WARN logged) -- which
  matches the report ("everything is still english") more precisely than raw
  keys would.

Either branch produces "persisted `af`, runtime English," and neither loads the
(verified-valid) `af` catalogue. The `af/` data, the schema `allowed_values`, and
the `ff-i18n` overlay mechanism are all correct; the single failure is that the
RUNNING binary is not given a root that contains the `af` directory.

The persisted value itself is fine: `set_user_value` writes the TOML before it
deadlocks (file write precedes the `inner.write()`), which is why the restart
shows `ui.locale = af` -- consistent with the report.

---

## Conclusions

- Symptom 1 root cause: **reentrant acquisition of the `ConfigSystem` `RwLock`**
  -- `set_user_value` holds the write lock across `reload_file` ->
  `callbacks.invoke` -> the `ui.locale` closure -> `get_string` (read lock on the
  same lock). File/symbol citations:
  `crates/ff-config/src/config_handle/runtime.rs` `set_user_value`;
  `crates/ff-config/src/reload/ops.rs` `reload_file` (Step 6);
  `crates/ff-desktop/src/main.rs` locale-reload closure;
  `crates/ff-desktop/src/shell/i18n_bootstrap.rs` `install_catalogue`;
  `crates/ff-config/src/config_handle/mod.rs:155` `get_string`.

- Symptom 2 root cause: **runtime `i18n/` base-path resolution is wrong for the
  production binary** (option (b) in the brief's taxonomy -- a base-path mismatch
  so the load finds no `af/` and falls back to English), compounded by **no
  packaging step that stages `i18n/` next to the binary**. The reload closure IS
  wired (so it is not option (c)); the catalogue mechanism and the `af` data are
  correct (so it is not a data problem). File/symbol citations:
  `crates/ff-desktop/src/shell/i18n_bootstrap.rs` `resolve_i18n_root`
  (candidate list, three-`..` dev fallback) and `build_catalogue` / `load_base`;
  `crates/ff-desktop/build.rs` (no copy step).

The two are independent: fixing the deadlock makes the live swap not freeze, but
the swap would still install English (symptom 2) until the root resolves to a
tree containing `af/`. Both must be fixed for `af` to work live AND after
restart.

---

## Fix plan (minimal, targeted -- NOT applied)

### Fix 1 -- break the reentrant lock (symptom 1)

Goal: callbacks must never run while the `ConfigSystem` write lock is held, OR
the callback must not re-read the config through the locked handle.

Preferred (smallest, safest): in
`ConfigHandle::set_user_value` / `remove_user_value`
(`crates/ff-config/src/config_handle/runtime.rs`), do the reload so that callback
invocation happens AFTER the write guard is dropped. Concretely, have
`reload_file` return the `ReloadEvent` WITHOUT invoking callbacks while the guard
is held, drop the guard, then invoke the matching callbacks. One way:

- Split `ReloadManager::reload_file` into "recompute + swap store, return the
  event" (under the lock) and "invoke callbacks for the event" (callable after
  the lock is released). Then in `set_user_value`:
  ```rust
  let event = {
      let mut system = self.inner.write().unwrap();
      system.manager.reload_file_compute(&user_path, ConfigLayer::User)?  // no invoke
  }; // guard dropped here
  if let Some(ev) = event {
      let callbacks = self.callbacks();   // read-lock, now safe
      callbacks.invoke(&ev);
  }
  ```
  This keeps the existing watcher-driven path (`reload/ops.rs`) behaviour intact
  if that path also moves invocation outside any held `ConfigSystem` lock; audit
  the watcher path too (`reload_all`, `load_project`, `unload_project`) for the
  same "invoke under lock" shape, since a watcher-thread reload firing the
  `ui.locale` closure would also call `get_string` and could contend. The
  watcher runs on its own thread so it would not SELF-deadlock, but keeping all
  invocation outside the write lock is the clean invariant.

Alternative (defence in depth, cheaper but narrower): make the `ui.locale`
callback NOT re-enter the handle. Pass the new locale value into the closure from
the `ReloadEvent` (the event already carries `changed_keys`; the effective value
could be threaded through, or the closure could read the value the panel just
set) so `install_catalogue` does not need `get_string`. This removes the specific
reentrancy but leaves the general "callbacks run under the write lock" hazard in
place, so Fix-1-preferred is the better structural fix.

Either way, keep the file write occurring before the reload (persistence already
works) and ensure the catalogue swap (`ff_i18n::set_catalogue`, an independent
`RwLock` in `ff-i18n/seam.rs`) is only taken after the config lock is released --
`set_catalogue` takes the ff-i18n write lock, which is fine as long as no config
lock is nested around it.

### Fix 2 -- make the runtime `i18n/` root resolve to the shipped catalogue (symptom 2)

Two parts; do both.

a. Correct the dev fallback in `resolve_i18n_root`
   (`crates/ff-desktop/src/shell/i18n_bootstrap.rs`). Add a candidate with the
   correct depth for the PRODUCTION binary (two `..` from `target/<profile>/` to
   the repo root, then `crates/ff-i18n/i18n`), while keeping the existing
   three-`..` candidate for the test binary. I.e. push BOTH:
   - `exe_dir/../../crates/ff-i18n/i18n`   (production: target/<profile>/)
   - `exe_dir/../../../crates/ff-i18n/i18n` (test: target/<profile>/deps/)
   A more robust option is to walk up from `exe_dir` (or `CARGO_MANIFEST_DIR` in
   debug) looking for a `crates/ff-i18n/i18n` marker, rather than hard-coding a
   fixed number of `..`.

b. Add a packaging/staging step so the installed binary has `i18n/` next to it
   (candidate 1, `exe_dir/i18n`, is the intended installed layout). Options:
   - A `build.rs` post-build copy of `crates/ff-i18n/i18n` into
     `target/<profile>/i18n` (mirrors how `help/` is handled -- check whether the
     help loader relies on a similar staged dir and match that mechanism), or
   - the installer/packaging script copies `i18n/` beside the exe.
   Without (b), only developer runs from the repo work; a shipped build still
   finds nothing.

After (a)+(b), startup `install_catalogue` will find a root containing `af/`,
`load_overlay(root, af)` will succeed, and both the live swap (post Fix 1) and the
restart path will render Afrikaans.

---

## Tests that should exist to lock the fix

All are currently MISSING for the production path (the one existing bootstrap
test passes only because of the test-binary depth coincidence described above).

1. **Startup-loads-persisted-locale** (ff-desktop, integration-level): with
   `ui.locale = af` persisted in a user config and a known-good `i18n/` root,
   assert that after the startup install the seam resolves an `af`-only key to
   its Afrikaans string (not English, not the key). This directly guards
   symptom 2 at startup.

2. **Locale-change-swaps-catalogue WITHOUT deadlock** (ff-config +/or ff-desktop):
   drive `set_user_value("ui.locale", "af")` with a registered `on_locale_reload`
   callback that re-reads the handle via `get_string`, under a watchdog/timeout,
   and assert it returns promptly (no hang) AND that the callback observed the new
   value. This guards symptom 1 and the reentrancy specifically. A focused
   ff-config unit test can register a callback that calls back into the handle and
   assert `set_user_value` completes -- it would deadlock today.

3. **Runtime-path-resolution** (ff-desktop unit): assert `resolve_i18n_root`
   returns a directory that actually contains `af/` for BOTH the production
   binary layout (`exe_dir = target/<profile>`) and the test layout
   (`exe_dir = target/<profile>/deps`). Since `current_exe()` is fixed in a test,
   test the pure candidate-building/first-existing logic against synthesised
   `exe_dir` values (a `resolve_i18n_root_from(exe_dir)` helper) rather than the
   real exe, so the production-depth case is covered deterministically.

4. **Packaging staging** (build/integration check): assert that a built
   `target/<profile>/i18n/af` exists (or that the chosen staging mechanism
   produced it), so a shipped layout is validated, not just the dev tree.

5. **End-to-end locale apply** (egui_kittest full-shell, per workspace testing
   rules): set `ui.locale = af`, drive the reload, and assert a visible label
   that differs between en and af renders the Afrikaans text -- the behavioural
   guard the owner's report is really about.
