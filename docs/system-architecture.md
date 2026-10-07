# System architecture

Knovum is one knowledge-space desktop application process with a Tauri/Rust host and Vue WebView UI. Spaces owns the primary product experience; Settings is an application-level preference destination.

## Runtime topology

```text
macOS / Windows
      |
      +-- window lifecycle / menu / tray / taskbar / dock
      +-- local credential database / notifications / dialogs / filesystem
      |
Tauri 2 Rust host
      |
      +-- official plugins and scoped custom commands
      +-- Rust dictation actor -> global shortcut / native microphone / guarded target insertion
      +-- long-running Rust work -> Tauri Channel
      +-- private, verified CPython inference worker -> JSON-RPC 2.0 over stdio
      +-- private, verified CPython Builtin Agent -> Deep Agents / LangGraph over bidirectional stdio
      +-- single-instance / window-state
      |
Tauri IPC boundary
      |
Vue 3 WebView
      |
      +-- app/         composition and app policy
      +-- shell/       desktop workbench
      +-- foundation/  commands, preferences, i18n, secrets, tasks
      +-- features/    Spaces product experience and application Settings
      +-- shared/      neutral contracts and durable UI composition
```

The source dependency graph is specified in `architecture.md` and enforced by dependency-cruiser.

Cross-application dictation remains in the Rust host when the WebView is hidden. One actor owns
the press/release session, temporary WAV lifetime and cancellation; final recognition enters the
same private Python speech capability used by media transcription. The captured native target is
revalidated before paste, and insertion failure is projected to the Vue result surface without
changing the recognized text.

## Startup sequence

```text
OS launches Knovum
      |
Tauri creates main (hidden)
      |
Vue mounts bootstrap/error shell in main
      |
load strict v1 preferences + apply embedded SQLx migrations
reconcile interrupted imports, derivations and Workflow runs
apply theme / type size / density / locale
restore disposable workspace session
install diagnostics / taskbar progress
      |
ready OR recoverable bootstrap error
      |
main reveals and focuses
```

There is no separate splash WebView. The Dock/taskbar owns launch feedback while the hidden main
window bootstraps, then the usable application or recoverable error state appears in one transition.
Rust starts recovery once during plugin setup. `plugin:spaces|await_space_runtime` awaits the same
initialization result; schedulers start only after recovery succeeds. Recovery failures reach the
recoverable bootstrap-error state, rather than allowing the application to report ready. This avoids both an unstyled white WebView flash
and a workspace that appears usable before its durable state is reconciled.

## Application composition

`app/router.ts` composes one Spaces application route and its Settings route. `features/spaces/workspace-definition.ts`
provides the Spaces-owned sidebar, navigator, toolbar, inspector and commands to the shell. There is no
product-module registry or application switcher. Rust image and AI capabilities remain internal
engines of the knowledge space rather than separate top-level products.

The Shell owns stable workbench zones:

```text
Unified Toolbar (Space tabs | Task Center | Inspector toggle)
optional Feature Sidebar | Workspace | optional contextual Inspector
```

The unified toolbar belongs to the knowledge space. Its feature-owned tab strip is a session view over open
Spaces, not a second persisted Space hierarchy: opening a Space from the sidebar adds or activates one tab,
closing a tab never archives its Space, and the final tab remains open so the workspace always has an explicit
active context. Back/forward traversal and recently closed tabs are session-only navigation state. The sidebar
and contextual inspector are Spaces-owned surfaces composed into stable shell zones. Settings temporarily
replaces the workspace content and does not participate in splitter state. One layout key persists the single
knowledge-space arrangement.

## Command flow

Document format parsing belongs to the Rust `documents/extraction` engine. Spaces publishes
its source passages through `assets/enrichment/content`; `spaces/previews` owns only canvas
visual thumbnails and their authorization/cleanup. PDF text uses `pdf-inspector 1.20.0`
with no optional rendering/OCR features. PDF page locators and revision-bound coverage are
retained. `documents/rendering` selects Core Graphics on macOS and Windows.Data.Pdf on
Windows at compile time, solely for background visual analysis.
PDF input distinguishes positioned image regions from explicitly classified recognition
pages. Normal text extraction does not schedule full-page AI. Both native adapters render
selected inputs directly, using the same Display-frame PDF-point coordinate contract.
Spaces owns the typed document resource/occurrence aggregate, persistent parse/visual jobs, atomic revision-guarded
publication and existing asset-reference lifecycle. Document parsing materializes Office embedded images and
PDF visual regions through the existing native renderer, so resource browsing never depends on AI configuration.
The AI capability owns optional visual interpretation, not document persistence or resource availability. No PDF
source-page render command is exposed to the WebView.

```text
Feature/App contributes AppCommand
        |
shared command registry
        |
        +-- Command Palette
        +-- window keyboard shortcut
        +-- macOS native Application Menu when applicable
```

There is no second menu action implementation. Application menu events emit the same command IDs used by the WebView registry.

Action availability is a neutral shared contract with `ready`, `checking`, `busy`, `blocked(reason)`, and `hidden` states. The same `AppCommand.availability` is read by command execution and the palette; a disabled visual state never replaces the execution guard. Feature modules combine local selection, source revision, edit session, and native capability results into that contract. Rust reports machine-readable capability states and reason codes for provider, credential, and database conditions; it never returns UI copy. Vue owns localization, help placement, and settings navigation. Image operation and layer capability checks refresh when their menus open, so returning from settings does not leave an old decision in view. Product-specific reason mapping remains in the owning feature rather than shared UI.

Document parsing owns structured inputs and logical locations; native PDF rendering only produces resources for requested pages or regions. Visual jobs own per-occurrence checkpoints, while Knowledge publication owns canonical revision and derived passages. Intermediate analysis does not rebuild Knowledge once per image: completed results are committed together on completion or failure. See [knowledge.md](knowledge.md) for checkpoint and resource lifetime details.

## Long-running task flow

```text
Feature Rust engine
      |
Tauri Channel<TaskUpdate>
      |
foundation task bridge
      |
Pinia Task Center
      +-- global toolbar Task Center
      +-- Dock/Taskbar progress
      +-- completion notification when appropriate
```

The bridge transports task presentation state. Cancellation and domain semantics stay with the feature engine. A rejected cancellation reports the rejection and leaves the producer responsible for settling its task. Resource preparation marks cancellation only after the native command accepts it. Spaces owns a single all-Space actionable enrichment query that joins persisted jobs with their live Space and node labels. The presentation excludes archived Spaces because those nodes cannot be opened; the scheduler still reads their jobs, and unarchiving restores the same job identity. Successful snapshots reconcile only the `spaces-enrichment` task producer, including removals; query errors retain the last successful projection. Enrichment events invalidate this query and supply progress to known running jobs. The same query feeds the canvas node indicator and the inspector, while Task Center adds global and current-Space views and navigation to the exact node. The Knowledge Health view has no separate enrichment queue.

Space asset publication and node persistence are separate steps. Database initialization must not sweep
unreferenced asset-directory entries: an asset can be written before its node is committed, including
clipboard images and staged imports. Asset deletion uses the explicit post-commit candidate path and
checks that durable node state no longer references it. A startup sweep cannot distinguish an orphan
from a valid in-flight publication and risks deleting the source before its node is saved.

Opening or refreshing a Space reads its nodes, scenes, relations, and structure snapshot in one SQLite
read transaction. The canvas receives the resulting structure, nodes, and relations as one scene input,
so a committed derivation or structure edit cannot be rendered with content from another revision.

Canvas node work state uses persisted node payload state for import, derivation and metadata; the Space-owned
actionable enrichment query for durable node jobs; and feature session state for an in-flight edit or media
analysis preview. Rust and the owning feature publish their own transitions; the canvas never infers job
completion from an animation or a timer. The canvas presents each node's relevant job state in its chrome,
while content acquisition state may appear inside unavailable content and edit apply state in the selected
image region. Task Center owns cancellation and detailed progress. No canvas-only status is persisted.
An asset revision marks published bytes; a pending derived node owns a distinct reserved output path and
does not copy the source revision or preview. The Rust derivation boundary rejects unpublished asset
outputs that already carry a revision or preview, both when work starts and before completion writes
back to the node. Operations with a primary asset source must use a different path from that source.
Video-edit outputs reserve a separate MP4 path while pending; the renderer writes its own managed
output file, and Rust checks the unpublished pending node before starting and publishing.
Canvas image decoding
has a separate local loading/error state and cannot make a pending output look complete.

Durable knowledge work uses the Space-owned Run scheduler rather than the presentation Task Center as
its state authority. A Run freezes its principal, grant, definition/binding identity, input snapshot
and budgets. StepInstances and Attempts carry resolved inputs, outputs, leases, checkpoints and
external reconciliation state. Workflow, Builtin Agent and externally initiated Actions all
reuse this authority. Token, elapsed-time, attempt, input-byte and output-byte budgets propagate through
parent/child Run trees; late workers and cancelled generations cannot publish.
Attempt completion, its output-byte budget and step/Run transitions commit in one write
transaction that checks the claimed epoch against the current Run. Workflow and Action
pause stops new step dispatch while allowing an already claimed step to finish. Remaining
work pauses; the last active step may settle the Run naturally when no work remains.
Ancestor pause and cancellation prevent new child dispatch. Parallelism counts running steps
across each ancestor's Run tree; an ineligible child cannot starve unrelated work. Budget
exhaustion revokes the unfinished tree, preserving completed results, and cancellation takes
precedence. Lease recovery invalidates and reconciles the whole expired execution epoch once,
including active siblings. Committed scheduling transitions notify observers; idle maintenance
is silent. The scheduler waits on committed Run changes or the next retry, lease or
budget deadline; idle work has no polling timer and takes no SQLite write lock.
Partial indexes cover unfinished controls, duration budgets, execution leases and
ready-step availability independently of completed Run history. Builtin Agent pause
uses its own durable checkpoint policy. Checkpoint retention shares the execution
deadline and cancellation signal: explicit deletions use SDK operations, while each
cleanup inspects at most 100 orphan thread heads and never decodes retained history.

Speech generation is a durable Action Run rather than an in-memory renderer job. A stable request ID
binds the accepted project revision. Segment generation and assembly have independent attempts, and
claimed work carries the Run execution epoch. Output reservation, segment completion and final
publication compare that epoch in their own write transactions. Recovery reconciles interrupted
attempts and can resume remaining durable work; cancelled or superseded attempts cannot publish.
Voice candidate synthesis is a dedicated durable Run. An external clone source is streamed into a
hash-bound managed snapshot before scheduling; the active Run owns that snapshot until a fenced
attempt publishes the normalized reference and audition assets. Candidate audition and
selected/rejected state survive restart, while an authorized selected candidate creates the
immutable voice revision used by later Runs.

Workflows are personal reusable chains captured from completed Action derivations, with explicit replaceable input slots and named outputs. Built-in production templates and their graph compiler are removed. Atomic generation, subtitle burn-in and document export use the same Action Run and publication facilities as workflow steps. The runtime accepts captured steps and durable Actions only; removed recipe graph kinds fail explicitly. Terminal Space packages preserve Run history for inspection without replay.
The built-in Agent exposes declared Space Actions and the user's selected MCP tools through
one typed catalog. The `extensions` owner maintains plugins, Skills, reusable external service
connector definitions, connection instances and connector accounts in `extensions.db`; credentials remain in `secrets.db`. Rust uses official rmcp for
inbound and outbound MCP. Python receives reviewed Skill bytes and authorized tool definitions,
with no upstream endpoint, process path or credentials. Each task freezes connector and connection revisions, schemas,
account bindings, authorization revisions and explicit tool/resource/prompt selection; each invocation rechecks revocation.
Plugin stdio declarations install without starting or discovering a process. Users select an
installed executable through the native dialog; the host resolves the declared parameters into
one configured endpoint. HTTP local-network access requires explicit user consent. Unconfigured
services remain visible and cannot be enabled for execution. Inspection and browsing sessions use
the same registry and account/plugin/connector revocation as task sessions.
Committed extension changes report post-commit cleanup failures separately. The UI refreshes
authoritative queries on both success and failure and shows pending cleanup without treating the
committed value as absent. Authorization validates submitted fields before disabling current
authority; changing an account binding disables the connection and clears its reviewed policies.
Reauthorization atomically changes the account revision and clears bound tool permissions for fresh
review. A frozen checkpoint cannot use replacement credentials, even when the account ID is unchanged.
Settings presents connectors as external services with capabilities, accounts and independent
connection instances. MCP transport configuration and public authorization requirements belong
to the service definition. Definitions are delivered by plugins or configured by the user; tools,
resources and prompts are discovered through rmcp. Definition updates revoke old task sessions;
endpoint or authorization changes disable every affected instance, remove account bindings and
clear tool review. Plugin component updates preserve definition identity and all instances.
Deleting a connection does not remove a reusable definition or its accounts. Removing a service
or uninstalling its plugin retires all instances and clears orphan credentials.
ACP sessions receive the existing private Knovum MCP endpoint, projecting the same selected
capabilities. Upstream tokens never enter ACP configuration or vendor global settings.
The external Agent engine reports session completion, including idle transport EOF,
without owning Spaces persistence. Spaces drains accepted updates and atomically settles
the conversation, active turns and Run. Late connection results cannot restore revoked
startup authority, and disconnect cannot overwrite cancellation or committed publication.
The persisted Run deadline also governs idle sessions. Extension leases are released
after runtime settlement; shutdown failure remains visible and retains resources until
the runtime actually stops.
The Builtin Agent conversation and message tables own the single user-visible history. A model task
or an ACP desktop Agent session is a Run bound to that history; Agent protocol identity, negotiated
capabilities and turn state remain execution records. Model and Agent turns share visible history,
while each Agent turn starts a fresh native session with explicit context. Historical grounded
answers and citation snapshots belong to the same conversation store. Obsolete prerelease
database baselines are rejected; no compatibility reader converts framework state. Space packages
export this history once and reject older package formats.
An Agent turn commits the user's message and its reply slot before the start command is accepted;
the renderer shows a temporary outbound message while the native session initializes, then replaces
it with the committed history. Conversation reads, context, branches and exports use insertion order
to resolve messages sharing a millisecond timestamp, so a reply cannot precede its question.
Conversation attachments have two explicit sources. Canvas references carry a node ID and stable
revision; Rust resolves them inside the active Space. Local files come from the native file dialog
and carry a scoped path plus content digest in the in-memory draft. Rust checks the dialog scope,
regular-file type, supported format, size and digest before each turn. It extracts bounded document
text or image content into the task context and records only file metadata in the Run manifest;
the file path is not persisted or converted into a canvas node. User messages retain only a
source-specific attachment summary for conversation history; canvas summaries can reveal their
node, while file summaries show the filename without reopening a private path. Desktop Agent
file inputs enter ACP as authorized canonical local resource links, without input staging or inline encoding.

## State and storage ownership

| Data                           | Owner/storage                                               |
| ------------------------------ | ----------------------------------------------------------- |
| local component UI state       | Vue component refs                                          |
| cross-view session state       | Pinia                                                       |
| remote/async cache             | TanStack Vue Query                                          |
| durable non-secret preferences | strict current v1 Tauri Store                               |
| last workspace route           | disposable localStorage value                               |
| structured product data        | feature-owned SQLite with embedded SQLx migrations          |
| credentials/tokens             | separate local `secrets.db`; user account and disk security |
| logs                           | Tauri log plugin with rotation                              |

Do not mirror one fact across several stores without an explicit synchronization design.

### Integration execution boundary

Agent descriptors include required connection help: native CLI / Node adapter /
standalone server, vendor documentation and upstream ACP setup/distribution URLs.
Connection type is derived from the Rust launch strategy rather than duplicated in UI
vendor checks. The strictly validated IPC descriptor is the sole help source; Settings
uses one Reka-based presentation for every Agent. Installation inspection confirms
entry presence only, not authentication or successful ACP negotiation. Help discloses
Node/npm/npx requirements and SDK-owned first-use adapter fetching where applicable;
Knovum does not own Agent installation, updates or credential provisioning.

Common desktop capabilities are not owned by UI components. Interactive image editing lives in the Vue Canvas preview and one strict `imageEdit` v1 draft contract. `src-tauri/src/images` replays that contract from an authorized managed source to produce the canonical PNG for download, local U²-Net background removal, configured-provider image operations and fal layer splitting. The WebView sends node identity and edit metadata, never full image buffers for processing. Spaces owns revision checks, pending nodes, derivation provenance, cancellation and output publication; Rust provider modules own networking, credentials and bounded result decoding. Layer splitting uses a separate fal credential from the credential database and publishes its transparent outputs atomically. LLM transport is owned by `src-tauri/src/ai` through the shared `genai` provider layer. The WebView sends only a stable connection/model or Agent reference; Rust loads the strict current settings, resolves the user-configured Base URL and model ID, derives the credential key, validates capability, reads the secret and owns provider networking. The credential database stores values separately from Spaces data and is read only by Rust through validated credential identifiers.

Layer preparation runs on the blocking executor and observes the same generation cancellation token
as provider execution. Publication admission and cancellation share one locked decision: accepted
cancellation prevents the database commit and releases prepared files; once publication is admitted,
cancellation is rejected explicitly while the atomic commit settles.

AI token accounting is part of that same Rust-owned provider boundary. Successful model turns persist only
provider-reported input, output, and total Token counts to the AI-owned `ai-usage.db` v1 database, together with
the model identity, capability purpose, event time, and the system-local activity date at occurrence. Text,
streaming, built-in Agent tool turns, translation, and visual analysis therefore share one accounting path. Missing
provider usage is left absent rather than estimated with a tokenizer. Settings reads bounded daily and
capability aggregates through a typed AI command; the WebView neither writes usage nor infers billing data. The
local Qwen embedding worker additionally reports the actual input token count from its model tokenizer.
Inputs beyond the fixed embedding token limit fail rather than being silently truncated.

Image edit state is a non-destructive node contract projected directly by the canvas content layer;
it never replaces the managed source asset. Video edit projects are feature-owned Space node data,
while the dedicated workbench owns only transient selection and playback state. Rust validates the
source identity and prepares a revision-keyed editing proxy in a narrowly scoped app cache; the
WebView reads video through the asset protocol and receives paths, never frames, over IPC. Final
video bytes cross a typed command into the Rust derivation boundary, which validates the source
revision, seeks past unused leading source media, runs one cancellable FFmpeg filter graph with
measured progress, persists a new managed asset and records `derives-from` provenance. The
renderer never receives an unrestricted shell or output path.
The video workbench keeps edit history and selection local to its mounted session. The WebView
plays the managed source or editing proxy through a media element and applies per-clip preview
volume through a Web Audio gain node; FFmpeg applies the same clip volume during final rendering.

## Product AI task execution

The user selects one default AI execution method, with optional feature overrides.
A current task's explicit target wins over its feature override, which wins over
`defaults.defaultTarget`. Enabling/detecting an Agent never implicitly selects it.
The Builtin Agent selector records an explicit choice as the global default; existing
feature overrides remain authoritative. Model connections are capability supplies,
not the admission gate for every product task.

`TaskTargetSelection` distinguishes a user model reference from an enabled desktop
Agent runtime identity. A model resolves to `ModelExecutionTarget`, declares native
capabilities and uses Rust-owned credentials/Provider transport. A desktop Agent
resolves without reading Knovum model credentials or requiring a model connection.
It uses its own authentication, tools and Skills. Its external account/fee source
is unknown to Knovum unless the runtime reports reliable evidence; a runtime name
is not proof of subscription coverage or zero external cost. There is no implicit
retry to another executor or billing account.

Transcript insights, image and document visual interpretation, writing, translation,
grounded answers and document creation use this task boundary. Image/video resource
creation and applicable image derivations also have an explicit Agent artifact
workflow, distinct from native model generation APIs. Models keep their capability
checks. Enabled Agents permit a controlled attempt, not a blanket support claim:
installation, executable startup, authentication, ACP input channels, native tools
and the current task's prepared inputs are separate facts. Resource links/local
paths support a tool route even without native ACP image blocks; they do not prove
that the chosen Agent can interpret or edit those files.

Bounded delegates reuse `external_agents/session`, authorized local attachments
and the existing artifact resolver. Each call has a unique transient workspace,
a 15-minute budget, cancellation acknowledgement and one named bounded result file.
Only a completed ACP turn with a valid regular result file is accepted. Chat output,
Markdown-wrapped JSON, missing fields and oversized/invalid results are not success.
The owning feature validates the domain result before its existing guarded publish.
There is no automatic format repair, executor switch or account fallback. Transient
workspaces are removed on completion/failure and at application startup.

Reasoning delegates receive no Knovum MCP inference Actions. Desktop conversation
MCP lists and admits only platform read/prepare/commit and deterministic Actions;
calling a routed inference Action fails before a child Run is created. The Agent
must reason with its own runtime and publish validated deliverables, avoiding both
Agent-to-Action-to-model hidden dependency and Agent-to-Action-to-Agent recursion.
The built-in runtime can explicitly invoke its own model-backed Actions through
its existing target snapshot and Run/grant/budget mechanisms.

Enrichment intents capture the visual execution selection in their node creation
transaction. Document parse jobs carry the policy to their subsequent visual job;
parsing itself is local. Selection is immutable across deduplication, retry and
interrupted-run recovery. Unselected intents await an explicit configured policy;
their first executable preparation records it atomically. Workflow and Action Run
inputs freeze explicit/inherited targets at creation. Active work does not reread
defaults to switch executors. Results record tagged `ExecutionSource`; Agent model
identity, tokens and costs remain absent when unknown.

AI routing preferences in the current v1 settings are optional: missing or null
`defaultTarget` means no global executor was selected, and missing or null function
overrides inherit the explicit global selection. Rust deserializes those fields as
`None`; the frontend schema decodes them to canonical `null` values. Neither path
chooses an account automatically or discards existing connections, enabled Agents
or overrides. Loading does not rewrite settings; the next user save writes the
canonical configuration. Invalid explicit targets, missing required settings data,
unknown fields and unsupported versions still fail validation. Absence and invalid
data are exercised independently in both Rust and frontend contract tests.
The single Spaces SQL baseline stores task choices and execution provenance;
obsolete schema/checksums fail visibly. No compatibility reader, automatic reset,
migration chain or dual write is introduced.

Settings mirrors that ownership through separate Providers & Models, Desktop Agents, Execution Routing, and Creation panes. A provider connection owns its credential reference and registered models. A desktop Agent owns only its runtime identity and enabled state; it never enters a provider form, implements `AiProvider`, or carries a fake endpoint, model, credential, or temperature. The shared Pinia integration stores are the live session source for AI settings and creation-library revisions, and their serialized writes prevent independently mounted panes from overwriting newer snapshots. Execution routing is a neutral application policy: API Models and Desktop Agents are peer target types, no type receives system priority, and the user selects a global execution default and may explicitly override it per product intent. A compatible Agent may satisfy visual understanding by reading the authorized local sources referenced by baseline ACP resource links, and may satisfy image, audio, video, or document creation through an Agent tool/MCP/Skill and the controlled workspace-artifact workflow. Every enabled Agent remains selectable; installation detection contributes only visible diagnostics and never revokes the user's choice. The static Agent directory and automatically loaded installation catalog are independent shared queries. Installation detection resolves configured execution entries in the system PATH and desktop application identities from native installation registration. macOS uses NSWorkspace Bundle ID lookup; Windows reads per-user and machine uninstall registration in both registry views. Desktop application identity and ACP launch identity are separate catalog fields: a GUI installation is never passed to the ACP launcher. Detection does not launch Agents or inspect authentication, and does not block route selection. Only the Desktop Agents settings pane enables automatic installation fetching; other execution-target consumers observe the shared cache without starting system discovery. Desktop Agent settings exposes a manual refresh beside its page title; its list and title action share the same query cache. Actual execution always performs protocol initialization and fails visibly if the chosen runtime cannot start. General artifact candidates are labeled as Agent-workflow rather than being presented as native model capabilities. No implicit fallback crosses Agent and API Model billing paths. The Rust IPC contract preserves the `Model` versus `Agent` distinction: model routes validate declared native model capabilities, while Agent routes use the common artifact workflow and retain the runtime's negotiated ACP capabilities as descriptive evidence. Connection and model drafts are validated before entering the strict `ai.models` contract, while reusable presets and workflows live in the independent `creation.library` contract and are created at their point of use. The installation IPC contract contains only Agent identity, installation presence and the separately resolved execution-entry path. Installation presence may be true while the execution-entry path is absent; this is not an ACP capability claim. ACP initialization and capability negotiation belong to actual execution, not settings inspection. macOS login-shell PATH discovery is asynchronous, bounded, and refreshed for each installation query rather than cached for the process lifetime; shell discovery failures are visible and never fall back to the GUI process PATH. Supported desktop Agents remain an explicit projection of the Agent Client Protocol Registry and use maintained native ACP modes or official ACP adapters bound to the user's installed host executable rather than Knovum-specific protocol forks.

Agent creation sessions run in a Rust-derived app-data workspace keyed by validated Space and session UUIDs. Built-in and desktop Agents execute autonomously under their authenticated task grants; SpaceActions do not create per-operation approval records. The built-in Agent exposes registered Actions bound to its explicit model account; conversation requests have no product mode. Desktop sessions expose platform Actions for local preparation, reading and knowledge publication; inference Actions are excluded. ACP permission requests are answered directly using a runtime-provided allow option, preferring allow-always; cancellation prevents further authorization. The adapter registry declares full-access session modes for Codex and Claude, which must be advertised and successfully selected before prompting. Other adapters use the same ACP automatic authorization policy without guessing vendor-specific mode names. Permission decision queues, commands, and UI cards are absent. Operating-system and Agent-runtime boundaries remain authoritative. Renderer attachments must already be inside the Tauri filesystem scope; Rust checks the supplied and canonical paths and rejects links and non-regular files. All local inputs use baseline ACP resource links plus explicit local source paths. Native desktop Agent tools own file and media inspection. Knovum does not copy input resources into the output workspace or encode local images/audio into ACP prompts. Input resources remain separate from output artifact collection. Rust attaches a private, authenticated, Space-scoped `knovum` MCP server to that ACP session, so the Agent uses the same knowledge queries and atomic `KnowledgeChangeSet` commit path as the desktop UI instead of accessing SQLite directly. The MCP endpoint and credential exist only for the session and are not a public WebView command or durable user setting. ACP image, audio, and embedded resource responses, together with files created by Agent tools, are materialized into the same workspace without product-defined size budgets. Direct media routes require exactly one expected output, validate detected MIME and verify the opened file identity. Each generic Agent artifact run is recorded in the Space creation-run tables; completed, unimported workspaces survive application restart and every recoverable session remains selectable, while interrupted runs become terminal failures and only their exact workspaces are cleaned. Agent creation still returns a path-free manifest for deliberate preview and import. For conversation turns, Rust publishes visible regular workspace artifacts and ACP output content through managed-asset acquisition. Direct message media has occurrence identity, so distinct blocks remain distinct even when their bytes match. Tool content is a complete resource projection keyed by tool and slot; replacement updates remove superseded slots. A strict v1 workspace manifest persists producer references: retired local file links are excluded from collection unless another tool or message still references them, and untracked workspace files remain eligible. Original linked files are retained until workspace disposal. Obsolete materialized tool files are reclaimed from the producer manifest only; filenames do not confer ownership, and other producer references prevent deletion. Workspace file links are validated without copying; credential-free HTTPS resource links are downloaded with public DNS addresses pinned independently for each redirect. Unsupported schemes and acquisition failures are visible. Before each asset, preview or extracted resource is written, an external-output staging manifest records its owning Run and every final and partial path. Native preview callbacks cannot write after generation closes. The existing preview/content import pipeline completes technical processing before node insertion; node insertion schedules durable semantic enrichment. Canvas nodes, conversation output references, edit history, staging release and Run completion commit in one Space database transaction. Attachments guide placement; explicit Actions own derivation and evidence relations. Both Agent execution paths collect created resources using the same principal/event and Action result contract. Knowledge Change creation includes structure/layout/history in its transaction. Session MCP Actions are children of the conversation Run, and finalization drains them and preserves child failure outcomes. Cancellation flags the Run tree and invalidates claimed execution epochs. Action output reservation, generation start and successful publication validate the claimed epoch in their own write transactions; registered executors are also cancelled cooperatively. Conversation finalization waits for durable descendant settlement. Publication, rollback and workspace cleanup share per-Run ownership, so cleanup cannot race a managed file writer. Publication failure stays owned by the external conversation; its publication state/error remains retryable at startup, and successful retry checks and settles the publication-failed Run. Startup removes manifested uncommitted files before retry. Package restoration restores committed canvas resources and historical conversations with publication already closed; packages do not carry live workspaces. The workspace is removed only after commit or explicit cancellation. A normal finalization failure rolls back the pending node and managed asset without leaving an undo entry, while a partial batch failure leaves only unfinished artifacts for retry. Explicit result discard and permanent Space deletion remove the corresponding workspace; startup reconciliation retries orphan cleanup for Space records that no longer exist. Space import and artifact consumption remain separate persistence boundaries and therefore do not claim process-crash-safe exactly-once semantics until the import contract owns a stable idempotency key. The ACP SDK owns subprocess stdio, process-group teardown, connection lifecycle and per-session update routing. Cancellation sends ACP `session/cancel`, continues consuming session updates while awaiting the terminal response, and only then permits SDK-owned transport teardown, using the same job identity exposed through Task Center.

Local resource preparation belongs to Rust. Exact catalog identities share one installation across
requests, while each request owns its cancellation demand. The last cancelled demand stops an
unfinished installation; observer loss does not. Execution holds verified model leases, so removal
cannot race inference. Resource snapshots include measured progress, real phases and bounded terminal
history. Settings and first-use capability consumers use the same foundation IPC and query.
Model groups and the optional media-import tool pair are Rust-owned preparation requests.
Active snapshots survive WebView reloads so Task Center reconstructs the same identity and
cancellation action. Settings persists an explicit international/mainland-China download profile;
a new installation captures that profile and never changes provider after a transport failure.
The Python runtime descriptor contains both regional archive URLs for the same immutable bytes.

`knovum_model_resources` invokes official Hugging Face/ModelScope SDKs in a bounded subprocess;
`knovum_inference` receives verified local paths and cannot download. Rust catalog snapshots group
all required model parts. Installations share one `(snapshot, region)` identity, use a snapshot
write lease, and retain SDK staging metadata on cancellation/failure for resume. Each file must
match the recorded size and SHA-256 before the completed directory is renamed into
`local-resources/models/<model>/<inventory-digest>/`. Inference holds one read lease per snapshot.
SDK state belongs to `local-resources/models/hub-cache/`, independently of personal Hub accounts.
Download-region preferences use the current v1 Tauri Store envelope. Reads check disk-load errors
explicitly, and reads/writes are serialized; only absent settings select the initial global profile.
No old-layout reader, automatic source switch or alternate model is installed. The current China
profile's U²-Net/Spleeter community endpoints remain a documented regional release gap; see
`oss-first.md` and `release.md` for required evidence. Kokoro's archive and gated diarization imports
retain their separate reviewed resource contracts.

Node writes derive enrichment intent inside the same Rust database transaction, so a WebView reload
cannot lose required work. Rust owns enrichment dispatch, per-node serialization, concurrency lanes,
execution targets, status transitions and retry validation. A missing configured target persists an
explicit configuration-wait state; invalid infrastructure stops visibly. Vue observes
`space-enrichment-update` and reads persisted actionable tasks. Media insight generation also runs
in Rust and rejects publication after source replacement or a concurrent manual transcript/insight
edit. Transcription uses the same persistent job for automatic and explicit user requests.
Tauri Channels carry progress inside Rust-owned executions; events project enrichment results to Vue.
Answer streams continue to use their owning conversation Channels.
Embedding/index storage, hybrid retrieval, and answer-with-sources transactions remain entirely Rust-owned;
the WebView never generates or transports embedding vectors or commits a partial grounded exchange.

The Rust local inference supervisor grants one heavy-model compute permit across capabilities,
including model loading. Interactive work takes priority while queued background work ages into
service. The application bundles its own Worker and an immutable platform download descriptor;
CPython and third-party packages are shared resources installed before models/inference. Rust passes
the application Worker root, dependency root and actual runtime manifest path explicitly.
It launches only a verified, managed, on-demand CPython interpreter with one selected
capability and model path per worker session. The worker retains a model briefly between requests
and is recycled on capability change, cancellation timeout, idle expiry or application exit.
`knovum_inference/runtime/` composes the inference process lifecycle and capability dispatch; `transport/` owns stdio
framing and validation, and `speech/` owns its engines and synthesis stream. Capability modules
do not depend on transport or worker state. Session preparation, including native imports and
thread configuration, runs off the
asyncio control loop. Only one initialization may be pending, and a preparation result cannot
activate a session after transport closure. Closing the transport stops admission, cancels current
computation and waits at most five seconds for pending initialization and request handlers before
terminating the process tree. Controls received before readiness return an explicit not-initialized
error. Protocol input is strictly UTF-8 JSON; non-finite numeric literals are rejected at parsing.
After spawning, Rust handles initialization cancellation/deadline inside the owned child lifecycle:
it terminates and waits for that child before returning and releasing memory or compute admission.
The Worker also owns bounded request sending: cancellation, an elapsed deadline or a failed write
terminates the child before the supervisor clears its slot. Successful sends retain one absolute
deadline through response reception. Python emits unexpected request failures through the generated
Rust-owned diagnostic contract; Rust accepts bounded structural evidence (error code, known method,
UUID request identity, exception type and at most eight basename/function/line frames). Exception
messages, source text, locals and full paths never enter that diagnostic stream.
Input and output wire validation use the same strict generated JSON schema semantics, including
JSON enum values; no permissive Python-object conversion or compatibility contract is used.
Current Python inference owns computation; Rust owns durable task state, revision checks, index
generations and result publication. Embedding uses the fixed Qwen snapshot through Sentence
Transformers with last-token pooling, 1,024-dimensional L2-normalized output and a strict 2,048
token input limit. Indexing logs report actual persisted counts and elapsed time; an incomplete
generation cannot become searchable.

Speech synthesis accepts WAV, MP3 and M4B publication. Segment audio is converted to integer sample
boundaries before chapter and assembly decisions. Mixing targets 24 kHz or 48 kHz according to the
selected output contract. Audiobook chapter PCM and loudness measurements have separate durable
checkpoints; final encoding applies the verified measurement, and publication retires those
checkpoints. Temporary normalization files remain attempt-owned. Video dubbing time-stretches
generated speech only within 0.9–1.1 of the target interval;
larger mismatches publish `needs_edit`. Overlapping dialogue occupies explicit lanes. The source voice
track is muted, the project uses a separately imported background track, or the user explicitly
accepts a locally generated Spleeter background candidate after audition. Spleeter's vocals versus
accompaniment split is never labeled as guaranteed dialogue removal; pending and rejected candidates
cannot enter the video mix. A video revision that selects a candidate freezes its separation ID,
model and source-video identity. Space packages carry the candidate row and both managed assets, so
restored revisions retain the same reviewed provenance.

Video-script translation uses the existing model settings and translation capability. Each request
selects one configured model, and the saved project revision freezes its provider, model, target
language and local-only policy alongside every translated segment's source identity, source revision
and original text. The translated text remains editable; synthesis caching still keys the resulting
segment text, so changing one translation invalidates only that segment and downstream assembly.
Local-only translation accepts only an Ollama connection on a loopback endpoint. If no such model is
configured, the editor requires a target-language script instead of calling a remote model.

Voice profiles are mutable library identities backed by immutable configuration revisions. Renaming
or archiving changes only the profile; language or supported speed changes create a new revision with
the engine, model, reference digest and authorization frozen from the previous revision. Existing
speech project revisions continue to point to their selected voice revision, including archived or
superseded revisions. Physical deletion requires an archived profile with no segment references.
Copying a voice to another Space creates a new profile and managed reference asset through the speech
asset publication journal; the whole Space package remains the complete voice export boundary.

Selected forced alignment is bounded to eight segments or 300 seconds and publishes stable units
against the transcript revision. Manual segment edits invalidate that segment's aligned units.
Optional Pyannote diarization is CPU-bound to two hours and speaker bounds 1–32. It uses four
user-authorized files from the pinned Community-1 revision plus a repository-owned pipeline config;
Rust never forwards a Hugging Face token or external YAML to Python.

## Security boundary

### Agent Space context

The conversation workspace is an output staging directory, never the Space's source of truth.
Every built-in or ACP Agent submission captures a fresh, bounded canvas inventory from the
Space database and persists it with its attachment manifest. The snapshot includes the Space ID,
title, total resource count, node identities, separate knowledge and source-file revisions, indexing state, and explicit pagination.
It includes unindexed resources and free canvas text; credential and portal nodes are excluded.
Metadata and resource contents are untrusted input, not instructions or evidence of visual contents.

`list-space-contents`, `search-knowledge`, and `read-node` are discovery/read Actions: they require
no preselected input resources. Built-in Actions and MCP share the same inventory, exact-node and
revision-aware keyword-search projections. Search chooses FTS or short-term substring matching
from the existing search plan, without retrying failed queries through another strategy. An empty
index match never establishes that a canvas is empty. Semantic search remains its explicit owning
retrieval capability; these Agent discovery tools do not implicitly download embedding models.

The session MCP server has the stable identifier `knovum`, declares tools and Space-scoped reading
instructions during initialization, and uses rmcp's HTTP protocol negotiation defaults. ACP supplies
its endpoint and session credential through `session/new.mcpServers`, not vendor configuration files.
`get_node_asset` authorizes the exact `assetRevision` returned by the node query and returns a managed source path; edited
images resolve to a rendered current view. The asset version reuses the derivation revision contract,
including the original bytes and image edits rather than the knowledge index version.
A changed revision or missing source fails explicitly.
Native Agent tools read that source and create a separate output; source files are not copied into
the artifact collector. The transport grants remain scoped to the authenticated Space.
Asset revision validation, managed-file authorization and rendering belong to `spaces/assets/node_asset`;
MCP, explicit attachments and built-in tool reads reuse this resolver.

Conversation has no Ask/Plan/Execute mode in its UI, request, database or Run policy.
The standard built-in graph supplies Todo and bounded read-only research without input composition
switches; the model decides when to use them. Todo activities remain an output projection.
Private RPC protocol 1, the v1 graph identity and Builtin Agent profile 1 reject earlier runtime
contracts; no fields are synthesized for old requests or checkpoints.

Builtin Agent model execution belongs to `knovum_agent_runtime`: Deep Agents 0.7.22,
LangGraph 1.2.13 and the async SQLite saver 3.1.1. The dedicated dependency lock and process
contain the official OpenAI, Anthropic, Google GenAI, Ollama, DeepSeek and Groq integrations.
OpenAI Chat/Responses is an explicit protocol choice; compatible endpoints do not trigger
protocol/provider failover. `genai` remains the transport for direct Rust AI capabilities.

Rust freezes the configured model profile, context/output limits, capability list and credential
reference per execution segment. Credentials are resolved into process memory over stdio only:
no graph state, checkpoint, process environment or log contains a model API key. Application
startup uses `truststore` with macOS Security/Windows CryptoAPI trust before constructing clients.
All model calls, including framework summaries and research, have host admission before HTTP.
SDK retries are disabled in the clients; a failed admitted call also blocks framework-level
retry through the Rust ledger. Input reservations use the SDK's token estimate plus the output
limit; actual provider usage is recorded even after cancellation or budget overrun. Unknown
usage/outcome remains explicit and retains its reservation. This does not promise a provider
will obey an exact token estimate or stop billing when its client disappears.

Each run supplies one stable input identity. The SDK checkpoint acknowledges it; replay checks
public graph state instead of appending the input again. Root public checkpoint history projects
complete main-agent messages by AIMessage ID into the single Rust history transaction. Internal
summary/research messages are excluded.
Projection selects the latest checkpoint identity per namespace from the SQLite checkpointer's
covering primary key, then uses the SDK to load those bodies and pending writes. Historic bodies
are not deserialized to rediscover namespaces; the main-message projection retains its cursor.
Rust settles accepted input identities and model receipts with set-based statements inside the
projection transaction. Receipt updates retain the execution-segment and graph-namespace fences;
already acknowledged inputs and committed receipts are not rewritten on projection replay.
Model-node output and its admission receipts are written together through the SDK's
`ExtendedModelResponse` state update, so an interrupted projection
acknowledgement can reconcile the result without another paid call. Summary receipts are captured
at the following before-model boundary. Public checkpoint/pending-write receipts reconcile the
paid-call ledger; no response replay cache or outbox is maintained. A missing model result enters
visible waiting, with separate unknown-outcome, known-response/uncommitted and unknown-usage
reasons. Another paid attempt requires the user's explicit command and current state version.

Every SDK tool is admitted against the run budget. Business tools use the versioned Action wire
name and a durable identity comprising execution revision, SDK tool namespace, AIMessage ID and
toolCall ID. Process identity/control epoch is a fence, not the operation's idempotency key.
Rust's existing Actions own argument conflicts, grants, atomic mutations and publication.
Generation Actions freeze their own selected business route rather than inheriting the reasoning
model. A long Action interrupts the graph with its existing child Run ID; Rust observes and
resumes that same run without model polling. Main-agent business writes cannot share a tool batch
with research delegation. Research is read-only, two instances maximum, depth one; general-purpose
and shell execution are disabled.

Pause stops new admissions immediately and is shown as paused only after a durable graph or
unaccepted-host-input boundary. Cancellation revokes Rust publication rights before notifying
Python; the process gets five seconds before whole-tree termination. Already received usage
transactions settle after pipe closure. Checkpoint versions/catalog changes retire an execution
segment visibly. A new branch, model/profile, catalog or unsuccessful prior run creates a
new segment seeded from product history. Same-binding successful follow-ups reuse the graph.
Superseded, closed segments are retained for 30 days, then deleted with the SDK public API only
after input/projection and model receipts have reconciled. Latest branch segments stay reusable.

Private drafts use StateBackend under `/workspace`; reviewed versioned Skills and the authorized
user-visible Space index snapshot use isolated read-only StoreBackends under `/skills` and
`/memory`. The snapshot survives context compression without copying an entire knowledge base.
Persistent notes remain normal inspectable/deletable Space nodes; updates use explicit Actions.
No user filesystem, Spaces database, secrets database or executable backend is mounted. Rust
resolves image attachments and tool-read images from authorized source revisions, freezes the
model's vision capability, and emits the official SDK multimodal message structure. Native
provider tool/image combinations and real endpoint behavior require qualification; mock endpoints
prove host/SDK contracts only. Received partial stream deltas are disposable UI state; only
checkpoint-proven complete answers enter durable history.

External desktop Agents retain ACP sessions and their own tools. Their bounded history prompt and
authorized local attachment paths do not use the built-in graph. Both execution kinds continue to
share product history, grants, resources and the Run/Task Center contracts.

The WebView is not automatically trusted with every registered Tauri plugin. Plugin registration and capability grants are separate.

High-impact capabilities such as Shell execution, arbitrary filesystem access, broad WebView network origins, and new window/Rust commands remain ungranted until a feature defines the exact scope. Updater check/install is intentionally exposed only through the application-owned Rust updater commands rather than granting the updater plugin directly to the WebView.

Custom secret and bootstrap-recovery commands use app-defined Tauri permissions and are scoped to the `main` window.

## Platform lifecycle

### macOS

- native Application Menu;
- Dock-first lifecycle;
- closing the main window keeps the application process alive;
- Dock reopen reveals/focuses main;
- menu-bar status item exposes application navigation and lifecycle actions.

### Windows

- no legacy File/Edit application menu bar;
- Taskbar + system Tray integration with application navigation and lifecycle actions;
- closing the main window hides it to the system Tray;
- explicit Quit from the Tray terminates the process.

Platform behavior is documented in `platform-matrix.md` and must not be unified merely for implementation convenience.

## Distribution boundary

The repository can build QA installers without organization credentials. Public production distribution is a separate boundary requiring real signing/notarization and updater configuration. See `packaging.md` and `release.md`.

First-use transcription authorizes a managed source and captures its content revision before model
preparation, then reauthorizes and verifies that revision before blocking media processing. Keyword
search is independent of semantic search. Explicit semantic use prepares the embedding model and
indexes current content before retrieval; saving notes does not implicitly download large weights.
Models live in Tauri's platform app-data directory, never the application bundle. The hidden startup
window does not wait for optional downloads. A process restart preserves verified installations, but
this implementation does not promise transfer resume or replay lost temporary user requests.

The Rust local inference admission policy grants one heavy-model compute permit across
transcription, dictation, embedding, image segmentation and synthesis. The permit stays with the
running Python worker until the calculation ends or its process tree exits. Feature owners keep
their own job leases and publication checks. Resource preparation completes before admission, so
network waits do not occupy a compute permit.

### Audio production contracts

The speech feature exposes a revision catalog (`profileId`, `revisionId`, revision number),
including historical and archived revisions. Editors offer active revisions and their exact
already-bound historical revisions; library history can audition each immutable revision.
Preset audition resolves the pinned Kokoro catalog directly without creating a throwaway profile.
The pinned v1_0 archive has 53 speakers (0–52). Its owned display catalog records native voice
language and gender; production language remains the supported Chinese/English profile.

Reference preparation authorizes the selected file, freezes a private snapshot, decodes the
chosen 5–30 second interval to the ASR/reference PCM16 profile and returns a review transcript.
A clip must be completely present in the source. Candidate creation verifies the reviewed
content digest, freezes the selected interval and confirmed text in its Run, and persists the
explicit uint32 synthesis seed. A Space can retain at most 16 unselected candidates for audition.
Saving, revising, copying, exporting and restoring a voice preserve its synthesis seed.

Synthesis chunks and retained masters are mono float32; finite values outside ±1 are retained.
The Rust preview channel sends a separately clamped PCM16 projection. Short and audiobook WAV
publication encodes the selected PCM16, PCM24 or float32 precision from float masters. Reference
and ASR audio continue to use their separate mono PCM16 input contract.

Audiobook imports use existing Office and PDF extraction for supported local documents,
MarkdownIt for TXT/Markdown paragraphs and inert DOM parsing of EPUB container/manifest/spine
markup read with the existing ZIP dependency. Image-only PDF pages reject script import rather
than producing an incomplete manuscript. Scripts obey the same current v1 paragraph/role bounds.
Paragraph and chapter pauses contribute actual samples and chapter boundaries, including
checkpoint/cache identities. Title, author and narrator are written as container metadata;
PNG/JPEG covers attach to MP3/M4B output. WAV rejects a configured cover. Cover content revisions
participate in authorization, asset retention, package token/digest validation and remapping.

Video projects bind voices per speaker and retain source-dialogue intervals independently from
translated text. Accepted separation can preserve stereo original sound outside those intervals,
with 10 ms transitions inside dialogue edges. Dialogue and background stems use the same render
and mixing code as final publication; stems remain float32. Separate original audio is an explicit
MP4 track choice. MP4 delivery encodes stereo AAC tracks. Speech that already fits its time
window bypasses tempo processing to preserve samples. Subtitle import/export
uses SRT/WebVTT timings. Multiple audio-track exports
select at most eight published revisions sharing an identical source video; their order determines
track order. Exports authorize native save-dialog destinations, validate immutable asset digests
before/after processing, support cancellation and use disposable, startup-reconciled workspaces.
Replacing a destination uses [Rust's native file rename](https://doc.rust-lang.org/std/fs/fn.rename.html)
after completing its sibling temporary file; an existing destination is not deleted first.

Signal inspection uses FFmpeg astats over 100 ms windows, bounded to two hours and 100 reported
findings. Silence, near-full-scale peaks, invalid samples and volume changes relative to median
active RMS provide seekable listening cues. It does not score naturalness, alter the master or
reject intentional pauses. Jobs remain feature-owned; no second inference queue or global audio
service was added. Changed pre-release configuration shapes are strict and fail visibly.

## Agent extension execution

Plugins follow Agent Plugins 1.0.0 with root `plugin.json`, optional `mcp.json`, and
`skills/<name>/SKILL.md`. Installation publishes immutable digest revisions before one database
transaction makes their components visible. Unsupported components are diagnosed individually.
Directory, ZIP, pinned HTTPS ZIP and HTTPS Git use the same bounded content boundary; Git reads
objects through gix without checkout, hooks, filters or credential helpers. Catalog sources are
explicit directory, HTTPS manifest or HTTPS Git sources; publisher IDs are not verified identities.
Personal Skills have their own immutable revisions. Plugin Skills are read-only and can be copied.
Task leases retain selected Skill and MCP plugin revisions; update and uninstall revoke new calls,
while leased bytes are reclaimed only after task release.

`spaces_conversation_tool_operations` is the common built-in/ACP tool ledger. Local Actions keep
real Action identity and existing idempotency. External MCP operations transition through
prepared, dispatching, completed, failed or outcome_unknown. A durable intent serializes concurrent
requests and only a recorded receipt is replayed. Restart, transport loss, oversized responses and
post-dispatch timeout never authorize another write. Unknown attempts remain immutable facts;
only an explicit user decision creates the next attempt. Run budgets, cancellation and current
account/connection policy fence new dispatch. Late receipts record facts without reviving a Run.

MCP catalogs include all bounded pages of tools, resources, URI templates and prompts. Resource
and prompt descriptors and resource contents retain the official SDK types in Rust. Generated
TypeScript views expose their UI fields; protocol metadata remains in the serialized SDK values.
Resource templates expand with RFC 6570 into explicitly selected concrete URIs, not wildcard authority.
Their parameters use a closed string/list/map/null contract at the input boundary.
Each connection has one explicit selection structure. Frozen resource and prompt bindings carry
their selected identities independently of the tool input Schema; the Schema validates model
arguments and never serves as the source of selection policy.
ACP lists those concrete resources under opaque proxy URIs. Sampling and arbitrary filesystem
Roots are not advertised. Progress and non-secret elicitation share the Run interaction/history
surface. Account authorization uses its separate OAuth/secret credential flow. Resource browsing
is an explicit user action; import rechecks the reviewed content digest and uses existing Space
asset processing, with immutable external provenance in the node payload.

Plugin source inspection, installation, update and catalog reads reuse the existing JobRegistry and Tauri Channels to report reading, validation and commit stages in Task Center. The host owns cancellation: it interrupts source reads and rejects commit admission after cancellation. Commit admission and cancellation share one lock; once admitted, the operation completes its existing atomic revision/database publication and exposes its actual result. Task Center disables cancellation in that stage and waits for host terminal acknowledgement.

The extension owner explicitly cancels pending source reads and OAuth flows at application exit, revokes all MCP sessions and terminates their owned stdio process trees, then clears OAuth client caches. Cleanup does not depend on dropping AppHandle cycles.
