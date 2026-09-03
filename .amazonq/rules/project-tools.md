# Project Tooling Rule

Use this rule whenever a task requires a script, data transformation, report
generator, migration, or repository maintenance helper.

## Location and reuse

- Reuse an existing project tool before writing a new script.
- Reusable project-specific tools belong under `tools/python/`,
  `tools/powershell/`, or another appropriate subfolder of `tools/`.
- Shared tools under `C:\tools\scripts` may be invoked when appropriate, but
  do not copy them into the project without a reason and recorded provenance.
- Do not create scripts in the repository root.

## Temporary versus reusable scripts

- One-off experiments, generated scripts, and failed approaches belong in the
  session workspace, not in the repository.
- Do not delete a project tool merely because the current task is complete.
- Before deleting or replacing a tool, check its references and ask for
  confirmation if its purpose or ownership is unclear.
- Promote a temporary script into `tools/` only when it is safe to rerun,
  documented, and likely to be reused.

## Safety and documentation

- Inspect a script before running it.
- Prefer read-only or dry-run modes for discovery and migration work.
- Never use broad recursive deletion or unresolved wildcard paths.
- Tools that modify files must describe their inputs, outputs, and overwrite
  behavior in a usage message or adjacent README.
- Keep generated output outside the repository unless it is an intentional
  project artefact.
- After adding or changing a tool, run its documented help or smallest safe
  validation command.
