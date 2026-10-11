# Desktop platform matrix

Knovum releases for macOS 14+ arm64, macOS 14+ x86_64, and Windows 10 2004+ (build 19041+) x64. Windows arm64 and
Linux are outside the product release scope. Unsupported targets fail explicitly; they do not receive
partial local inference or compatibility execution paths.

This document owns platform differences and real-machine acceptance requirements. Shared lifecycle
and capability rules belong to [desktop-foundation.md](desktop-foundation.md). Model choices, runtime
profiles, hardware admission, and download sizes belong to [local-models.md](local-models.md).
Artifact construction and promotion belong to [packaging.md](packaging.md) and
[release.md](release.md).

## Product behavior

| Concern              | macOS arm64 / x86_64                                        | Windows x64                                                      | Shared contract                                                          |
| -------------------- | ----------------------------------------------------------- | ---------------------------------------------------------------- | ------------------------------------------------------------------------ |
| Launch               | Dock feedback                                               | Taskbar feedback                                                 | Hidden main window until usable or recoverable bootstrap state           |
| Window chrome        | Native traffic lights; documented overlay titlebar/material | Native caption; Mica content where supported                     | Native controls remain OS-owned                                          |
| Background lifecycle | Close hides to Dock/menu-bar status item                    | Close hides to system Tray                                       | Explicit Quit terminates through the Rust lifecycle                      |
| Reopen               | Dock or status item focuses main                            | Tray focuses main                                                | Single-instance host owns focus and restore                              |
| Application commands | Native Application Menu and `Command` shortcuts             | No classic menu bar; `Ctrl` shortcuts                            | One `AppCommand` registry and palette                                    |
| Progress             | Rust-owned Dock progress                                    | Rust-owned Taskbar progress                                      | Rust producer state remains observable without the renderer              |
| Dialogs              | Native macOS dialogs                                        | Native Windows dialogs                                           | Official Tauri dialog boundary                                           |
| Clipboard images     | Native read, validated managed PNG                          | Native read, validated managed PNG                               | Rust-owned persistence; no broad WebView write scope                     |
| File drop            | Finder paths                                                | Explorer paths                                                   | Session authorization followed by Rust validation                        |
| Dictation capture    | CoreAudio                                                   | WASAPI                                                           | One Rust press/release session and private ASR worker                    |
| Dictation insertion  | Accessibility-gated paste to the same frontmost app         | UIPI-aware paste to the same foreground window                   | Target revalidation, private clipboard marker, text preserved on failure |
| Document rendering   | Core Graphics                                               | Windows.Data.Pdf                                                 | Background analysis only; no custom reader                               |
| System preview       | Quick Look adapter                                          | Windows preview adapter                                          | One authorized managed asset; explicit unavailability                    |
| Locale and text      | BCP-47; system/SF and PingFang via Core Text                | BCP-47; Segoe UI Variable and Microsoft YaHei UI via DirectWrite | System fonts; no packaged UI font                                        |
| Release trust        | Signed and notarized application/update                     | Code-signed installer/update                                     | Credentials remain outside source and build logs                         |

Both macOS architectures are separate release targets with distinct runtime publications, signatures,
and update artifacts. An arm64 result does not qualify x86_64, and an adapter test does not qualify an
installed application.

Both platforms use the application context-menu policy defined in [desktop-foundation.md](desktop-foundation.md).
Installed release acceptance must verify that non-editable surfaces expose no default browser menu,
capability-owned native menus still open, editable controls retain their editing menus, and the WebView
Inspector cannot be opened.

## Compiled resources and inference

FFmpeg is a bundled core resource on every supported target. The retained Agent environment, model weights, and optional
media tools are prepared on demand into verified application-owned storage. Offline startup and basic
Space editing do not require on-demand resources. Dictation and media transcription share the selected
ASR runtime and model rather than installing independent copies.

A durable inference request freezes its target, runtime, model, and device profile before worker
startup. GPU execution is chosen only when that target and model declare a qualified backend and current
capacity satisfies admission. Otherwise CPU is selected before startup. An inference failure is returned
to the user; the application does not retry on another device or engine.

Downloaded executables remain subject to system signing, quarantine, and execution policy. Hash
verification alone does not satisfy macOS signing/notarization or Windows execution acceptance. Exact
models and hardware requirements are maintained in [local-models.md](local-models.md).

Windows runtime executables that use Win32 narrow-string filesystem APIs require the documented UTF-8
process manifest and must verify the active code page on the installed target. Python UTF-8 mode alone
does not change those Win32 APIs.

## Permissions and optional integrations

Capability configuration follows the owning feature. File associations, external filesystem writes,
Shell commands, system HTTP/WebSocket destinations, updater endpoints, camera or screen capture, and
secondary windows are absent until a real product flow defines their scope. Microphone and native input
permissions are limited to recording and dictation. Installing a Tauri plugin never grants WebView
authority by itself.

## Real-machine acceptance

CI compilation is a required source gate, not platform acceptance. A release candidate must be tested
as the actual signed or signing-equivalent installed artifact on all three targets.

| Area            | Required evidence on each target                                                                                                                                                                                      |
| --------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Installation    | Clean non-developer install, launch, resource location, uninstall/upgrade path, and no dependency on checkout tools or `PATH`                                                                                         |
| Lifecycle       | Hidden bootstrap including entry/mount failure, native Retry/Quit, close/quit ACK loss, long save decisions, background hide, reopen, single instance, and saved window/panel state                                   |
| Data recovery   | Native backup/restore dialogs, exclusive maintenance progress, active-work rejection, interrupted publication recovery, credential reauthorization and malformed/obsolete archive rejection                           |
| Native UI       | Correct caption/traffic lights, focused Edit menu, native canvas context menu and keyboard invocation, IME input, modal command scope, dialogs, background notifications, preview, and progress after renderer reload |
| Permissions     | Microphone, Accessibility or Windows input restrictions, clipboard protection, selected-path scope, and clear denial recovery                                                                                         |
| Core media      | Bundled FFmpeg verification and representative decode/probe without external installation                                                                                                                             |
| Local resources | First-use download, progress, cancellation, restart recovery, offline reuse, integrity rejection, and safe retirement while leased                                                                                    |
| CPU inference   | Required baseline features complete within declared memory and timeout limits and publish valid output                                                                                                                |
| GPU inference   | Every advertised Metal, CUDA, or Vulkan path proves device admission, execution, cancellation, and output validity; unavailable hardware does not expose that path                                                    |
| Durable work    | Pause/cancel/restart, lease expiry, stale-result fencing, atomic publication, cleanup, and no duplicate output                                                                                                        |
| Quality         | Representative ASR, alignment, diarization, generation, and media outputs receive human review where the feature claims quality                                                                                       |
| Release trust   | System signature, notarization or Windows trust, updater signature, architecture identity, licenses/notices, and rollback/promotion controls                                                                          |

Evidence is target- and release-candidate-specific. It must record the application artifact, runtime and
model identities, device/backend, inputs, cancellation or recovery scenario, outputs, and reviewer.
Evidence from another architecture, a raw adapter, or an earlier candidate cannot satisfy the gate.

Any change to lifecycle, window chrome, menus, shortcuts, files, clipboard, notifications, updater,
processes, compiled resources, or device selection reopens the relevant acceptance rows. Release evidence
retention and promotion rules are defined in [release.md](release.md).

## Native inference target gate

All targets ship the private Rust worker and hash-sealed native libraries. Apple Silicon uses Candle/
Whisper Metal and admitted audio.cpp Metal; Intel macOS uses CPU for embedding/ASR and admitted
audio.cpp devices; Windows uses CPU embedding/ASR with optional qualified audio.cpp Vulkan. Images,
Sherpa and MOSS use CPU. Target-specific compile, real package, installed launch, cancellation and
model tests are required independently. Intel macOS evidence cannot approve Apple Silicon or Windows.

The native ONNX Runtime 1.22.1 supplier raises the product's Windows floor to 2004 (19041),
matching its [official release contract](https://github.com/microsoft/onnxruntime/releases/tag/v1.22.1).
The host and worker reject older builds visibly before loading native libraries. Windows version
detection imports Microsoft's `windows-version` 0.1.7 directly; no replacement OS probe or
older-runtime compatibility path is maintained.
