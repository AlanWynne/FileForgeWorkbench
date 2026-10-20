# Project Rules Live in Kiro Steering

The authoritative rules for FileForgeWorkbench are maintained as Kiro steering
files across two locations, following the tiered steering model (see
`c:\workspace\_steering\02-TIERING-STRATEGY.md`):

- **User level** (`~\.kiro\steering\`) holds the universal rules that apply to
  every project on this machine.
- **Workspace level** (`.kiro/steering/`) holds only the rules specific to this
  repository.

Universal rules are NOT edited here. They are maintained one level up in
`c:\workspace\_steering\user-space\steering\` and deployed down; a change made
directly in a workspace is overwritten on the next sync.

## Universal rules (user level, `~\.kiro\steering\`)

| Topic | File | Covers |
|-------|------|--------|
| Workflow | `workflow.md` | Prompt triage, wrong-window guard, requirements gate, operating modes |
| Terminal discipline | `terminal-discipline.md` | pwsh7 non-interactive wrapper, one-command-one-job, dedicated tools over shell |
| Tooling | `tooling.md` | Project tool reuse, safety, mandatory script-output capture to logs |
| Testing | `testing.md` | TDD cycle, deterministic tests, coverage annotations, the TCR |
| Documentation | `documentation-charset.md` | Mandatory plain-ASCII character rules for docs and source |
| Task format | `task-format.md` | `[ ]`/`[x]` only, titles required, status on sub-bullets |
| Shell exit codes | `shell-exit-code.md` | Trust the sentinel, not the reported `Exit Code` |

## Project rules (workspace level, `.kiro/steering/`)

| Topic | File | Covers |
|-------|------|--------|
| Rust standards | `rust-standards.md` | Coding standards, error handling, 400-line file limit |
| Rust FFWB layout | `rust-ffwb-layout.md` | `ff-desktop` module layout and enforcement |
| Specs | `specs.md` | Spec folder structure, sub-project list, UI terminology, tasks.md format |
| Framework conformance | `framework-conformance.md` | Build ON the framework; the load-bearing mechanisms |
| Wiring standard | `wiring-standard.md` | The canonical `ff-desktop` feature-wiring recipe |
| Workspace conformance | `workspace-conformance.md` | The tab-order focus contract for this shell |
| Workflow orchestration | `workflow-orchestration.md` | Delegation model, liveness probe |

These rules are mandatory. In particular: run the requirements gate in
`workflow.md` before touching any source file, follow TDD in `testing.md`, and
keep every document plain ASCII per `documentation-charset.md`.
