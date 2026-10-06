# Desktop foundation

Desktop Agent installation, updates and sign-in remain user/vendor-owned. Each Agent
with a missing command exposes guidance through its status text; detected commands have
no help control. There is no separate row label or page-level help entry.
The Rust Agent directory owns required
command names, official guidance and upstream setup/distribution links. Connection type
is derived from the actual launch strategy: native CLI, Node adapter, or standalone ACP
server. Settings consumes the strictly validated descriptor instead of maintaining a
vendor-specific help table. Existing Reka popovers own help interaction and the opener
plugin opens official links in the system browser. Node-adapter help explicitly discloses
the SDK's npx dependency and possible first-use npm download. A detected GUI without a
resolved command is labeled explicitly. Detection does not inspect login or establish
ACP readiness; no product-owned Agent installer or update manager is introduced.

On macOS, command discovery obtains PATH from a non-interactive login shell. User-owned
PATH configuration belongs in `.zprofile`, not the interactive-only `.zshrc`. Discovery
does not source interactive shell configuration or guess vendor installation paths.

## Rust-owned lifecycle

Already implemented:

- hidden main window until Vue bootstrap reaches ready/recoverable-error state, with no intermediate splash WebView;
- single process with single-instance handling;
- macOS menu-bar status item and Windows system Tray with Show and Quit;
- desktop close semantics: macOS hides the main window while the app stays available from Dock/status item; Windows hides to the system Tray and explicit Quit terminates the process;
- macOS Dock reopen;
- window size/position/maximized/fullscreen persistence without persisting hidden visibility;
- macOS native Application Menu with Space tab commands, Settings and Command Palette; Windows deliberately has no classic menu bar;
- rotating persistent logs;
- official Tauri plugin registration.

These stay in Rust because they must work when the Vue WebView is hidden.

On macOS, Settings and Command Palette are inserted into the Knovum application submenu. Knovum replaces the generic File submenu with the current Space-tab lifecycle commands so `Command+T`, `Command+Shift+T` and `Command+W` create, restore and close workspace tabs instead of pretending the single main window is a browser window. These items follow the selected app locale and their enabled state follows the live tab session. Window/Help retain native platform behavior. The generic Edit submenu is removed: its predefined Cut/Copy/Paste items always register native `Command+X/C/V` accelerators, which consume canvas shortcuts before the WebView can dispatch them. The focused WebView editor or canvas owns editing chords through tinykeys on both platforms (`docs/commands.md`). Windows uses the same `AppCommand` identifiers and `Ctrl` shortcuts without a classic application menu bar.

## Workbench failure isolation

Operation failures belong to the initiating capability and its existing notification or task
state. An uncaught event, watcher or lifecycle-effect error is logged by Vue's application
error handler without unmounting the working interface. The application has no global
stack-trace page or runtime WebView-reload recovery control.

The settings content and Space work area each own a local rendering boundary. Only component
setup, render and asynchronous component-loading failures replace that area's content; the
surrounding workbench remains mounted. A user can explicitly reopen the failed content, and
settings navigation resets its boundary. Reopening remounts the affected content, including
its component-local state. The boundary adds no business-operation retries or Rust-job restarts;
initialization effects remain owned by the remounted capability.
Technical details belong to persistent logs, not the recovery presentation. Native process or
WebView crashes are outside this Vue boundary. Required startup-service failures continue to
use the separate bootstrap failure state before the main window is revealed.

## Shell/OS integration ready for features

Application composition registers appearance preferences, AI model settings and the independent creation library with the
existing window-close save barrier. Close, Quit and updater installation flush debounce-pending edits
and await admitted preference and AI configuration/credential mutations. All registered saves settle before the
transition proceeds; any failure prevents hiding, exit or installation and is reported by its owner.

The runtime registers only plugins used by current product flows: clipboard, dialogs, filesystem, opener, OS information, Store, SQLite, logging, single-instance handling, window-state persistence, notifications and updater. Shell, WebSocket and arbitrary process execution remain unregistered.

Notification permission is feature-owned by the main WebView and only important state transitions are surfaced. The updater is Rust-owned and receives its endpoint/public verification key at release build time.

The base capability grants only low-risk desktop primitives already owned by the framework plus notification access for the main window. Clipboard and filesystem writes remain feature-scoped. Model networking no longer grants a wildcard HTTP client to the WebView; those calls stay behind validated Rust commands.

## Cross-application dictation

Dictation is a Rust-owned desktop session. `CommandOrControl+Shift+Space` starts microphone
capture on press and submits the bounded WAV to the qualified Python ASR profile on release, so
capture and recognition continue while the main WebView is hidden. Cancel destroys the temporary
audio and prevents insertion. The recording is not a Space asset unless a separate explicit save
operation is added. The first qualified profiles are Apple Silicon macOS and x64 Windows; other
architectures do not register the shortcut and have no compatibility execution path.

The session captures the foreground application before recording and validates the same native
identity again before and immediately after staging the clipboard. macOS requires Accessibility
permission; Windows may reject `SendInput` across an integrity boundary. Focus changes and
permission denial preserve the recognized text in the Knovum result dialog. Automatic insertion
uses a private clipboard marker and restores the prior clipboard only when the marker remains
unchanged, preserving clipboard content copied by the user during recognition.

## Taskbar / Dock progress

The app task presentation store is mirrored to Tauri `Window.setProgressBar`: no running task clears progress, unknown-progress tasks are indeterminate, and determinate tasks produce aggregate progress. On Linux this OS decoration is only available on supported desktop environments; failure is logged and does not invalidate the actual job.

## Files and drag/drop

Do not grant `$HOME/**`. Runtime paths selected through the official Dialog or dropped through Tauri are session-scoped by Tauri. Register persisted-scope only for a feature that intentionally retains references to external paths across launches; the current Space copies imported resources into AppData and therefore does not persist source scopes. Use Tauri's window drag/drop API directly inside the owning feature when file/folder drop is part of that feature's UX. Folder ingestion is recursive only after Rust validation and rejects symbolic links, special files, oversized trees and oversized roots. Markdown image drop uses the same webview drag/drop path boundary: the whole active document is the target, dropped OS paths are session-authorized, Rust validates media type/size and copies the image into Space-owned AppData before the editor inserts it.

Clipboard image paste in Spaces uses the `tauri-plugin-clipboard` `read_image_binary` API for PNG-encoded reads, then a typed Rust command validates and writes the encoded PNG only into `$APPDATA/spaces/assets`. Space does not grant WebView mkdir/write/remove filesystem permissions; editor images and audio recordings use the same Rust-owned persistence boundary. Knovum-to-Knovum node copy/paste duplicates private resource ownership. Operating-system file-list paste is not emulated through browser `File` objects or broad filesystem scope; it requires a separately selected clipboard-file engine.

## Overlay titlebar ownership

macOS keeps native traffic lights and the Tauri overlay titlebar. The controls use Tauri's fixed `{ x: 16, y: 19 }` traffic-light position, optically aligned with the controls in the rendered 40px unified toolbar. The hidden-title window must not call `setTitle()` after creation because Tauri currently resets custom traffic-light positioning when the title changes. One unified toolbar spans the window. Its leading group owns shell navigation; the Space tab strip holds open tabs and the new-tab control directly following them, not document actions or task controls. Space Outline belongs to the sidebar, with Recently Deleted in its persistent footer. Space management uses native context menus on tabs, cards, Inbox and the current Space title. The Space view owns canvas creation tools and viewport controls. The shell owns a persistent Task Center control in the unified toolbar beside the right sidebar toggle. Its popover shows tasks across Spaces with an optional current-Space filter and links node-owned work back to its source node. Individual knowledge imports retain their canvas-local progress. The toolbar's leading controls start after the native-control exclusion zone. The sidebar splitter starts below the toolbar, so its resize hit target never crosses the traffic lights; the right workspace sidebar is a resizable work-area pane with a stable toolbar toggle. Windows keeps the native caption and does not inherit this traffic-light spacing.

## Multi-window

Only the `main` window is authorized today. A multi-window feature must define its window type, lifetime, route/context transfer and exact capability before a new label is added. Use official `WebviewWindow` APIs rather than a speculative window manager.

## Networking and processes

- AI provider traffic is owned by the Rust `ai` module through `genai`; the WebView passes a resolved execution target plus a stable credential identifier, never the secret value.
- Translation and editor AI reuse the shared model-provider runtime.
- WebSocket and Shell plugins are not registered until a concrete feature requires them.

## Updater and signing

The updater plugin is registered only when the build contains validated updater configuration. `pnpm package --release` embeds the permanent HTTPS manifest URL and verification key before plugin initialization, requires signing configuration and generates updater artifacts. QA builds remain disabled. Rust owns scheduled checks, the selected version, download/verification and installation; a shared Pinia snapshot projects that state into About and Task Center. Installation uses the existing registered-save barrier and explicit user confirmation. Signing, candidate promotion, native acceptance and GitHub/COS migration are owned by `release.md`.

## Cross-platform compile gate

`.github/workflows/platform-smoke.yml` runs the web gates and builds the desktop binary with `tauri build --no-bundle` on macOS and Windows runners for desktop-affecting pull requests. Linux remains the primary quality runner for full Rust/Web gates. This is a compile gate, not a substitute for manual OS behavior QA.

## Feature-owned desktop access

Spaces (owned internally by the `spaces` feature) is the first feature to activate feature-scoped desktop primitives. Its capability grants only typed Space actions: opening private imported resources/default applications, clipboard image reads, private asset mkdir/write/remove inside `$APPDATA/spaces/assets`, and narrow custom commands for persistence/import/refresh/duplication/export plus Saved Website preview capture and local media transcription. Space SQLite reads and writes stay behind Rust commands; the WebView has no SQL capability. Imported files and cached preview media are exposed to the WebView only from `$APPDATA/spaces/assets/**/*` and `$APPDATA/spaces/previews/**/*` through the asset protocol. `system_preview` exclusively owns document thumbnail rendering through Quick Look or Windows Shell. `pdf-inspector` owns PDF page-level text extraction without a native PDF runtime; `office_oxide` owns Office text/Markdown extraction, and FFmpeg owns video posters. The evidence inspector opens the original document through its associated desktop application rather than rendering PDF pages itself. Imported HTML is copied into Knovum-owned AppData and previewed in a sandboxed asset-protocol frame. Directory resource imports are rejected; archive files are treated as opaque binary file assets without extraction. Remote websites are never framed. Saved Website network access stays behind the custom Rust validation boundary rather than granting arbitrary dynamic HTTP origins to the WebView. Spaces also uses Tauri popup menus for right-click actions and grants only menu construction/popup plus resource-close capability; menu resources are closed after every popup. WebView-owned editable text is excluded from interception so OS text-editing semantics remain intact.

### Local media analysis tools

Spaces authorizes media assets and FFmpeg decodes each source revision to a 16 kHz mono PCM cache under `$APPCACHE/spaces/media-analysis`; startup clears stale cache entries. The Python inference worker owns ASR execution under the shared Rust compute admission. Rust verifies the audio reference and returned transcript before publishing media knowledge. Stage events use a Tauri `Channel` for Task Center and Dock/Taskbar progress. ASR model files are prepared on demand from pinned per-platform snapshots.

FFmpeg discovery belongs to `src-tauri/src/local_resources/tools/mod.rs`; managed Python
publication, installation and verification belong to `local_resources/runtimes/python`. Release/runtime configuration is explicit:

During development, `scripts/desktop/tauri.mjs` binds FFmpeg to the provisioned host directory.
Python development requires an explicit manifest override or a configured trusted publication.
Packaged FFmpeg resolves from Resources; Python resolves from its immutable app-data installation.

- `KNOVUM_FFMPEG_PATH`, or bundled `bundled-runtime/<platform>/ffmpeg[.exe]`;
- `KNOVUM_PYTHON_INFERENCE_MANIFEST_PATH` and `KNOVUM_PYTHON_AGENT_MANIFEST_PATH` are explicit development/CI overrides for their verified runtime profiles. Release builds use the trusted descriptor and verified app-data runtime.
- `KNOVUM_FASTER_WHISPER_*_PATH` and `KNOVUM_MLX_WHISPER_*_PATH` are development/CI overrides for individually verified model files. Release weights live in application data, outside the application bundle.

The desktop release targets are `macos-arm64` and `windows-x64`. Missing bundled core tools are configuration errors; Python components and models are prepared on first capability use. Packaging and runtime manifests reject other architectures rather than presenting partial native inference support.

The `integrations` capability grants clipboard access shared across features. Filesystem access remains limited to Tauri session-scoped dropped/Dialog-selected paths and user-selected exports. All provider traffic goes through shared Rust AI commands. No static `$HOME/**` scope is granted; provider credentials stay behind Rust secret commands.

## System preview

Document understanding is separate from preview integration. Background PDF rasters use
Core Graphics on macOS and Windows.Data.Pdf on Windows, with bounded pixels and native
thread/resource lifetimes. They are private analysis inputs, not a source-page viewer;
no rendering command, external filesystem scope or additional renderer process is exposed
to the WebView. Office embedded rasters use the existing document parser and image codec.
Original files are still opened through the associated desktop application.

`src-tauri/src/system_preview/*` owns OS preview integration. On macOS its adapter uses `QLPreviewPanel`, which is main-thread AppKit state. The `spaces` command authorizes a managed asset and then calls this internal engine; there is no separate public file-preview plugin and no general Renderer path capability. Unsupported platforms report unavailability explicitly, and the canvas omits the action there.
