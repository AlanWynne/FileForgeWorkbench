# Analysis Record: connector-extensibility (W5.13)

- **Wave**: 5 (emulators, tools, connectors)
- **Backing crate**: `ff-connector-extensibility` (connector plugin trait, provider
  registration, capability advertisement, provider lifecycle, authentication framework,
  future-connector hooks, error mapping -- the framework FTP/SFTP/z/OS/cloud connectors
  "MUST implement" to integrate with the VFS)
- **Spec files**: requirements.md (151 lines, 7 requirements), tasks.md
  (84 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 5 pass

---

## 1. Split candidacy + cap (PA-STD-068)

7 reqs / 151 lines -- not a split candidate. One cap file: `registry.rs` = 460 non-test
(over the 400 cap). Recorded PA-STD-068 (MEDIUM -- cap): split registry.rs by concern
(registration protocol / capability advertisement / lifecycle / lookup). REFACTOR.

---

## 2. Cross-unit consistency -- PA-CONFLICT-021 (SEVENTH orphan: connector FRAMEWORK no connector implements)

Two linked findings:

### (a) The connector family is mostly UNBUILT

The spec family names connectors for local-fs, network-fs, FTP/SFTP, cloud, and
mainframe. But only ONE connector crate EXISTS: `ff-connector-local-fs`. There is no
ff-connector-network-fs / -ftp-sftp / -cloud / -mainframe crate. So the connector
family is largely spec-only (to be confirmed per-connector in W5.14+; those units may be
spec-without-crate). Noted here as context.

### (b) ff-connector-extensibility is an ORPHAN (SEVENTH) -- its trait is implemented by NOBODY

The crate's entire purpose is "the plugin trait ... that future VFS connectors MUST
implement." But:
- The ONE existing connector, `ff-connector-local-fs`, does NOT depend on
  ff-connector-extensibility (ext-dep = 0) -- it depends on ff-vfs directly (vfs-dep = 1)
  and implements the lower-level VFS provider trait instead of this framework.
- 0 external code refs to `ff_connector_extensibility` / ConnectorPlugin / ConnectorProvider.

So the connector-EXTENSIBILITY framework (trait + registration + capability + lifecycle +
auth) is implemented by NO connector -- the sole connector bypasses it for ff-vfs. This is
the SEVENTH orphan crate, and the SAME PATTERN as language-service (PA-CONFLICT-018): a
foundation/framework crate whose stated consumers ignore it and implement the lower layer
directly. Here it is even starker -- the framework exists to be implemented, and nothing
implements it.

Recorded PA-CONFLICT-021 (owner-gated, MEDIUM-HIGH): decide the connector architecture --
(a) make ff-connector-local-fs (and future connectors) implement ff-connector-extensibility
(the designed framework: registration, capability advertisement, lifecycle, auth,
error-mapping), with the framework layered over ff-vfs -- PREFERRED if the extensibility/
capability/auth features are wanted; or (b) if connectors implement ff-vfs directly by
design, delete/absorb the framework + reconcile the spec (the "future connectors" premise
never materialised). Pairs with the orphan cluster (PA-W4.1) + language-service
(PA-CONFLICT-018). Code + owner decision.

### Positive: ff-logging USED

Unlike most Wave-4/5 crates, ff-connector-extensibility ACTUALLY LOGS (4 ff-logging calls)
-- registration/lifecycle events. Positive (no dead-dep, no PA-LOG). Even though orphaned,
its own code follows the logging standard.

### Public types and ownership

- ConnectorPlugin trait, ProviderRegistry, capability advertisement, lifecycle, auth
  framework, error mapping -- sole-owned by `ff-connector-extensibility` but ORPHANED
  (PA-CONFLICT-021); the auth + capability layer has no counterpart in ff-vfs (so if the
  framework is deleted, those features are lost -- argues for wiring, not deleting).

### Cross-reference integrity

The spec's "connectors MUST implement this" premise is NOT honoured (0 implementers) --
PA-CONFLICT-021.

---

## 3. Completeness

Tracking: all 84 sub-tasks `[x]`. The framework is complete + (partly) tested AS A CRATE
(trait, registration, capability, lifecycle, auth, hooks, error mapping). FUNCTIONALLY
complete -- but orphaned (PA-CONFLICT-021), so the extensibility layer is not realized. No
PA-INCOMPLETE for the crate; the gap is architectural (no implementers) + the broader
family being unbuilt.

### TCR gap (PA-TCR-034) -- TOTAL ABSENCE

TCR.md has 0 rows for ff-connector-extensibility across 7 reqs. Recorded PA-TCR-034
(consistent with the orphan crates' total-TCR-absence pattern).

---

## 4. Logging audit

- `ff_logging` / `log_*!`: 4 -- USED (registration/lifecycle). ff-logging dep NOT dead.
- `std::fs`: 0.

POSITIVE -- the crate follows the logging standard (contrast the many dead-ff-logging
crates). No PA-LOG item. (Once wired, connector auth + lifecycle logging is security-
relevant, but the foundation already logs.)

---

## 5. Task revision proposals

- **PA-CONFLICT-021 (owner-gated, MEDIUM-HIGH)**: SEVENTH orphan -- make connectors
  implement ff-connector-extensibility (framework over ff-vfs) [PREFERRED, preserves
  capability/auth features], OR delete/absorb + reconcile the "future connectors" spec.
  Also surfaces that the connector family is mostly UNBUILT (only local-fs exists).
  Code + owner decision.
- **PA-STD-068 (MEDIUM -- cap)**: split registry.rs (460) by concern. REFACTOR.
- **PA-TCR-034**: add TCR rows (0 for 7 reqs). No code.

No PA-LOG (logging present -- positive). No ASCII item (0 non-comment). No raw-fs.

---

## Summary

connector-extensibility (`ff-connector-extensibility`) is a complete, well-logged (4
ff-logging calls -- a positive, follows the standard) connector-plugin FRAMEWORK (trait,
registration, capability advertisement, lifecycle, authentication, error mapping; 84/84).
The headline finding is PA-CONFLICT-021 (MEDIUM-HIGH): it is the SEVENTH orphan crate, and
starkly so -- its entire purpose is "the trait future VFS connectors MUST implement," yet
the ONE existing connector (ff-connector-local-fs) does NOT depend on it (0) and implements
ff-vfs directly instead, and there are 0 external refs to it. Same pattern as language-
service (foundation crate its consumers ignore). It also surfaces that the connector family
is largely UNBUILT -- only local-fs has a crate (no network-fs/ftp-sftp/cloud/mainframe
crates), to confirm per-connector in W5.14+. Decide: wire connectors onto the framework
(preserves the capability/auth layer that ff-vfs lacks) or delete/absorb + reconcile the
"future connectors" spec. Minor: registry.rs 460 over cap (PA-STD-068), total TCR absence
(PA-TCR-034). NOTE the positive: unlike most orphans, this crate ACTUALLY LOGS.
