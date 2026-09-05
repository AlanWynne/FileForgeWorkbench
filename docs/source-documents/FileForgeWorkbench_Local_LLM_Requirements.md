# FileForgeWorkbench Local LLM and AI Assistant Requirements

## 1. Document Control

| Field | Value |
|---|---|
| Document title | FileForgeWorkbench Local LLM and AI Assistant Requirements |
| Project | FileForgeWorkbench |
| Document type | Software Requirements and Architecture Specification |
| Status | Draft for incorporation into the project requirements baseline |
| Version | 1.0 |
| Date | 2026-09-05 |
| Licence consideration | Project licence and third-party model/runtime licences shall be assessed independently |

## 2. Purpose

This document defines the requirements for incorporating a small, locally hosted large language model into FileForgeWorkbench. The capability shall provide optional AI-assisted functions for mainframe file handling, JCL, COBOL, REXX, dataset interpretation, requirements engineering, documentation, repository search, and automated dialog-test generation.

The AI capability shall be implemented as a modular subsystem. FileForgeWorkbench shall remain fully operational when the subsystem is disabled, unavailable, incompatible, or not installed.

## 3. Objectives

The Local LLM and AI Assistant subsystem shall:

1. Provide useful AI assistance without requiring source files or organisational information to leave the user's machine or approved environment.
2. Support small models that can run on typical developer workstations.
3. Use retrieval-augmented generation to ground answers in selected project and technical documentation.
4. Avoid coupling FileForgeWorkbench to one model, model vendor, runtime, or inference API.
5. Allow AI-generated outputs to be reviewed before they affect files, configurations, repositories, tests, or commands.
6. Preserve the deterministic behaviour of the editor and mainframe-emulation functions.
7. Provide traceability between an answer and the local sources used to construct it.

## 4. Scope

### 4.1 In Scope

- Local or organisation-approved LLM inference.
- A provider-neutral AI service interface.
- Connection to local inference runtimes through an approved local API.
- Optional support for OpenAI-compatible inference endpoints.
- Retrieval-augmented generation over user-approved repositories and documents.
- Contextual assistance for text files, source code, JCL, REXX, COBOL, copybooks, dataset names, requirements, test scripts, and project documentation.
- Generation of proposed content, explanations, searches, requirements, and test scripts.
- Clear presentation of sources, confidence limitations, warnings, and generated-content status.
- Administrative controls, privacy controls, resource limits, logging controls, and model licence records.

### 4.2 Out of Scope for the Initial Release

- Training a foundation model.
- Unsupervised modification of source files.
- Autonomous execution of generated operating-system, TSO, JCL, SQL, Git, or application commands.
- Automatic upload of project files to public AI services.
- Treating model output as authoritative validation of JCL, source code, security, compliance, or production readiness.
- Replacing deterministic parsers, compilers, linters, schema validators, or policy engines with an LLM.

## 5. Guiding Design Principles

### 5.1 Optional and Non-Blocking

The AI subsystem shall be optional. Failure of the model runtime, retrieval index, model download, or inference request shall not prevent normal FileForgeWorkbench operation.

### 5.2 Local-First Processing

Local inference shall be the preferred deployment mode. Remote providers may be supported only when explicitly configured and permitted by applicable policy.

### 5.3 Provider Neutrality

The application shall communicate through an internal provider abstraction so that runtimes and models can be changed without redesigning editor functions.

### 5.4 Human Review

Generated text, code, requirements, commands, and test scripts shall be proposals. The user shall remain responsible for reviewing and accepting them.

### 5.5 Deterministic Core

LLM output shall not replace deterministic handling of FB, VB, VSAM, ISAM, PDS, GDG, encoding, record length, copybook parsing, or file conversion rules.

### 5.6 Source Grounding

Where an answer uses indexed documentation, the interface shall identify the supporting sources and distinguish retrieved material from model-generated interpretation.

### 5.7 Least Privilege

AI tools shall receive only the minimum context and permissions required for the requested operation.

## 6. Conceptual Architecture

```text
+------------------------------------------------------------+
|                    FileForgeWorkbench                      |
|                                                            |
|  +----------------+    +-------------------------------+   |
|  | Editor and UI  |--->| AI Assistant Orchestrator     |   |
|  +----------------+    +---------------+---------------+   |
|                                      |                    |
|                     +----------------+----------------+   |
|                     | Provider Abstraction Interface  |   |
|                     +----------+--------------+-------+   |
|                                |              |           |
|                 +--------------+--+       +---+---------+ |
|                 | Local Provider  |       | Approved     | |
|                 | Adapter         |       | Remote       | |
|                 +--------+--------+       | Adapter      | |
|                          |                +-------------+ |
|  +-----------------------+----------------------------+   |
|  | Retrieval, policy, prompt, audit, and tool services |   |
|  +----------+------------------+-----------------------+   |
|             |                  |                           |
|      +------v------+    +------v------------------+        |
|      | Local Model |    | Local Document Index   |        |
|      | Runtime     |    | and Metadata Store     |        |
|      +-------------+    +-------------------------+        |
+------------------------------------------------------------+
```

### 6.1 Primary Components

1. **AI Assistant User Interface**: Chat, editor actions, context selection, source display, safety warnings, and output review.
2. **AI Assistant Orchestrator**: Builds requests, applies policy, invokes retrieval, calls the selected provider, validates responses, and reports errors.
3. **Provider Abstraction Interface**: Presents a stable internal API independent of model runtime.
4. **Provider Adapters**: Implement communication with supported local or approved remote inference endpoints.
5. **Retrieval Service**: Indexes approved content, retrieves relevant passages, and returns source metadata.
6. **Prompt and Template Service**: Provides versioned task prompts for explanation, requirements extraction, test generation, and other supported functions.
7. **Policy and Permission Service**: Enforces privacy, workspace, file-type, endpoint, and tool-execution controls.
8. **Audit and Diagnostic Service**: Records configurable technical events without exposing protected source content by default.
9. **Model Registry**: Stores model identity, version or digest, context limits, provider, licence, source, and approval status.

## 7. Functional Requirements

### 7.1 AI Subsystem and Provider Management

#### FFW-AI-FR-001: Optional AI subsystem

**EARS requirement:** While the AI subsystem is disabled or unavailable, FileForgeWorkbench shall provide all non-AI editor, file-management, emulation, and automation capabilities without requiring an AI runtime.

**Acceptance criteria:**

- The application starts without an installed model.
- Normal file operations remain available.
- AI actions are hidden, disabled, or marked unavailable.
- A failed AI request does not terminate the application or corrupt the active document.

#### FFW-AI-FR-002: Provider abstraction

**EARS requirement:** The system shall expose an internal provider-neutral interface for chat completion, text generation, streaming responses, cancellation, health checking, and model capability discovery.

**Acceptance criteria:**

- UI components do not call a vendor-specific endpoint directly.
- A provider adapter can be replaced without changing editor command implementations.
- Provider-specific errors are converted to standard FileForgeWorkbench error categories.

#### FFW-AI-FR-003: Multiple provider configurations

**EARS requirement:** Where more than one approved AI provider is configured, the system shall allow the user or administrator to select the active provider and model.

#### FFW-AI-FR-004: Provider health check

**EARS requirement:** When an AI provider is selected, the system shall verify endpoint availability and model readiness before submitting an inference request.

#### FFW-AI-FR-005: Request cancellation

**EARS requirement:** When the user cancels an active AI request, the system shall stop response processing, release associated resources, and preserve any existing editor content.

#### FFW-AI-FR-006: Streaming output

**EARS requirement:** Where the selected provider supports streaming, the system shall display the response incrementally and shall allow the user to cancel it.

### 7.2 Context and Privacy Controls

#### FFW-AI-FR-010: Explicit context selection

**EARS requirement:** When the user invokes an AI action, the system shall identify the content that will be supplied as context before the request is submitted.

#### FFW-AI-FR-011: Minimum necessary context

**EARS requirement:** The system shall send only the selected text, required metadata, retrieved passages, and task instructions necessary to perform the requested AI operation.

#### FFW-AI-FR-012: Sensitive-content warning

**EARS requirement:** If the selected provider is remote and the request includes file or repository content, the system shall display the destination and an appropriate disclosure warning before submission unless an approved organisational policy has already governed that action.

#### FFW-AI-FR-013: Excluded paths and patterns

**EARS requirement:** The system shall support exclusion rules for files, directories, extensions, dataset patterns, secrets, credentials, keys, certificates, and other content that shall not be indexed or sent to an AI provider.

#### FFW-AI-FR-014: Secret detection

**EARS requirement:** When selected context appears to contain a configured secret pattern, the system shall block or redact that content and inform the user before an AI request is submitted.

#### FFW-AI-FR-015: No implicit background upload

**EARS requirement:** The system shall not upload workspace content to a remote AI provider solely because a workspace was opened or indexed locally.

### 7.3 Retrieval-Augmented Generation

#### FFW-AI-FR-020: Approved source indexing

**EARS requirement:** When a user with appropriate permission selects a source for indexing, the system shall index only supported and non-excluded content from that source.

#### FFW-AI-FR-021: Supported source types

**EARS requirement:** The retrieval service shall support an extensible set of source readers, including Markdown, plain text, source code, configuration files, and other formats enabled by installed readers.

#### FFW-AI-FR-022: Index metadata

**EARS requirement:** For each indexed unit, the system shall retain sufficient metadata to identify its source file, location, content version or hash, indexing date, and applicable access scope.

#### FFW-AI-FR-023: Incremental re-indexing

**EARS requirement:** When an indexed file changes, the system shall update the affected index entries without requiring a complete rebuild of unrelated content.

#### FFW-AI-FR-024: Source citations

**EARS requirement:** When retrieved content contributes to an AI response, the system shall display source references that allow the user to navigate to the supporting local content where possible.

#### FFW-AI-FR-025: Retrieval transparency

**EARS requirement:** When no sufficiently relevant local source is retrieved, the system shall indicate that the response was not grounded in project documentation.

#### FFW-AI-FR-026: Index removal

**EARS requirement:** When a source is removed from the index, the system shall delete its searchable content and associated retrieval metadata, subject to configured audit-retention rules.

### 7.4 Mainframe and FileForgeWorkbench Assistance

#### FFW-AI-FR-030: Dataset-name explanation

**EARS requirement:** When the user selects a z/OS-style dataset name and requests an explanation, the system shall generate a structured explanation of the recognised qualifiers, GDG generation and version notation, and any uncertain interpretation.

#### FFW-AI-FR-031: File-structure assistance

**EARS requirement:** When the user requests file-structure assistance, the system shall use available metadata and a bounded sample to propose possible FB, VB, delimited, encoded, or copybook-related interpretations without overriding deterministic detection results.

#### FFW-AI-FR-032: JCL assistance

**EARS requirement:** When the user requests JCL assistance, the system shall support explanation, drafting, review suggestions, and documentation-grounded help while clearly identifying generated JCL as unvalidated.

#### FFW-AI-FR-033: COBOL and REXX assistance

**EARS requirement:** When the user requests source assistance, the system shall support explanation, documentation, draft generation, refactoring suggestions, and test-case suggestions for supported languages.

#### FFW-AI-FR-034: Deterministic validation precedence

**EARS requirement:** Where a deterministic parser, compiler, linter, schema validator, or emulator produces a result, the system shall present that result separately from AI suggestions and shall not represent the AI suggestion as the deterministic result.

#### FFW-AI-FR-035: Repository-aware search translation

**EARS requirement:** When the user expresses a repository search in natural language, the system shall convert it into a reviewable search plan before executing the search.

#### FFW-AI-FR-036: Command proposal control

**EARS requirement:** When the model proposes a command, the system shall display the command and its target environment for review and shall not execute it without an explicit user action through the applicable controlled command interface.

### 7.5 Requirements Engineering Assistance

#### FFW-AI-FR-040: EARS requirement generation

**EARS requirement:** When the user requests requirements extraction or generation, the system shall produce proposed uniquely identifiable requirements using configured EARS templates.

#### FFW-AI-FR-041: Source traceability

**EARS requirement:** When an extracted requirement is based on indexed source material, the system shall associate the proposed requirement with the relevant source reference and passage location.

#### FFW-AI-FR-042: Requirement status

**EARS requirement:** The system shall mark AI-generated requirements as proposed until a user reviews and accepts them into the controlled requirements set.

#### FFW-AI-FR-043: Requirement quality checks

**EARS requirement:** When an AI-generated requirement is produced, the system shall support checks for ambiguous terms, compound obligations, missing conditions, missing actors, unverifiable outcomes, and duplicate intent.

### 7.6 Automated Dialog-Test Assistance

#### FFW-AI-FR-050: Test-script generation

**EARS requirement:** When the user describes a dialog workflow in natural language, the system shall generate a proposed script conforming to the installed FileForgeWorkbench dialog-automation schema.

#### FFW-AI-FR-051: Schema validation

**EARS requirement:** Before a generated dialog-test script can be executed, the system shall validate it against the applicable automation schema and report all validation errors.

#### FFW-AI-FR-052: Test review

**EARS requirement:** The system shall require the generated test script to be presented for review before its first execution.

#### FFW-AI-FR-053: Restricted test actions

**EARS requirement:** If a generated test contains a restricted or destructive action, the system shall block execution until the applicable permission and confirmation requirements are satisfied.

### 7.7 Output Review and Application

#### FFW-AI-FR-060: Generated-content labelling

**EARS requirement:** The system shall label AI-generated output so that it is distinguishable from existing file content, retrieved source text, deterministic diagnostics, and user-authored content.

#### FFW-AI-FR-061: Preview before modification

**EARS requirement:** When an AI action proposes a document modification, the system shall display a diff or equivalent preview before applying the change.

#### FFW-AI-FR-062: Atomic application

**EARS requirement:** When the user accepts an AI-generated edit, the system shall apply it as a single undoable editor operation where technically feasible.

#### FFW-AI-FR-063: Partial acceptance

**EARS requirement:** Where the generated output contains separable changes, the system shall allow the user to accept or reject individual changes where technically feasible.

#### FFW-AI-FR-064: Output provenance

**EARS requirement:** The system shall retain, for the active session or configured history period, the provider, model identifier, prompt-template version, selected context description, retrieved-source references, and generation timestamp associated with an accepted output.

### 7.8 Configuration and Administration

#### FFW-AI-FR-070: Model registry

**EARS requirement:** The system shall maintain configuration metadata for each model, including model name, provider, version or digest where available, context limit, licence identifier, source, approval status, and enabled capabilities.

#### FFW-AI-FR-071: Approved provider list

**EARS requirement:** Where administrative policy is enabled, the system shall allow only approved providers, endpoints, models, and embedding models to be configured.

#### FFW-AI-FR-072: Resource limits

**EARS requirement:** The system shall support configurable limits for request size, retrieved context, output size, concurrent requests, memory usage, and inference duration.

#### FFW-AI-FR-073: Per-workspace controls

**EARS requirement:** The system shall support workspace-specific AI enablement, indexed sources, exclusions, model selection, and tool permissions.

#### FFW-AI-FR-074: Configuration portability

**EARS requirement:** The system shall separate portable AI configuration from machine-specific endpoint paths, credentials, and local model locations.

## 8. Non-Functional Requirements

### 8.1 Security

#### FFW-AI-NFR-001: Secure endpoint handling

Provider credentials and tokens shall be stored through an operating-system or organisation-approved secret-storage mechanism and shall not be written in plaintext to project files or normal logs.

#### FFW-AI-NFR-002: Transport protection

Remote provider communication shall use an organisation-approved encrypted transport configuration.

#### FFW-AI-NFR-003: Prompt-injection containment

Retrieved documents shall be treated as untrusted data. Instructions found inside retrieved content shall not automatically override system policy, workspace policy, user intent, or tool restrictions.

#### FFW-AI-NFR-004: Tool isolation

Model-generated tool requests shall be validated against an allow-list, parameter schema, permission boundary, and user-confirmation policy before execution.

#### FFW-AI-NFR-005: Untrusted output

Generated code, commands, HTML, Markdown, paths, links, and configuration values shall be treated as untrusted until validated or accepted by the user.

### 8.2 Privacy

#### FFW-AI-NFR-010: Local data boundary

When the active provider is configured as local-only, request content shall remain within the configured local inference and retrieval boundary.

#### FFW-AI-NFR-011: Content logging default

Source content, prompts, retrieved passages, and model responses shall not be written to persistent diagnostic logs by default.

#### FFW-AI-NFR-012: History control

The user or administrator shall be able to disable AI conversation history and delete locally retained AI history and indexes.

### 8.3 Performance and Resource Management

#### FFW-AI-NFR-020: UI responsiveness

Inference, embedding, indexing, and model-health operations shall execute without blocking the main user-interface thread.

#### FFW-AI-NFR-021: Graceful resource pressure

If insufficient memory, storage, or compute capacity prevents an AI operation, the system shall fail the operation gracefully and leave the active document unchanged.

#### FFW-AI-NFR-022: Bounded context

The orchestrator shall enforce configured token or size budgets for selected context, retrieval results, chat history, and generated output.

#### FFW-AI-NFR-023: Indexing progress

Long-running indexing operations shall expose progress, cancellation, error reporting, and resumable or incremental behaviour where supported.

### 8.4 Reliability

#### FFW-AI-NFR-030: Failure isolation

A provider, model, embedding, or retrieval failure shall be isolated from the editor and core file-processing subsystems.

#### FFW-AI-NFR-031: Timeout handling

AI requests shall use configurable connection, response, and overall operation timeouts.

#### FFW-AI-NFR-032: Reproducibility metadata

The system shall record enough non-sensitive metadata to identify the model and prompt-template versions used to generate an accepted artefact.

### 8.5 Usability and Accessibility

#### FFW-AI-NFR-040: Clear state

The interface shall clearly show whether AI is disabled, ready, indexing, generating, unavailable, cancelled, or in error.

#### FFW-AI-NFR-041: Keyboard operation

AI assistant functions shall be operable using the keyboard and shall integrate with the accessibility conventions used by FileForgeWorkbench.

#### FFW-AI-NFR-042: Explain limitations

The interface shall state that AI output may be incomplete or incorrect and that generated technical artefacts require validation.

### 8.6 Maintainability and Extensibility

#### FFW-AI-NFR-050: Adapter extensibility

New provider adapters shall be addable without modifying the public behaviour of existing adapters.

#### FFW-AI-NFR-051: Versioned prompts

Task prompts and output schemas shall be version-controlled and independently testable.

#### FFW-AI-NFR-052: Automated tests

The AI subsystem shall include unit tests for policy, redaction, prompt construction, response parsing, provider errors, retrieval metadata, and generated-edit application.

#### FFW-AI-NFR-053: Mock provider

The project shall provide a deterministic mock provider for automated testing without loading a real model.

### 8.7 Cross-Platform Compatibility

#### FFW-AI-NFR-060: Supported platforms

The provider abstraction and AI user interface shall operate consistently across the Windows, Linux, and macOS platforms supported by FileForgeWorkbench, subject to the capabilities of the selected runtime.

#### FFW-AI-NFR-061: Runtime independence

FileForgeWorkbench shall not require that every supported model runtime be available on every supported platform.

## 9. Model and Runtime Selection Requirements

The implementation shall not mandate one model. Candidate models and runtimes shall be assessed against:

- Model and runtime licence compatibility.
- Commercial-use and redistribution terms.
- Model size and quantisation options.
- Memory and storage requirements.
- CPU, GPU, and accelerator support.
- Context-window requirements.
- Structured-output reliability.
- Code and technical-documentation capability.
- Local inference performance.
- Security maintenance and provenance.
- Availability on the FileForgeWorkbench target platforms.
- Ability to pin a model version or content digest.

The installation design should prefer downloading or connecting to models separately from the main FileForgeWorkbench binary unless redistribution has been explicitly approved.

## 10. Licensing and Legal Requirements

#### FFW-AI-LIC-001: Independent licence assessment

Each model, tokenizer, embedding model, runtime, library, and supplied dataset shall undergo an independent licence and usage-terms assessment before inclusion or recommendation.

#### FFW-AI-LIC-002: Licence record

The model registry shall record the declared licence or usage terms and the location from which the model was obtained.

#### FFW-AI-LIC-003: Redistribution control

A model shall not be bundled with FileForgeWorkbench unless redistribution, commercial use, modification, and notice obligations have been reviewed and approved.

#### FFW-AI-LIC-004: Attribution and notices

Where required, FileForgeWorkbench distributions shall include the applicable copyright, attribution, licence, and model-use notices.

#### FFW-AI-LIC-005: Source-document rights

Indexing or using documentation with the AI subsystem shall not be interpreted as granting rights to redistribute, publish, or create externally distributed derivative copies of that documentation.

## 11. Audit and Observability

The subsystem shall support configurable recording of:

- Request identifier.
- Timestamp.
- Workspace identifier or non-sensitive alias.
- Provider and model identifier.
- Operation type.
- Prompt-template version.
- Retrieval source identifiers.
- Duration and outcome.
- Cancellation, timeout, policy block, or validation failure.
- Whether generated output was accepted, rejected, or partially applied.

The subsystem shall avoid recording source content, credentials, secrets, full prompts, or full responses unless an explicitly approved diagnostic mode is enabled.

## 12. Suggested Initial Use Cases

The first implementation increment should concentrate on low-risk, reviewable assistance:

1. Explain selected JCL, COBOL, REXX, dataset names, copybook fields, and project documentation.
2. Ask questions over locally indexed FileForgeWorkbench documentation.
3. Generate draft EARS requirements with source traceability.
4. Generate draft dialog-automation scripts that must pass schema validation.
5. Propose repository searches and regular expressions for user review.
6. Generate comments, summaries, and documentation without directly changing source files.

Later increments may add controlled edit proposals, deeper repository context, provider plugins, and policy-governed tool invocation.

## 13. Acceptance Test Scenarios

### ATS-AI-001: Start without AI

**Given** no AI runtime or model is installed  
**When** FileForgeWorkbench starts  
**Then** all core non-AI functions remain available  
**And** AI functions report that no provider is configured.

### ATS-AI-002: Local grounded question

**Given** an approved local document has been indexed  
**And** a local model provider is ready  
**When** the user asks a question answered by the indexed document  
**Then** the response identifies the supporting source  
**And** the user can navigate to the cited content where supported.

### ATS-AI-003: No relevant retrieval result

**Given** the local index does not contain relevant material  
**When** the user asks a question  
**Then** the interface indicates that the response is not grounded in project documentation.

### ATS-AI-004: Remote-provider disclosure

**Given** a remote provider is selected  
**And** file content is selected as context  
**When** submission is not already governed by an approved policy  
**Then** the system displays the destination and disclosure warning before submission.

### ATS-AI-005: Secret protection

**Given** selected context contains a configured secret pattern  
**When** the user invokes an AI operation  
**Then** the system blocks or redacts the secret  
**And** informs the user of the action taken.

### ATS-AI-006: Generated edit review

**Given** the model proposes a source-file change  
**When** the response is returned  
**Then** the system displays a diff  
**And** does not modify the file until the user accepts the change  
**And** the accepted change can be undone as a single editor operation where technically feasible.

### ATS-AI-007: Provider failure

**Given** an inference request is active  
**When** the provider becomes unavailable  
**Then** the system reports a standard provider error  
**And** preserves the active document  
**And** remains usable.

### ATS-AI-008: Generated dialog test

**Given** the user describes a dialog workflow  
**When** the model generates a test script  
**Then** the system validates it against the installed schema  
**And** reports all schema violations  
**And** requires review before first execution.

### ATS-AI-009: Prompt injection in retrieved content

**Given** an indexed document contains instructions directed at the model  
**When** that document is retrieved as context  
**Then** those instructions do not override configured system policy, workspace policy, or tool restrictions.

### ATS-AI-010: Model traceability

**Given** the user accepts AI-generated content  
**When** provenance information is inspected  
**Then** the system identifies the provider, model, prompt-template version, generation timestamp, and retrieved-source references without exposing stored credentials.

## 14. Risks and Mitigations

| Risk | Required mitigation |
|---|---|
| Hallucinated technical guidance | Label generated content, provide sources, preserve deterministic validation, and require user review. |
| Exposure of sensitive files | Use local-first processing, explicit context, exclusions, secret detection, endpoint controls, and minimal context. |
| Prompt injection from indexed documents | Treat retrieved text as untrusted data and enforce non-overridable policy boundaries. |
| Excessive workstation resource use | Support small or quantised models, background workers, limits, cancellation, and graceful failure. |
| Vendor or runtime lock-in | Use a provider abstraction and portable request and response models. |
| Model licence incompatibility | Maintain a model registry and complete licence review before bundling or recommendation. |
| Non-reproducible output | Record model, provider, template, source, and timestamp metadata for accepted artefacts. |
| Unsafe generated commands | Use review, allow-lists, schemas, permission checks, and explicit execution actions. |
| Stale retrieval index | Detect content changes and support incremental re-indexing. |
| AI feature blocks core work | Isolate failures and make the complete subsystem optional. |

## 15. Phased Delivery Recommendation

### Phase 1: Provider Foundation

- Provider abstraction.
- One local provider adapter.
- Health checks, streaming, cancellation, configuration, and mock provider.
- Read-only chat and selected-text explanation.

### Phase 2: Local Retrieval

- Local document readers.
- Chunking, embeddings, vector retrieval, source metadata, exclusions, and index lifecycle.
- Question answering with source links.

### Phase 3: Domain Assistance

- JCL, COBOL, REXX, dataset, copybook, FB/VB, GDG, and documentation task templates.
- EARS requirement generation and quality checks.
- Dialog-test script generation and schema validation.

### Phase 4: Controlled Editing and Tools

- Diff-based edit proposals.
- Partial acceptance and undo integration.
- Reviewable repository searches.
- Policy-governed and permission-controlled tool invocation.

### Phase 5: Enterprise Controls

- Approved-provider policies.
- Central configuration options.
- Enhanced audit integration.
- Remote-provider support where authorised.
- Model and prompt evaluation suites.

## 16. Definition of Done for the Initial Capability

The initial Local LLM capability shall be considered complete when:

1. FileForgeWorkbench operates normally with AI disabled or absent.
2. A configured local provider can be discovered, health-checked, invoked, streamed, and cancelled.
3. Selected text can be explained without automatically modifying the source document.
4. At least one approved local documentation source can be indexed and cited in a grounded response.
5. Excluded paths and configured secret patterns are not supplied to the provider.
6. Provider failure does not affect editor stability or file integrity.
7. Generated content is labelled and reviewable.
8. The deterministic mock provider supports automated integration testing.
9. Model, provider, prompt-template, and source metadata are available for accepted generated artefacts.
10. Third-party runtime, model, and library licence records are documented.

## 17. Open Design Decisions

The following decisions shall be resolved during detailed design:

- Default local runtime and supported API protocol.
- Initial model-size and workstation-resource targets.
- Embedding model and vector-index implementation.
- Chunking strategies for Markdown, source code, copybooks, manuals, and requirements.
- Whether the AI subsystem runs in-process, in a separate FileForgeWorkbench service, or through an external runtime only.
- Provider plugin packaging and version compatibility.
- Local index encryption and retention requirements.
- Enterprise policy integration.
- Supported structured-output schema and retry strategy.
- Evaluation datasets for JCL, COBOL, REXX, EARS requirements, and dialog automation.
- Distribution approach for models, including download, user-supplied models, or organisation-managed deployment.

## 18. Traceability Summary

| Objective | Primary requirements |
|---|---|
| Optional AI capability | FFW-AI-FR-001, FFW-AI-NFR-030 |
| Local-first privacy | FFW-AI-FR-010 to FFW-AI-FR-015, FFW-AI-NFR-010 to FFW-AI-NFR-012 |
| Provider neutrality | FFW-AI-FR-002 to FFW-AI-FR-006, FFW-AI-NFR-050 |
| Grounded answers | FFW-AI-FR-020 to FFW-AI-FR-026 |
| Mainframe assistance | FFW-AI-FR-030 to FFW-AI-FR-036 |
| Requirements engineering | FFW-AI-FR-040 to FFW-AI-FR-043 |
| Test automation | FFW-AI-FR-050 to FFW-AI-FR-053 |
| Human-controlled changes | FFW-AI-FR-060 to FFW-AI-FR-064 |
| Secure operation | FFW-AI-NFR-001 to FFW-AI-NFR-005 |
| Licence compliance | FFW-AI-LIC-001 to FFW-AI-LIC-005 |

---

**End of document**
