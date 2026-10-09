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

- Exact-pinned Konva/vue-konva own canvas scene interaction. Adding another scene graph or graph
  editor would create a second spatial source of truth. Vue owns workbench composition and Knovum
  persists neutral canvas facts.
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
- `docx-rs` owns editable Word export; Typst owns fixed-layout PDF export. The already pinned
  [`office_oxide` 0.1.13 PPTX writer](https://github.com/yfedoseev/office_oxide/blob/v0.1.13/src/pptx/write.rs)
  owns presentation package serialization. Its `PptxWriter`/`SlideData` APIs accept title/body text
  and `Write + Seek`, so the same private Rust process can return bytes without file access, a new
  dependency or another runtime. Knovum retains section mapping and the 100-slide limit.
  [`python-pptx`](https://python-pptx.readthedocs.io/en/latest/) and
  [`PptxGenJS`](https://github.com/gitbrent/PptxGenJS) cover broader authoring but require an additional
  Python/Node execution and packaging boundary that this bounded Rust export does not need.
  The three formats remain separate capabilities rather than a generic Office framework. Real
  PowerPoint/Keynote viewing remains a platform acceptance requirement.

Detailed data and interaction contracts live in [spaces.md](spaces.md),
[document-export.md](document-export.md) and [data-ownership.md](data-ownership.md).

## Media processing and acquisition

- FFmpeg is the single local audio/video processing engine for normalization, cuts, concatenation,
  filters, subtitles and supported hardware encoding. GStreamer Editing Services is credible for a
  full nonlinear editor, but its compiled runtime and project model are excessive for Knovum's bounded
  timeline. Web timeline packages that embed Pixi/WebCodecs engines were rejected because they would
  introduce a competing media model.
- Subtitle import retains a bounded SRT/WebVTT parser because the reviewed libraries do not cover
  its reject-the-whole-import contract. [Media Captions](https://github.com/vidstack/captions)
  `0.0.18` and `1.0.4` silently discard reversed or zero-duration cues even in strict mode;
  [`subtitle`](https://github.com/gsantiago/subtitle.js) accepts out-of-range timestamp fields and
  exposes Node-stream APIs with partial WebVTT support;
  [`srt-vtt-parser`](https://github.com/plussub/srt-vtt-parser) lacks strict timestamp validation.
  Adding a second parser to recover discarded cues would duplicate the same responsibility. Reassess
  this exception when an upstream parser passes the malformed and mixed-validity import regressions.
- Core FFmpeg uses pinned static builds from [Martin Riedl](https://ffmpeg.martin-riedl.de/) on macOS and [Gyan](https://www.gyan.dev/ffmpeg/builds/) on Windows, both upstream-listed third-party suppliers. Conda-forge's maintained recipe was evaluated and previously used; its many-file shared-library closure adds relocation/signing surface without additional Knovum functionality or demonstrated performance benefit. Older `ffmpeg-static` npm artifacts were rejected because they lag the selected FFmpeg release. The producer downloads exact immutable archives by size/hash, extracts only FFmpeg plus notices, rejects non-system dynamic dependencies and qualifies the actual export/filter/encoding capabilities. Source and complete license review remain required; packaging owns exact sources and target evidence.
- yt-dlp's extractor registry owns supported rich-media sites. Rust invokes the pinned standalone
  executable with ambient config, plugins, cookies, playlists, self-update and remote components
  disabled. Provider-specific URL parsers and oEmbed clients would duplicate that registry.
- VoiceStudio was reviewed as a workflow and engine reference, not imported as application code.
  Knovum reuses its own Space authorization, managed assets, node ownership and durable task model.

FFmpeg and its compiled dependency closure require exact inventory, license and corresponding-source review before
distribution. Runtime topology is in [system-architecture.md](system-architecture.md); package
construction and approval gates are in [packaging.md](packaging.md) and [release.md](release.md).

## Local models and inference

- The application-managed CPython worker uses capability-specific locked profiles. Rust owns resource
  identity, process lifecycle, cancellation and publication; the worker receives verified local paths,
  stays offline during inference and loads only the selected capability.
- Qwen3 Embedding through Sentence Transformers remains the fixed semantic-index engine because the
  persisted vector dimension and model identity are database contracts. Local ASR uses MLX Whisper
  on Apple Silicon and faster-whisper/CTranslate2 on supported CPU/CUDA targets. FFmpeg owns input
  normalization.
- Qwen3 Forced Aligner was selected over WhisperX because WhisperX requires an older Torch line that
  conflicts with the secure runtime. CTC alternatives require a second language-specific catalog.
  The official Qwen aligner API supplies multilingual alignment while Knovum maps results to immutable
  transcript ranges.
- Speaker diarization uses Sherpa-ONNX with public ungated segmentation and embedding resources.
  This avoids Pyannote account-token plumbing and its training stack while retaining a maintained
  inference API.
- Spleeter through Sherpa-ONNX owns vocals/accompaniment separation. Demucs code is permissive, but
  the author's
  [checkpoint clarification](https://github.com/facebookresearch/demucs/issues/508#issuecomment-1663033356)
  restricts the reviewed pretrained weights; a third-party model-card tag cannot override it.
- Hub downloads use the official
  [huggingface-hub API](https://huggingface.co/docs/huggingface_hub/en/guides/download) and ModelScope
  SDK public APIs for their respective repository protocols. Rust owns allowlists, hashes,
  cancellation and atomic publication. The full ModelScope inference framework and Rust hf-hub file
  path were rejected because they add unrelated machinery or fail the required resume contract.

Source selection is explicit; there is no locale/IP routing or provider failover. A second SDK may be
removed only after one qualified protocol covers every approved snapshot, redistribution permission
and supported-network acceptance. Model identities, user-visible choices, GPU admission and resource
preparation are owned by [local-models.md](local-models.md).

## Speech generation

MOSS-TTS-Nano is the default compact CPU-capable synthesis and reference-cloning engine. Knovum uses
the
[official standalone ONNX runtime](https://github.com/OpenMOSS/MOSS-TTS-Nano/blob/8b7bcc9341b3b4ef3a3a58ba1338a7d85ff133eb/ort_cpu_runtime.py)
rather than its full PyTorch application. Supertonic lacked Chinese support; the reviewed OmniVoice
weights were noncommercial; CosyVoice's footprint was materially larger; Kokoro's reviewed archive
did not provide sufficient eSpeak distribution evidence. Those implementations are not retained as
aliases or dormant profiles.

VoxCPM2 owns optional high-quality cloning and voice design. Its published Python dependency graph
requires a Datasets line below the security floor, so metadata rewriting, advisory suppression and
environment splitting were rejected. The audio.cpp path uses the upstream-recommended
[audio.cpp](https://github.com/0xShug0/audio.cpp/tree/a1c24b2f90c6e6bcb9a525210d826717e96cdfa9)
at the frozen reviewed revision with verified
GGUF resources. Knovum adds a narrow callback bridge because upstream's public pull API exposes output
only after synchronous generation; the bridge preserves bounded chunks and cancellation without
modifying vendor source.

The worker initializes only the selected engine. Failure remains visible and does not rerun on a
different device. Exact snapshots, controls and platform qualification remain in
[local-models.md](local-models.md) and [release.md](release.md).

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
