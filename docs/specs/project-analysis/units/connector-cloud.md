# Analysis Record: connector-cloud (W5.17)

- **Wave**: 5 (emulators, tools, connectors)
- **Backing crate**: NONE (no cloud connector crate -- DEFERRED, not in initial release)
- **Spec files**: requirements.md (195 lines, 12 requirements, 10 EARS criteria),
  design.md -- NO tasks.md
- **Analysed**: Wave 5 pass (DEFERRED future spec)

---

## 1. Split candidacy

N/A -- no crate. Nothing to split.

---

## 2. Cross-unit consistency -- CONFIRMS the deferred-connector pattern + PA-CONFLICT-021

Same deferred-spec pattern as network-fs (W5.15) + ftp-sftp (W5.16):
- NO crate, NO tasks.md; design.md marked "STATUS: DEFERRED -- Not in initial release."
- 12 reqs / 195 lines / 10 EARS; 0 crate dependents (expected).
- Targets the `connector-extensibility` trait (ConnectorPlugin, ConnectorRegistry) --
  another confirming data point for the PA-CONFLICT-021 refinement (deferred remote
  connectors are the intended implementers of ff-connector-extensibility).

### Scope specifics -- SharePoint Online / OneDrive / OAuth 2.0

Unlike the other deferred connectors, cloud targets SPECIFIC providers: SharePoint
Online, OneDrive, with OAuth 2.0 authentication (design shows client_creds.rs +
OAuth flows). Two forward-looking notes (NOT findings -- no code exists):
- SECURITY (when built): OAuth tokens / client credentials are sensitive; the
  implementation must use secure credential storage (OS keychain / encrypted store), not
  plaintext config -- this is exactly what the connector-extensibility AUTHENTICATION
  framework (Req 5, W5.13) should provide, so cloud is a strong test of that auth layer.
- Provider SDK licences (when built): the SharePoint/OneDrive/Graph SDK dependencies will
  need a licence review at implementation time.
Recorded as forward notes on PA-CONFLICT-021 (the auth layer will be exercised here) --
no action now.

### Consistent layering

cloud targets connector-extensibility (VfsProvider + lifecycle + auth + capability),
consistent with the deferred family. OAuth/credential handling is the clearest
justification for the extensibility framework's auth layer existing (local-fs needs no
auth; cloud very much does) -- further supporting the "intentional base" reading of
PA-CONFLICT-021.

---

## 3. Completeness

N/A for a deferred spec (no crate, no tasks.md, nothing claimed done). HONEST deferral.
THIRD deferred connector spec (w/ network-fs, ftp-sftp) -- rolls under PA-TRACK-006.

---

## 4. Logging audit

N/A -- no crate. (When built, auth/token-refresh + request logging will matter, and must
avoid logging secret values.)

---

## 5. Task revision proposals

- **PA-TRACK-006 (extend)**: connector-cloud is the THIRD deferred spec-only connector --
  include in the "mark deferred connector specs clearly" action.
- **PA-STD-071 (extend, docs ASCII)**: design.md has the same mojibake STATUS line
  (corrupted warning-emoji + em-dash). Same ASCII fix. Docs; no gate.
- **PA-CONFLICT-021 (confirming evidence + auth-layer note)**: cloud's OAuth/credential
  needs justify the connector-extensibility auth layer; it will be the strongest test of
  that layer + secure credential storage when built. No action now.

No code (no crate). No new PA-CONFLICT / PA-LOG / PA-TCR / PA-SPLIT.

---

## Summary

connector-cloud is a DEFERRED, honestly-scoped FUTURE spec (no crate, no tasks.md,
design.md "DEFERRED -- not in initial release"; 12 reqs / 10 EARS) targeting SharePoint
Online, OneDrive, and OAuth 2.0. It repeats the deferred-connector pattern (network-fs
W5.15, ftp-sftp W5.16) and CONFIRMS the PA-CONFLICT-021 refinement (targets the
connector-extensibility trait -- a deferred remote connector). Its OAuth/credential scope
is the clearest justification for the connector-extensibility AUTHENTICATION layer (local
needs no auth; cloud does), further supporting the "intentional base" reading. Two
forward-looking notes for when it is built (no action now): secure credential storage for
OAuth tokens (via the extensibility auth framework, not plaintext) and a provider-SDK
licence review. Rolls under PA-TRACK-006 (mark deferred specs) + PA-STD-071 (design.md
mojibake). No code, no new findings.
