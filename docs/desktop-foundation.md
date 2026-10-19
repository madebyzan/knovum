# Desktop foundation

The Rust host owns behavior that must remain correct while the WebView is hidden or unavailable. Vue
owns product presentation and invokes only typed, scoped desktop commands. Platform differences are
listed in [platform-matrix.md](platform-matrix.md); visual composition belongs to
[ui-system.md](ui-system.md), and release mechanics belong to [packaging.md](packaging.md).

## Lifecycle

- The main window starts hidden and appears only after bootstrap reaches a usable or recoverable-error
  state. There is no splash WebView. Rust starts the 60-second startup deadline before renderer
  initialization; entry/import/mount failures report to its native Retry/Quit error dialog. A retry
  reloads the existing main WebView and starts a new startup generation. Explicit data maintenance
  holds the deadline while native progress reports the operation. No timer forces an exit.
- A single-instance Rust lifecycle owns reopen, focus, close-to-background, explicit quit, window-state
  persistence, and rotating logs.
- macOS uses the native Application Menu and menu-bar status item. Windows keeps the native caption and
  uses the system Tray. Both dispatch the shared `AppCommand` identifiers.
- Close, Quit, and updater installation pass through the registered-save barrier. Leave decisions
  run in registration order. Explicit cancellation stops the transition; failed saves retain the
  editor and produce visible owner feedback. Admitted writes settle before transition completion.
  Settings and Space editor owners prepare their decisions before the enclosing leave barrier
  finalizes discarded drafts. A later cancellation keeps earlier unsaved drafts available.
- Rust creates numbered close/quit requests. The renderer acknowledges them and sends a heartbeat
  every two seconds while awaiting saves or user decisions. A response may complete only its own
  pending request. Fifteen seconds without a heartbeat produces a native error: Close retains the
  window; Quit offers an explicit warning about unsaved edits and a choice to keep open or quit.
  Rust hides the window only after an approved close. Renderer reload invalidates its ready state.
- Rust owns OS progress, notification delivery, and lifecycle events because these must continue when
  the WebView is hidden.

The Shell preserves native window controls. macOS may use the documented overlay titlebar with native
traffic lights; Windows retains its native caption. New custom caption controls require an architecture
decision. Only the `main` window is authorized. A new window requires an explicit use case, lifetime,
route/context transfer, and dedicated Tauri capability scope.

## Failure isolation

Operation failures remain with the initiating capability and its task or notification state. Vue's
application error handler logs uncaught UI errors without replacing the whole workbench. Settings and
the Space workspace have local rendering boundaries that may remount only their failed content.
Remounting does not retry business operations or restart Rust jobs. Required startup-service failures
use the bootstrap error state before the main window is revealed.

## Capability policy

The resolved display language updates native application menus, Tray actions, and Task Center
notifications together. Rust accepts only the current display-language identifiers; `system` must
be resolved before IPC. Interface language support is documented in `src/foundation/i18n/locales/README.md`.
Traditional Chinese is selected for `zh-Hant` or Taiwan, Hong Kong, and Macau system locales;
an explicit `zh-Hans` script selects Simplified Chinese. Other supported language families use
their registered display locale; Portuguese uses `pt-BR`. An unsupported OS language selects English.

Register only plugins used by current product flows. Shell, arbitrary process execution, WebSocket,
and broad system HTTP access remain absent until an owning feature has a concrete, scoped need.
Installing a plugin does not grant its commands to a WebView.

Base capabilities cover low-risk lifecycle primitives. Clipboard, filesystem, network, preview, and
custom commands are feature-scoped. Space databases remain behind Rust commands, and the WebView has no
SQL capability. Provider and model traffic stays behind validated Rust boundaries; credentials are
referenced by stable identifiers and never cross into the WebView.

External desktop Agent installation, updates, and sign-in remain user- or vendor-owned. Detection
returns a validated descriptor for a platform CLI, Node adapter, or standalone ACP server; it does not
infer login readiness or introduce a product-owned installer. Missing commands expose the official
vendor guidance. On macOS, discovery reads a non-interactive login-shell `PATH`, does not source
interactive shell configuration, and does not guess vendor installation paths.

## Files, clipboard, and dialogs

The application entry installs one document-level context-menu policy before mounting Vue. Development
keeps the WebView default menu. Production frontend builds cancel the default menu outside editable
controls, including overlays mounted outside the application root. Capture-phase cancellation does not
stop event propagation: capability-owned handlers still display their Tauri native menus. Inputs,
textareas, selects and rich-text editors retain their native editing menus. Inspector availability
follows Tauri's build contract: development/debug builds enable it; release builds do not enable the
`devtools` Cargo feature.

The application does not grant `$HOME/**`. Native dialogs and Tauri drops authorize paths for the
current session. A feature that needs durable ownership copies or streams the resource into its managed
AppData boundary. Persisted external scopes require an explicit product contract.

Rust validates file type, size, tree limits, symbolic links, special files, and destination before
copying. Directory support is feature-specific. Clipboard image reads are encoded and validated before
Rust writes them to managed storage. Operating-system file-list paste is not emulated with browser
objects or broad filesystem permission.

Imported and generated files are exposed to the WebView only through their feature's narrow asset
protocol scope. A popup menu owns explicit native item resources, including submenu children, so their
action channels survive construction. `Menu.popup()` resolves on display, before selection; the shared
owner retains one popup resource tree until the next popup replaces it. Replacement and failed display
release its resources; WebView destruction releases the final tree. Other OS resources are closed after
use. Editable WebView text keeps
native keyboard editing behavior rather than being intercepted by feature context menus. The macOS
Edit menu dispatches palette-hidden AppCommands without installing competing accelerators. For a
focused WebView input or rich-text editor, one narrow desktop command maps a strict Cut/Copy/Paste
enum to AppKit's fixed first-responder selectors on the main thread. AppKit therefore preserves its
native rich-text clipboard and undo behavior. With no editable target, the AppCommand dispatches the
existing `spaces.cut`, `spaces.copy` or `spaces.paste` canvas command. The WebView does not synthesize
clipboard payloads or replay editor transactions for application-menu editing.

System-thumbnail caches are keyed by managed-resource identity, source modification time, requested
dimensions, and generator version, then written atomically. Preview-provider availability never changes
semantic document extraction or knowledge publication.

## Dictation and native input

Cross-application dictation is one Rust-owned press/release session. It owns microphone capture,
temporary audio, cancellation, recognition submission, and guarded insertion while the WebView is
hidden. Cancel removes temporary media and prevents insertion.

The session captures the foreground target and validates the same foreground-window identity before insertion.
Permission denial, focus change, or OS integrity restrictions preserve recognized text for explicit
copying. Automatic insertion uses a private clipboard marker and restores the previous clipboard only
when that marker is still present, so user clipboard changes are not overwritten.

Microphone, Accessibility, global-shortcut, and foreground-insertion behavior is registered only for
qualified targets. Unsupported targets expose unavailability rather than a compatibility execution
path. ASR runtime and model behavior belongs to [local-models.md](local-models.md).

## Tasks, notifications, and progress

Rust aggregates committed Run state, enrichment state and resource preparation snapshots independently
of the renderer. Session-only tasks publish a bounded projection through a desktop command; Rust
producer identities override matching session entries. No queued/running work clears Dock/taskbar
progress; unknown progress is indeterminate and known progress is averaged. Paused/attention tasks do
not keep a running indicator. The WebView has no direct set-progress-bar permission.

The official Tauri notification plugin delivers background task completion/failure notifications after
at least ten seconds of work, only when the main window is hidden or unfocused. Historical terminal
snapshots and cancelled tasks do not notify. Notifications contain no prompts, output or producer
errors. The strict preference controls delivery; OS notification settings remain authoritative.
The main WebView grants only the notification plugin's initialization-time permission query
(`notification:allow-is-permission-granted`); Rust remains the notification delivery owner.
The official desktop plugin does not expose delivery acknowledgement or the real OS authorization
state, so enabling the preference does not claim that the OS delivered a notification. Feature engines retain cancellation and terminal authority.

## Native preview and document integration

System preview opens one authorized managed asset through the OS adapter. It does not grant a general
renderer path or filesystem capability. Unsupported preview providers report unavailability explicitly.

Background document rendering for analysis is separate from interactive system preview. It uses bounded
native resources and produces private analysis input; it does not expose a source-page rendering command
or custom document reader to the WebView. Original files open through their associated desktop
application.

## Local resources

Core compiled tools resolve from fixed checkout resources in development and application Resources after
installation. Python environments and models are prepared into verified application-owned storage when
their capability is first used. Project environment variables and `PATH` do not select substitute
resources. Exact runtime and model contracts are defined in [local-models.md](local-models.md), and
production layout is defined in [packaging.md](packaging.md).

## Updater and signing

The updater is registered only when validated release configuration is embedded. Rust owns checks,
selection, download, verification, and installation. Installation requires explicit user confirmation
and the registered-save barrier. Endpoint, signing, promotion, and artifact policy belong to
[release.md](release.md).

CI compilation on macOS and Windows proves source compatibility only. Lifecycle, menus, shortcuts,
dialogs, permissions, preview, native input, progress, installation, signing, and updating require the
real-machine acceptance defined in [platform-matrix.md](platform-matrix.md).

## Local backup and restore

Settings exposes explicit local backup and restore through native file dialogs. Both operations first
pass the registered save barrier and restart into exclusive maintenance before product storage opens.
Backup uses SQLx/SQLite snapshots and the existing ZIP library for the current database baselines,
strict settings and managed assets. Active or replayable Space work must settle before backup.
Before scheduling either restart, the Renderer checks Task Center work and Rust checks durable Space
work plus active package transfers, imports, canvas exchanges, model preparation and extension
publication owned by their engines. Spaces reconciles interrupted package-import namespaces before
admitting a backup, so uncommitted imported assets cannot enter it. Rust atomically reserves each
idle engine's admission before reconciliation or durable checks. These reservations reject new
jobs through pending-operation publication and native restart; any scheduling failure releases them.

Restore accepts the current format only at its original canonical application data location. Space
packages own transfer between installations. Hashes, bounded archive paths, database baselines and
integrity are validated before publication. A restore journal reconciles interrupted publication
before normal startup; an unresolved journal blocks storage initialization. One journal records the
prepared or committed publication phase. Committed restore data remains active if obsolete staging
cleanup fails; the retained journal retries cleanup and the receipt exposes the cleanup warning.
Other interrupted maintenance stages use canonical UUID-owned directories. Startup removes only
these real directories after journal reconciliation; cleanup failures remain visible and prevent
normal initialization until resolved. A successful backup reports any staging cleanup warning.
Credentials, downloaded models, runtimes, caches and logs are excluded. Restore removes existing credentials together with
publication, requiring explicit reauthorization. Maintenance results remain visible until dismissed.

The supported OS floor is owned by `docs/platform-matrix.md`. Rust inference requires Windows
10 2004/build 19041 or newer; a lower build fails visibly before loading the sealed runtime.
