# Dataset Lifecycle Ownership Documentation

> **Governance Reference:** [Dataset Ownership Model](./../.kiro/specs/dataset-ownership-model/requirements.md) (ADR-001)

This document describes the complete dataset lifecycle with clear ownership at each stage. Every operation in the create→use→modify→delete sequence has an explicitly named owning crate.

---

## 1. Dataset Creation Lifecycle

**Validates:** Requirement 13 AC 1

### Via IDCAMS DEFINE Command

```
┌──────────────┐      ┌───────────────────────┐     ┌───────────────────┐
│  ff-idcams   │      │  ff-dataset-catalog   │     │  ff-vsam-services │
│              │      │                       │     │                   │
│ 1. Parse     │      │                       │     │                   │
│    DEFINE    │─────>│  2. create_dataset()  │     │                   │
│    command   │      │    (catalog entry +   │     │                   │
│              │      │     physical storage) │────>│ 3. initialize_    │
│              │      │                       │     │    dataset()      │
│              │      │                       │     │   (VSAM structs,  │
│              │      │                       │     │    if VSAM type)  │
└──────────────┘      └───────────────────────┘     └───────────────────┘
    OWNS:                   OWNS:                       OWNS:
    Command parsing         Catalog entry creation      VSAM record
    Parameter validation    Physical storage alloc      structure setup
```

**Ownership sequence:**
1. **ff-idcams** parses the DEFINE command and extracts attributes (OWNS: parsing)
2. **ff-dataset-catalog** creates the catalog entry + physical storage via `create_dataset()` (OWNS: catalog CRUD)
3. **ff-vsam-services** initializes VSAM record structures via `initialize_dataset()` if VSAM type (OWNS: VSAM setup)

### Via JCL Allocation (DD DISP=NEW)

```
┌──────────────┐            ┌───────────────────────┐     
│  ff-dsalloc  │            │  ff-dataset-catalog   │     
│              │            │                       │     
│ 1. Parse DD  │            │                       │     
│ 2. Interpret │            │                       │     
│    DISP=NEW  │            │                       │     
│ 3. Assemble  │───────────>│ 4. create_dataset()   │     
│    attrs from│            │    (catalog entry +   │     
│    DCB/SPACE │            │     physical storage) │     
└──────────────┘            └───────────────────────┘     
    OWNS:                   OWNS:
    DD parsing              Catalog entry creation
    DISP interpretation     Physical storage allocation
    Attribute assembly
    Default application
```

---

## 2. Dataset Access Lifecycle

**Validates:** Requirement 13 AC 2

```
Consumer ─────> ff-dataset-catalog ─────> ff-vfs ─────> ff-vsam-services
                (resolve DSN)             (content I/O)  (record-level,
                                                          if VSAM)
```

**Ownership sequence:**
1. Consumer provides a DSN
2. **ff-dataset-catalog** resolves DSN to physical path via `resolve_dsn()` (OWNS: resolution)
3. Content I/O flows through **ff-vfs** using `vfs://catalog/DSN` URIs (OWNS: abstraction)
4. For VSAM datasets, record-level access flows through **ff-vsam-services** (OWNS: record ops)

---

## 3. Dataset Modification Lifecycle (Attribute Changes)

**Validates:** Requirement 13 AC 3

```
┌──────────────┐            ┌───────────────────────┐
│  ff-idcams   │            │  ff-dataset-catalog   │
│  (ALTER cmd) │            │                       │
│              │            │                       │
│ 1. Parse     │───────────>│ 2. update_dataset()   │
│    ALTER     │            │    (modify catalog    │
│    command   │            │     entry attributes) │
└──────────────┘            └───────────────────────┘
    OWNS:                   OWNS:
    Command parsing         Metadata persistence
    Attribute extraction    Validation of new attrs
```

**Ownership sequence:**
1. **ff-idcams** parses ALTER command OR workbench UI initiates change (OWNS: parsing)
2. **ff-dataset-catalog** updates the catalog entry via `update_dataset()` (OWNS: metadata CRUD)

---

## 4. Dataset Deletion Lifecycle

**Validates:** Requirement 13 AC 4

```
┌──────────────┐     ┌───────────────────┐     ┌───────────────────────┐
│  ff-idcams   │     │  ff-vsam-services │     │  ff-dataset-catalog   │
│  (DELETE cmd)│     │                   │     │                       │
│              │     │                   │     │                       │
│ 1. Parse     │────>│ 2. destroy_       │────>│ 3. delete_dataset()   │
│    DELETE    │     │    dataset()      │     │    (remove entry +    │
│    command   │     │   (if VSAM: clean │     │     physical storage) │
│              │     │    up record      │     │                       │
│              │     │    structures)    │     │                       │
└──────────────┘     └───────────────────┘     └───────────────────────┘
 OWNS:                OWNS:                     OWNS:
 Command parsing      VSAM cleanup              Catalog entry removal
                      (if applicable)           Physical storage removal
```

**Ownership sequence:**
1. **ff-idcams** parses DELETE command (OWNS: parsing)
2. IF VSAM, **ff-vsam-services** cleans up record structures via `destroy_dataset()` (OWNS: VSAM cleanup)
3. **ff-dataset-catalog** removes catalog entry and physical storage via `delete_dataset()` (OWNS: catalog CRUD)

---

## 5. GDG Generation Lifecycle

**Validates:** Requirement 13 AC 5

```
┌──────────────┐         ┌───────────────────────┐
│  ff-dsalloc  │         │  ff-dataset-catalog   │
│  or ff-idcams│         │                       │
│              │         │                       │
│ 1. Detect    │────────>│ 2. create_generation()│
│    (+1) ref  │         │    - Assign gen number│
│    or DEFINE │         │    - Enforce roll-off │
│              │         │    - Create entry     │
└──────────────┘         └───────────────────────┘
 OWNS:                   OWNS:
 Reference detection     Generation numbering
 (+1) syntax parsing     Roll-off policy
                         Catalog entry creation
```

**Ownership sequence:**
1. **ff-dsalloc** resolves `(+1)` reference OR **ff-idcams** processes DEFINE for new generation (OWNS: reference parsing)
2. **ff-dataset-catalog** handles generation numbering, roll-off enforcement, and catalog entry creation via `create_generation()` (OWNS: GDG management)

---

## 6. Resolution Lifecycle (DSN → Physical Path)

**Validates:** Requirement 14 AC 1–5

```
┌────────────────────────────────────────────────────────────┐
│                    ff-dsalloc (owns stages 1-3)            │
│                                                            │
│  ┌──────────┐   ┌─────────────┐   ┌─────────────┐          │
│  │1. Parse  │──>│2. Substitute│──>│3. Resolve   │          │
│  │  DD stmt │   │  symbolics  │   │  referbacks │          │
│  └──────────┘   └─────────────┘   └─────────────┘          │
│                                          │                 │
└──────────────────────────────────────────┼─────────────────┘
                                           ▼
                              ┌──────────────────────┐
                              │  ff-dataset-catalog  │
                              │                      │
                              │  4. resolve_dsn()    │
                              │     (catalog lookup →│
                              │      physical path)  │
                              └──────────────────────┘
                                OWNS:
                                Catalog lookup
                                Path resolution
```

**Ownership sequence:**
1. **ff-dsalloc** parses the JCL DD statement (OWNS: DD parsing)
2. **ff-dsalloc** performs symbolic substitution on the DSN (OWNS: symbolic substitution)
3. **ff-dsalloc** resolves referbacks if present (OWNS: referback resolution)
4. **ff-dataset-catalog** performs catalog lookup and returns physical path via `resolve_dsn()` (OWNS: resolution)

### GDG Relative Reference Resolution

When the resolution involves a GDG relative reference:
1. **ff-dsalloc** detects `(+n)`/`(0)`/`(-n)` syntax (OWNS: GDG reference detection)
2. **ff-dsalloc** invokes `ff-dataset-catalog::resolve_generation()` with base name and offset (DELEGATION)
3. **ff-dataset-catalog** computes the target generation and returns its path (OWNS: generation resolution)

### New Allocation During Resolution (DISP=NEW)

When resolution results in a new allocation:
1. **ff-dsalloc** determines allocation is required (OWNS: DISP interpretation)
2. **ff-dsalloc** assembles attributes from DCB/SPACE/defaults (OWNS: attribute assembly)
3. **ff-dsalloc** invokes `ff-dataset-catalog::create_dataset()` (DELEGATION)
4. **ff-dataset-catalog** creates entry and storage (OWNS: catalog CRUD)

---

## Architectural Invariant

> **At NO POINT in any lifecycle SHALL a subsystem bypass the owning subsystem's API to perform an operation directly.** This is the fundamental architectural invariant of the Dataset Ownership Model (ADR-001).

| Operation | Only Authorized Via |
|-----------|-------------------|
| Catalog entry creation/deletion/update | `CatalogService` trait |
| DSN resolution | `CatalogService::resolve_dsn()` |
| VSAM record operations | `VsamService` trait |
| JCL DD parsing | `AllocatorService` trait |
| IDCAMS command parsing | `ff-idcams` internal parser |

---

## Future Extensibility

When introducing new dataset-related subsystems, follow this process:

1. **ADR Amendment** — Produce an ADR amendment defining:
   - What the new subsystem owns
   - What it does NOT own
   - Permitted and prohibited dependencies
   - Its authority rule

2. **Trait Interface** — Define the new subsystem's public API as a trait in the owning crate

3. **Fitness Function Update** — Extend the architectural compliance tests in `ff-governance-tests` with prohibition rules for the new crate

4. **DAG Preservation** — Verify the dependency graph remains acyclic (no cycles)

5. **API Extension** — If existing traits need new methods, add them to the owning crate's trait through a PR to that subsystem

**Template:** See `docs/adr/template-dataset-subsystem.md` for the ADR amendment template.
