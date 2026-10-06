# Desktop platform matrix

This is the framework-level contract for macOS and Windows. Business features should not reimplement these behaviors.

The supported Apple desktop baseline is macOS 14 or later on arm64. The application bundle
declares the same minimum through Tauri, matching the controlled PyTorch and TorchCodec wheels;
macOS 13 and earlier are outside the release scope.

| Concern                    | macOS                                                                                       | Windows                                                              | Foundation                                                                         |
| -------------------------- | ------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| Startup                    | Dock launch feedback; main appears when usable                                              | Taskbar launch feedback; main appears when usable                    | main starts hidden until usable/recoverable bootstrap state                        |
| Window chrome              | native traffic lights + overlay titlebar/material                                           | native caption + Mica content material                               | platform-specific Tauri config                                                     |
| Leading pane chrome        | unified toolbar starts after traffic lights; pane splitters begin below it                  | ordinary pane geometry; native caption stays untouched               | shell-owned pane/titlebar contract                                                 |
| Theme                      | follows system or explicit light/dark                                                       | follows system or explicit light/dark                                | configured + live OS listener                                                      |
| Window state               | size/position/maximized/fullscreen                                                          | size/position/maximized/fullscreen                                   | persisted                                                                          |
| Close button               | hides main window; app remains available from Dock/menu-bar status item                     | hides main window to the system Tray                                 | Rust-owned desktop lifecycle                                                       |
| Explicit quit              | native Application Menu / menu-bar status item                                              | Windows system Tray                                                  | Rust-owned                                                                         |
| Reopen                     | Dock/status item restores or focuses main                                                   | Tray focuses main window while running                               | Rust-owned                                                                         |
| Command palette            | Cmd+K via native Application Menu                                                           | Ctrl+K via WebView shortcut                                          | shared command registry + command palette                                          |
| Shortcut display           | symbols                                                                                     | text modifiers                                                       | `UiKbd`                                                                            |
| Tray / status item         | installed as a menu-bar status item with app actions                                        | installed as system Tray with app actions                            | shared Rust-owned desktop menu                                                     |
| Task progress              | Dock progress                                                                               | taskbar progress                                                     | mirrored from Task Center                                                          |
| System dialogs             | macOS picker/dialog                                                                         | Windows picker/dialog                                                | official dialog plugin ready                                                       |
| Spaces clipboard images    | native clipboard image read -> Rust-managed private AppData PNG                             | native clipboard image read -> Rust-managed private AppData PNG      | feature-scoped clipboard read + Rust persistence                                   |
| Cross-app dictation        | Apple Silicon: CoreAudio capture; Accessibility-gated Command+V into the same frontmost app | x64: WASAPI capture; UIPI-aware Ctrl+V into the same foreground HWND | Rust press/release session, private clipboard marker, qualified Python ASR profile |
| Spaces file/folder drop    | Finder drag -> validated private resource copy                                              | Explorer drag -> validated private resource copy                     | Tauri drag paths; no persisted external scope                                      |
| Spaces document thumbnails | system Quick Look page render cached as a private PNG                                       | Windows Shell provider render cached as a private PNG                | one system-rendering policy; unavailable thumbnails keep the resource presentation |
| Spaces system preview      | spacebar or canvas menu opens the system panel for one private asset                        | native preview host for a private asset                              | Spaces asset preview operation over Rust platform adapters                         |
| Locale                     | OS BCP-47 locale                                                                            | OS BCP-47 locale                                                     | system/en/zh-CN mapping                                                            |
| Font stack                 | system/SF family                                                                            | Segoe UI Variable                                                    | tokenized                                                                          |
| Generated text             | PingFang SC via Core Text                                                                   | Microsoft YaHei UI via DirectWrite                                   | installed system fonts; no packaged font file                                      |
| Release trust              | signing + notarization                                                                      | code-signed installer recommended                                    | release configuration, credentials external                                        |

## Scope-dependent integrations

Document visual processing uses background Core Graphics PDF rendering on macOS and
Windows.Data.Pdf on Windows. No PDF renderer binary or rendering IPC is shipped.
This is separate from system document thumbnails and does not create a custom reader.
Other platforms return an explicit unsupported error. Native rendering/threading must
be manually validated on both supported desktop platforms.

These are intentionally not globally enabled because the correct configuration cannot be known before the owning feature exists:

- file associations / Open With: requires real extensions and ownership semantics;
- Shell: requires the Git/tool command and exact argument scope;
- filesystem writes outside app storage: requires a user-selected path or explicit feature scope;
- Tauri HTTP/WebSocket: requires known hosts/endpoints;
- updater endpoint/public key: requires the real release channel and signing keys;
- camera/screen capture entitlements: only when a real feature needs them; microphone usage is scoped to audio recording and Rust-owned dictation;
- secondary windows: requires a real window type, lifetime and capability set.

Installing a Tauri plugin does not grant the WebView permission to use it. Capability changes accompany the feature that needs the permission.

## QA rule

Any change to lifecycle, window, menu, keyboard, file, notification, updater or process behavior must be manually verified on both macOS and Windows in addition to CI compile checks.

## Optional runtime resources

Inference and Assistant Python components are separate verified on-demand profiles; the installer
carries only application capability source and two trusted platform descriptors. Both reuse the
same installer and native CPython baseline, with independent package roots and locks. Resource absence is a preparation state, not
an unsupported-device state. First-download/offline acceptance is required on both targets.

The approved desktop release scope for native Python inference is macOS 14+ arm64 and Windows x64.
Qualification is tracked per architecture, not only per OS:

| Target          | Current native ASR path               | Release state                                                                |
| --------------- | ------------------------------------- | ---------------------------------------------------------------------------- |
| macOS 14+ arm64 | MLX Whisper                           | Implementation present; model, latency, and clean-install acceptance pending |
| Windows x64     | faster-whisper / CTranslate2 CPU INT8 | Implementation present; clean-install and quality acceptance pending         |

Spleeter 2-stem source separation uses the same CPU FP16 ONNX profile on both supported targets.
The implementation and verified on-demand resource manifest are present. Native latency, peak
memory, spoken-dialogue residual and background-loss acceptance remain release gates on both
targets. The admitted profile is CPU with an 8 GiB inference reservation and a ten-minute input
limit; no candidate is used until the user auditions and accepts it.

Designed-voice synthesis has a separate conservative admission profile:

| Target      | VoxCPM2 execution profile | Admission gate                                                           | Release state                                                                 |
| ----------- | ------------------------- | ------------------------------------------------------------------------ | ----------------------------------------------------------------------------- |
| macOS arm64 | MPS float32               | 48 GiB physical-memory floor plus available-memory/reservation admission | Implementation present; native memory, latency and quality acceptance pending |
| Windows x64 | CPU bfloat16              | 32 GiB physical-memory floor plus available-memory/reservation admission | Implementation present; native memory, latency and quality acceptance pending |

There is no automatic device, precision or engine selection after admission fails. These memory
values are conservative product gates and must not be presented as measured minimum requirements.
VoxCPM2 inputs are verified 16 kHz mono voice revisions sourced from authorized 5–30 second samples;
the synthesis result is 48 kHz audio on both targets.

macOS x64 and Windows arm64 are outside the release scope. Unsupported targets return an explicit
error; there is no emulation or legacy engine route. A future scope expansion requires a product
decision, native dependency verification, and a release note.

On both desktop platforms, models and optional media-import tools are prepared into Tauri app-data
when the corresponding action needs them. Offline startup and basic Space editing do not require
these resources. FFmpeg remains a bundled native dependency; dictation uses the same selected
Python ASR model/runtime as media transcription. Downloaded
executables retain the platform's signing and execution constraints; hash verification does not
replace macOS signing/notarization or Windows execution checks. Release evidence belongs to
`packaging.md` and `release.md`.
