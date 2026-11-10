# ffwb Build Closure -- Authoritative Findings (Phase 1, repo B)

Read-only investigation. No source or manifest was edited in this step.

Goal: derive the TRUE set of workspace member crates that the ff-desktop /
ffwb (FFWB.exe) binary actually compiles, so a later step can set workspace
`default-members` to that set and narrow the DEFAULT gate run. Parked crates
stay in `members` and get pulled back in later, one at a time.

## Method

The closure was derived from cargo's RESOLVED dependency graph (not estimated
from reading manifests). cargo metadata emits `resolve.nodes`, each node
carrying its dependency edges tagged by kind (normal / build / dev). The
closure is the transitive walk from the ff-desktop root node following ONLY
normal and build edges (dev edges excluded, because dev-dependencies are not
compiled into the shippable binary), intersected with the set of workspace
member crates (path dependencies under `crates/`).

### Exact commands run

All via the clean non-interactive pwsh7 wrapper, output redirected to a log and
read back (verify by effect; the `Exit Code` line is unreliable on this host).

1. Enumerate the full dependency closure (human-readable cross-check):

   ```
   C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo tree -p ff-desktop --prefix none --no-dedupe *> c:\workspace\apps\FileForgeWorkbench\tools\logs\ff-desktop-tree.log"
   ```

2. Emit the resolved dependency graph as JSON (authoritative source):

   ```
   C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "cargo metadata --format-version 1 --manifest-path c:\workspace\apps\FileForgeWorkbench\Cargo.toml *> c:\workspace\apps\FileForgeWorkbench\tools\logs\ff-metadata.json"
   ```

3. Compute the closure from the metadata JSON (walks resolve.nodes,
   normal+build only, intersect with workspace members under crates/):

   ```
   C:\tools\powershell7\pwsh.exe -NoProfile -NonInteractive -Command "C:\tools\python\python.exe c:\workspace\apps\FileForgeWorkbench\tools\python\compute_closure.py c:\workspace\apps\FileForgeWorkbench\tools\logs\ff-metadata.json c:\workspace\apps\FileForgeWorkbench\tools\logs\closure-compute.txt"
   ```

The computation script lives at
`tools/python/compute_closure.py` (reusable; documented header, read-only).
Its full output is at `tools/logs/closure-compute.txt`.

### Cross-checks performed

- Root `Cargo.toml` `members` list: 92 path entries under `crates/` (counted
  via grep). cargo metadata reports 92 workspace members under crates/ -- no
  discrepancy. (The task brief's "~95" was an estimate; the authoritative
  count is 92.)
- `crates/ff-desktop/Cargo.toml` direct `[dependencies]` path-deps: 50 member
  crates listed directly. All 50 appear in the closure. The closure's 4
  additional members (see below) are pulled in TRANSITIVELY via those 50, not
  listed directly on ff-desktop -- expected and correct.
- Sanity: closure (54) + parked (38) = 92 = total members. Balanced.

## IN-CLOSURE members -- compiled by the ffwb binary (54)

These are the crates the ffwb binary actually compiles (normal + build
dependency closure of ff-desktop, including ff-desktop itself).

```
ff-catalog-dialog
ff-catalog-registry
ff-ce-host-fs
ff-ce-ntfs
ff-ce-posix
ff-command
ff-command-semantics
ff-config
ff-connector-local-fs
ff-context-menu
ff-core
ff-dataset-alloc-dialog
ff-desktop
ff-display-line-mapping
ff-document-model
ff-dscatalog
ff-edit-operations
ff-editor-panel
ff-exclude-manager
ff-exclude-show-filter
ff-explorer-view
ff-fftest
ff-file-ops
ff-file-tree
ff-files-panel
ff-find-and-replace
ff-gcc-toolchain
ff-global-search
ff-help
ff-i18n
ff-idcams
ff-keys
ff-layout
ff-logging
ff-nav-model
ff-navigation-commands
ff-plugin
ff-posix-provider
ff-rust-toolchain
ff-screen-model
ff-scrm
ff-scroll-amount
ff-session
ff-shell
ff-theme
ff-theme-editor
ff-toolchain-api
ff-toolchain-panel
ff-undo-redo
ff-vfs
ff-viewport-scrolling
ff-volume
ff-workflow
ff-zoom
```

### Transitive-only members (in closure, NOT direct deps of ff-desktop)

Pulled in via other closure crates, so they belong in the closure even though
ff-desktop does not list them directly:

```
ff-ce-ntfs
ff-ce-posix
ff-undo-redo
ff-workflow
```

## PARKED members -- NOT compiled by the ffwb binary (38)

Workspace members that are NOT reached by the ffwb build closure. These stay in
`members` but would be excluded from `default-members`, so the default gate
skips them. They are pulled back in one at a time as functionality is wired.

```
ff-asa
ff-auto-indent
ff-background-io
ff-caret-selection
ff-clipboard
ff-compare-merge
ff-completion
ff-connector-extensibility
ff-database-tool
ff-dsalloc
ff-encoding
ff-external-mod
ff-forge
ff-governance-tests
ff-hex
ff-html-export
ff-idle-processing
ff-jes
ff-language-service
ff-large-file-performance
ff-line-commands
ff-lua
ff-md-viewer
ff-mdx-app
ff-mdx-installer
ff-mdx-plugin
ff-menu
ff-pdf-export
ff-select
ff-seqnum
ff-structure-catalog
ff-syntax-highlighting
ff-tabmask
ff-tabs
ff-text-decorations
ff-viewers
ff-whitespace-guides
ff-wrap
```

## Ambiguity / edge-case flags

- **Dev-dependency-only members:** NONE. No workspace member is reachable from
  the closure solely via dev-dependency edges. (The script computed full
  reachability INCLUDING dev edges and found zero extra members beyond the
  normal+build closure, so excluding dev edges did not drop any member from the
  binary's true closure.)
- **Optional / feature-gated member deps inside the closure:** NONE. No
  in-closure crate declares a workspace-member dependency as `optional` (behind
  a feature). So the closure is not sensitive to feature selection for
  member-to-member edges -- the 54 are compiled regardless of feature flags.
- **ff-fftest** is a DIRECT `[dependencies]` entry of ff-desktop (a normal dep,
  not a dev-dependency), so despite its "test" name it is genuinely part of the
  shipped binary's closure and is correctly IN-CLOSURE. Flagged for the design
  step in case it was expected to be test-only.
- **ff-mdx-app / ff-mdx-installer / ff-md-viewer / ff-html-export /
  ff-pdf-export / ff-mdx-plugin** (the Viewer/Exporter family) are all PARKED --
  they are separate binaries/plugins, not part of the ffwb binary closure.
- **Separate binary crates among parked members:** the mdx family above builds
  its own executables; narrowing `default-members` to the ffwb closure will
  also drop those from the default build/gate. This is consistent with the goal
  (gate only what ffwb compiles) but the design step should decide whether any
  parked binary needs its own gate profile later.

## Counts

| Set                          | Count |
|------------------------------|-------|
| Total workspace members      | 92    |
| IN-CLOSURE (ffwb compiles)   | 54    |
| PARKED (not in ffwb closure) | 38    |

Sanity: 54 + 38 = 92 (balanced against total members).
