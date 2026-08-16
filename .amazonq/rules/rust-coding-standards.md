# Rust Coding Standards

Apply these rules to every Rust file in this project without being asked.

## Requirements-First Gate — MANDATORY BEFORE ANY CODE CHANGE

> **When given a new requirement:** follow `.amazonq/rules/new-requirements-gate.md` first.
> That rule covers requirements.md, design.md, tasks.md, project-master, and TCR updates
> before any code is written. The steps below apply once that gate is complete.

No implementation code and no test code may be written until all three steps are complete:

1. Update the relevant `docs/specs/<sub-project>/requirements.md` — use EARS format: `WHEN ... THE ... SHALL ...`
   - If the change adjusts or extends an existing requirement, edit that criterion in place and note the change.
   - If the change introduces new behaviour with no existing criterion, add a new numbered criterion.
   - Requirements documents are the source of truth — code must never get ahead of them.
2. Update `docs/TCR.md` — add a 🔴 NOT COVERED row for each new criterion
3. Write the failing test — must carry `// Validates: Requirement X.Y` and fail before implementation

**Full gate sequence:**
```
1. requirements.md updated   ← ALWAYS FIRST, no exceptions
2. docs/TCR.md updated
3. Test written and FAILING
4. Implementation written
5. Test passing
6. TCR.md status updated to ✅ PASS
```

Skipping any step is a violation of project standards.

**Scope of the requirements update:**
- Changes to `ff-desktop` shell behaviour → update `docs/specs/startup-and-session/requirements.md` and/or the relevant sub-project spec (e.g. `edit-operations`, `file-operations`, `menu-and-statusbar`).
- New UI interactions (mouse click, keyboard shortcut) → update the spec that owns that interaction domain.
- Bug fixes that reveal a missing or incorrect criterion → correct the criterion before fixing the code.

## Naming Conventions

| Item | Convention | Example |
|------|-----------|---------|
| Types, traits, enums | `UpperCamelCase` | `DocumentSession` |
| Functions, methods, variables | `snake_case` | `edit_line` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_LINE_WIDTH` |
| Modules | `snake_case` | `file_engine` |

- Prefer full descriptive names: `line_number` not `ln`
- Booleans must read as predicates: `is_modified`, `has_pending_changes`

## Error Handling

- Library code: use `thiserror`. One `Error` enum per crate with enough context to diagnose.
- Application/binary code: use `anyhow` with `.context("what was being attempted")` on every `?` crossing a module boundary.
- Never use `unwrap()` or `expect()` in library code. In tests, `expect("why")` is preferred over `unwrap()`.
- Never use `panic!` except to enforce documented programmer-error invariants.
- Never silently discard errors with `let _ = ...` without a comment.

## Ownership and Borrowing

- Prefer borrowing over cloning. Comment any `clone()` inside a hot loop.
- Prefer `&str` over `&String`; prefer `&[T]` over `&Vec<T>` in function signatures.
- Use `Arc<T>` for shared ownership across threads; `Rc<T>` only for single-threaded contexts.
- Acquire `Mutex`/`RwLock` guards in the narrowest scope possible; never hold across an `await` or blocking call.

## Types and Data Modelling

- Make illegal states unrepresentable — use enums with data-carrying variants.
- Use the newtype pattern: `struct LineNumber(usize)` not a bare `usize`.
- Prefer `Option<T>` over sentinel values (`-1`, `""`, `0`).
- Derive `Debug` on every public type. Derive `Clone`, `PartialEq`, `Eq` only when semantically meaningful.
- Use `#[non_exhaustive]` on public enums that may gain variants.

## Functions and Methods

- One function, one responsibility. Max ~40 lines; refactor longer functions into named helpers.
- No output parameters — return a value or tuple instead.
- Constructors: `new` (infallible), `try_new` / `from_*` (fallible).

## Structs and Impl Blocks

- `impl` item order: constants → associated functions → public methods → private methods.
- Avoid `pub` fields unless the type is a plain data container. Use accessor methods.
- One trait per `impl` block.

## Modules and Visibility

- One concern per file. `mod.rs` re-exports only — no logic.
- Use `pub(crate)` for items that cross module boundaries but are not public API.
- Max 3 levels deep: `crate::module::submodule`.

## Unsafe Code

- `unsafe` is forbidden except in FFI boundary code.
- Every `unsafe` block must have a `// SAFETY:` comment.
- Never use `unsafe` to work around a borrow checker error.

## Comments and Documentation

- Every public item must have a `///` doc comment describing *what* and *why*, not *how*.
- Use `# Errors`, `# Panics`, `# Examples` sections where applicable.
- Delete commented-out dead code — use git to recover it.

## Formatting

- All code formatted with `rustfmt` before committing.
- Imports grouped: `std` → external crates → crate-local, blank line between groups.
- Max line length: 100 characters.

## Clippy

- Must build cleanly with `cargo clippy -- -D warnings`.
- Suppress a lint only with `#[allow(clippy::lint_name)]` on the smallest scope, with a comment explaining why.
