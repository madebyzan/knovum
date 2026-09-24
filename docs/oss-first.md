# OSS-first gate

## Canvas image editing and video workbench reviewed 2026-09-23

- The existing exact-pinned **Konva/vue-konva** canvas stack and browser Canvas 2D primitives own
  interactive image projection. Crop, rotation, flip, tonal controls and alpha erasing do not
  justify another image-editor framework or a second scene graph. Rust's maintained `image` crate
  owns canonical raster materialization and validates encoded outputs before managed persistence
  or export. The shared strict image-edit contract keeps preview and output behavior aligned.
- Local U²-Net ONNX inference remains the background removal engine. Mature Rust image crates do
  not provide the requested generative layer decomposition or viewpoint synthesis. The existing
  DashScope image provider is extended for [Wan image super-resolution](https://help.aliyun.com/en/model-studio/wanx-image-edit-api-reference)
  and [Qwen image editing](https://help.aliyun.com/en/model-studio/qwen-image-edit-guide) rather
  than adding a parallel AI routing stack. [Qwen-Image-Layered via fal](https://fal.ai/models/fal-ai/qwen-image-layered/api)
  supplies actual separate transparent layers; its credential stays in the OS secret store and
  layer publishing remains a Rust multi-output transaction. Local Qwen deployment needs a
  Python/PyTorch/CUDA runtime and does not fit the Tauri desktop packaging contract.
- The official **FFmpeg filter graph** remains the video rendering engine. It already supplies
  trim/atrim, concat, volume, rotation, flip and libass timed-text rendering, while the platform H.264 encoder
  and Task Center cancellation are already provisioned by Knovum.
- GStreamer Editing Services is a credible full nonlinear editing engine with official Rust
  bindings, but would require a new native runtime and macOS/Windows packaging pipeline. The
  current product contract is one managed source, ordered cuts and timed text. FFmpeg remains
  the selected execution engine for that bounded scope; a future multi-source, multi-track
  realtime editor requires a new engine decision rather than a second concurrent timeline model.
- `vue-timeline-editor`, `vue-video-editor`, WebCut and small draggable timeline packages were
  reviewed. The focused timeline packages are immature, while the larger editors introduce their
  own WebCodecs/Pixi media engine and project model. Either choice would create a competing media
  source of truth beside Knovum's managed assets and FFmpeg pipeline. Knovum therefore owns only
  its small typed timeline/project interaction layer and delegates proxy preparation and final
  rendering to the existing Rust/FFmpeg boundary. The browser media element plays only managed
  source media during initial proxy preparation and the short-GOP proxy afterward.

## Desktop Agent installation discovery reviewed 2026-09-17

- Installation discovery and ACP execution are distinct requirements. Google publishes the [Antigravity ACP server separately](https://github.com/agentclientprotocol/registry/blob/main/antigravity-acp/agent.json); the GUI application's presence is not evidence that the ACP server is in PATH.
- Selected implementation: Apple's [NSWorkspace Bundle ID query](<https://developer.apple.com/documentation/appkit/nsworkspace/urlforapplication(withbundleidentifier:)>) through the existing `objc2` runtime, and Microsoft's [installed-program registration](https://learn.microsoft.com/en-us/windows/win32/msi/uninstall-registry-key) through the existing `windows` bindings. Only the target-local Windows Registry feature is added; no new package or WebView filesystem/process permission is introduced.
- Tauri's OS and opener plugins do not expose installed-application identity queries. Broad filesystem scanning, application-name/path guesses, shell-script discovery and launching Agent processes to test installation do not satisfy the identity-based, read-only requirement.
- Knovum owns the explicit application/ACP identities and maps native evidence into its existing installation contract. Native lookup failures are visible; no compatibility reader or alternate execution path is added. Desktop Agent settings is the sole automatic installation-query owner; execution still initializes ACP using only the configured server/host entry.

Custom infrastructure is the exception.

Before implementing a non-trivial capability:

1. Check Tauri/Vue first-party options.
2. Compare credible GitHub projects: recent releases/commits, maintainer depth, issue/PR health, license, security and platform support.
3. Check npm or crates.io/docs.rs release/adoption/dependency signals.
4. Select a maintained stable solution that satisfies the requirements.
5. Write custom logic only when every suitable maintained candidate misses a named, testable requirement.

"We can write it ourselves", API naming preference or avoiding a small integration seam is not a rejection reason.

## No wrapper inflation

OSS-first does not mean wrapping the OSS API. Import vendor APIs directly unless Knovum adds policy, validation, security, domain translation or multi-step orchestration.

## PR evidence

For new non-trivial dependencies/subsystems record:

```text
Capability / requirements:
GitHub candidates:
npm or crates.io candidates:
Maintenance and release signal:
Selected implementation:
Rejected alternatives + exact unmet requirement:
Tauri/security capability impact:
Why custom code is necessary (custom only):
```

## Foundation selections reviewed 2026-08-13

- Tauri 2 official plugins for desktop integration.
- Vue 3 + TypeScript + Vite.
- Reka UI for accessible headless behavior; Knovum owns the visual language.
- Vue Sonner for an imperative, accessible toast API instead of a custom toast queue.
- Pinia, TanStack Vue Query, VueUse and Zod for their focused responsibilities.
- TanStack Virtual and stable TanStack Table v8 for high-volume list/grid work.
- Oxlint + Oxfmt for lint/format; dependency-cruiser for Vue-aware dependency-graph rules; Knip + jscpd for dead/duplicate code.
- tinykeys for application-local shortcut matching instead of a custom keyboard parser.
- keyring-rs for OS credential storage rather than custom encryption/keychain code.

Feature engines remain deliberately deferred until their business requirements are concrete.

## Spaces selections reviewed 2026-08-14

- Exact-pinned `konva` and `vue-konva` are the infinite-canvas runtime. Konva's Stage, layered scene graph, hit graph, drag events and Transformer replace the former Vue Flow interaction surface. Vue remains the workbench and business-orchestration owner.
- Tauri 2 `Menu.popup()` owns right-click context menus in Spaces. This keeps context-menu rendering and interaction OS-native instead of recreating macOS/Windows menus in the WebView; standard OS behavior such as Canvas-background Paste uses Tauri's predefined native menu role. Reka remains the headless behavior layer for left-click Dialog/Listbox/Dropdown surfaces.
- SQLite FTS5 is used for local Space search rather than adding a second indexing service.
- Existing VueUse `watchDebounced` is used for bounded Note/Markdown autosave.
- Tiptap Vue 3 + StarterKit is selected for structured Note/Markdown editing because it supplies the ProseMirror editing model, history, standard blocks/marks and Vue integration while leaving Knovum in control of desktop UI. TaskList/TaskItem, TableKit, Placeholder and CharacterCount are first-party Tiptap extensions rather than custom equivalents.
- `@tiptap/markdown` is adopted only for the Markdown item: Markdown text remains canonical and only round-trippable features are enabled there. Knovum-owned Notes instead persist one validated structured document tree; the two products share the Tiptap engine without sharing a persistence contract.
- Tauri official plugins remain the desktop integration baseline. Clipboard image paste uses the official Clipboard Manager image API for image reads and a typed Rust command for validated persistence into the private Space asset directory. Saved Website metadata is fetched by the Rust `web_capture` capability with HTTP status, MIME and response-size boundaries. Mozilla `@mozilla/readability` 0.6.0 owns article/main-content extraction; Knovum only maps the parsed article into its KnowledgeSection contract instead of maintaining a parallel readability heuristic.
- Document ingestion is Rust-first. Exact-pinned `pdf-inspector` 1.20.0 owns PDF page-level Markdown and reading order, with default features disabled (no PDFium/OCR runtime). `office_oxide` owns DOC/DOCX/XLS/XLSX/PPT/PPTX plain-text and Markdown extraction. PDF source evidence displays extracted text and opens the original file; it does not render pages in the WebView. Office pixel-fidelity rendering remains separate from semantic ingestion.

PDF parser choice: [upstream Rust API](https://github.com/firecrawl/pdf-inspector/blob/v1.20.0/docs/rust-api.md) confirms the pinned version's public positioned image API. Knovum uses `extract_text_with_positions_mem_in_frame(..., PositionFrame::Display)` and `ItemType::Image`, not a private PDF walker. Normal text pages avoid visual calls; known recognition pages use explicit page inputs. Inline-image/vector coverage remains limited and visible. MIT/default-features-disabled extraction replaces PDFium and its dynamic libraries. Upstream speed claims are not Knovum benchmarks. Native Core Graphics (`objc2-core-graphics =0.3.2`) and Windows.Data.Pdf (existing `windows` bindings) render bounded selected regions without another renderer dependency. Existing `image` normalizes lossless PNGs; existing `zip` bounds Office expansion, never interprets Office semantics. `office_oxide =0.1.8` owns structure/images; its public XLSX cell API avoids IR row truncation. Knovum maps results and coordinates existing AI/job/asset contracts, with no alternate parser or compatibility reader.

## Editor and Rust-processing selections reviewed 2026-08-15

- Existing Tiptap Vue 3 + first-party Markdown/extensions remain the editor engine. `@tiptap/extension-file-handler` owns image drop/paste semantics, the first-party Image extension owns image nodes/resizing, TableKit owns table commands, and the first-party `@tiptap/extension-find-and-replace` owns editor search/highlighting/navigation. Knovum glue only persists selected image bytes into its private asset scope and applies desktop visual composition; it does not reimplement editor transactions.
- Image conversion/resizing uses the maintained Rust `image` crate rather than browser Canvas. The current output matrix is PNG/JPEG/BMP/TIFF; resize uses the crate's Lanczos3 filter and rotate/flip use its built-in transforms.
- Rust background removal uses exact-pinned `tract-onnx` 0.23.5 and the existing `image` crate. The U²-Net small model is embedded with an explicit source, license record, engine version, and SHA-256. Browser-only `modern-rembg` and `onnxruntime-web` were removed because CPU inference and image compositing in the WebView made the desktop shell unresponsive.
- Subtitled-video derivation reuses Knovum's provisioned FFmpeg runtime and current Whisper transcript contract. FFmpeg's libass `subtitles` filter plus the macOS VideoToolbox or Windows Media Foundation H.264 encoder own local subtitle burn-in, avoiding a GPL x264 distribution dependency. Knovum owns SubRip serialization, Space-scoped cancellation, source-revision validation, output persistence and lineage. No wrapper crate is added because the repository already has a bounded Rust process-control adapter.
- Spaces binary document/media import/export uses Tauri dialog/filesystem/opener boundaries and preserves original bytes. Native PDF page rasters and supported Office images are private document resources for background analysis, not source-page previews. Their durable manifest participates in the existing package and asset-reference lifecycle. FFmpeg generates persistent video posters. Audio/video playback remains activation-only while heavy editing stays a professional-app handoff.
- Sensitive Information nodes reuse the existing `keyring`-backed secret store. SQLite stores only non-secret metadata, avoiding a parallel encryption subsystem or application-managed master key.
- Spaces resource ingestion uses Rust `std::fs` for bounded private copies. The file-inspection boundary rejects directories before a canvas import job exists, while archive formats remain opaque binary file assets; workspace package ZIP transfer remains separate. Code/text preview is a bounded Rust read. Imported HTML runs only inside a sandboxed asset-protocol frame backed by Knovum-owned AppData; site-directory import is not supported.
- External professional editing stays an explicit opener handoff. Resource refresh occurs when the Knovum window resumes rather than adding a file-watcher dependency solely for this workflow.

## Spaces infinite canvas and media knowledge

- Infinite-canvas interaction uses Konva's official Stage, Layer, hit detection, drag events, transforms and redraw controls. Knovum adds only knowledge-object presentation, domain orchestration and the neutral persistence/exchange contract.
- Long media transcripts use `@tanstack/vue-virtual`, the Vue adapter for TanStack Virtual. The Inspector keeps the complete transcript model while mounting only visible rows.
- Local speech transcription uses the maintained whisper.cpp `whisper-cli` full JSON contract. FFmpeg normalizes imported audio/video once to 16 kHz mono PCM, then one accelerated Whisper process transcribes the complete track with built-in Silero VAD. The media domain persists token-timed subtitle cues split only at speech pauses, sentence boundaries, or elapsed-audio boundaries. FFmpeg's libass renderer owns glyph measurement and Unicode line wrapping against the real video frame; Knovum does not estimate line width from character counts. This keeps the model resident for the job, avoids intermediate chunk files, preserves original-timeline timestamps, and drives Task Center progress through the CLI callback. Browser speech APIs and cloud media-upload transcription are intentionally not part of the architecture.
- Semantic Knowledge Search executes locally in-process using `Qwen3-Embedding-0.6B` through `fastembed` and `candle` at 1,024 dimensions. This keeps vector generation 100% offline, private, and removes external API/cloud dependencies for knowledge indexing and semantic search.
- Translation uses the shared maintained `genai` multi-provider boundary and the same connection/model capability settings as other AI actions. Qwen-MT targets use Alibaba's documented `translation_options`; other explicitly translation-capable chat models use a provider-neutral translation-only chat protocol. Protocol selection is explicit from provider/model identity rather than a fallback, and Rust does not select a product-default model.
- Token usage accounting reuses `genai`'s provider response usage instead of installing a second tokenizer or approximating vendor billing. For the local embedding model, the existing Hugging Face `tokenizers` encoding reports its exact post-truncation input count during inference. Provider omissions stay explicit; Knovum does not synthesize cloud usage from prompt text.
- Background removal executes the bundled U2Net ONNX model directly with Tract's ONNX API. This avoids an incomplete ONNX Runtime C-API compatibility layer and preserves one pure-Rust inference implementation across Intel/Apple Silicon macOS, Windows and Linux.
- Chapter/Summary generation reuses Knovum's existing Rust generative-AI provider integration. It is opt-in, receives timestamped text rather than media binaries, and Chapter timestamps must resolve to real Transcript segment starts before persistence.
- Desktop Agent sessions reuse `agent-client-protocol` v1 MCP server declarations and the existing `rmcp` streamable-HTTP server. Each creation session receives a loopback-only, bearer-authenticated, Space-scoped endpoint that is cancelled with the ACP session; no Agent-specific knowledge protocol or direct database access is implemented.

## Release-candidate OSS decisions reviewed 2026-08-18

- Saved Website readable-content extraction uses Mozilla `@mozilla/readability` 0.6.0 rather than Knovum-owned DOM stripping/selector heuristics. Knovum keeps only security-sensitive Rust fetching, metadata normalization and domain translation.
- Space transactions use the already selected `sqlx` dependency in Rust. A generic renderer-to-Rust SQL transaction bridge was rejected because it would widen authority and obscure domain invariants; commands are named for their business mutation.
- `tract-onnx` remains exact-pinned. Runtime or model upgrades require cross-platform release builds, checksum/license review, representative-image fixtures, and a real inference smoke on macOS and Windows.
- Editable Word export uses `docx-rs` in the Rust layer. Fixed-layout PDF export uses Typst's official compiler and PDF exporter through exact-pinned `typst-as-lib`, with its system-font discovery feature enabled; filesystem/package/network resolvers remain disabled. Presentation export uses `PptxGenJS` inside a dedicated Web Worker so slide assembly cannot monopolize the canvas event loop. PDF and PPTX select the current platform's installed UI sans family. The light video editor uses exact-pinned `fontdb` (already present through Typst) to enumerate installed font families and face weights; libass burns the selected family and weight into the rendered video. Missing fonts are rejected before export. Editable projects still depend on fonts installed on the computer that renders them. None of these dependencies becomes a general office-document abstraction.
- Video production reuses the existing provider adapters, including the built-in Qwen ecosystem, and the existing FFmpeg runtime for deterministic media work. A generated editable script is an explicit intermediate knowledge resource; selected images are optional references, not a prerequisite or substitute for the script.
- SQLite mutations that read before writing start with `BEGIN IMMEDIATE` through the Spaces-owned transaction boundary. This gives one explicit write-serialization policy without retry loops, busy-error fallbacks or a renderer-visible generic database service.

## Knowledge work system selections reviewed 2026-09-22

- `rig-core` and `rig-agent` 0.42.0 provide only the built-in Assistant's stepwise model/tool control state. Knovum retains its `genai` model transport, durable transcript, tool Invocation/Receipt, authorization and Run checkpoints; Rig provider, MCP, memory and vector-store features remain disabled. Serialized Rig state is never the long-term product contract.
- `petgraph` 0.8.3 owns topological sorting and cycle detection for Workflow compilation. Knovum owns typed ports, branch coverage, execution policy and persistence because graph libraries do not define those product semantics.
- `cel` 0.14.5 evaluates bounded pure conditions and data transforms. The exposed environment contains only recorded input data; file, network, secrets, implicit clock and arbitrary code are unavailable.
- `cron` 0.17.0 and the existing `chrono-tz` timezone database parse and enumerate schedule candidates. Knovum owns authorization expiry, DST choice, overlap, misfire windows, deduplication and occurrence persistence.
- `rmcp` 3.1.3 remains the official Rust MCP transport for inbound Space tools and reviewed external tools. `agent-client-protocol` 2.1.0 remains the external desktop-Agent protocol. Both are exact-pinned because protocol or SDK changes require contract and recovery verification.
- `schemars` 1.2.2, `ts-rs` 12.0.1 and `jsonschema` 0.56.0 share Rust-owned static contracts with TypeScript and validate untrusted dynamic tool schemas in Rust. Remote schema references are not fetched, and Vue does not become a second dynamic-schema authority.
- Space Action parameters are generated from typed Rust contracts with `schemars`, emitted as JSON Schema 2020-12 and validated by `jsonschema`. The former hand-written `text/number/json` descriptor was removed because it duplicated the standard while losing enums, ranges and object structure.
- Automation event types follow the CloudEvents reverse-DNS convention (`dev.knovum.space.action.<action-id>.committed.v1`). The official Rust CloudEvents SDK was reviewed but not added because its own project documentation still marks all APIs unstable; Knovum does not claim that its local SQL outbox is a full CloudEvents transport envelope.
- `@vue-flow/core` 1.48.2 owns controlled graph interaction in Workflow Studio. The graph and keyboard-accessible outline edit one draft; Knovum does not infer state from undocumented viewport/change events.
- Apalis, Temporal and a second Agent framework were rejected: their queue, persistence or provider abstractions would create competing truths for the existing SQLx/Tokio Run scheduler and `genai` transport. The custom code is limited to Knovum-specific Run, authorization, publication and recovery semantics that these libraries cannot own without violating domain boundaries.

## Space catalog thumbnails reviewed 2026-09-16

Space catalog covers and Saved View previews are separate from file previews. `html-to-image` was removed because it cloned the live canvas DOM and made thumbnail work proportional to mounted UI complexity. Full-window and WebView snapshot APIs were rejected because they capture product chrome and couple derived content to platform window geometry. Knovum instead projects the neutral canvas scene through one non-interactive Konva thumbnail renderer: whole-Space covers use content bounds, while Saved Views use their persisted viewport. The renderer draws only canvas knowledge content and loads bounded local preview images; selection, handles, toolbars and zoom controls are not part of its input.

## Quick Look thumbnails and preview panel reviewed 2026-09-05

- macOS document thumbnails come from Apple's own QuickLookThumbnailing framework through the maintained `objc2-quick-look-thumbnailing` 0.3.2 bindings, with `objc2-image-io` 0.3.2 encoding the returned `CGImage` to PNG. Both live in a `cfg(target_os = "macos")` dependency block with narrow, non-default features. Knovum owns only the request policy, the completion-to-channel bridge and the private output path under the existing previews directory; the drawing itself is the system's.
- The macOS preview panel uses `quicklook` 0.2.2 (MIT OR Apache-2.0), a thin binding to Apple's QuickLookUI `QLPreviewPanel`, in the same `cfg(target_os = "macos")` block. It depends on `objc2` 0.6.4 and the 0.3.2 framework crates the thumbnail path already pins, so it adds no second Objective-C runtime version to the lockfile. Knovum owns only the path authorization, the main-thread dispatch and the toggle rule; the panel is AppKit's, so a Space file renders exactly as it does in Finder.
- `tauri-plugin-quicklook` 0.2.5 was reviewed and rejected for thumbnails: it exposes only the preview panel, fixes its own plugin command key, and Knovum needs the panel and the thumbnail to share one policy and one output directory. It was rejected for the panel as well: it registers seven commands that accept whatever path the WebView supplies, with no hook for a product-specific authorization rule, so adopting it would widen the WebView ACL from one authorized Space-asset preview to seven arbitrary-path panel operations. Its mature main-thread dispatch pattern was studied and re-implemented locally rather than adopting the plugin.
- Credential-shaped names (`.env`, `.env.*`, `.npmrc`, private keys, keystores, password stores) are never rendered into AppData: a canvas thumbnail must not become a second, less protected copy of a secret.

## Rich media links and Windows Shell preview reviewed 2026-09-10

- Media-reference recognition and local import use exact-pinned official yt-dlp standalone executables. This replaces provider-specific URL parsers and oEmbed clients: yt-dlp's extractor registry is the source of truth for supported sites, while Knovum owns the normalized persisted contract and managed-asset lifecycle.
- yt-dlp is invoked only by Rust with fixed machine-readable arguments. Ambient configuration, plugins, remote components, browser cookies, playlists and self-update are disabled. Downloads run in an isolated workspace and publish media plus poster into the Spaces managed-asset directory only after successful completion.
- YouTube extraction uses the JavaScript support bundled with the official yt-dlp executable and an exact-pinned application-owned Deno runtime. The yt-dlp source is Unlicense, while official PyInstaller standalone executables are GPL-3.0-or-later combined works because of bundled components. Release URLs and SHA-256 hashes live in the bundled-runtime manifest.
- Windows document thumbnails use `IShellItemImageFactory` through the target-only `windows` crate with only Foundation, GDI, COM and Shell features enabled. `THUMBNAILONLY` deliberately rejects generic icons. Extraction runs on a dedicated STA worker and publishes a PNG into the existing private preview cache.
- Windows full preview remains a separate `IPreviewHandler` host capability because it requires a native HWND, STA message loop and deterministic handler lifecycle. It is not approximated with a Vue panel or custom Office renderer.

## Regional model downloads reviewed 2026-09-17

Hugging Face supports endpoint overrides and full commit revisions in its [download API](https://huggingface.co/docs/huggingface_hub/main/en/package_reference/file_download#huggingface_hub.hf_hub_url). The official Rust [hf-hub client](https://github.com/huggingface/hf-hub) also supports `HF_ENDPOINT`; [hf-mirror](https://hf-mirror.com/) documents that same mechanism for its third-party community mirror. ModelScope was not selected as an interchangeable endpoint: different repository identities and revision schemes require independently verified artifact mappings.

Knovum retains its existing reqwest transport and model installation/receipt policy rather than adding a second Hub cache, authentication and installation lifecycle. Reqwest already owns HTTPS, redirects, proxy environment handling and [connection/read timeout primitives](https://docs.rs/reqwest/0.12.28/reqwest/struct.ClientBuilder.html). Knovum-specific code owns only endpoint precedence, bounded source selection and fresh-file restart after transport failures for public artifacts, and catalog size enforcement. It does not implement HTTP, resumable transfers or a generic mirror framework. Full commit revision and independently recorded SHA-256 remain identical across endpoints; no access token is sent. Explicit endpoint configuration disables automatic source switching.

# 文档提取与原生资源生命周期

office_oxide 0.1.8 的公共工作表图片、文字形状、单元格接口直接复用，
不为 XLS/XLSX 图片提取构造整份 IR；Word/Slides 使用借用的 IR 内容，
包括库实际提供的页眉页脚。逐元素带位置分块因公开渲染接口不足而保留产品映射，
不复制私有解析器，不克隆含图片的 IR。

pdf-inspector 1.20.0 提供文本及 Display 坐标，不提供公开嵌入图片字节提取接口。
必要区域由 CoreGraphics / Windows.Data.Pdf 渲染；不引入第二套 PDF 引擎。
macOS 位图翻转原地进行；Windows 页面、流、读取器在作用域退出时释放，
包括错误路径，不增加常驻原生渲染线程或跨线程句柄缓存。
