# System architecture

Knovum is one local-first desktop application. A Tauri 2/Rust host owns the operating-system,
storage, security, and native-runtime boundaries; one Vue 3 WebView owns product presentation.
Spaces is the primary product experience and Settings is an application-level destination. Source
and dependency ownership is defined in [architecture.md](architecture.md); Space and node behavior
belongs to [spaces.md](spaces.md), and visual rules belong to [ui-system.md](ui-system.md).

Supported runtime identities are macOS arm64, macOS x64, and Windows x64. Target-specific behavior
and real-machine qualification are defined in [platform-matrix.md](platform-matrix.md). Runtime
production, bundling, signing, and release promotion belong to [packaging.md](packaging.md) and
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
      +-- verified native tools, private CPython workers and private document worker processes
      +-- private MCP/ACP integration boundary
      |
typed Tauri IPC and Channel events
      |
Vue 3 WebView
      +-- app/         composition and application policy
      +-- shell/       stable desktop workbench
      +-- foundation/  IPC, preferences, tasks, i18n and shared infrastructure
      +-- features/    product capabilities
      +-- shared/      neutral contracts and durable UI compositions
```

The WebView never receives unrestricted SQL, filesystem, process, shell, credential, or model
network access. Rust commands authorize and validate requests at the boundary, then return typed
data and machine-readable reason codes. Vue owns localized copy and presentation.

Native inference runs in verified, application-owned CPython environments over JSON-RPC on stdio.
The application selects and prepares the required environment and model when a capability is used;
users do not select internal runtimes. Native tools and workers are resolved from fixed development
or installed application locations, never from `PATH` or project environment overrides. Required
resources fail explicitly when unavailable or invalid.

## Startup and recovery

```text
OS launches Knovum
      |
create hidden main window
      |
mount bootstrap/error shell
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
ready.

Pre-release storage uses strict current baselines. Obsolete formats are rejected or explicitly reset;
the application does not silently reinterpret them through compatibility readers. Recovery may resume
only durable work whose identity, source revision, authority, resources, and execution epoch still
match.

## Commands and desktop events

Features and application policy contribute typed `AppCommand` values to one registry. The command
palette, WebView shortcuts, and native application menu dispatch the same command identifiers and
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
      |
Tauri Channel<TaskUpdate>
      |
foundation task bridge
      |
Task Center + Dock/taskbar + notification
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

Durable knowledge work, Actions, Workflows, speech production, and Agent turns use the Space-owned Run
scheduler. A Run freezes its principal, grant, definition or binding revision, input snapshot,
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

Document export uses the current application executable in one private worker mode before desktop initialization. It receives a bounded, versioned source/format frame and returns bounded bytes; it never receives paths, Run authority, or database access. The Run attempt token cancels the process tree, and the host waits for process exit before releasing scheduler capacity. Spaces retains source/epoch revalidation and publication. Spawn, protocol and render failures are explicit.

## Native resources and local inference

FFmpeg is a bundled, verified core media resource. Python environments, models, and optional media
tools are application-managed resources prepared on capability use. All resolve through signed or
hash-verified manifests in application-owned locations. Installed runtime leases prevent retirement
while a worker or Agent still uses a generation; explicit lease release makes retired generations
eligible for cleanup.

Idle-worker admission checks `runtime.health` for the current session, empty request slot and actual
resident bytes. Invalid or unavailable health fails visibly, and health never reads in-flight stdio.
Memory pressure or an incompatible interactive request reclaims an idle worker before admission.
The supervisor owns one-worker reservation, priority fairness and the 60-second idle TTL; an active
compute permit remains held until calculation settles or the owned process tree exits.

Resource preparation is a cancellable long-running task. A recipe and target identity determine the
publication and installation path. Preparation publishes atomically only after file manifests,
producer identity, executable policy, and required notices pass verification. Missing core resources,
unsupported targets, corrupt manifests, and failed verification are explicit errors; no alternate
backend, device, executable, or legacy resource is selected after execution starts.

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

Plugin installation validates metadata and code before publishing one active revision. Stdio
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
  owner; the WebView has no wildcard native HTTP authority.
- External callbacks, tool results, files, runtime manifests, and worker messages are untrusted inputs
  and must pass schema, identity, size, and authorization checks before publication.
- Commit admission and cancellation share an ordering boundary. Once a commit is admitted, it reaches
  its atomic result and reports that result; cancellation cannot leave an ambiguous half-commit.

Agent graph identity contains the exact locked DeepAgents and LangGraph versions. Contract generation synchronizes that identity with Python, while profile/lock/installed-SDK regression checks detect a stale source identity. Changed identities retire incompatible execution segments through the existing strict boundary; no compatibility reader is introduced.

## Distribution gate

The supported targets are shipped only when their runtime descriptors, native resources, application
binary, signing policy, and updater identity agree. A compile succeeds only as a compile gate; it does
not establish clean installation, GPU execution, OS integration, cancellation, recovery, media quality,
signing, or update acceptance. Each release candidate requires the real-machine evidence listed in
[platform-matrix.md](platform-matrix.md) and the artifact controls in
[release.md](release.md).
