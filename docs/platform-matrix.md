# Desktop platform matrix

This is the framework-level contract for macOS and Windows. Business features should not reimplement these behaviors.

The supported Apple desktop baseline is macOS 14 or later on arm64 and x86_64. The application bundle
declares the same minimum through Tauri, matching the controlled runtime deployment target;
macOS 13 and earlier are outside the release scope.

| Concern                    | macOS                                                                                               | Windows                                                              | Foundation                                                                         |
| -------------------------- | --------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| Startup                    | Dock launch feedback; main appears when usable                                                      | Taskbar launch feedback; main appears when usable                    | main starts hidden until usable/recoverable bootstrap state                        |
| Window chrome              | native traffic lights + overlay titlebar/material                                                   | native caption + Mica content material                               | platform-specific Tauri config                                                     |
| Leading pane chrome        | unified toolbar starts after traffic lights; pane splitters begin below it                          | ordinary pane geometry; native caption stays untouched               | shell-owned pane/titlebar contract                                                 |
| Theme                      | follows system or explicit light/dark                                                               | follows system or explicit light/dark                                | configured + live OS listener                                                      |
| Window state               | size/position/maximized/fullscreen                                                                  | size/position/maximized/fullscreen                                   | persisted                                                                          |
| Close button               | hides main window; app remains available from Dock/menu-bar status item                             | hides main window to the system Tray                                 | Rust-owned desktop lifecycle                                                       |
| Explicit quit              | native Application Menu / menu-bar status item                                                      | Windows system Tray                                                  | Rust-owned                                                                         |
| Reopen                     | Dock/status item restores or focuses main                                                           | Tray focuses main window while running                               | Rust-owned                                                                         |
| Command palette            | Cmd+K via native Application Menu                                                                   | Ctrl+K via WebView shortcut                                          | shared command registry + command palette                                          |
| Shortcut display           | symbols                                                                                             | text modifiers                                                       | `UiKbd`                                                                            |
| Tray / status item         | installed as a menu-bar status item with app actions                                                | installed as system Tray with app actions                            | shared Rust-owned desktop menu                                                     |
| Task progress              | Dock progress                                                                                       | taskbar progress                                                     | mirrored from Task Center                                                          |
| System dialogs             | macOS picker/dialog                                                                                 | Windows picker/dialog                                                | official dialog plugin ready                                                       |
| Spaces clipboard images    | native clipboard image read -> Rust-managed private AppData PNG                                     | native clipboard image read -> Rust-managed private AppData PNG      | feature-scoped clipboard read + Rust persistence                                   |
| Cross-app dictation        | Apple Silicon / Intel: CoreAudio capture; Accessibility-gated Command+V into the same frontmost app | x64: WASAPI capture; UIPI-aware Ctrl+V into the same foreground HWND | Rust press/release session, private clipboard marker, qualified Python ASR profile |
| Spaces file/folder drop    | Finder drag -> validated private resource copy                                                      | Explorer drag -> validated private resource copy                     | Tauri drag paths; no persisted external scope                                      |
| Spaces document thumbnails | system Quick Look page render cached as a private PNG                                               | Windows Shell provider render cached as a private PNG                | one system-rendering policy; unavailable thumbnails keep the resource presentation |
| Spaces system preview      | spacebar or canvas menu opens the system panel for one private asset                                | native preview host for a private asset                              | Spaces asset preview operation over Rust platform adapters                         |
| Locale                     | OS BCP-47 locale                                                                                    | OS BCP-47 locale                                                     | system/en/zh-CN mapping                                                            |
| Font stack                 | system/SF family                                                                                    | Segoe UI Variable                                                    | tokenized                                                                          |
| Generated text             | PingFang SC via Core Text                                                                           | Microsoft YaHei UI via DirectWrite                                   | installed system fonts; no packaged font file                                      |
| Release trust              | signing + notarization                                                                              | code-signed installer recommended                                    | release configuration, credentials external                                        |

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

Inference, model downloads and the Builtin Agent use five independently verified on-demand Python
profiles: Resources, Light Inference, Torch Inference, Voice Generation and Agent. The installer
carries the application capability source and one trusted descriptor per profile. All profiles
use the frozen native CPython baseline with independent dependency roots and locks. Missing
resources trigger preparation; users do not select environments. First-use/offline acceptance
is required on all three native targets. Core FFmpeg is a separately verified bundled native
resource, not a Python profile; the same native producer prepares it before Vite in development.

The approved desktop release scope for native Python inference is macOS 14+ arm64/x64 and Windows 10 1903+ x64.
Qualification is tracked per architecture, not only per OS:

| Target          | Current native ASR path                           | Release state                                                                |
| --------------- | ------------------------------------------------- | ---------------------------------------------------------------------------- |
| macOS 14+ arm64 | MLX Whisper                                       | Implementation present; model, latency, and clean-install acceptance pending |
| macOS 14+ x64   | faster-whisper / CTranslate2 CPU INT8             | Implementation present; native runtime build and model acceptance pending    |
| Windows x64     | faster-whisper / CTranslate2 CUDA F16 or CPU INT8 | Implementation present; clean-install and quality acceptance pending         |

Spleeter separation offers FP16 and FP32 native ONNX snapshots using the same CPU engine. Both
produce distinct, exact-length 44.1 kHz stereo vocal/accompaniment assets for audition. Sherpa
speaker diarization uses the public segmentation/ERes2Net snapshot. These engine adapters are
currently CPU-only; enumerating a GPU does not create a supported provider. Images offer
compact/full U²-Net through the same Light Inference profile.

Speech generation uses MOSS-TTS-Nano as the small CPU default for preset voices and reference
cloning. VoxCPM2 is an optional original-precision native GGUF engine for quality cloning and
voice design. It requires at least 32 GiB physical memory and admission against the selected
profile's declared reservation. Before inference, the managed producer/runtime probes usable
Metal, CUDA or Vulkan and their current capacity; it selects CPU when no eligible GPU profile
fits. Inference errors are surfaced without a backend retry. Both engines return 48 kHz audio;
unsupported engine controls are absent from the corresponding form.

| Target          | Optional VoxCPM2 execution             | Qualification                                                      |
| --------------- | -------------------------------------- | ------------------------------------------------------------------ |
| macOS 14+ arm64 | native Metal or CPU, frozen before run | Native device/model and lifecycle acceptance required              |
| macOS 14+ x64   | native supported GPU or CPU            | Actual Intel CPU synthesis/clone/design recorded; GPU not accepted |
| Windows x64     | native CUDA / Vulkan or CPU            | Native non-administrator installation and GPU acceptance required  |

Windows arm64 and Linux remain outside the release scope. Unsupported targets return an explicit
error. Intel macOS uses frozen native dependency closures, with no emulation or legacy engine.
Both macOS architectures use native signing, distinct runtime publications and updater artifacts.
The [current Intel adapter record](evidence/local-audio-acceptance-2026-10-08.json) records actual
model/runtime identities, cancellation and produced media. Adapter execution does not establish
native application packaging, installer acceptance, human quality or another architecture.

On both desktop platforms, models and optional media-import tools are prepared into Tauri app-data
when the corresponding action needs them. Offline startup and basic Space editing do not require
these resources. FFmpeg remains a bundled native dependency; dictation uses the same selected
Python ASR model/runtime as media transcription. Downloaded
executables retain the platform's signing and execution constraints; hash verification does not
replace macOS signing/notarization or Windows execution checks. Release evidence belongs to
`packaging.md` and `release.md`.

The October 7 supplier, native voice and dependency-audit records describe earlier candidate
investigations. They are historical evidence: the current application uses the adopted native
VoxCPM2 execution path and MOSS default rather than the blocked Python VoxCPM2 package. The
current acceptance record retains unresolved corresponding-source/license review, publishing,
signing and real native-platform gaps instead of reusing old qualification claims.

### Optional forced alignment models

All supported desktop targets expose Qwen3 Forced Aligner FP32 and native Q8_0 model choices.
Q8_0 shares the frozen audio.cpp runtime closure with VoxCPM2 and uses its public C API; the
supplier is compiled for both families. Native backend discovery qualifies currently free GPU
capacity before execution: Metal on macOS and the compiled CUDA/Vulkan backends on Windows,
otherwise CPU is selected before worker startup. The full Transformers profile uses Torch's
qualified CPU/MPS/CUDA backend. A durable request freezes the model, and neither adapter retries
on another device after failure. Real Apple Silicon/Windows GPU and output-quality acceptance
remain separate from the shared code and native supplier contract.

The delivered CPython executable uses a UTF-8 Win32 process code page. Runtime production extracts and preserves its original activation manifest, adds `activeCodePage=UTF-8` with the Windows SDK Manifest Tool, then applies the existing signing and resource-file verification. Native relocation smoke checks `GetACP()==65001`; Python `-X utf8` alone does not configure native narrow-string filesystem APIs. This follows the [Microsoft UTF-8 process guidance](https://learn.microsoft.com/en-us/windows/apps/design/globalizing/use-utf8-code-page) without requiring a user to change Windows regional settings.
