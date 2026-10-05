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
- `workflows/`: capture, versioned library and replay of verified derivation chains over SpaceActions;
- `assistant/`: Space-specific conversations, context, tasks, tool calls and checkpoints;
- `mcp/`: inbound Space protocol projection over Actions and Authorization, never direct mutation;
- `knowledge/`: semantic contracts, provenance, patches, events and knowledge governance;
- `knowledge/indexing/`: extraction of revisions, passages and knowledge chunks from Space nodes;
- `knowledge/node_contracts/document_coverage/`: validation of the persisted document-coverage payload; document enrichment writes this contract, while `knowledge/indexing/node/document.rs` alone projects its semantic revision and chunks;
- `knowledge/chunk_store/`: transactional persistence and revision rules for derived knowledge chunks;
- `knowledge/retrieval/`: Space-owned vector indexing, semantic search and hybrid ranking;
- `knowledge/grounding/`: answer-with-sources contracts, prompt construction, citations and reference cleanup;
- `derivation/`: application use cases that transform current Space resources into
  traceable derivative resources. Each capability owns its preparation, engine orchestration,
  completion and failure lifecycle; shared lifecycle policy stays at this boundary rather than in
  commands or engine adapters;
- `commands/`: Space-level state-changing application operations that do not have a more specific
  capability owner;
- `queries/`: read-only database projections and response mapping; `package_snapshot/` collects one transaction-consistent archive snapshot and `knowledge_projection/` owns knowledge-access protocol projections;
- `nodes/`: content mutation, lifecycle and node-owned derived-state updates; it does not own structure or layout;
- `structure/`: the authoritative Structure/Layout/Edit-State aggregate, validation, mutation planning, history, clipboard, deletion and cross-Space structural integrity; `commands.rs` owns mutations, `queries.rs` owns reads, and `deletion/` owns the deletion lifecycle;
- `canvas_view_state.rs`: disposable per-Space camera persistence; camera state is intentionally outside Structure/Layout history;
- `assets/`: Space asset authorization, import, enrichment, references and lifecycle;
- `assets/transcription/`: the Space-owned transcription use case, including asset authorization and knowledge persistence;
- `previews/`: Space preview-cache paths and mapping of neutral engine results into `PreparedAsset`;
- `package/`: Space package read, write, validation and transactional restore;
- `agent_artifacts/`: agent artifact workspace creation, preview, import, recovery and consumption;
- `recovery.rs`: ordered reconciliation of interrupted Space-owned workflows during bootstrap.

Content and platform engines are independent crate-level capabilities: `documents/` owns extraction
and generation (including the sole PPTX engine under `generation/pptx.rs`), `media/` owns acquisition and media transformation, and `images/` owns image
operations. `file_inspection/`, `web_capture/`, `system_preview/` and `local_resources/` remain
separate capabilities. `local_resources/models` owns downloaded model installation and integrity;
`local_resources/tools` owns fixed tool identity, verified app-data installation and bundled executable discovery;
`local_resources/preparation` coordinates shared model installation and execution leases. None depends on `spaces` or emits
Space domain types. A Space command authorizes an asset before calling `system_preview`; the preview
engine never becomes a general Renderer file capability.

`ai/local_runtime` owns the verified private CPython process, bounded JSON-RPC transport,
one heavy-compute permit and process cleanup. It keeps no Run or project state. Capability
modules authorize model paths and map typed requests; Python adapters invoke upstream model
libraries and return temporary results. Python cannot write Space data, publish assets or
choose a replacement model after failure. `spaces` owns durable Attempts, input revisions,
checkpoints and publication. The WebView never starts or addresses Python directly.

Inside `python/src/knovum_inference`, `runtime/` owns worker admission, session preparation,
request lifecycle and capability dispatch; `transport/` owns bounded stdio framing, JSON-RPC
and wire validation. `speech/` owns transcription, alignment, diarization and separation;
`speech/synthesis/` separates engine selection, Kokoro, VoxCPM2 and stream credits/publication.
The focused embedding and image segmentation implementations remain file owners.
Capability implementations depend on their own modules, generated contracts and neutral errors;
they do not import worker lifecycle or transport. Only runtime composition connects these owners.
Package `__init__.py` files do not re-export implementation symbols.

Repository tooling owns `tools/generate_python_contracts.py` and the native runtime publication
entries `tools/build_python_runtime.py` / `tools/verify_python_runtime_signatures.py`.
`tools/runtime_packaging/` owns controlled artifact handling, binary inspection, installed inventory
and native signing policy. Its tests live in `tools/runtime_packaging/__tests__/`.
Model qualification belongs to `python/tests/acceptance/`: strict corpus/capture-manifest/execution contracts,
hash-bound provenance and the four capability scorers. Pydantic owns structural validation,
NumPy owns vector arithmetic and percentiles, and JiWER owns transcription error counts.
Only corpus coverage, release identity, retrieval judgments, thresholds and blind-review rules remain
project logic. `pnpm inference:acceptance` evaluates retained native evidence; pytest exercises its
policies with clearly identified synthetic fixtures. Contract generation, publication and acceptance
dependencies have separate development groups and are excluded from the worker wheel/runtime.
Worker tests remain grouped by runtime, transport and speech ownership under `python/tests/`.

`file_inspection/classification` is the single authority for imported resource classification.
Inspection, Space import and system-thumbnail eligibility consume its typed result; they must not
maintain parallel extension lists or infer semantic categories from persisted node-kind strings.

`job_registry/mod.rs` is the crate-private lifecycle primitive for in-memory work. It owns atomic
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
  execution, dispatch, configured-target resolution, retry and system-authored node updates. Vue
  observes events and persisted tasks, and submits explicit transcription/retry/cancel intents;
- `assets/enrichment/media_knowledge/` owns transcript chunking, insight prompts, bounded model
  calls, chapter construction and publication against the captured media-knowledge input;
- Rust computes `knowledgeRevision` for user node writes and returns the committed payloads.
  Vue keeps draft validation and undo state, without implementing a second hash algorithm;
- `package/` owns archive validation, ID/reference remapping, managed assets, transactional restore
  and failed-import cleanup. Vue owns dialogs, progress and opening the imported Space;
- `canvas_exchange/` owns JSON Canvas file reading, import mapping, asset acquisition and atomic
  commit. Vue projects the current visual frames for export and presents committed imports;
- `knowledge/retrieval/` owns the fixed embedding spec identity, batching, LanceDB writes, semantic search
  and hybrid rank fusion. Its publication and invalidation outbox share the existing SQLite write
  transaction boundary; embedding computation stays outside that boundary. Vue submits query text
  or an indexing intent and never transports vectors;
- `assistant/task/` owns built-in conversation execution and streaming persistence, with read APIs in
  `assistant/queries/` and mutation APIs in `assistant/commands/`, while
  `knowledge/grounding/` owns its reusable semantic contracts, prompt assembly, citation rules and
  reference cleanup. Vue owns assistant presentation only.

Raw persistence transitions, embedding vectors and partial grounded-AI steps are not IPC contracts.

Canvas generation and document exports enter the typed `actions/canvas` acceptance boundary. Rust pins authorized source identities and execution choice, then creates an Action Run through the existing invocation contract. A Workflow replays a saved, verified derivation chain through those same Actions under a persisted grant and idempotency key. Interactive image tools, multi-output layer splitting, and video-editor rendering use one capability-owned Rust command each; they record Derivation provenance but are not replayable Workflow Actions. Built-in recipe definitions and their separate compiler are removed. Document assembly and publication belong to Rust, not a Renderer coordinator. Vue owns selection, input binding and observation; it never marks a step complete.

The knowledge embedding model, preprocessing and vector dimensions form one Spaces retrieval-index contract. They are intentionally not a user-switchable generation preference: changing any part of the embedding spec requires a new index generation and complete rebuild. `ai/capabilities/embedding` owns the fixed Qwen execution spec and validates Python output; Spaces alone decides which generation may be searched.

Expose one command per user/application intent so the WebView cannot assemble a second domain
implementation from low-level Rust primitives.

`queries/` never mutates durable business state or owns execution control. `derivation/queries.rs` reads readiness and metadata; `derivation/commands.rs` invokes generation and rendering. Search cancellation is a command over the registry owned by `knowledge/retrieval/search_control.rs`. `commands/` and `derivation/` may depend on
`knowledge/`, `knowledge/indexing/` and infrastructure adapters; semantic policy must not be duplicated in
a command, derivation capability or query adapter. `knowledge/` never depends on
`derivation/`: governance and provenance policy remain independent of the use cases that
produce derivative resources. It also does not depend on `assets/` or `queries/`; the Rust dependency
boundary is reviewed through Rust module ownership and code review. The removed source-token scanner
could not resolve relative imports or aliases and therefore did not enforce this dependency rule. Action query dispatch calls `queries/` directly.

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

Large bounded contexts such as Spaces split by business capability first. `model/` is reserved for
shared pure data models, invariants and projections within its owning capability; it is not a
container for all interfaces. `schema/` owns executable structural validation, and `contracts/`
owns explicit implementation boundaries or generated wire types. `spaces/model` and
`knowledge/model` have different business scopes; they do not duplicate one entity. Do not add
generic `types/` containers or singular `contract/` directories. This does not imply an independent
domain layer.
UI shape (`components`, `views`) and technical lifecycle (`state`, `runtime`) remain subordinate to
their business owner; they do not become peer product domains.

Within Spaces, workspace persistence belongs to `workspace/`, content persistence to `nodes/`,
Structure/Layout edit policy and IPC adapters to `structure/`, camera persistence to `canvas/persistence/`, relation
persistence to `knowledge/relations/`, scene persistence to `scenes/`, credential persistence to
`credentials/`, and search transport to `search/`.
`session/` may assemble their contracts but does not become their implementation owner.
`model/structure/schema/edit-snapshot.ts` owns the neutral validated edit snapshot. `structure/` owns edit commands and pure policies; its `space-edit-operations` and `structure-transfer-operations` are IPC adapters and cannot be imported by presentation. Scene validation belongs to `scenes/`; relation validation belongs to `model/relations/`; package formats compose those contracts. Node payload change shapes belong to `model/nodes`, not history.

Asset submission budgets live in `assets/concurrency-budget.ts`; media inference has its own `media/inference-budget.ts`. Their process-wide queue sizes and cancellation checks are unchanged. They do not depend on session composition or represent durable jobs.

Rust structure planning is grouped under `spaces/structure/planner/`; planner modules are private except the structure-scoped clone, deletion, bounds and spatial/tree-policy boundaries consumed by transactions and transfer. Images accepts caller-authorized paths and owns only rendering/resource limits. Spaces authorizes every managed image source before engine invocation; the images engine must not import Spaces.

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

`app/router.ts` owns the fixed Spaces and Settings routes. `features/spaces/workspace-definition.ts` exposes the single
composition contract for Spaces-owned shell surfaces and commands. Do not introduce a product-module
registry: Knovum is one knowledge-space application, not a container for peer applications.

The `AppCommand` contract is the single source of truth for command-palette entries and application-local shortcuts. Application menu items emit the same command IDs instead of owning a second action implementation.

## Direct vendor API rule

Use Vue, Reka, VueUse, TanStack and official Tauri APIs directly. A local abstraction is allowed only when it owns Knovum policy, validation, security, domain translation or real multi-API orchestration.

Use `cloneDeep` from the existing `es-toolkit` dependency for independent editable drafts and copies of repository-owned data, including Vue reactive data. Do not introduce clone wrappers or serialization-based clone substitutes. Boundary schemas still own input validation; cloning is not sanitization. Native `structuredClone` remains appropriate when platform structured-clone semantics are the requirement, including tests of transferable submission data.

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

| Kind                 | Edits the same identity                                                                        | Creates a derivative when requested                                                            |
| -------------------- | ---------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| Note, text, Markdown | Body, title and tags                                                                           | Summary, drafted document, visualization or other independently useful output                  |
| Image                | Crop, rotation, flip, tone and comments                                                        | Background removal, AI object removal, upscale, new angle, separated layers or generated image |
| Video                | Edit project, title, tags and annotations                                                      | Rendered video, video with burned-in subtitles or generated clip                               |
| Audio                | Title, tags and corrected node-owned transcript/analysis                                       | Separately published transcript or document resource                                           |
| Website              | URL, captured metadata and annotations of that saved resource                                  | Summary, document or visualization grounded in its captured revision                           |
| File, code           | Title, tags and other node metadata; an explicit source replacement changes its asset revision | Converted document, generated code or other new resource                                       |
| Knowledge object     | Typed properties, status, title and evidence relations                                         | A new knowledge resource only when the intent explicitly creates one                           |
| Credential           | Metadata and dedicated local credential-database reference                                     | No credential derivative; generated artifacts may consume an authorized credential             |
| Portal               | Target and presentation metadata                                                               | No content derivative; duplicate is a clone                                                    |

For images, the same-identity path writes only the source node's validated `imageEdit` recipe
(adjustments, crop, flip and rotation) and its undo history. Background removal, object removal,
layer splitting, multi-angle generation and upscaling never write that recipe or replace the
source asset. Each reserves a new output node, consumes the pinned rendered source, and records
Derivation provenance plus `derives-from` only after a successful publish. A brush selection for
object removal is a transient derivation input, not an image edit. Shared image dimensions are
read-only presentation data and do not determine which persistence path owns an operation.

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
  recipe used to render them. Image crop/flip/tone and video render projects change this
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
untouched video frames consumes `asset`; subtitle burn-in pins the video's file revision and its `knowledge` revision (including any
existing transcript). Subtitle cues are internal media data; the only published derivative is a video. Presentation and video generation consume
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

Optional preference additions are not incompatible data changes. Their owning schema must define
the meaning of absence and normalize it at decoding, identically in Rust and TypeScript where both
consume the setting. A missing optional preference is distinct from an invalid explicit value:
only absence receives the declared default; malformed values, unknown fields, missing required
records and unsupported envelope versions remain errors. Reading performs no write or reset.
Normal saves write the complete canonical value through the existing serialized Store mutation.
Default state derives from the same schema rather than a second list of optional field defaults.
Tests must preserve populated user configuration with optional preferences absent and reject
invalid explicit values. Adding a field or changing its meaning must include these contract tests;
incompatible required-field or semantic changes need an explicit version-policy decision.

## Runtime architecture

Source dependency direction is intentionally separate from process/runtime topology. The system-level view, hidden-main startup sequence, IPC/task flow, storage ownership and platform lifecycle are documented in `system-architecture.md`.

## Integration settings and Rust engines

Model connection settings shared by otherwise isolated features belong to `foundation/ai/`. Provider
connections own user-editable endpoints and model definitions; official OpenAI and
OpenAI-compatible endpoints are explicit protocol choices. External Agent discovery and sessions are
owned by the crate-level `external_agents/` boundary and are not `AiProvider` implementations or model
fallbacks. `ai/models/ModelExecutionTarget` and `ModelTargetSelection` describe only direct
model calls. `ai/tasks/TaskTargetSelection` and `TaskExecutionTarget` are product task boundaries:
they route to a direct model call or a desktop Agent without changing Provider transport contracts.
`ai/tasks/completion` is the bounded text/vision dispatch seam, not a second Agent framework or
scheduler. The built-in assistant keeps its existing Knovum-controlled orchestration runtime;
user model connections currently supply its inference. Future hosted supply may use that same
runtime, but no hosted service, subscription entitlement or billing implementation exists yet.
Deterministic document/media/image processing remains independent of AI task routing.
SpaceAction execution requirements resolve independently to these task executors or local processors. Rust catalogs project Provider, External Agent,
SpaceAction and Workflow definitions to the Renderer. Secret values are persisted only through
`foundation/secrets/` and its dedicated local credential database.

Image processing, media processing, AI execution and storage are Rust or foundation engines consumed by
Spaces capabilities. They are not independent Renderer products. Each engine exposes only the commands and
permissions required by its owning knowledge-space workflow. External paths enter through webview drop or OS
dialogs, receive session-scoped authorization, and are never covered by a broad product-level filesystem scope.

### Resource lifecycle composition

`app_resources` assembles the local-resources plugin and owns removal policy that takes an exclusive resource lease and unloads the AI embedding engine before deleting its model. `local_resources` owns path resolution, verification, download and filesystem operations; it does not import AI or Spaces. Source directories must not contain a file and a sibling directory with the same basename. Rust modules use conventional paths. A file-only module uses `<name>.rs`; a module with a `<name>/` subdirectory owns its entry at `<name>/mod.rs`, never a sibling `<name>.rs`. `tools/checks/source-layout.mjs` enforces the naming rule for `src/`, `src-tauri/src/` and `tools/`, and runs through both `pnpm check:web` and `pnpm check:rust`. Stable wire IDs are explicitly mapped to UI translation keys.

### IPC error boundary

`app_error::AppError` is the single product error contract: `{ code, message }`; `AppResult<T>` is its result alias. Application commands, AI execution, and classified domain failures share it. Codes are assigned explicitly at the source and use snake_case. Plain engine messages become `operation_failed`, preserving their message; neither conversion nor business policy extracts a code from message text. Native library errors and internal control enums remain local to their owning engine. Do not create parallel product error classes or convert a classified error back to a string before the receiving policy has acted on its code.

Renderer application calls use `foundation/ipc/invoke-command` and `foundation/errors/app-error`. A validated rejection envelope becomes a native `Error` with the envelope as `cause`; existing `Error` identity and invoke arguments are preserved. Business policy reads `decodeAppError(error)?.code`. Translation and cancellation do not need custom subclasses: local classified failures use `createAppError`. Native Tauri/plugin string rejections are normalized at this active vendor boundary; malformed product envelopes retain their original cause without being reinterpreted.

`foundation/errors/error-format` owns presentation: `getErrorMessage` supplies user-facing text; `formatErrorDiagnostic` supplies stack, machine code, causes and aggregate failures for logs. Callers do not stringify caught errors themselves. Native `AggregateError` is reserved for actual independent failures (for example an operation and its rollback), with the original operation as `cause`; it is not another product error hierarchy. Recovery must not silently replace the original failure. An operation has one notification owner; a handled failure may notify and return, while a propagated failure is reported by its caller. Committed results remain committed when subsequent renderer projection fails.

AI inference, enrichment and derivation preserve `AppError` through execution. Run retry decisions inspect codes; attempt failure/retry events store the complete envelope and attempt `error_code` stores only its machine code. Existing terminal knowledge/checkpoint display fields may retain message text; they are not classification inputs. MCP translates the product code into protocol error data without inventing another application error type. No string-prefix recovery, account fallback or obsolete envelope readers belong to this boundary.

Import `invokeCommand` under its actual name. This function owns error normalization only;
Tauri owns transport, serialization and Channels. It does not own UI notifications,
query caching, retries, cancellation or business recovery. The Spaces mutation receipt
protocol remains in `structure/confirmed-mutation`, where explicit command failures are
distinguished from an ambiguous transport outcome.

Vue components and presentation modules consume capability-owned operations or query
options, never the application command transport. For example, `video-editing/font-catalog`
owns the validated installed-font query and `video-editing/use-video-edit-proxy` owns
preparation, cancellation and stale-result suppression. Application composition may
orchestrate startup commands; foundation capabilities own their native commands.
Oxc restricts the native `invoke` import to the error boundary, and dependency-cruiser
enforces the presentation boundary. Official Tauri/plugin APIs and `Channel` remain
direct vendor imports wherever their owning capability needs them.

## IPC contract ownership

Rust owns serialized request/response/event types. ts-rs exports neutral types to `src/shared/contracts/generated`; Spaces-owned types are exported directly to `src/features/spaces/contracts/generated` through per-type ts-rs export paths; neutral runtime identity arrays and feature-owned limits come from the corresponding Rust definitions. Feature action/workflow JSON Schemas remain in Spaces. Foundation must not import feature implementations to obtain wire types. Runtime validation and UI projections keep their actual responsibilities rather than redefining wire DTOs. Speech callers import generated request and read-model declarations directly: `SpeechProjectOverview` describes the project catalog, `SpeechProjectSnapshot` contains a revision and its segments, and `VoiceProfileHistory` includes revision history. Inference wire outputs name their content (`SpeechTranscription`, `SpeechSynthesis`, `EmbeddingEncoding`); JSON-RPC retains its standard `result` envelope field.

Use generated request types to check IPC payloads and generated response/event types for
wire data. Video-editing requests, progress, fonts and completed derivation outputs follow
this contract. Validate consumed IPC values before entering the frontend model; model
schemas may impose stronger constraints than a serialized Rust DTO, such as supported
project versions, allowed rotations and valid node kinds. These refinements are model
policy, not a second wire protocol.

ts-rs generates DTOs, not command bindings. Neither the native `invoke<T>` nor
`invokeCommand<T>` statically pairs a command name with its arguments and result.
Capability functions expose concrete operation contracts to callers; do not add a
handwritten global command map or a custom signature parser to imitate generated
bindings. Adopting a full command-binding generator requires the normal dependency
decision and migration of the existing plugin, DTO and Channel contracts.

Space agent artifacts, action catalog and authorized asset preview belong to Spaces. The app router supplies the action catalog loader to Settings through explicit props, preserving feature isolation without a global registry.

### Runtime preparation boundaries

Spaces owns the intent to transcribe or use semantic search. It explicitly prepares resources before
processing, revalidates the asset after waiting, and owns result publication. AI/media engines execute
only on verified resources; the embedding capability receives a `PreparedModels` execution lease and
never initiates network installation. Model leases are acquired once per operation and passed down;
nesting fair read locks while a removal writer is waiting is forbidden.

Settings manages resources through `foundation/runtime`, using the same preparation IPC and query
as capability consumers. Rust owns installation facts and cancellation; Task Center is a projection.
Observer disconnection does not cancel preparation. Cancelling one request releases its own demand;
other requests for the exact artifact keep the shared installation alive. There is no generic plugin
platform, renderer downloader, compatibility command alias, PATH fallback or alternate model.

### Schema ownership and supporting modules

Rust DTOs generate the TypeScript IPC contracts and the Python runtime JSON schema through
`pnpm contracts:generate`. `tools/generate_python_contracts.py` then produces strict Pydantic
models under `python/src/knovum_inference/contracts/generated/` from that schema. The existing
[generator's dotted namespace support](https://datamodel-code-generator.koxudaxi.dev/module-exports/) separates control, audio and capability contracts; its explicit
ownership map changes Python file placement only, never canonical wire names or field definitions.
Consumers import defining modules directly. `contracts/wire_model.py` owns strict Pydantic policy.
CI regenerates both outputs and requires zero differences; generated contracts are not edited
independently. Every Rust schema definition must have exactly one module owner.

Independent Renderer runtime schemas use the owning capability or model’s `schema/<subject>.ts` layout, as specified in `naming-conventions.md`. Generated IPC contracts and database baselines retain their separate boundaries. In Rust, `spaces/knowledge/object_types/` owns custom Knowledge Type lifecycle and decoding; it is not a generic type container. Retrieval index schemas own Arrow structure, while `retrieval/validation.rs` owns query-vector input policy.

Canvas editor/surface interfaces belong to `canvas/contracts/`; pure preview policy belongs to `canvas/preview/`. Workspace persistence state belongs to `workspace/`. Knowledge object type editing belongs to `knowledge/object-types/`, not a generic TypeScript types folder.

## Source layout and size

`naming-conventions.md` owns sibling-name uniqueness and test-directory conventions.
`tools/checks/source-layout.mjs` verifies repository entry directories and all source roots,
and runs fixture-based policy regressions before inspecting the checkout.

Rust and Python source files are limited to 350 nonblank lines, including generated Python
contracts. Split by responsibility; keep bindings as reproducible modular generator output.
Frontend files retain the Oxc 180-line and complexity limits. Declarative schemas, static fixtures and documentation
are maintained according to their own contracts.
