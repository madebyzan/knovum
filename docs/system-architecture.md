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
      +-- long-running Rust work -> Tauri Channel
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
Recovery is awaited through `plugin:spaces|reconcile_workspace`; it is not a detached setup task and cannot
silently fail after the application reports ready. This avoids both an unstyled white WebView flash
and a workspace that appears usable before its durable state is reconciled.

## Application composition

`app/router.ts` composes one Spaces application route and its Settings route. `features/spaces/workspace-definition.ts`
provides the Spaces-owned sidebar, navigator, toolbar, inspector and commands to the shell. There is no
product-module registry or application switcher. Rust image and AI capabilities remain internal
engines of the knowledge space rather than separate top-level products.

The Shell owns stable workbench zones:

```text
Unified Toolbar
optional Feature Sidebar | Workspace | optional contextual Inspector
                                           |
                                   contextual Task Center
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
      +-- contextual toolbar Task Center
      +-- Dock/Taskbar progress
      +-- completion notification when appropriate
```

The bridge transports task presentation state. Cancellation and domain semantics stay with the feature engine.

Space asset publication and node persistence are separate steps. Database initialization must not sweep
unreferenced asset-directory entries: an asset can be written before its node is committed, including
clipboard images and staged imports. Asset deletion uses the explicit post-commit candidate path and
checks that durable node state no longer references it. A startup sweep cannot distinguish an orphan
from a valid in-flight publication and risks deleting the source before its node is saved.

Opening or refreshing a Space reads its nodes, scenes, relations, and structure snapshot in one SQLite
read transaction. The canvas receives the resulting structure, nodes, and relations as one scene input,
so a committed derivation or structure edit cannot be rendered with content from another revision.

Canvas node work state has two authorities: persisted node payload state for import, derivation and metadata,
and feature session state for an in-flight edit or media analysis. Rust and the owning feature publish their
own transitions; the canvas never infers job completion from an animation or a timer. The canvas presentation
projects these sources into one active/failed view, then places the same dot visual in unavailable content,
node chrome beside available content, or the selected image region during an edit apply. The Task Center
remains the place for cancellation and real task progress. No canvas-only status is persisted.
An asset revision marks published bytes; a pending derived node owns a distinct reserved output path and
does not copy the source revision or preview. The Rust derivation boundary rejects unpublished asset
outputs that already carry a revision or preview, both when work starts and before completion writes
back to the node. Operations with a primary asset source must use a different path from that source.
Canvas image decoding
has a separate local loading/error state and cannot make a pending output look complete.

Durable knowledge work uses the Space-owned Run scheduler rather than the presentation Task Center as
its state authority. A Run freezes its principal, grant, definition/binding identity, input snapshot
and budgets. StepInstances and Attempts carry resolved inputs, outputs, leases, checkpoints and
external reconciliation state. Workflow, Assistant and externally initiated Actions all
reuse this authority. Token, elapsed-time, attempt, input-byte and output-byte budgets propagate through
parent/child Run trees; late workers and cancelled generations cannot publish.

The built-in recipe graph accepts Input, Action, Review, Join and Output nodes. Personal
workflows instead freeze a verified derivation sequence with explicit input and output slots.
The runtime rejects removed generic graph node kinds; it does not reinterpret them as
personal workflows. Terminal Space packages preserve Run history for inspection without replay.
The built-in assistant exposes Knovum's declared Space Actions. Desktop Agents manage their own
external tools through their protocol runtime. The product does not maintain a second MCP
connection catalog for the built-in assistant.
The Assistant conversation and message tables own the single user-visible history. A model task
or an ACP desktop Agent session is a Run bound to that history; Agent protocol identity, negotiated
capabilities and turn state remain execution records. Model and Agent turns share visible history,
while each Agent turn starts a fresh native session with explicit context. Historical grounded
answers and citation snapshots are migrated once into the same conversation store. Space packages
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
file inputs enter the Agent protocol's existing scoped attachment staging path.

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

Common desktop capabilities are not owned by UI components. Interactive image editing lives in the Vue Canvas preview and one strict `imageEdit` v5 draft contract. `src-tauri/src/images` replays that contract from an authorized managed source to produce the canonical PNG for download, local U²-Net background removal, DashScope image operations and fal layer splitting. The WebView sends node identity and edit metadata, never full image buffers for processing. Spaces owns revision checks, pending nodes, derivation provenance, cancellation and output publication; Rust provider modules own networking, credentials and bounded result decoding. Layer splitting uses a separate fal credential from the credential database and publishes its transparent outputs atomically. LLM transport is owned by `src-tauri/src/ai` through the shared `genai` provider layer. The WebView sends only a stable connection/model or Agent reference; Rust loads the strict current settings, resolves the user-configured Base URL and model ID, derives the credential key, validates capability, reads the secret and owns provider networking. The credential database stores values separately from Spaces data and is read only by Rust through validated credential identifiers.

AI token accounting is part of that same Rust-owned provider boundary. Successful model turns persist only
provider-reported input, output, and total Token counts to the AI-owned `ai-usage.db` v1 database, together with
the model identity, capability purpose, event time, and the system-local activity date at occurrence. Text,
streaming, assistant tool turns, translation, and visual analysis therefore share one accounting path. Missing
provider usage is left absent rather than estimated with a tokenizer. Settings reads bounded daily and
capability aggregates through a typed AI command; the WebView neither writes usage nor infers billing data. The
local Qwen embedding engine additionally records the exact post-truncation input count from the same maintained
`tokenizers` encoding used for inference, without running a second estimator.

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

Settings mirrors that ownership through separate Providers & Models, Desktop Agents, Execution Routing, and Creation panes. A provider connection owns its credential reference and registered models. A desktop Agent owns only its runtime identity and enabled state; it never enters a provider form, implements `AiProvider`, or carries a fake endpoint, model, credential, or temperature. The shared Pinia integration stores are the live session source for AI settings and creation-library revisions, and their serialized writes prevent independently mounted panes from overwriting newer snapshots. Execution routing is a neutral application policy: API Models and Desktop Agents are peer target types, no type receives system priority, and the user selects the default target for each product intent. A compatible Agent may satisfy visual understanding through baseline ACP resource links or negotiated image input, and may satisfy image, audio, video, or document creation through an Agent tool/MCP/Skill and the controlled workspace-artifact workflow. Every enabled Agent remains selectable; installation detection contributes only visible diagnostics and never revokes the user's choice. The static Agent directory and automatically loaded installation catalog are independent shared queries. Installation detection resolves configured execution entries in the system PATH and desktop application identities from native installation registration. macOS uses NSWorkspace Bundle ID lookup; Windows reads per-user and machine uninstall registration in both registry views. Desktop application identity and ACP launch identity are separate catalog fields: a GUI installation is never passed to the ACP launcher. Detection does not launch Agents or inspect authentication, and does not block route selection. Only the Desktop Agents settings pane enables automatic installation fetching; other execution-target consumers observe the shared cache without starting system discovery. Desktop Agent settings exposes a manual refresh beside its page title; its list and title action share the same query cache. Actual execution always performs protocol initialization and fails visibly if the chosen runtime cannot start. General artifact candidates are labeled as Agent-workflow rather than being presented as native model capabilities. No implicit fallback crosses Agent and API Model billing paths. The Rust IPC contract preserves the `Model` versus `Agent` distinction: model routes validate declared native model capabilities, while Agent routes use the common artifact workflow and retain the runtime's negotiated ACP capabilities as descriptive evidence. Connection and model drafts are validated before entering the strict `ai.models` contract, while reusable presets and workflows live in the independent `creation.library` contract and are created at their point of use. The installation IPC contract contains only Agent identity, installation presence and the separately resolved execution-entry path. Installation presence may be true while the execution-entry path is absent; this is not an ACP capability claim. ACP initialization and capability negotiation belong to actual execution, not settings inspection. macOS login-shell PATH discovery is asynchronous, bounded, and refreshed for each installation query rather than cached for the process lifetime; shell discovery failures are visible and never fall back to the GUI process PATH. Supported desktop Agents remain an explicit projection of the Agent Client Protocol Registry and use maintained native ACP modes or official ACP adapters bound to the user's installed host executable rather than Knovum-specific protocol forks.

Agent creation sessions run in a Rust-derived app-data workspace keyed by validated Space and session UUIDs. ACP permission requests are persisted as Run events and remain pending until the user chooses one runtime-provided option in the conversation or cancels the request. Knovum never fabricates `allow_always`, treats an Agent request as human approval, or silently substitutes another permission mode. Operating-system and Agent-runtime boundaries remain authoritative. Renderer attachments must already be inside the Tauri filesystem scope; Rust checks the supplied and canonical paths, rejects links and non-regular files, and revalidates the opened file identity while staging without adding Knovum-specific size or count limits. Images and audio use native ACP content blocks when advertised by the initialized runtime, while all unsupported native media plus video, documents and other files use baseline ACP resource links. Successfully staged inputs are deleted immediately after the Agent run and are never retained only for artifact recovery. Rust attaches a private, authenticated, Space-scoped Knovum Knowledge MCP server to that ACP session, so the Agent uses the same knowledge queries and atomic `KnowledgeChangeSet` commit path as the desktop UI instead of accessing SQLite directly. The MCP endpoint and credential exist only for the session and are not a public WebView command or durable user setting. ACP image and audio responses and files created by Agent tools are materialized into the same workspace without product-defined size budgets. Direct media routes require exactly one expected output, validate detected MIME and verify the opened file identity. Each generic Agent artifact run is recorded in the Space creation-run tables; completed, unimported workspaces survive application restart and every recoverable session remains selectable, while interrupted runs become terminal failures and only their exact workspaces are cleaned. The Renderer receives typed progress and a path-free artifact manifest; preview, import and consumption submit only validated Space, session and relative artifact identities, and Rust requires that identity to remain a member of the current manifest. Rust resolves the app-owned source and enters the existing Space import preparation pipeline without granting WebView filesystem access. Space import creates a pending node against the captured Space ID; Agent artifact consumption is the source finalizer, and only successful finalization commits history, membership, reveal and knowledge indexing. A normal finalization failure rolls back the pending node and managed asset without leaving an undo entry, while a partial batch failure leaves only unfinished artifacts for retry. Explicit result discard and permanent Space deletion remove the corresponding workspace; startup reconciliation retries orphan cleanup for Space records that no longer exist. Space import and artifact consumption remain separate persistence boundaries and therefore do not claim process-crash-safe exactly-once semantics until the import contract owns a stable idempotency key. The ACP SDK owns subprocess stdio, process-group teardown, connection lifecycle and per-session update routing. Cancellation sends ACP `session/cancel`, continues consuming session updates while awaiting the terminal response, and only then permits SDK-owned transport teardown, using the same job identity exposed through Task Center.

Runtime model bundles are downloaded as one Rust-owned job with one cancellation token and aggregate progress. Active download snapshots survive WebView reloads, allowing Task Center to reconstruct the same task identity and cancellation action without restarting or duplicating the download. Model catalog entries describe immutable Hub artifacts (repository, full commit revision, filename, size and SHA-256), independently of transport endpoints. For these public, unauthenticated artifacts, the default order is Hugging Face then the third-party hf-mirror.com community mirror; connection/header failures advance to the next source with a bounded 15-second header deadline. Body transport failures restart from zero on the next source, truncating the temporary file rather than mixing bytes. Cancellation, progress delivery, disk and integrity failures stop the job. KNOVUM_MODEL_ENDPOINT takes precedence over HF_ENDPOINT; an explicit HTTPS endpoint is exclusive and can include a corporate proxy path prefix. Invalid endpoint settings fail visibly. No model download sends credentials. Each connection has a 10-second connect timeout and a 30-second read idle timeout, without a total time limit for large weights. Installation still requires the catalog SHA-256 before publishing the file. Community mirror availability is not guaranteed; users can explicitly choose a mirror, official Hub or corporate endpoint without changing artifact identity.

Node writes derive enrichment intent inside the same Rust database transaction, so a WebView reload
cannot lose required work. Rust owns enrichment status transitions and execution concurrency; Vue
reads the actionable queue, resolves the current user-selected execution target, and projects results
into the active canvas. Tauri Channels carry transcription and answer-stream progress back to Vue.
Embedding/index storage, hybrid retrieval, and answer-with-sources transactions remain entirely Rust-owned;
the WebView never generates or transports embedding vectors or commits a partial grounded exchange.

The local embedding engine owns a single inference permit, held by the blocking computation
even if its asynchronous caller is cancelled. Model loading also stays on that blocking thread.
Inputs are tokenized without padding, sorted by length, and evaluated in bounded microbatches
(8 rows, 2,048 padded tokens, and 2,048² attention cells); vectors retain input order.
Development builds optimize the host and numerical dependencies while retaining debug information.
Indexing logs report actual persisted counts and elapsed time; an incomplete write fails explicitly.

## Security boundary

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
