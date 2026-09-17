# Architecture

## Dependency direction

```text
main.ts / KnovumApp.vue    composition root
        |
        +--> app/          application composition and app-wide policy
        |     +--> features/*
        |     +--> foundation/*
        |     +--> shared/*
        |
        +--> shell/        stable desktop workspace chrome
        |     +--> foundation/*
        |     +--> shared/*
        |
        +--> features/*    product capabilities
        |     +--> foundation/*
        |     +--> shared/*
        |
        +--> foundation/*  cross-cutting product infrastructure
              +--> shared/*

shared/* -----------------> shared/* only
```

`dependency-cruiser` enforces the graph, including feature-to-feature isolation. Oxlint owns code-quality rules; it is not used as a dependency-graph engine.

## Ownership

- `app/`: composition and policy that genuinely spans the application: router, workspace-session wiring, runtime diagnostics and bootstrap orchestration.
- `foundation/`: reusable cross-cutting infrastructure needed by product capabilities: bootstrap state, commands/shortcuts, i18n, preferences, secrets and background-task presentation/Channel integration.
- `shell/`: workspace/sidebar/inspector layout, command surface, task center and app/feature error boundaries.
- `features/<name>/`: business ownership. Start flat and split only after real responsibilities emerge.
- `shared/contracts/`: stable neutral types used by several owners.
- `shared/ui/`: Knovum visual components. They add desktop styling/semantics while Reka owns interaction behavior.
- `src-tauri/src/desktop/`: lifecycle that must remain correct while the WebView is hidden or unavailable.

## Spaces ubiquitous language

`Spaces` is the product bounded context. A `Space` is its aggregate root: nodes,
relations, scenes, knowledge types, knowledge state, assets, retrieval records and package data are
owned through one `spaceId`. The `Inbox` is the one system Space.

The Spaces Browser is application navigation that lists Inbox and user Spaces; it is not a second
domain model. `spaces` names the bounded context and its external namespace, while `space` names behavior
scoped to one aggregate. Source directories, database tables, IPC commands, types and UI components
must use this language directly. Aliases, compatibility readers and dual command registrations are
not permitted before the first public release.

The Spaces Rust boundary is divided by responsibility, not CRUD mechanics:

- `knowledge/`: semantic contracts, provenance, patches, events and knowledge governance;
- `knowledge/indexing/`: extraction of revisions, passages and knowledge chunks from Space nodes;
- `knowledge/chunk_store/`: transactional persistence and revision rules for derived knowledge chunks;
- `knowledge/retrieval/`: Space-owned vector indexing, semantic search and hybrid ranking;
- `knowledge/grounding/`: grounded-answer contracts, prompt construction, citations and reference cleanup;
- `derivation/`: application use cases that transform current Space resources into
  traceable derivative resources. Each capability owns its preparation, engine orchestration,
  completion and failure lifecycle; shared lifecycle policy stays at this boundary rather than in
  commands or engine adapters;
- `commands/`: Space-level state-changing application operations that do not have a more specific
  capability owner;
- `queries/`: read-only database projections and response mapping;
- `nodes/`: node mutation, lifecycle and node-owned derived-state updates; current canvas geometry belongs only to `canvas_state`;
- `assets/`: Space asset authorization, import, enrichment, references and lifecycle;
- `assets/transcription/`: the Space-owned transcription use case, including asset authorization and knowledge persistence;
- `previews/`: Space preview-cache paths and mapping of neutral engine results into `PreparedAsset`;
- `package/`: Space package read, write, validation and transactional restore;
- `agent_artifacts/`: agent artifact workspace creation, preview, import, recovery and consumption;
- `recovery.rs`: ordered reconciliation of interrupted Space-owned workflows during bootstrap.

Content and platform engines are independent crate-level capabilities: `documents/` owns extraction
and generation, `media/` owns acquisition and media transformation, and `images/` owns image
operations. `file_inspection/`, `web_capture/`, `system_preview/` and `local_resources/` remain
separate capabilities. `local_resources/models` owns downloaded model installation and integrity;
`local_resources/tools` owns discovery of bundled executables and native libraries. None depends on `spaces` or emits
Space domain types. A Space command authorizes an asset before calling `system_preview`; the preview
engine never becomes a general Renderer file capability.

`file_inspection/classification` is the single authority for imported resource classification.
Inspection, Space import and system-thumbnail eligibility consume its typed result; they must not
maintain parallel extension lists or infer semantic categories from persisted node-kind strings.

`job_registry.rs` is the crate-private lifecycle primitive for in-memory work. It owns atomic
registration, cancellation lookup and lease-based cleanup; each capability owns a separate typed
registry for its jobs. `ai/executions.rs` therefore represents active AI executions rather than a
second job framework, while model installation keeps its domain snapshot in a typed registry.
Long-lived recording sessions, singleton model engines, credential sessions and serialization locks
are state owners rather than jobs and do not use this primitive.

Within `media/`, `compute_budget` owns interactive CPU allocation policy and `subprocess` owns child
process waiting, cancellation and bounded diagnostics. These names describe concrete responsibilities;
they are not general concurrency or process-management frameworks.

Spaces processing and retrieval are Rust application use cases, not WebView workflows:

- `assets/enrichment/jobs/` owns durable processing-job state, Rust concurrency lanes, media/vision
  execution and system-authored node updates. Vue submits a job and presents Channel progress;
- `knowledge/retrieval/` owns embedding instructions, model identity, batching, LanceDB writes, semantic search
  and hybrid rank fusion. Vue submits query text or an indexing intent and never transports vectors;
- `commands/grounded/` owns the conversation use case and streaming exchange persistence, while
  `knowledge/grounding/` owns its reusable semantic contracts, prompt assembly, citation rules and
  reference cleanup. Vue owns assistant presentation only.

Raw persistence transitions, embedding vectors and partial grounded-AI steps are not IPC contracts.

Creation Workflows may combine Rust derivations with named Renderer adapters such as Web Worker
document rendering. Rust owns the registered definition, dependency planning, legal state transitions,
durable run persistence and interrupted-run reconciliation. The Renderer executes only the next step ID
authorized by Rust, presents progress and returns the named adapter output. Workflow adapters never
receive SQL, raw persistence transitions or engine-owned mutable state.

The knowledge embedding model and vector dimensions form one Spaces retrieval-index contract. They are intentionally not a user-switchable generation preference: changing either requires an explicit index-contract change and complete rebuild. The generic `ai::embedding` adapter accepts a resolved target and dimensions but does not own the selected model identity.

Expose one command per user/application intent so the WebView cannot assemble a second domain
implementation from low-level Rust primitives.

`queries/` never mutates durable state. `commands/` and `derivation/` may depend on
`knowledge/`, `knowledge/indexing/` and infrastructure adapters; semantic policy must not be duplicated in
a command, derivation capability or query adapter. `knowledge/` never depends on
`derivation/`: governance and provenance policy remain independent of the use cases that
produce derivative resources.

Read and write files for the same aggregate are not duplicate implementations when they form an
intentional command/query split. Two files are duplicate capability implementations only when both
decide the same policy, validate the same invariant, translate the same boundary or own the same
side effect. Shared policy moves inward; adapters remain separate at their external boundaries.

## Module decomposition

Directories express ownership and stable boundaries, not temporary implementation steps. A small
capability stays flat. Split it into a directory only after it owns at least two distinct
responsibilities; once split, all of its implementation moves under that owner instead of remaining
as prefixed siblings. Renderer features use capability names such as `canvas/`, `assistant/`,
`indexing/` and `queries/`. Persistence adapters live with the capability whose data contract they
implement; a feature-wide `persistence/` directory that collects unrelated aggregates is forbidden.
They do not mirror backend architecture layers with generic `application/`, `domain/` or
`infrastructure/` directories. A feature's `session/` directory
is reserved for its Pinia composition root and the cross-capability actions assembled by that root;
business policy remains with the owning capability.

Large bounded contexts such as Spaces split by business capability first. Shared Renderer contracts
and pure presentation transforms live in `model/`; this does not imply an independent domain layer.
UI shape (`components`, `views`) and technical lifecycle (`state`, `runtime`) remain subordinate to
their business owner; they do not become peer product domains.

Within Spaces, workspace persistence belongs to `workspace/`, node persistence to `nodes/`, relation
persistence to `knowledge/relations/`, scene persistence to `scenes/`, credential persistence to
`credentials/`, search transport to `search/`, and canvas snapshots to `canvas/persistence/`.
`session/` may assemble their contracts but does not become their implementation owner.

The Spaces UI composition root keeps cross-capability visual composition under `presentation/`.
Canvas surfaces, knowledge presentation, rich-editor controls, media playback and derivation dialogs
live under `presentation/canvas`, `presentation/knowledge`, `presentation/editor`,
`presentation/media` and `presentation/derivations` respectively. The Konva adapter owns geometry,
hit testing, selection and transforms. Product-owned rich node content remains in Vue presentation
components mounted through the neutral canvas content slot; those components do not own canvas
geometry or engine state. Presentation may consume capability actions and query contracts but never
imports persistence adapters directly.

Do not create catch-all ownership such as `application`, `domain`, `infrastructure`, `services`,
`utils`, `helpers`, `composables`, `common`, `misc`, or `platform` inside Renderer features. Direct
imports preserve the real owner; barrel modules and rename-only
wrappers do not create architecture boundaries. Detailed path, file, suffix, identifier and test
rules live only in `naming-conventions.md`.

## One source of truth

`app/router.ts` owns the fixed Spaces and Settings routes. `features/spaces/workspace.ts` exposes the single
composition contract for Spaces-owned shell surfaces and commands. Do not introduce a product-module
registry: Knovum is one knowledge-space application, not a container for peer applications.

The `AppCommand` contract is the single source of truth for command-palette entries and application-local shortcuts. Application menu items emit the same command IDs instead of owning a second action implementation.

## Direct vendor API rule

Use Vue, Reka, VueUse, TanStack and official Tauri APIs directly. A local abstraction is allowed only when it owns Knovum policy, validation, security, domain translation or real multi-API orchestration.

Rename-only wrappers are rejected. `readClipboardText()` around Tauri `readText()` is not an architecture boundary.

## State ownership

- local visual state: component-local refs;
- cross-view session state: Pinia;
- async/server cache: TanStack Vue Query;
- durable non-secret preferences: Tauri Store through the strict current v1 preferences schema;
- structured local product data/indexes: feature-owned SQLite; before the first public release each
  database has one current SQLx v1 baseline and no compatibility migration chain;
- credentials/tokens/secrets: foundation keyring commands backed by OS credential stores;
- long Rust work: feature-owned Rust implementation streamed to the shared Task Center through Tauri Channels.

Do not mirror the same state across several stores without an explicit synchronization design.

## Validation boundary

Validate data that crosses a real trust or persistence boundary: user input, Store/SQLite data, filesystem/network data and Rust IPC payloads. Do not add runtime assertions for static registries and other invariants entirely owned by this repository when TypeScript/Rust structure already guarantees them.

## No compatibility fallback

Knovum targets Tauri desktop only. Required startup infrastructure fails explicitly. Expected user/platform outcomes such as cancelled dialogs, denied notifications, unavailable global shortcuts or unsupported taskbar decoration are handled as normal runtime outcomes, not hidden by a generic compatibility layer.

## Size/complexity gates

Oxlint fails at 180 non-blank/non-comment lines per TS/Vue file, complexity above 10 and other correctness rules. These are ceilings, not targets. Function length is reviewed through responsibility, dependency direction and complexity instead of a physical line-count limit. dependency-cruiser, Knip and jscpd catch architecture violations, dead code and substantial copy/paste duplication.

No source path alias is configured. Relative paths intentionally expose ownership and dependency direction.

## Persistence split

Preferences use a strict v1 Tauri Store envelope. Before first public release, only the current v1 schema is accepted; incompatible development data is reset explicitly instead of carrying compatibility readers or migration chains. The same rule applies to other internal Store/package/protocol envelopes. The first public release must explicitly replace this development policy with a production migration policy if persisted compatibility becomes a supported product contract. The non-sensitive last-workspace route is a single WebView `localStorage` string so optional session restore cannot make the Store file a bootstrap dependency. Feature structured data belongs to feature-owned SQLite schemas. Secrets belong only in the OS credential store.

## Runtime architecture

Source dependency direction is intentionally separate from process/runtime topology. The system-level view, hidden-main startup sequence, IPC/task flow, storage ownership and platform lifecycle are documented in `system-architecture.md`.

## Integration settings and Rust engines

Provider and Desktop Agent configuration shared by otherwise isolated features belongs to `foundation/ai/`. It owns the strict current v1, non-secret `ai.models` contract and the tagged `ModelTarget | AgentTarget` execution-routing contract. Provider connections own user-editable endpoints and model definitions; official OpenAI and OpenAI-compatible endpoints are explicit protocol choices. Agent runtimes own installation readiness and enabled state and are not `AiProvider` implementations. Model and Agent are peer execution-target types: routing has no built-in type priority, the user selects per-intent defaults, and cross-type fallback requires explicit user configuration. Compatibility is derived from the intent's input/output contract plus runtime capability evidence; it must not be encoded as a permanent text-only Agent allowlist. An Agent may satisfy media creation through a verified native/tool capability or a scoped workspace-artifact workflow. Reusable generation presets and versioned creation workflows use the separate `creation.library` contract. Rust registries project Provider, Agent, Knowledge Action and Workflow definitions to the Renderer; Rust resolves stable target IDs to endpoint, credential and adapter at execution time. Secret values are persisted only through `foundation/secrets/` and the OS credential store.

Image processing, media processing, AI execution and storage are Rust or foundation engines consumed by
Spaces capabilities. They are not independent Renderer products. Each engine exposes only the commands and
permissions required by its owning knowledge-space workflow. External paths enter through webview drop or OS
dialogs, receive session-scoped authorization, and are never covered by a broad product-level filesystem scope.
