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

## Shell/OS integration ready for features

The runtime registers only plugins used by current product flows: clipboard, dialogs, filesystem, opener, autostart, OS information, Store, SQLite, logging, single-instance handling, window-state persistence, notifications and updater. Shell, WebSocket and arbitrary process execution remain unregistered.

Notification permission is feature-owned by the main WebView and only important state transitions are surfaced. The updater is Rust-owned and receives its endpoint/public verification key at release build time.

The base capability grants only low-risk desktop primitives already owned by the framework plus notification access for the main window. Clipboard and filesystem writes remain feature-scoped. Model networking no longer grants a wildcard HTTP client to the WebView; those calls stay behind validated Rust commands.

## Taskbar / Dock progress

The app task presentation store is mirrored to Tauri `Window.setProgressBar`: no running task clears progress, unknown-progress tasks are indeterminate, and determinate tasks produce aggregate progress. On Linux this OS decoration is only available on supported desktop environments; failure is logged and does not invalidate the actual job.

## Files and drag/drop

Do not grant `$HOME/**`. Runtime paths selected through the official Dialog or dropped through Tauri are session-scoped by Tauri. Register persisted-scope only for a feature that intentionally retains references to external paths across launches; the current Space copies imported resources into AppData and therefore does not persist source scopes. Use Tauri's window drag/drop API directly inside the owning feature when file/folder drop is part of that feature's UX. Folder ingestion is recursive only after Rust validation and rejects symbolic links, special files, oversized trees and oversized roots. Markdown image drop uses the same webview drag/drop path boundary: the whole active document is the target, dropped OS paths are session-authorized, Rust validates media type/size and copies the image into Space-owned AppData before the editor inserts it.

Clipboard image paste in Spaces uses the `tauri-plugin-clipboard` `read_image_binary` API for PNG-encoded reads, then a typed Rust command validates and writes the encoded PNG only into `$APPDATA/spaces/assets`. Space does not grant WebView mkdir/write/remove filesystem permissions; editor images and audio recordings use the same Rust-owned persistence boundary. Knovum-to-Knovum node copy/paste duplicates private resource ownership. Operating-system file-list paste is not emulated through browser `File` objects or broad filesystem scope; it requires a separately selected clipboard-file engine.

## Overlay titlebar ownership

macOS keeps native traffic lights and the Tauri overlay titlebar. The controls use Tauri's fixed `{ x: 16, y: 19 }` traffic-light position, optically aligned with the controls in the rendered 40px unified toolbar. The hidden-title window must not call `setTitle()` after creation because Tauri currently resets custom traffic-light positioning when the title changes. One unified toolbar spans the window. Its leading group owns shell navigation; the Space tab strip holds open tabs and the new-tab control directly following them, not document actions or task controls. Space Outline belongs to the sidebar, with Recently Deleted in its persistent footer. Space management uses native context menus on tabs, cards, Inbox and the current Space title. The Space view owns canvas creation tools and viewport controls. The shell owns a bottom-right content-area Task Center that opens upward and avoids the floating Inspector; individual knowledge imports retain their canvas-local progress. The toolbar's leading controls start after the native-control exclusion zone. The sidebar splitter starts below the toolbar, so its resize hit target never crosses the traffic lights; the Inspector is an independent canvas overlay. Windows keeps the native caption and does not inherit this traffic-light spacing.

## Multi-window

Only the `main` window is authorized today. A multi-window feature must define its window type, lifetime, route/context transfer and exact capability before a new label is added. Use official `WebviewWindow` APIs rather than a speculative window manager.

## Networking and processes

- AI provider traffic is owned by the Rust `ai` module through `genai`; the WebView passes a resolved execution target plus a stable credential identifier, never the secret value.
- Translation and editor AI reuse the shared model-provider runtime.
- WebSocket and Shell plugins are not registered until a concrete feature requires them.

## Updater and signing

The updater plugin is registered, but a build is considered update-enabled only when both `KNOVUM_UPDATER_ENDPOINT` and `KNOVUM_UPDATER_PUBKEY` are present at compile time. Production release builds use `tauri.release.conf.json`, which enables updater artifact generation. The application never invents placeholder release endpoints or verification keys; update signing, macOS code signing/notarization and Windows signing remain release-environment configuration documented in `release.md`.

## Cross-platform compile gate

`.github/workflows/platform-smoke.yml` runs the web gates and builds the desktop binary with `tauri build --no-bundle` on macOS and Windows runners for desktop-affecting pull requests. Linux remains the primary quality runner for full Rust/Web gates. This is a compile gate, not a substitute for manual OS behavior QA.

## Feature-owned desktop access

Spaces (owned internally by the `spaces` feature) is the first feature to activate feature-scoped desktop primitives. Its capability grants only typed Space actions: opening private imported resources/default applications, clipboard image reads, private asset mkdir/write/remove inside `$APPDATA/spaces/assets`, and narrow custom commands for persistence/import/refresh/duplication/export plus Saved Website preview capture and local media transcription. Space SQLite reads and writes stay behind Rust commands; the WebView has no SQL capability. Imported files and cached preview media are exposed to the WebView only from `$APPDATA/spaces/assets/**/*` and `$APPDATA/spaces/previews/**/*` through the asset protocol. `system_preview` exclusively owns document thumbnail rendering through Quick Look or Windows Shell. `pdf-inspector` owns PDF page-level text extraction without a native PDF runtime; `office_oxide` owns Office text/Markdown extraction, and FFmpeg owns video posters. The evidence inspector opens the original document through its associated desktop application rather than rendering PDF pages itself. Imported HTML is copied into Knovum-owned AppData and previewed in a sandboxed asset-protocol frame. Directory resource imports are rejected; archive files are treated as opaque binary file assets without extraction. Remote websites are never framed. Saved Website network access stays behind the custom Rust validation boundary rather than granting arbitrary dynamic HTTP origins to the WebView. Spaces also uses Tauri popup menus for right-click actions and grants only menu construction/popup plus resource-close capability; menu resources are closed after every popup. WebView-owned editable text is excluded from interception so OS text-editing semantics remain intact.

### Local media analysis tools

Media transcription engines are owned by `src-tauri/src/media/*`. The Space adapter validates the source through the Space-owned asset authorization boundary, performs blocking FFmpeg/whisper work off the async runtime, and streams stage/progress events to the WebView through a Tauri `Channel` so Task Center and Dock/Taskbar progress stay in sync. A temporary prepared WAV lives under a job-scoped `$APPCACHE/spaces/media-analysis` directory and is deleted before the command returns. One accelerated `whisper-cli` process transcribes the complete track with built-in Silero VAD, so long silence is skipped without repeatedly loading the Whisper model or extracting intermediate chunk files. Timestamps remain on the original media timeline when knowledge citations are persisted.

Bundled tool discovery is centralized in `src-tauri/src/local_resources/tools/mod.rs`; preview and transcription code do not implement separate lookup rules. Release/runtime configuration is explicit:

During local desktop development, `tools/run-tauri.mjs` binds these variables to the single
provisioned host runtime before launching Tauri. Packaged builds continue to resolve the same tools
from their target-specific application resource directory.

- `KNOVUM_FFMPEG_PATH`, or bundled `bundled-runtime/<platform>/ffmpeg[.exe]`;
- `KNOVUM_WHISPER_PATH`, or bundled `bundled-runtime/<platform>/whisper-cli[.exe]`;
- `KNOVUM_WHISPER_MODEL_PATH` and `KNOVUM_WHISPER_VAD_MODEL_PATH` are development/CI overrides for explicit Knovum-managed model installs. Release model weights live in application data and are never bundle resources.

The platform directory is one of `macos-arm64`, `macos-x64`, `windows-arm64`, `windows-x64`, `linux-arm64`, or `linux-x64`. Missing required tools/models are configuration errors; there is no browser speech-recognition path, remote media streaming path, or alternate silent implementation.

The `integrations` capability grants clipboard access shared across features. Filesystem access remains limited to Tauri session-scoped dropped/Dialog-selected paths and user-selected exports. All provider traffic goes through shared Rust AI commands. No static `$HOME/**` scope is granted; provider credentials stay behind Rust secret commands.

## System preview

Document understanding is separate from preview integration. Background PDF rasters use
Core Graphics on macOS and Windows.Data.Pdf on Windows, with bounded pixels and native
thread/resource lifetimes. They are private analysis inputs, not a source-page viewer;
no rendering command, external filesystem scope or additional renderer process is exposed
to the WebView. Office embedded rasters use the existing document parser and image codec.
Original files are still opened through the associated desktop application.

`src-tauri/src/system_preview/*` owns OS preview integration. On macOS its adapter uses `QLPreviewPanel`, which is main-thread AppKit state. The `spaces` command authorizes a managed asset and then calls this internal engine; there is no separate public file-preview plugin and no general Renderer path capability. Unsupported platforms report unavailability explicitly, and the canvas omits the action there.
