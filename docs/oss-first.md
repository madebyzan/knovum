# OSS-first gate

Knovum adopts maintained upstream implementations before creating a subsystem. The goal is not to
maximize dependency count; it is to keep product code focused on Knovum policy, ownership and user
experience while established libraries own commodity protocol and engine behavior.

## Selection process

Before implementing a non-trivial capability:

1. Define the requirement, authority boundary, supported targets and failure contract.
2. Check official Tauri/Vue/platform APIs and dependencies already in the repository.
3. Compare credible GitHub and npm/crates.io candidates for maintenance, releases, issue health,
   license, security, platform support, dependency cost and public API fit.
4. Select the smallest maintained implementation that satisfies every named requirement.
5. Write custom code only for product policy or when all suitable candidates miss a testable
   requirement.

“We can write it ourselves”, naming preference and avoidance of a small integration seam are not
rejection reasons. Conversely, an OSS package is not selected merely because it exists: it must not
introduce a competing source of truth, broaden WebView authority or carry an unacceptable runtime,
license or maintenance burden.

Import upstream APIs directly. Add an adapter only when it enforces validation, security, domain
translation, orchestration or lifecycle policy. Do not create rename-only wrappers.

## Existing-code remediation

The same gate applies to maintained code, not only new capabilities. Remove a custom implementation
when an installed library or platform API covers its full contract, including cancellation,
ordering, resource bounds and supported targets. Keep product policy at the owning capability and
let upstream own scheduling, parsing, model serialization and primitive interaction.

Validate once at the owning trust boundary. Repository-owned typed constants and values already
validated in the same execution path do not need another structural parser. User edits, disk data,
network responses, IPC and external files still require validation. A TypeScript generic on an IPC
call does not validate its payload. A local desktop process is not a reason to remove path
authorization, bounded reads, credential handling or atomic publication checks.

Avoid serialization as cloning or as an intermediate step between an already parsed value and a
model that accepts it directly. Before removing a conversion, verify upstream semantics: strict
Pydantic JSON validation intentionally accepts enum strings that strict Python-object validation
rejects, and wire aliases or vendor-specific content blocks can require domain translation.

Do not replace a short standard-library expression with a new dependency merely to reduce custom
syntax. Assess the responsibility removed and the resulting code, rather than enforcing a deletion
quota or wrapping a library behind a new generic subsystem.

For a new dependency or custom subsystem, record:

```text
Capability and requirements:
GitHub and registry candidates:
Maintenance, security, license and platform evidence:
Selected implementation:
Rejected alternatives and exact unmet requirements:
Tauri/security capability impact:
Why custom code remains necessary (custom implementation only):
```

Exact version inventories belong to lockfiles and manifests. Dependency lifecycle rules are in
[dependency-policy.md](dependency-policy.md).

## Foundation and desktop integration

- Tauri 2 and its official plugins own supported desktop integrations; native framework bindings
  are used only where Tauri has no suitable API. Vue 3, Reka UI, Pinia, TanStack Vue Query, VueUse,
  Zod and the Oxc toolchain retain their focused responsibilities.
- Native task notifications use the [official Tauri notification plugin](https://v2.tauri.app/plugin/notification/)
  and its maintained [plugins-workspace source](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/notification).
  Direct `notify-rust` would duplicate Tauri's application identity/Windows adapter integration;
  custom AppKit/WinRT bindings would duplicate supported official behavior. Rust retains only
  task identity, privacy, background-delivery and user-preference policy. No WebView notify authority
  or browser-notification replacement is introduced.
- SQLx owns Rust-side SQLite access. The WebView does not receive SQL authority, and Knovum does not
  add a generic database service.
- Native application discovery uses Apple's
  [NSWorkspace application lookup](https://developer.apple.com/documentation/appkit/nsworkspace)
  and Microsoft's
  [installed-program registration](https://learn.microsoft.com/en-us/windows/win32/msi/uninstall-registry-key).
  Tauri's OS/opener plugins do not expose identity-based discovery. Filesystem scanning, path guesses
  and launching an Agent to test installation were rejected because they cannot prove identity and
  widen authority.
- Cross-application dictation uses Tauri's official global-shortcut plugin, RustAudio CPAL and narrow
  macOS/Windows input bindings. Browser `MediaRecorder` cannot survive a hidden WebView;
  `global-hotkey` would duplicate the Tauri boundary; Enigo does not own the required foreground
  identity and clipboard transaction. Knovum custom code is limited to authorization, bounded
  capture, cancellation and foreground revalidation.
- macOS thumbnails use QuickLookThumbnailing and the preview panel uses QuickLookUI. The reviewed
  Tauri quick-look plugin exposed arbitrary WebView-provided paths and did not supply thumbnails, so
  narrow native bindings preserve Space-owned path authorization. Windows thumbnails use
  `IShellItemImageFactory`; full preview remains a separate native `IPreviewHandler` capability.

The resulting lifecycle, permissions and interaction contracts are owned by
[desktop-foundation.md](desktop-foundation.md) and
[system-architecture.md](system-architecture.md).

## Canvas, editing and documents

- Exact-pinned Konva/vue-konva own canvas scene interaction, including the video editor's preview
  stage, where the transformer owns layer and overlay manipulation. Adding another scene graph or
  graph editor would create a second spatial source of truth. Vue owns workbench composition and
  Knovum persists neutral canvas facts.
- Tiptap and its first-party extensions own rich-text editing, tables, file handling and find/replace.
  Markdown text remains canonical only for Markdown items; Notes keep their validated structured
  document. Vendor JSON is not a compatibility model.
- Browser-native module Worker/OffscreenCanvas own canvas image-preview execution. VueUse owns shared
  consumer lifecycle; Spaces only orchestrates latest-surface requests, active priority and preview budgets.
  `useWebWorkerFn` does not expose the module-import and bitmap-transfer contract required by the existing
  edit renderer; a resize-only library such as Pica does not implement Knovum tone semantics. No additional
  worker framework or raster engine is installed. Rust `image` remains the canonical export/materialization
  authority.
- Rust `image` owns canonical raster materialization. U²-Net runs through the existing ONNX Runtime
  worker for background removal; browser ONNX and browser compositing were rejected because they
  block the desktop renderer. The maintained
  [rembg U²-Net adapter](https://github.com/danielgatis/rembg/blob/main/rembg/sessions/u2net.py)
  confirms the full model contract, but installing rembg would duplicate the existing adapter and
  image pipeline.
- `pdf-inspector` owns semantic PDF extraction; `office_oxide` owns Office semantic extraction.
  Platform PDF rendering is used only for bounded visual regions. A private PDF walker, second Office
  parser or custom fidelity renderer would duplicate mature parsers without satisfying Office
  preview fidelity.
- Mozilla Readability owns readable webpage extraction. Knovum retains security-sensitive fetching,
  normalization and persistence rather than maintaining selector heuristics.
- `docx-rs` owns Markdown Word export; Typst owns content PDF export. `office_oxide` owns
  DOCX serialization from its existing Office/Note IR. This overlap is retained for an explicit
  unmet requirement: the current vendor DOCX writer does not serialize `Image.hyperlink`, while
  the admitted Markdown writer preserves linked images. A direct swap also requires visible-title,
  flowing-quote, nested-list, table-style and page-fit-image translation corrections. Neither writer
  is a fallback for the other. Removing the dedicated writer requires an upstream linked-image
  implementation and equivalent user-visible output; packaging XML patches are not an alternative.
  The already pinned
  [`office_oxide` 0.1.13 PPTX writer](https://github.com/yfedoseev/office_oxide/blob/v0.1.13/src/pptx/write.rs)
  owns presentation package serialization. Its `PptxWriter`/`SlideData` APIs accept title/body text
  and `Write + Seek`, so the same private Rust process can return bytes without file access, a new
  dependency or another runtime. Knovum retains section mapping and the 100-slide limit.
  [`python-pptx`](https://python-pptx.readthedocs.io/en/latest/) and
  [`PptxGenJS`](https://github.com/gitbrent/PptxGenJS) cover broader authoring but require an additional
  Python/Node execution and packaging boundary that this bounded Rust export does not need.
  The three formats remain separate capabilities rather than a generic Office framework. Real
  PowerPoint/Keynote viewing remains a platform acceptance requirement.
- PDF/Office semantic content export reuses the pinned [pdf-inspector](https://github.com/firecrawl/pdf-inspector)
  and [office_oxide](https://github.com/yfedoseev/office_oxide) crates. Rich Tiptap Notes translate directly
  to the existing Office IR; no Markdown round-trip or second persistence model is introduced.
  The existing platform PDF adapter (Core Graphics PDF on macOS, `Windows.Data.Pdf` on Windows)
  supplies source-page/image rasters. A rendered page raster is a revision-qualified
  extraction artifact, not an AI fallback or a cached preview text.
- Original-layout Office PDF was evaluated separately from content export. A full local layout
  engine such as [LibreOffice](https://help.libreoffice.org/latest/en-US/text/shared/guide/start_parameters.html)
  would add hundreds of megabytes and runtime installation/maintenance for a capability outside
  Knovum's knowledge-authoring scope. On-demand installation moves that cost outside the default
  installer but does not remove it. The lightweight product boundary therefore excludes Office
  original-layout PDF conversion and engine provisioning; users who need the original layout export
  PDF in their external Office application. Semantic Office content reaches PDF through the same
  reflowed typesetting as every other document, and no such route is presented as original-layout
  conversion.
- Presentation pagination reuses [unicode-segmentation](https://github.com/unicode-rs/unicode-segmentation)
  and [unicode-width](https://github.com/unicode-rs/unicode-width), already present in the compiled
  dependency closure, as declared direct dependencies. They own grapheme boundaries and CJK width
  estimates; handwritten Unicode ranges and byte slicing were rejected because they split emoji
  and combining characters. A general font/layout engine or textwrap framework does not remove
  Knovum's concrete slide-capacity/table/image policy and would add a second authoring runtime.
  office_oxide owns serialization; Knovum only maps source content to bounded, readable slides.

Detailed data and interaction contracts live in [spaces.md](spaces.md),
[resource-export.md](resource-export.md) and [data-ownership.md](data-ownership.md).

## Media processing and acquisition

- FFmpeg is the single local audio/video processing engine for normalization, cuts, concatenation,
  filters, subtitles and supported hardware encoding. GStreamer Editing Services is credible for a
  full nonlinear editor, but its compiled runtime and project model are excessive for Knovum's bounded
  timeline. A web editing SDK that owns its own project model was rejected again below; the preview
  instead reuses maintained demux/decode and scene-graph libraries under Knovum's own project.
- The video editor's preview reads frames with [Mediabunny](https://github.com/Vanilagy/mediabunny)
  and composes them with the already pinned Konva scene graph. Rust/FFmpeg keeps normalization and
  export; the WebView only decodes the preview file Rust produced.

  ```text
  Capability and requirements: present any timeline position of a bounded edit, in sync with its
  sound, on the supported macOS and Windows WebViews; manipulate clip and text layers directly;
  decode only what the viewport needs; never drive playback through a media element.
  GitHub and registry candidates: WebAV (@webav/av-cliper, av-canvas), Mediabunny, etro,
  mp4box.js, hand-written HTMLVideoElement transport (the previous implementation).
  Maintenance, security, license and platform evidence: Mediabunny 1.61 is actively released,
  MPL-2.0, consumed unmodified, pure TypeScript with no compiled-code or worker build inputs, and
  documents Safari support; Konva 10 is already pinned for Space canvas interaction.
  Selected implementation: Mediabunny for container parsing, WebCodecs video decoding, frame
  sinks and filmstrip batches; Konva for the stage scene graph, hit areas and transformer;
  Web Audio for decoding and sample-accurate scheduling of clip sound.
  Rejected alternatives and exact unmet requirements: WebAV supports Chrome and Edge only, so it
  cannot run in the macOS WKWebView, and it owns a competing clip/sprite project model; etro is
  unmaintained for this contract; mp4box.js covers demuxing alone and leaves decode, sinks and
  thumbnails to Knovum; the previous HTMLVideoElement transport cannot satisfy the requirement at
  all, because the WebView's autoplay policy rejects `play()` and element seeking cannot hold a
  frame-accurate position for two overlapping clips.
  WebCodecs audio decoders are absent from the Safari versions Knovum supports, so audio is never
  decoded through WebCodecs: Mediabunny copies the audio track out of the preview file without
  re-encoding and Web Audio decodes it.
  Tauri/security capability impact: none. No new command, permission or WebView authority; the
  preview reads only the managed preview file Rust already authorizes.
  Why custom code remains necessary: the project model, transition parity with `xfade`, the ASS
  text parity, the audio-clock transport and the bounded thumbnail budget are Knovum policy. No
  library owns them, and no library is wrapped by a rename-only adapter.
  ```

- Subtitle import retains a bounded SRT/WebVTT parser because the reviewed libraries do not cover
  its reject-the-whole-import contract. [Media Captions](https://github.com/vidstack/captions)
  `0.0.18` and `1.0.4` silently discard reversed or zero-duration cues even in strict mode;
  [`subtitle`](https://github.com/gsantiago/subtitle.js) accepts out-of-range timestamp fields and
  exposes Node-stream APIs with partial WebVTT support;
  [`srt-vtt-parser`](https://github.com/plussub/srt-vtt-parser) lacks strict timestamp validation.
  Adding a second parser to recover discarded cues would duplicate the same responsibility. Reassess
  this exception when an upstream parser passes the malformed and mixed-validity import regressions.
- Core FFmpeg uses pinned static builds from [Martin Riedl](https://ffmpeg.martin-riedl.de/) on macOS and [Gyan](https://www.gyan.dev/ffmpeg/builds/) on Windows, both upstream-listed third-party suppliers. Conda-forge's maintained recipe was evaluated and previously used; its many-file shared-library closure adds relocation/signing surface without additional Knovum functionality or demonstrated performance benefit. Older `ffmpeg-static` npm artifacts were rejected because they lag the selected FFmpeg release. The media-libraries Cargo/CMake build downloads exact immutable archives by hash, extracts only FFmpeg plus notices, rejects non-system dynamic dependencies and qualifies the actual export/filter/encoding capabilities. Complete notices and pinned source records remain required; packaging owns exact sources and target evidence.
- yt-dlp's extractor registry owns supported rich-media sites. Rust invokes the pinned standalone
  executable with ambient config, plugins, cookies, playlists, self-update and remote components
  disabled. Provider-specific URL parsers and oEmbed clients would duplicate that registry.
- VoiceStudio was reviewed as a workflow and engine reference, not imported as application code.
  Knovum reuses its own Space authorization, managed assets, node ownership and durable task model.

FFmpeg and its compiled dependency closure require exact inventory, license and corresponding-source review before
distribution. Runtime topology is in [system-architecture.md](system-architecture.md); package
construction and approval gates are in [packaging.md](packaging.md) and [release.md](release.md).

## Local models and inference

- The Rust inference worker uses ONNX Runtime 1.22.1 via exact-pinned ort with the API-21
  contract, Candle 0.11 for Qwen3 embeddings, official whisper.cpp 1.8.3 C APIs, Sherpa-ONNX
  1.13.8 C APIs and frozen audio.cpp C APIs. Bindgen generates supplier ABI declarations.
  The host retains resource, memory, process, cancellation and publication policy.
- ONNX Runtime covers all nine image models. Tract was compared but does not meet the frozen
  graphs and mature cancellation/session requirements. Existing verified model exports remain fixed.
- Candle's official Qwen3 implementation replaces Sentence Transformers/Torch. The last-token
  output is qualified against the official F32 reference, and the new index identity rejects old data.
- whisper.cpp replaces MLX and Faster Whisper. The upstream MIT C API is used directly; whisper-rs
  was rejected because its Unlicense dependency is outside the repository's approved policy.
- Sherpa owns public ungated diarization and Spleeter separation. Its compiled build disables TTS
  and eSpeak, Python, PortAudio and server examples. No training framework is conveyed.
- reqwest is the existing maintained streaming client for pinned Hub HTTP protocols. hf-hub was
  compared but does not cover ModelScope plus this strict resume contract. The narrow Rust code
  owns HTTP validation, cancellation, hashes and atomic publication rather than another download
  subsystem or a second SDK.

## Speech generation

MOSS-TTS-Nano uses the original frozen ONNX graphs and built-in ten-speaker tables. ONNX Runtime,
SentencePiece, rubato and rand_pcg own graph execution, tokenization, reference resampling and RNG.
The maintained moss-tts-nano Rust candidate was evaluated: its private fixed 375-frame limit and
adaptive codec chunking cannot satisfy the required streaming/reference/cancellation contract.
audio.cpp exposes MOSS only as a synchronous result. Knovum therefore retains a narrow graph
orchestrator for this concrete gap; it does not implement neural layers, codecs, resamplers or RNG.
Its new frozen identity is `moss-tts-nano-100m-onnx-bundled-v2`.

VoxCPM2 uses the frozen upstream audio.cpp supplier and GGUF exports. A narrow C++ callback bridge
is required because the public pull API exposes output after synchronous generation; callback ABI 3
passes an explicit context, bounds chunks and supports cancellation. Vendor source is unchanged.
Forced alignment uses the public C API directly and F16/Q8 model exports.

Node media production uses yauzl for selected ZIP extraction and OS signing/dependency tools.
Model qualification uses actual engine tests and reviewed measurement artifacts; do not maintain
a separate corpus/report framework or scoring dependency stack in repository tooling.
Source/model identities, controls and platform acceptance remain in the owning model/release docs.

## Agents, protocols and extensibility

- Deep Agents/LangGraph own the built-in Agent graph, summaries, private workspace and checkpointing.
  Rust owns product actions, authorization, fees, publication and recovery. A second scheduler or
  custom graph/checkpoint framework would cross those boundaries.
- `rmcp` owns MCP client/server and OAuth protocol behavior; `agent-client-protocol` owns desktop
  Agent sessions. Knovum adds bounded transport, credential storage and Space authorization rather
  than implementing either protocol.
- Gitoxide reads reviewed HTTPS Git objects without shelling out to Git or honoring ambient user Git
  configuration. `yaml-rust2`, `iri-string` and Unicode normalization own their standard parsing and
  identity responsibilities.
- Tauri/Vue/TanStack/Zod primitives remain the plugin UI foundation. No frontend plugin framework or
  Python MCP client is added.

Upstream references: [Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk),
[Gitoxide](https://github.com/GitoxideLabs/gitoxide),
[yaml-rust2](https://github.com/Ethiraric/yaml-rust2),
[iri-string](https://github.com/lo48576/iri-string),
[unicode-normalization](https://github.com/unicode-rs/unicode-normalization),
[Agent Plugins](https://agent-plugins.org/) and [Skills](https://skills.sh/).

Protocol and authority boundaries are defined in [system-architecture.md](system-architecture.md)
and [secrets.md](secrets.md). Exact package versions and license gates remain lockfile- and
[dependency-policy.md](dependency-policy.md)-owned.

### Bundled extension workflows

The optional catalog reuses the existing Agent Plugins package inspector, immutable publication path,
account store and rmcp transports/OAuth. Skill progressive disclosure stays with DeepAgents
SkillsMiddleware and CompositeBackend rather than a second loader. GitHub, Notion and Linear use their
official remote MCP services ([GitHub](https://github.com/github/github-mcp-server/blob/main/docs/remote-server.md),
[Notion](https://developers.notion.com/guides/mcp/build-mcp-client),
[Linear](https://linear.app/docs/mcp)); no third-party proxy, installer or provider-specific transport is added.
The package references and templates orchestrate Knovum source reading/publication and service tools,
without adding inference engines or arbitrary script execution.

## Local recovery archives

Backup reuses maintained [SQLx](https://github.com/launchbadge/sqlx) and SQLite's
[`VACUUM INTO`](https://www.sqlite.org/lang_vacuum.html) for consistent snapshots before feature
pools open, and the existing [zip crate](https://github.com/zip-rs/zip2) for the bounded archive.
Adding rusqlite as a second SQLite access stack or a second archive engine would duplicate existing
responsibilities without meeting an additional requirement. Knovum owns the explicit storage
whitelist, baseline/hash validation, credential exclusion and restore publication journal because
those are product authority and lifecycle decisions.

## Desktop release tooling

Use the maintained official [tauri-apps/tauri-action](https://github.com/tauri-apps/tauri-action)
for app drafts, asset uploads and the standard updater JSON, and the official GitHub CLI for
Agent asset retrieval, provenance verification and resource releases. Direct `gh` calls suffice;
an Octokit dependency or custom GitHub client adds no required policy. The former candidate,
channel-promotion and hosting-migration framework is removed. Reviewed platform acceptance and
manual publication use GitHub's release interface. CI actions are pinned to reviewed commits.

[Apple-Actions/import-codesign-certs](https://github.com/Apple-Actions/import-codesign-certs)
imports and cleans up the temporary macOS signing keychain before resource signing. It replaces
handwritten keychain lifecycle steps; the Tauri bundler remains responsible for app signing and
notarization. FFmpeg preparation remains direct because its pinned supplier validation, required
filters/encoders, license notices and resource manifest are concrete Knovum requirements.

## Desktop build ownership

Cargo owns compiled build outputs in `OUT_DIR`; CMake ExternalProject owns the immutable supplier
download/configure/build/install graph. The compiled library crates use the CMake CLI directly, without Rust
build dependencies whose feature unification can produce duplicate supplier build units. This is a
standard project build, without a parallel recipe hash, cache or provisioner.

The single uv project owns editable Agent development. PyInstaller's maintained onedir mode owns
the distributable Python interpreter/module graph and platform binary relocation; Knovum adds
only its stdio/UTF-8, complete inventory, license and signing policy. Its official POSIX symlink
requirements are honored through Tauri's macOS directory-copy option. Onefile was rejected because
per-launch extraction adds startup/lifecycle surface; standalone Python archives and Conda
assembly were rejected because they make Knovum own interpreter installation and environment repair.
Nuitka was considered; compiling this orchestration process has no demonstrated requirement and
adds a C/C++ build graph for the retained Python SDKs. Existing SDK versions and compiled suppliers
remain selected; this change transfers their build ownership, not the engines.

References: [Cargo build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html),
[CMake ExternalProject](https://cmake.org/cmake/help/latest/module/ExternalProject.html),
[uv projects](https://docs.astral.sh/uv/guides/projects/),
[PyInstaller pitfalls](https://pyinstaller.org/en/stable/common-issues-and-pitfalls.html), and
[Tauri configuration](https://v2.tauri.app/reference/config/).
