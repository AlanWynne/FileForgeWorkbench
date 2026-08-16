# ADR Amendment: [New Subsystem Name]

**Status:** Proposed | Accepted | Superseded
**Date:** YYYY-MM-DD
**Amends:** ADR-001 (Dataset Ownership Model)

---

## Context

[Describe why this new subsystem is needed and what problem it solves.]

---

## Ownership

### What This Subsystem Owns

| Capability | Description |
|-----------|-------------|
| [capability-1] | [description of what this means] |
| [capability-2] | [description] |

### What This Subsystem Does NOT Own

| Capability | Owned By | Reason |
|-----------|----------|--------|
| [capability-1] | [owning crate] | [why it belongs there] |
| [capability-2] | [owning crate] | [reason] |

---

## Prohibited Dependencies

This subsystem SHALL NOT depend on:

| Crate | Reason |
|-------|--------|
| [crate-name] | [architectural reason] |

This subsystem IS permitted to depend on:

| Crate | Via | Purpose |
|-------|-----|---------|
| ff-vfs | Direct | Resource access abstraction |
| [crate] | [Trait name] | [purpose] |

---

## Trait Interface

```rust
/// [Description of the trait]
pub trait [TraitName]: Send + Sync {
    /// [method description]
    fn method_name(&self, ...) -> Result<..., Error>;
}
```

---

## Integration Pattern

### How Other Crates Consume This Subsystem

[Describe how dependent crates interact with this subsystem through trait interfaces.]

### How This Subsystem Integrates With Existing Infrastructure

[Describe VFS provider registration, command registration, etc.]

---

## Fitness Function Updates

The following rules SHALL be added to `ff-governance-tests`:

```rust
DependencyRule {
    crate_name: "[new-crate]",
    prohibited_dependency: "[prohibited]",
    reason: "[reason]",
    requirement_ref: "ADR-001 Amendment: [this ADR]",
},
```

---

## Consequences

- [Positive consequence 1]
- [Positive consequence 2]
- [Risk or tradeoff 1]
