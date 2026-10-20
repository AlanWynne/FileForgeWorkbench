---
inclusion: always
---

# FFWB Rust layout (project-specific, Tier 3)

Split out of rust-standards.md; the generic Rust standards now live at user/central level.

### ff-desktop module layout
```
src/
  main.rs                  -- entry point, startup wiring only
  shell/
    mod.rs                 -- WorkbenchShell struct, new(), re-exports
    state.rs               -- fields, FocusStop, helper types
    commands.rs            -- handle_command(), is_shell_command()
    render.rs              -- update(), render_* methods
    dialogs.rs             -- render_dialogs() -- all modal dialog dispatch
  files_panel/
    mod.rs                 -- FilesPanelState, re-exports
    state.rs               -- data model, add_dataset, load_entries
    render.rs              -- render(), render_catalog_tree(), render_content_area()
    context.rs             -- context menu types and item lists
  dataset_alloc_dialog.rs  -- <= 400 lines (split tests if borderline)
  catalog_manager_dialog.rs
```

### Enforcement
Before submitting an implementation task, any file over 400 non-test lines blocks
the task until split:
```
rg --count-matches '' crates/ff-desktop/src/**/*.rs | awk -F: '$2 > 400'
```
