# Project Rules Live in Kiro Steering

The authoritative project rules for FileForgeWorkbench are maintained as Kiro
steering files under `.kiro/steering/`. They were consolidated from the former
`.amazonq/rules/*.md` set to remove duplication and reduce token cost.

Before doing any work in this repository, read the relevant steering file(s):

| Topic | File | Covers |
|-------|------|--------|
| Workflow | `.kiro/steering/workflow.md` | Prompt triage, the requirements gate, and the spec/code operating modes |
| Rust standards | `.kiro/steering/rust-standards.md` | Coding standards, error handling, module layout, 400-line file limit |
| Testing | `.kiro/steering/testing.md` | TDD cycle, cargo commands, coverage annotations, the TCR |
| Documentation | `.kiro/steering/documentation.md` | Mandatory plain-ASCII character rules for docs and source |
| Specs | `.kiro/steering/specs.md` | Spec folder structure, sub-project list, UI terminology, tasks.md format |
| Tooling | `.kiro/steering/tooling.md` | Project tool reuse, safety, and mandatory script-output capture |

These rules are mandatory. In particular: run the requirements gate in
`workflow.md` before touching any source file, follow TDD in `testing.md`, and
keep every document plain ASCII per `documentation.md`.