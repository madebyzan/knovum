# Desktop foundation

The Rust host owns behavior that must remain correct while the WebView is hidden or unavailable. Vue
owns product presentation and invokes only typed, scoped desktop commands. Platform differences are
listed in [platform-matrix.md](platform-matrix.md); visual composition belongs to
[ui-system.md](ui-system.md), and release mechanics belong to [packaging.md](packaging.md).

## Lifecycle

- The main window starts hidden and appears only after bootstrap reaches a usable or recoverable-error
  state. There is no splash WebView.
- A single-instance Rust lifecycle owns reopen, focus, close-to-background, explicit quit, window-state
  persistence, and rotating logs.
- macOS uses the native Application Menu and menu-bar status item. Windows keeps the native caption and
  uses the system Tray. Both dispatch the shared `AppCommand` identifiers.
- Close, Quit, and updater installation pass through the registered-save barrier. Admitted preference
  and configuration writes must settle; a failure blocks the transition and is reported by its owner.
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

Register only plugins used by current product flows. Shell, arbitrary process execution, WebSocket,
and broad native HTTP access remain absent until an owning feature has a concrete, scoped need.
Installing a plugin does not grant its commands to a WebView.

Base capabilities cover low-risk lifecycle primitives. Clipboard, filesystem, network, preview, and
custom commands are feature-scoped. Space databases remain behind Rust commands, and the WebView has no
SQL capability. Provider and model traffic stays behind validated Rust boundaries; credentials are
referenced by stable identifiers and never cross into the WebView.

External desktop Agent installation, updates, and sign-in remain user- or vendor-owned. Detection
returns a validated descriptor for a native CLI, Node adapter, or standalone ACP server; it does not
infer login readiness or introduce a product-owned installer. Missing commands expose the official
vendor guidance. On macOS, discovery reads a non-interactive login-shell `PATH`, does not source
interactive shell configuration, and does not guess vendor installation paths.

## Files, clipboard, and dialogs

The application does not grant `$HOME/**`. Native dialogs and Tauri drops authorize paths for the
current session. A feature that needs durable ownership copies or streams the resource into its managed
AppData boundary. Persisted external scopes require an explicit product contract.

Rust validates file type, size, tree limits, symbolic links, special files, and destination before
copying. Directory support is feature-specific. Clipboard image reads are encoded and validated before
Rust writes them to managed storage. Operating-system file-list paste is not emulated with browser
objects or broad filesystem permission.

Imported and generated files are exposed to the WebView only through their feature's narrow asset
protocol scope. Popup menu and other native resources are closed after use. Editable WebView text keeps
native editing behavior rather than being intercepted by feature context menus.

Native thumbnail caches are keyed by managed-resource identity, source modification time, requested
dimensions, and generator version, then written atomically. Preview-provider availability never changes
semantic document extraction or knowledge publication.

## Dictation and native input

Cross-application dictation is one Rust-owned press/release session. It owns microphone capture,
temporary audio, cancellation, recognition submission, and guarded insertion while the WebView is
hidden. Cancel removes temporary media and prevents insertion.

The session captures the foreground target and validates the same native identity before insertion.
Permission denial, focus change, or OS integrity restrictions preserve recognized text for explicit
copying. Automatic insertion uses a private clipboard marker and restores the previous clipboard only
when that marker is still present, so user clipboard changes are not overwritten.

Microphone, Accessibility, global-shortcut, and foreground-insertion behavior is registered only for
qualified targets. Unsupported targets expose unavailability rather than a compatibility execution
path. ASR runtime and model behavior belongs to [local-models.md](local-models.md).

## Tasks, notifications, and progress

The shared task projection mirrors running work to Dock or taskbar progress. No running work clears the
indicator; unknown progress is indeterminate; determinate work is aggregated. The producer remains the
authority for cancellation and terminal state. Notifications require feature-owned permission and are
used only for meaningful transitions.

## Native preview and document integration

System preview opens one authorized managed asset through the OS adapter. It does not grant a general
renderer path or filesystem capability. Unsupported preview providers report unavailability explicitly.

Background document rendering for analysis is separate from interactive system preview. It uses bounded
native resources and produces private analysis input; it does not expose a source-page rendering command
or custom document reader to the WebView. Original files open through their associated desktop
application.

## Local resources

Core native tools resolve from fixed checkout resources in development and application Resources after
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
