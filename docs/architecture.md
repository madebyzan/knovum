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

- `actions/`: the versioned `SpaceAction` contract and read-only catalog; concrete action behavior
  remains in the domain owner that validates and commits it;
- `authorization/`: principals, scopes, grants and effect-specific approvals;
- `runs/`: durable Run, StepInstance and Attempt state, scheduling, recovery and events;
- `workflows/`: versioned definitions, compilation and execution over SpaceActions;
- `assistant/`: Space-specific conversations, context, tasks, tool calls and checkpoints;
- `mcp/`: inbound Space protocol projection over Actions and Authorization, never direct mutation;
- `knowledge/`: semantic contracts, provenance, patches, events and knowledge governance;
- `knowledge/indexing/`: extraction of revisions, passages and knowledge chunks from Space nodes;
- `knowledge/chunk_store/`: transactional persistence and revision rules for derived knowledge chunks;
- `knowledge/retrieval/`: Space-owned vector indexing, semantic search and hybrid ranking;
- `knowledge/grounding/`: answer-with-sources contracts, prompt construction, citations and reference cleanup;
- `derivation/`: application use cases that transform current Space resources into
  traceable derivative resources. Each capability owns its preparation, engine orchestration,
  completion and failure lifecycle; shared lifecycle policy stays at this boundary rather than in
  commands or engine adapters;
- `commands/`: Space-level state-changing application operations that do not have a more specific
  capability owner;
- `queries/`: read-only database projections and response mapping;
- `nodes/`: content mutation, lifecycle and node-owned derived-state updates; it does not own structure or layout;
- `structure/`: the authoritative Structure/Layout/Edit-State aggregate, validation, mutation planning, history, clipboard, deletion and cross-Space structural integrity;
- `canvas_view_state.rs`: disposable per-Space camera persistence; camera state is intentionally outside Structure/Layout history;
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

Workflows compile and execute entirely through the Rust-owned Run scheduler. A Workflow Action invokes
the same domain use case as UI or MCP, under a persisted grant and idempotency key. Renderer-only
document assembly remains an implementation detail of its owning derivation rather than a second
Workflow coordinator. Vue owns draft interaction and observation only; it never receives SQL, raw
persistence transitions, leases or authority to mark a step complete.

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
as prefixed siblings. Renderer features use capability names such as `canvas/`, `conversation/`,
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

Within Spaces, workspace persistence belongs to `workspace/`, content persistence to `nodes/`,
Structure/Layout persistence to `structure/`, camera persistence to `canvas/persistence/`, relation
persistence to `knowledge/relations/`, scene persistence to `scenes/`, credential persistence to
`credentials/`, and search transport to `search/`.
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
- credentials/tokens/secrets: foundation commands backed by the dedicated local `secrets.db`;
- long Rust work: feature-owned Rust implementation streamed to the shared Task Center through Tauri Channels.

Do not mirror the same state across several stores without an explicit synchronization design.

For Spaces, SQLite Structure/Layout is the only authority for item identity, parentage, order,
local transforms, Section frames, lock state, appearance and relation-style overrides. Content rows
own knowledge payloads and source state. The Konva document is an ephemeral projection adapter and
the camera is separate disposable view state. Renderer mutations submit one typed intent with an
expected edit revision and mutation identity; Rust validates and commits the complete invariant.

## Space operation effects and source identity

Classify an operation by its durable effect and identity contract, not by the toolbar that exposes
it, whether it uses AI, or whether it runs in Rust. This classification is policy for each owning
capability; it is not a new universal executor or a second action catalog.

| Effect          | Identity and persistence                                                                                               | Product surface                         |
| --------------- | ---------------------------------------------------------------------------------------------------------------------- | --------------------------------------- |
| Inspect         | Reads current state; may create disposable cache files, never Space history or lineage                                 | Open, preview, search                   |
| Edit existing   | Retains the target node, relation or Structure identity; commits a validated change with undo/history where supported  | Edit, adjust, annotate, arrange         |
| Create primary  | Gives an independently authored or acquired resource a new identity, without `derives-from`                            | Add, import, capture                    |
| Clone           | Gives a copy a new identity and preserves the copied state; it does not claim transformation provenance                | Duplicate                               |
| Derive          | Creates one or more new ordinary resource nodes from pinned inputs; records a Derivation and `derives-from` relations  | Create from selection, generate, render |
| Export external | Writes outside the Space through an explicit destination or system handoff; does not create a Space node or Derivation | Download, export file                   |

Temporary interaction occupancy is separate from durable effect. A node editor, image tool,
comment gesture, video editor or canvas drag owns the current interactive draft until Apply,
Cancel or gesture completion. Read-only inspection and viewport navigation may continue. Starting
another edit, structural mutation, primary creation, clone, export or derivation while that draft is active is
blocked at the originating UI action **and** the command/submit entry point. The gate projects
existing feature session state; it does not store a second copy of an editor's draft. A submitted
background derivation releases the interactive gate and runs against its pinned input revision;
Rust revision validation remains the authority at commit. Image tool switching replaces the
node bar with the active tool panel, while image generation popovers share one open state.
Image comment placement uses one toolbar inside the image for point, drawing, region and exit;
the selection action bar is hidden for that session without cancelling its draft.

The same rule applies to every node kind: note/text/Markdown editing, image editing and comments,
video editing, and structured knowledge editing are exclusive interactive work; details and
read-only previews are inspection. Groups and Sections use Structure interaction occupancy rather
than content-node editing. A new operation must declare both its durable effect above and whether
it holds an interactive draft before it is exposed in a bar, menu, shortcut or automation surface.

An image or media transformation either edits its input node or publishes a separate resource; a
command that does both must be split into separately named intents. A KnowledgeChangeSet may
atomically contain several validated knowledge mutations without becoming a media Derivation. A
derived output may later be edited as its
own node; editing it does not rewrite the input or its historical Derivation. A failed or cancelled
Derivation may leave a clearly failed pending output, never a completed lineage claim. Cheap local
processing is not automatically an edit, and AI processing is not automatically a Derivation.

The node-kind decision is explicit:

| Kind                 | Edits the same identity                                                                        | Creates a derivative when requested                                                |
| -------------------- | ---------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| Note, text, Markdown | Body, title and tags                                                                           | Summary, drafted document, visualization or other independently useful output      |
| Image                | Crop, rotation, flip, tone, eraser recipe and comments                                         | Background removal, upscale, new angle, separated layers or generated image        |
| Video                | Edit project, title, tags and annotations                                                      | Rendered video, subtitle resource, burn-in output or generated clip                |
| Audio                | Title, tags and corrected node-owned transcript/analysis                                       | Separately published subtitle, transcript or document resource                     |
| Subtitle             | Language, cues and corrections                                                                 | Burned-in video or a separately authored document                                  |
| Website              | URL, captured metadata and annotations of that saved resource                                  | Summary, document or visualization grounded in its captured revision               |
| File, code           | Title, tags and other node metadata; an explicit source replacement changes its asset revision | Converted document, generated code or other new resource                           |
| Knowledge object     | Typed properties, status, title and evidence relations                                         | A new knowledge resource only when the intent explicitly creates one               |
| Credential           | Metadata and OS-vault secret reference                                                         | No credential derivative; generated artifacts may consume an authorized credential |
| Portal               | Target and presentation metadata                                                               | No content derivative; duplicate is a clone                                        |

Groups and Sections are Structure entities rather than content nodes. Move, resize, reorder, lock,
align, reparent, rename and dissolve mutate Structure. Saved Views mutate their own scene record;
Relations mutate their own relation record. Creating a blank node, importing a file or capturing a
website is primary creation. Duplicating any of these is a clone. Indexing, thumbnails, proxy media
and node-owned enrichment are projections or maintained state, not user-facing Derivations. The
existing `SpaceActionImpact` (`read_only`, `mutates_resource`, `creates_resource`) remains the
authorization level; `creates_resource` alone does not imply derivation lineage.

Revision selection follows the data actually consumed:

- `sourceRevision` identifies immutable managed source bytes. It does not change when an image
  edit recipe, video edit project or comment changes.
- A rendered-content revision identifies the canonical source bytes plus the normalized edit
  recipe used to render them. Image crop/flip/tone/eraser and video render projects change this
  revision. Comments, title, canvas position and selection do not.
- `knowledgeRevision` identifies semantic node content used for retrieval and grounding. It is
  distinct from rendered pixels and from the raw asset revision. A knowledge-producing edit must
  invalidate or recompute the knowledge derived from the affected content.
- Structure's `editRevision` fences layout and membership mutations; it is never substituted for a
  resource revision.

Derivation inputs declare which revision basis they consume (`asset`, `rendered` or
`knowledge`) and persist that basis, the consumed revision and its edit/project snapshot. Rust
validates the base revision when recording the input. The snapshot contains the edit/project
parameters required to identify a rendered input. Source-current status compares the declared
basis, never just the raw file revision. Source validation requires callers to name the consumed
basis instead of guessing between knowledge and asset revisions. Image operations also compare
their captured edit recipe before publishing. A newer source edit marks an older output as historical; it does not mutate or
silently regenerate that output. Preview and external export consume the current rendered content
and share its renderer.
For example, background removal and upscale consume an image's `rendered` revision; analysis of
untouched video frames consumes `asset`; subtitle burn-in consumes the video's asset
revision and the subtitle node's `knowledge` revision. Presentation and video generation consume
the document's `knowledge` revision and unedited image reference files by `asset` revision. A canvas
move consumes none of these.

Node content updates submit the complete expected persisted payload alongside the replacement.
Rust compares them and commits the edit in one transaction; a stale editor fails visibly instead
of overwriting a newer edit or comment. Structure retains its separate numeric `editRevision`.
Field-scoped node mutations remain a possible future refinement when a concrete operation needs
independent concurrent edits. Derivation requests carry source identity and base revision, while
Rust resolves and records the consumed revision and snapshot;
the WebView never becomes the authority for rendered bytes or provenance. Capability owners keep
their own implementation modules (`nodes/`, `structure/`, `derivation/`, `assets/`); presentation
only selects the correct intent and shows its availability.

The input revision contract applies to new records in the current pre-release SQLite baseline;
obsolete development databases and packages are rejected instead of being silently interpreted.
Image and video rendering also recheck the edit/project while work is running. Image visual-
knowledge jobs use the same rendered revision and Rust image renderer as image Derivations, so an
edit invalidates stale jobs and the analysis sees displayed pixels. Operation-specific writers that
bypass general node content updates still need an audit. Video generation completion reloads the current pending output in its write
transaction and verifies the output path, preserving node metadata edits made during generation.

## Validation boundary

Validate data that crosses a real trust or persistence boundary: user input, Store/SQLite data, filesystem/network data and Rust IPC payloads. Do not add runtime assertions for static registries and other invariants entirely owned by this repository when TypeScript/Rust structure already guarantees them.

## No compatibility fallback

Knovum targets Tauri desktop only. Required startup infrastructure fails explicitly. Expected user/platform outcomes such as cancelled dialogs, denied notifications, unavailable global shortcuts or unsupported taskbar decoration are handled as normal runtime outcomes, not hidden by a generic compatibility layer.

## Size/complexity gates

Oxlint fails at 180 non-blank/non-comment lines per TS/Vue file, complexity above 10 and other correctness rules. These are ceilings, not targets. Function length is reviewed through responsibility, dependency direction and complexity instead of a physical line-count limit. dependency-cruiser, Knip and jscpd catch architecture violations, dead code and substantial copy/paste duplication.

No source path alias is configured. Relative paths intentionally expose ownership and dependency direction.

## Persistence split

Preferences use a strict v1 Tauri Store envelope. Before first public release, only the current v1 schema is accepted; incompatible development data is reset explicitly instead of carrying compatibility readers or migration chains. The same rule applies to other internal Store/package/protocol envelopes. The first public release must explicitly replace this development policy with a production migration policy if persisted compatibility becomes a supported product contract. The non-sensitive last-workspace route is a single WebView `localStorage` string so optional session restore cannot make the Store file a bootstrap dependency. Feature structured data belongs to feature-owned SQLite schemas. Credentials belong only in the foundation-owned `secrets.db`.

## Runtime architecture

Source dependency direction is intentionally separate from process/runtime topology. The system-level view, hidden-main startup sequence, IPC/task flow, storage ownership and platform lifecycle are documented in `system-architecture.md`.

## Integration settings and Rust engines

Model connection settings shared by otherwise isolated features belong to `foundation/ai/`. Provider
connections own user-editable endpoints and model definitions; official OpenAI and
OpenAI-compatible endpoints are explicit protocol choices. External Agent discovery and sessions are
owned by the crate-level `external_agents/` boundary and are not `AiProvider` implementations or model
fallbacks. SpaceAction execution requirements resolve independently to a model, deterministic
processor or explicitly delegated external session. Rust catalogs project Provider, External Agent,
SpaceAction and Workflow definitions to the Renderer. Secret values are persisted only through
`foundation/secrets/` and its dedicated local credential database.

Image processing, media processing, AI execution and storage are Rust or foundation engines consumed by
Spaces capabilities. They are not independent Renderer products. Each engine exposes only the commands and
permissions required by its owning knowledge-space workflow. External paths enter through webview drop or OS
dialogs, receive session-scoped authorization, and are never covered by a broad product-level filesystem scope.
