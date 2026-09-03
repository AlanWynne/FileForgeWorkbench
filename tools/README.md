# Project Tools

This folder contains reusable scripts that support FileForge Workbench
development, documentation, validation, and maintenance. These tools are not
part of the product runtime.

## Organization

| Folder | Purpose |
|--------|---------|
| `python/` | Reusable Python utilities |
| `powershell/` | Reusable PowerShell utilities |
| `rust/` | Small Rust-based maintenance tools, if needed |
| `fixtures/` | Small input files required by a tool or validation |

Current reusable utilities:

- [`powershell/ffwb_make.ps1`](powershell/ffwb_make.ps1) - common build and
  test commands.
- [`powershell/push-to-github.ps1`](powershell/push-to-github.ps1) - explicitly
  confirmed commit, tag, and push workflow.

Each reusable tool should include:

- A descriptive filename.
- A short usage comment or help message.
- Explicit input and output paths.
- Safe default behavior that does not delete or overwrite data unexpectedly.
- A note in this README or a nearby README when the tool needs special setup.

## Temporary scripts

Do not place one-off experiments or failed patches here. Use the session
workspace or another explicitly temporary location for those files. A script
may be promoted into this folder when it is useful for a second task, has
clear ownership and usage, and is safe to rerun.

## Runtime prerequisites

Use the repository's documented commands first. When Python is required, the
standard local interpreter is `C:\tools\python`. Shared scripts in
`C:\tools\scripts` may be used, but project-specific reusable tools belong in
this folder so they are versioned with the project.
