# System architecture

Knovum is one local-first desktop application. A Tauri 2/Rust host owns the operating-system,
storage, security, and process-runtime boundaries; one Vue 3 WebView owns product presentation.
Spaces is the primary product experience and Settings is an application-level destination. Source
and dependency ownership is defined in [architecture.md](architecture.md); Space and node behavior
belongs to [spaces.md](spaces.md), and visual rules belong to [ui-system.md](ui-system.md).

Supported runtime identities are macOS arm64, macOS x64, and Windows x64. Target-specific behavior
and real-machine qualification are defined in [platform-matrix.md](platform-matrix.md). Runtime
production, bundling, signing, and release publication belong to [packaging.md](packaging.md) and
[release.md](release.md). Models, profiles, download sizes, and hardware admission belong to
[local-models.md](local-models.md).

## Runtime topology

```text
macOS / Windows
      |
Tauri 2 Rust host
      +-- window, menu, tray/status item, Dock/taskbar, dialogs and notifications
      +-- scoped filesystem, clipboard, secrets and SQLite ownership
      +-- durable feature engines and Run scheduler
      +-- verified compiled tools, Rust inference workers, bundled frozen Agent
      +-- private MCP/ACP integration boundary
      |
typed Tauri IPC and Channel events
      |
Vue 3 WebView
      +-- app/         composition and application policy
      +-- shell/       stable desktop workbench
      +-- foundation/  IPC, preferences, tasks, i18n and shared infrastructure
      +-- features/    product capabilities
      |     +-- Spaces image-preview module Worker (bitmap input/output; no IPC or storage authority)
      +-- shared/      neutral contracts and durable UI compositions
```

The WebView never receives unrestricted SQL, filesystem, process, shell, credential, or model
network access. Rust commands authorize and validate requests at the boundary, then return typed
data and machine-readable reason codes. Vue owns localized copy and presentation.

Spaces shares one image-preview Worker among mounted preview consumers. The browser owns Worker and
OffscreenCanvas execution; Spaces owns bounded resolution, active-image priority, stale-result disposal and
consumer lifecycle. This worker receives decoded bitmaps and validated image edit facts, never credentials,
filesystem paths or database authority. Failure stays visible and does not rerun tone processing on the main
thread. Vue's long-document statistics Worker remains separate.

Local inference runs in the private Rust `knovum-inference-worker` process over bounded JSON-RPC stdio.
The host owns downloads, model identity, resource verification, admission, cancellation and publication.
The Rust worker owns model execution only; it has no database or secret authority. Stable DTOs live in
`src-tauri/crates/inference-contracts` and generate TypeScript contracts. Deep Agents/LangGraph is the
only retained Python business runtime. Compiled libraries and the release worker are bundled, signed
and sealed in one manifest; model weights remain on demand. Fixed development and installed paths
replace executable discovery. Required resources fail visibly. Local IPC queries and mutations use TanStack
`networkMode: always`. Only Renderer-owned network requests use browser connectivity admission.
Mixed local/remote extension and MCP IPC always reaches Rust, whose transport reports actual network
failures without blocking local directory, ZIP or stdio operations.

Document conversion uses a bounded private worker for content delivery. Markdown and Note PDF
output uses Typst; Office original-layout PDF rendering is outside the product boundary. Knovum
neither installs an Office engine nor discovers an ambient converter. Users export original-layout
PDF in their external Office application and import the result.

## Startup and recovery

```text
OS launches Knovum
      |
create hidden main window; arm Rust startup deadline
      |
mount bootstrap/error shell
      |
finish explicit backup/restore maintenance before opening product stores
      |
load strict preferences and embedded database baselines
recover interrupted durable work and resource state
apply appearance and restore disposable workspace session
start schedulers and task projection after recovery succeeds
      |
reveal usable application or recoverable bootstrap error
```

There is no splash WebView. Dock or taskbar feedback covers startup until the main window can show a
usable or recoverable state. Rust starts Space recovery once; every consumer awaits that same result.
Schedulers cannot dispatch before recovery succeeds, and a recovery failure cannot be reported as
ready. Explicit backup/restore maintenance runs at the next process start before SQLite pools,
Tauri Store reads or interrupted-job recovery. Every storage consumer awaits one retained maintenance
result. Unreconciled restore publication blocks initialization; ordinary operation failure is reported
with the original data preserved. The native launch progress remains active during maintenance.

Extension account cleanup and External Agent task workspace cleanup start independently after the
retained backup-maintenance result. Space recovery awaits extension account readiness and then passes
its retained extension snapshot IDs for lease reconciliation before dispatch. Desktop task producer
observation is wired by application composition; the desktop hub only merges neutral task snapshots.

Pre-release storage uses strict current baselines. Obsolete formats are rejected or explicitly reset;
the application does not silently reinterpret them through compatibility readers. Recovery may resume
only durable work whose identity, source revision, authority, resources, and execution epoch still
match.

## Commands and desktop events

Features and application policy contribute typed `AppCommand` values to one registry. The command
palette, WebView shortcuts, and the native Application Menu dispatch the same command identifiers and
execution guards. Visual disabled state never replaces the guard.

Action availability is represented as `ready`, `checking`, `busy`, `blocked(reason)`, or `hidden`.
The owning feature combines its selection and revision state with Rust capability results. Menu and
capability checks refresh when opened so a settings or permission change is not hidden by stale state.

Desktop lifecycle and capability rules are defined in
[desktop-foundation.md](desktop-foundation.md). Cross-application dictation remains Rust-owned while
the WebView is hidden: one session owns capture, temporary media, cancellation, target validation,
and guarded insertion. Recognition uses the same private speech capability as media transcription.

## Long-running work

```text
Feature engine / Run scheduler
      +-- committed Rust task projections --> Dock/taskbar + native notifications
      |
Tauri Channel<TaskUpdate>
      |
foundation task bridge --> Task Center
      |
bounded session-only desktop projection
```

Task Center is a presentation projection, not the authority for work. The feature or Run engine owns
state, cancellation, retries, cleanup, and terminal settlement. Cancellation becomes visible only
after the producer accepts it; rejection leaves the producer responsible for settling the task.
Queries reconcile their own producer snapshot and retain the last successful projection on read
failure. Resource observation outages use non-terminal `attention`, retaining progress and cancellation; an observation retry reconnects without restarting the producer. The next authoritative snapshot restores or settles the task.

Accepted conversation tasks send one `Finished` Channel update when their producer exits. The
update carries the task outcome and an explicit error when settlement fails. `settlement_failed`
means that durable Run settlement could not be recorded or confirmed; it is a Channel outcome,
never a persisted Run state. The Renderer settles its submission from that update and displays its
error without issuing another database read to recover the same cause. A failed cleanup cannot
rewrite an already committed result.

Durable knowledge work, Actions, Workflows, speech production, and Agent turns use the Run
scheduler owned by the Spaces capability. `RunScope` is explicit: Space work carries its Space ID,
while global voice generation uses Application scope. Application grants require a local user and
empty Space/action permissions; they cannot execute Space routes. Children inherit their parent’s
exact scope, and Run retries preserve the original Space scope and authorized action set. A Run freezes its principal, grant,
definition or binding revision, input snapshot,
resource choice, and budgets. Step instances and attempts own resolved inputs, outputs, leases,
checkpoints, retry state, and external reconciliation state.

The scheduler enforces these invariants:

- token, elapsed-time, attempt, input-byte, and output-byte budgets propagate through parent and child
  Run trees;
- completion, output accounting, and step/Run transitions commit atomically against the current
  execution epoch;
- pause stops new dispatch but allows already claimed work to settle; cancellation prevents new child
  dispatch and takes precedence over budget exhaustion;
- expired leases invalidate and reconcile the affected execution epoch, so late or superseded work
  cannot publish;
- scheduling waits on committed changes and real retry, lease, or budget deadlines rather than an idle
  polling timer;
- generated identity, source freshness, human review, resource availability, and Run state remain
  separate facts.

Feature-specific work may split preparation, inference, assembly, and publication into independent
attempts. Each attempt must compare the frozen source revision and Run epoch before writing output.
Recovery can continue accepted remaining work, but cancelled, superseded, or stale attempts cannot
publish.

Every durable Action handler uses that Attempt's cancellation token. Explicit cancellation, budget
expiry and lease loss therefore interrupt the same execution, whose settlement precedes releasing
scheduler capacity.

## Publication and storage ownership

Feature structured data uses feature-owned SQLite databases. Credentials and tokens use the dedicated
secret boundary. Preferences use the strict Store envelope; disposable navigation state may use the
documented session store. No global database service or cross-feature table ownership is introduced.

Opening or refreshing a Space reads the structure and visible content needed for one scene in one
SQLite read transaction. A write that changes durable state and its publication markers commits as
one transaction. External file moves use a journal or reserved-output protocol so startup recovery can
distinguish incomplete publication from committed ownership.

An asset can exist before its node transaction commits. Startup must therefore not sweep unreferenced
asset paths. Deletion starts only from an explicit post-commit candidate and rechecks that durable
state no longer references the path. Derived media uses a new reserved path and cannot overwrite its
source. Published bytes receive a revision only when publication commits.

Package export freezes a consistent Space snapshot and preserves stable ownership, remapping, history,
and managed-asset references. Package restore validates before commit and cannot replay terminal Runs.
Space-package format belongs to [knowledge.md](knowledge.md); application distribution belongs to
[packaging.md](packaging.md).

Document serialization uses the current application executable in one private worker mode before desktop initialization. It receives a bounded, strict current v2 source/format frame and returns bounded bytes. Markdown and Note inputs carry authoring content, citations and authorized image snapshots; PDF inputs also carry revision-qualified literal recognition. File identities are Rust-authorized Space-owned absolute paths and SHA-256 revisions. The worker reads a bounded snapshot and verifies its digest before vendor parsing; it never receives Run authority, database access or Renderer-chosen filesystem authority. The export cancellation token cancels the process tree, and the host waits for process exit before releasing scheduler capacity. Spaces retains path authorization, file/source revalidation and destination authorization. Spawn, protocol and serialization failures are explicit. Supported formats and fidelity limits are owned by [resource-export.md](resource-export.md).

## Compiled resources and local inference

FFmpeg is a bundled, verified core media resource. It contains one statically linked executable plus notices and build evidence; supplier/download contracts belong to `packaging.md`. The frozen Agent and Rust inference libraries are immutable installer resources. Development uses
Cargo-owned native outputs and the Agent project’s editable uv environment. Models and optional
media tools are prepared on capability use. The application never assembles, installs, repairs or
retires a Python environment. Required resource inventories fail visibly when missing or altered.

Idle-worker admission checks `runtime.health` for the current session, empty request slot and actual
resident bytes. Invalid or unavailable health fails visibly, and health never reads in-flight stdio.
Memory pressure or an incompatible interactive request reclaims an idle worker before admission.
The supervisor owns one-worker reservation, priority fairness and the 60-second idle TTL; an active
compute permit remains held until calculation settles or the owned process tree exits.

Model/tool acquisition is a cancellable long-running task. Built-in Agent and native inference
resources are compiled application inputs: the host verifies the embedded trusted inventory,
executable policy and notices before starting their processes. Installed-resource validation does
not resolve dependencies, repair environments or choose a system interpreter.

Optional media import runs only through the verified application-managed `yt-dlp` and Deno absolute
paths. The host disables ambient configuration, plugins, remote components, self-update, playlists,
and cookies; places the validated URL after `--`; bounds output and duration; supports cancellation;
and publishes from isolated staging only after validation. It is a recognized-content import path,
not a generic webpage probe.

Model selection, GPU admission, CPU behavior, and model-specific capabilities are defined once in
[local-models.md](local-models.md). A durable request freezes the chosen model, runtime, and device
profile before worker startup.

## Integrations and Agent execution

The `extensions` owner stores plugins, Skills, external-service definitions, connections, accounts,
and reviewed authorization metadata. Secrets remain outside that database. A task freezes component,
connection, schema, account, authorization, and selected tool/resource/prompt revisions; every
invocation rechecks revocation.

Plugin installation validates metadata, package containment and declared components before publishing
one active revision. Stdio
declarations do not start or discover processes during installation. Users choose executables through
the native dialog, and local-network HTTP requires explicit consent. Updating an endpoint,
authorization requirement, or account binding disables affected execution until the user reviews the
new authority. Removing a service retires its connections and orphan credentials.

Builtin model Agents and external ACP Agents share the durable conversation store and Run authority.
The private Knovum MCP endpoint projects only the capabilities selected for that task; upstream tokens,
process paths, and unrestricted Space context never enter Agent configuration. Each turn starts with a
frozen context and deadline. Spaces owns persistence and atomically settles accepted updates; transport
EOF or disconnect cannot overwrite cancellation or committed publication.

Extension processes and resources are leased until their runtime settles and accepted descendant
Actions finish. Shutdown failure remains visible and retains those resources. Application exit cancels
pending reads and authorization flows, revokes sessions, terminates owned process trees, and clears
ephemeral credential caches.

The extension settings distinguish complete packages (Plugins), package-provided service integrations
(Apps), independent personal instructions (Skills), and raw protocol services (MCP).
Apps manage the declared service's identities and reviewed tool access; MCP owns custom endpoint
creation and transport configuration. `ConnectorDefinition` remains the shared internal service
contract. Apps require explicit service metadata in the package manifest; raw `mcp.json` declarations
remain MCP services, including package-owned ones. Inspection marks declared integrations as `app`
components so all settings views share the same distinction without another database or transport.
Packaged Skills are inspected within their owning plugin and have no independent enablement command.
Skill and packaged connection reads project their owning plugin's current enablement; personal Skills
and standalone MCP connections retain independent enablement. A plugin owns exactly one connection
binding per declared service; users configure its endpoint, account and tool authorization, but cannot
create, delete or toggle individual packaged connections. Copying a packaged Skill creates personal
ownership; plugin updates retain component identity, not child toggle intent.

Conversation selection carries `pluginIds`, personal `skillIds`, and standalone `connections`.
Rust rejects direct packaged-component selections and expands each selected plugin atomically to all
its Skills and declared service bindings. Unsupported or missing components, ambiguous bindings,
unavailable accounts, missing grants for required tools and changed schemas reject the whole package instead
of admitting a partial capability. Packaged MCP catalogs supply tools, listed resources and prompts
as a unit; only explicitly granted tools are exposed and each call retains its authorization policy.
Every declared Skill tool binding must resolve or the entire snapshot is rejected; bindings include
MCP tools, resource reads and prompt retrieval. Complete packages expose their catalog's resource
templates through the same confirmed resource-read tool. The frozen schema restricts template identity,
variable names and string/list/map/null values; `iri-string` performs RFC 6570 expansion after argument
validation and before dispatch. Personal MCP selection instead freezes user-supplied template values
as concrete URIs. Neither path permits arbitrary resource URIs. Snapshots retain package revisions as well as
component, account and catalog revisions, so disabling, updating or uninstalling a package revokes
its tasks even when it contains no Skills. Obsolete pre-release conversation selections/snapshots
without package identity are rejected by the strict contracts; there is no compatibility reader.

`create_mcp_server` validates a custom service and creates its definition and first disabled
connection in one SQLite transaction. Additional standalone connections reuse a configured service.
Protocol services are configured through MCP inspection; declared application accounts and access are reached through Apps or the owning plugin. Long
configuration forms live in the settings pane and participate in the existing settings leave
coordinator; they do not open nested modal dialogs.
Connection configuration saves name and tool permission drafts together against the admitted
revision. A concurrent revision change retains the draft and requires explicit discard and
reload; it never silently overwrites unsaved permissions.

The application ships an offline, optional plugin catalog using the first-class `bundled` source.
It offers knowledge-work, evidence-research, meeting-notes, document-delivery, study-companion,
github-work, notion-work and linear-work. Users review and install chosen packages, then enable,
disable or uninstall them as complete units. Catalog identities are stable and metadata comes from
the package manifest. External marketplace releases cannot impersonate bundled packages.
Already-installed bundled content updates through the normal inspected revision publication path,
retaining installation identity and enablement. Absent packages are never automatically reinstalled.
GitHub uses its official remote MCP endpoint with a user-provided bearer credential; Notion and Linear
use their official Streamable HTTP endpoints and SDK-managed OAuth discovery. No provider credentials
or fabricated accounts are shipped. Malformed or unsupported declared components reject installation rather than publishing a partial
package. Inspection exposes the precise diagnostics, including unsupported vendor extensions.

Each bundled Skill contains focused instructions, task-specific references and an adaptable template.
DeepAgents' existing SkillsMiddleware discovers metadata first, loads SKILL.md on selection, and reads
referenced resources only when the skill needs them. No custom skill loader or script executor is
added. These skills orchestrate existing Space Actions and connected catalogs; they grant no extra
authority.
These instructions distinguish business Action IDs from advertised tool names and respect the
executor's capabilities. External Agents generate new deliverables with their own runtime and
submit files through the existing task-output publication path; they do not call built-in inference
Actions or assume the built-in memory mount or research subagent exists in an external session.

Runtime skill files retain their `/skills/` paths and are uploaded through the SDK's
`CompositeBackend`, which routes them into the skills store. Skill discovery and `read_file`
use that same mounted backend; direct writes to the child store would bypass its path routing.

External Agents receive the same frozen package selection through the private Space MCP server,
supplied by the standard [ACP session MCP binding](https://agentclientprotocol.com/protocol/v1/session-setup).
`list_selected_skills` exposes selected Skill metadata, resource paths and admitted alias-to-tool bindings without loading file bodies;
`read_selected_skill` reads only exact frozen files. The desktop prompt and MCP instructions guide
progressive discovery and binding resolution. External services' tools, resources and prompts remain
behind Knovum authorization, revision checks and task receipts; provider credentials are not forwarded.
No global CLI Skill directories or provider plugin settings are modified. HTTP MCP is an optional ACP
capability: initialization must advertise it before session creation, otherwise execution fails visibly.
This integration supplies Skill workflows through MCP; it does not claim native plugin installation,
arbitrary Skill script execution or universal support across external Agent versions.

The portable package loader targets [Agent Plugins 1.0.0](https://agent-plugins.org/specification)
with locally pinned schemas, and Skills use the [Agent Skills format](https://agentskills.io/specification),
including Unicode lowercase names, directory identity and metadata character limits. Unknown vendor
metadata is diagnosed and ignored; it is not a required workflow component. Knovum's execution policy
admits a selected package's implemented Skills and declared service dependencies together, while
inspection can still report valid and invalid components separately. This does not claim support for
Codex's separate `.codex-plugin` package layout or another client's lifecycle hooks.

External MCP uses the [official Rust SDK](https://github.com/modelcontextprotocol/rust-sdk), pinned in
Cargo, for protocol versions, capabilities, STDIO and Streamable HTTP. Knovum adds authorization,
frozen schemas, deadline/cancellation handling, bounded results and receipt settlement. The live
catalog is frozen when a turn is prepared; settings readiness is not a promise that a provider remains
reachable or that a future schema is unchanged. Legacy HTTP+SSE, MCP Apps UI hosting and arbitrary Skill
script execution are not advertised. `allowed-tools` metadata cannot grant host permissions; executable
dependencies still need implemented capabilities and explicit authorization. Built-in Skills orchestrate
existing Space Actions rather than installing new engines. Local SDK/protocol integration checks do
not establish live-provider OAuth interoperability or the quality of a model's completed deliverable.

Service-editor drafts include structured public header/environment pairs, so incomplete names and
duplicate fields remain a protected draft until explicitly saved or discarded. Serialization rejects
duplicate names rather than overwriting them; HTTP header names are case-insensitive. STDIO arguments
are ordered literal strings, including empty arguments and spaces. User transport changes reset endpoint
configuration and authorization requirements together. Service names are required; descriptions are optional.
Account-name and secret-field
drafts also participate in the settings leave coordinator. Authorization in progress must complete
or be cancelled before the owning view leaves; secrets are cleared before IPC and on disposal.

## Captured external documents

Web capture decodes a recognized BOM or declared HTTP charset through `encoding_rs` without replacement characters. With neither declaration, bytes must be valid UTF-8. HTML metadata encoding sniffing is outside the supported capture contract; a legacy page without an HTTP charset fails explicitly rather than publishing damaged text. Unknown encodings and malformed bytes are errors.

## Security boundary

- Tauri capabilities grant only the commands and paths required by the owning feature.
- Dialog-selected and dropped paths are session-scoped unless a feature explicitly persists a managed
  copy or durable scope.
- Imported content is copied or streamed through Rust validation into application-owned storage before
  normal feature use.
- Credentials never enter Pinia persistence, local storage, feature data, logs, query caches, Python
  arguments, or Agent configuration. AI connection URLs must be HTTP(S), have a host and contain no userinfo; saving and execution both enforce this before secret resolution.
- Network destinations, redirects, response sizes, and local-network access are validated by the Rust
  owner; the WebView has no wildcard system HTTP authority.
- External callbacks, tool results, files, runtime manifests, and worker messages are untrusted inputs
  and must pass schema, identity, size, and authorization checks before publication.
- Commit admission and cancellation share an ordering boundary. Once a commit is admitted, it reaches
  its atomic result and reports that result; cancellation cannot leave an ambiguous half-commit.

Agent graph identity contains the exact locked DeepAgents and LangGraph versions. Contract generation synchronizes that identity with Python, while project/lock/installed-SDK regression checks detect a stale source identity. Changed identities retire incompatible execution segments through the existing strict boundary; no compatibility reader is introduced.

### Provider and Agent output network policy

`public_network` owns the shared public-resource transport policy for provider image/video output and ACP resource links: credential-free HTTPS, all resolved addresses public, DNS-pinned connections, no ambient proxy, bounded manually validated redirects, and credentials scoped to the initial origin. Callers own their content-size limits and cancellation. AI API requests retain configured endpoint authority and reject redirects to another origin; Gemini operation names are relative resource names and resumable upload URLs must retain the configured origin. Provider output is untrusted input and never grants authority to access loopback/private networks or send API credentials to a new origin. Images are limited to 64 MiB and videos to 512 MiB including decoded inline payloads; cancellation also interrupts video submission, polling and download I/O.

IPv6 resource addresses must belong to ordinary public unicast space (`2000::/3`), excluding the special protocol range `2001::/23`, documentation ranges `2001:db8::/32` and `3fff::/20`, and 6to4 `2002::/16`. Knovum does not make exceptions for special protocol assignments inside these ranges. IPv4-mapped addresses use the same IPv4 public-address policy. These exclusions follow the [IANA IPv6 Special-Purpose Address Registry](https://www.iana.org/assignments/iana-ipv6-special-registry).

## Distribution gate

The supported targets are shipped only when their sealed Agent/native inventories, compiled resources, application
binary, signing policy, and updater identity agree. A compile succeeds only as a compile gate; it does
not establish clean installation, GPU execution, OS integration, cancellation, recovery, media quality,
signing, or update acceptance. Each release candidate requires the real-machine evidence listed in
[platform-matrix.md](platform-matrix.md) and the artifact controls in
[release.md](release.md).

## Local image execution ownership

`local_resources` owns the image model catalog, immutable multi-file snapshots and preparation leases.
The image runtime owner shares the existing WorkerSlot, admission, deadline, cancellation and idle
reclamation with other inference capabilities. Operation-specific ONNX adapters live in
`inference-worker/src/images`; the host owns image decoding, rendered coordinates, alpha composition and PNG assets.
New image methods use typed bounded tensor references, independently of U²-Net's existing 320px contract.

Spaces registers image transformations as ordinary durable SpaceActions. Renderer controls never
reserve output assets or submit a parallel image executor. The existing Run owns preparation/execution;
publication requires its authority and freezes the input revision/edit and explicit model target.
Ephemeral smart-selection previews have a cancellable request owner and produce no history or assets.
Canvas controls reuse the existing operation gate, selection surface and bar density/placement policy.

Model weights load lazily on the first admitted computation, under that computation’s deadline.
Worker initialization validates the sealed runtime and capability identity within its startup budget;
it does not spend that budget parsing model graphs or allocating model weights. The worker retains
only its owning capability’s loaded engine until shutdown.
