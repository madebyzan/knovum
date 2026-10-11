# Local AI runtime and model distribution

Knovum separates bundled core media runtimes from on-demand capability resources. Core knowledge features do not require Ollama or another model server.

## Shipped with the signed application

Platform-specific runtime binaries belong to the application release and are never fetched from arbitrary URLs at runtime:

- FFmpeg
- the Rust inference worker with its verified ONNX Runtime, audio.cpp, Sherpa and Whisper libraries
- one trusted descriptor for the retained on-demand CPython 3.11.17 Deep Agent profile

`pnpm desktop:dev` prepares native suppliers and the Rust worker before starting Tauri. CI publishes compiled resources for all three desktop targets and embeds the resulting immutable descriptors in the application. Models use the same verified application-owned installation locations in development and release; no individual-file environment overrides exist. Release builds package FFmpeg from `src-tauri/resources/bundled-runtime/<platform>/` into `resources/bundled-runtime/<platform>/`. The bundled native manifest lists every worker/library/notice file, size and SHA-256; the retained Agent has its own published Python inventory. Rust verifies the complete runtime before starting its private worker; Vue has no process-launch permission. Missing resources do not block ordinary startup or Space editing; feature execution prepares its required runtime and model through the shared cancellable Task Center.

## Prepared on first use

Resource status queries report media-import tools as unavailable on targets absent from the pinned
manifest without failing the status of other resources. The status retains the unsupported-platform
identity; preparing those tools still fails explicitly. Feature preparation distinguishes an unsupported
build architecture from missing runtime publication descriptors.

A supported web-media import prepares the exact-pinned yt-dlp executable and Deno JavaScript runtime
from the repository-owned manifest. Both are installed under the versioned
`local-resources/tools/` application-data namespace after SHA-256 verification. Concurrent imports
share the same installation, and the Task Center shows preparation progress. Settings reports their
status, but users do not need to understand or manually install them before using the feature.

## Local model preparation

Knovum does not download model weights during application installation or first launch. A capability
that needs a missing model prepares it after the user starts that capability and reports the work in
the shared Task Center. The Models settings page lists local models alongside generative provider connections. Local entries show their function, download size, installed bytes, verification state, download progress and cancellation. Qwen3 embeddings and platform-matched transcription are essential; the embedding identity is fixed. Optional models can be removed while idle without deleting generated content. A corrupt essential snapshot can be downloaded again through its existing atomic installer.

Opening or editing a Space checks local embedding readiness before background indexing. Missing
or unverified models leave the index unprepared without starting a download or reporting an indexing
failure. Enabling semantic search or choosing Related Content's preparation action prepares the
verified embedding snapshot explicitly; Related Content then schedules a full Space index.

| Capability                     | Supported models                                                               |
| ------------------------------ | ------------------------------------------------------------------------------ |
| Text embedding                 | Fixed Qwen3-Embedding-0.6B, Candle F32 last-token pooling                      |
| Transcription                  | Whisper Tiny / Small (default) Q8_0 / Large-v3 F16; whisper.cpp on all targets |
| Speech synthesis and cloning   | MOSS-TTS-Nano (small default); VoxCPM2 Q8_0 / original-precision GGUF          |
| Voice design                   | VoxCPM2 Q8_0 / original-precision GGUF                                         |
| Transcript alignment           | Qwen3 Forced Aligner Q8_0 (default) / F16 GGUF; audio.cpp                      |
| Speaker recognition            | Sherpa Pyannote 3.0 INT8 (default) / FP32 + ERes2Net embeddings                |
| Vocal/accompaniment separation | Spleeter FP16 (default) / FP32, Sherpa CPU engine                              |
| Image background removal       | U²-Net Compact (default) / full, ONNX Runtime                                  |
| Image upscaling                | Real-ESRGAN General X4v3 / X4plus, tiled ONNX Runtime                          |
| Image repair                   | MI-GAN, selected-region ONNX Runtime                                           |
| Image selection                | EfficientSAM Ti / S, cached ONNX Runtime                                       |
| Relative depth                 | Depth Anything V2 Small UINT8 / F32, ONNX Runtime                              |

All non-Agent models execute in the Rust worker. Rust uses reqwest for the exact Hugging Face and
ModelScope repository protocols; direct suppliers and archive members retain their frozen hashes.
Downloads validate 206 Content-Range before appending, restart on 200, stream bounded progress and
preserve partial bytes on cancellation. The installer verifies complete snapshots before atomic
publication. No Python Hub SDK, Torch, MLX or inference environment remains.

Whisper tiny/small use the official Q8_0 exports and Large-v3 uses the official F16 export.
All use the separately pinned whisper.cpp Silero 6.2.0 VAD binary, preserving the source timeline.
Silero's public VAD API selects and filters speech regions on the original waveform. Whisper's
state-based decode receives the original offset and duration, so both segment and word timestamps
retain source silence and gaps; no compressed-audio timestamp reconstruction is used.
With VAD enabled, automatic language detection uses the first speech window through the public
Whisper API, so leading silence is excluded from language detection while retaining source offsets.
Word timings combine complete Unicode token pieces. Chinese, Japanese, Thai, Lao, Burmese and
Cantonese retain these units; other languages merge non-space subwords using the upstream
[Whisper tokenizer language policy](https://github.com/openai/whisper/blob/main/whisper/tokenizer.py).
The upstream decoder pads its final window with 30 seconds of zeros. Native timestamps are
validated against that bounded context, then clipped to the source audio. Published word spans
must have positive duration within their segment; untimed punctuation remains in segment text.
Both dictation and media transcription use the native profile's compute budget. Tiny/Small and
Metal execution allow four times the input duration with a two-minute minimum. Large-v3 CPU
allows 120 times the input duration with a thirty-minute minimum; every profile is capped at six
hours. These are cancellable task deadlines, not quality or performance acceptance thresholds.

Forced alignment and speaker diarization are separate optional speech stages. Neither is required
to complete plain ASR. Qwen3 Forced Aligner is prepared only for a user-selected range no longer
than eight transcript segments or 300 seconds. Settings offers a smaller Q8_0 GGUF model (default)
and the native F16 GGUF. Both exports are fixed at
`audio-cpp/audio.cpp-gguf` revision `09fe073ba154561f4474162e8bd4ab233a848eca`.
Q8_0 weighs 1,129,966,496 bytes and F16 weighs 1,840,097,696 bytes; the catalog also owns
model-license evidence. The original HF file set cannot satisfy this supplier's required generation
configuration, so it is replaced completely. No fabricated sidecars, model-spec overrides or old
checkpoint readers exist. Public audio.cpp C APIs own loading, session execution and alignment.
Supported Metal/Vulkan devices are selected before inference after capacity checks; CPU is the
explicit profile on machines without an admitted accelerator. Execution failures remain errors.
Each durable alignment request freezes a model specification independent of its execution device.
Retries use that specification; transcript provenance retains the selected model and upstream revision.
The synchronous audio.cpp C API owns one session per worker and does not expose an inference abort hook.
Cancellation is checked before/after inference and between segments; the host's existing bounded
worker shutdown terminates a call that does not settle. No additional cancellation protocol is simulated.
Alignment writes stable word or character units against the selected transcript revision; partial
results remain visible, while a manual edit invalidates timing only for the edited segment.
The locked aligner profile covers Chinese, English, Cantonese, French, German,
Italian, Portuguese, Russian and Spanish. Japanese and Korean require optional
upstream tokenizers absent from this reviewed runtime and return
`unsupported_language`. Prepared PCM16 WAV samples are validated in Rust and passed to the native supplier as finite float samples.

Speaker diarization offers the official Sherpa-ONNX Pyannote 3.0 INT8 (default) and FP32 segmentation
exports, both with 3D-Speaker ERes2Net base embeddings. Weights and MIT/Apache-2.0 notices total
41,146,685 bytes for INT8 and 45,599,092 bytes for FP32. The Rust-owned catalog fixes each
upstream revision, path, size and SHA-256;
verified staging is atomically published. Downloads require no token, account, manual model import
or terms form. No external configuration enters the worker. The node action and Settings model
entry share the same cancellable capability preparation request and native runtime inventory.

Diarization is bounded to two hours of mono 16 kHz PCM16 WAV. The user may leave the speaker count
automatic or provide one exact known count from 1 through 32. Sherpa clustering does not expose
arbitrary minimum/maximum constraints; the product does not simulate them. Existing durable job
columns retain the automatic/exact-count invariant. Target-platform acceptance must measure automatic
speaker-count and turn quality; a successful known-count sample does not qualify automatic counting.
Video source separation uses the two fixed
selected Spleeter FP16 or FP32 ONNX pair through sherpa-onnx on CPU. Rust converts the source to 44.1 kHz stereo
PCM, verifies both stems on the same sample timeline and persists both candidate stems.
The frozen Sherpa STFT/ISTFT reconstructs complete 1,024-sample hops. The adapter pads the input
to a complete hop and at least one 4,096-sample window before inference, then crops both stems
back to the exact original frame count. It rejects an unexpected reconstruction length; the Rust
publication boundary retains its exact source-timeline check. This follows the
[pinned supplier's STFT implementation](https://github.com/k2-fsa/sherpa-onnx/blob/v1.13.8/sherpa-onnx/csrc/offline-source-separation-spleeter-impl.h).
Because sherpa's Spleeter implementation materializes the full STFT and masks, one request is
limited to ten minutes; longer videos must be split explicitly before separation and are never
silently truncated.
The model separates vocals and accompaniment; it does not establish that spoken dialogue was fully
removed or that every environmental sound was preserved. The candidate therefore remains unusable
as a video background until the user auditions and explicitly accepts it.

Every local model file has a pinned revision, size and SHA-256. The embedding snapshot includes the Sentence Transformers module and last-token pooling configuration, not just weights and tokenizer. Installation uses a temporary file, streams real byte progress into the shared Task Center, verifies the checksum, and only then atomically activates the model.
Candle 0.11 owns Qwen3 execution and tokenizers owns tokenization. Knovum applies the frozen query
instruction, document input, last-token selection and L2 normalization. The identity is
`qwen3-embedding-0.6b:candle-0.11-f32-lasttoken-v2`; persisted indexes with an older identity
are rejected and rebuilt explicitly. Inputs are limited to 32 texts and 2,048 tokens; output is
1,024 finite normalized dimensions.
Installed weights live under the current `local-resources/models/` application-data namespace; pre-release legacy paths are not read.

## Runtime ownership

Rust verifies resources and freezes the engine, model, device, owner, attempt, audio identity and
deadline before starting a private worker. Workers load verified snapshots offline, return validated
results or request-scoped artifacts, and cannot write the Knowledge database or vector index.
Process admission, resident-memory checks, idle reuse, cancellation and publication are defined in
[system-architecture.md](system-architecture.md); corresponding-source and license approval remains a
[release requirement](release.md). Ollama is an optional generative provider, not an embedding or ASR dependency.

## Generative AI and privacy

Knowledge indexing runs locally using the private Qwen3 worker. Vector embeddings and extracted chunk texts do not leave the machine. Grounded answering sends only the explicit prompt plus bounded relevant text context when cloud generative AI is configured. Source files and media binaries are not uploaded by these paths.

Summary and Chapters are optional generative media insights. A transcript is persisted locally before any generative step, so transcription remains useful when Builtin Agent AI is disabled.

## Device selection

GPU preference is application policy. A GPU path must be supported by the selected engine and
managed runtime, available on the device, and satisfy the execution profile's resource budget.
CPU is selected only when no usable supported GPU path exists. A GPU inference failure is surfaced;
the application does not retry that request on CPU or switch models. Users do not choose dependency
packages or implementation backends.

Qwen3 uses Candle Metal on Apple Silicon and CPU on Intel macOS and Windows. Whisper uses
Metal on Apple Silicon and CPU on the other targets. Images, Sherpa and MOSS use explicit CPU
profiles with bounded threads. audio.cpp alignment and VoxCPM2 use its public native device
probe, admitted Metal/Vulkan capacity and portable CPU dispatch. CUDA is not shipped or advertised.
Backend probing runs in a separate verified native process; it does not load Python or Torch.
Windows Vulkan remains subject to real hardware qualification. No request retries on another device.

## Hardware guidance

- Preset speech and the smaller local models retain the ordinary supported-target acceptance matrix.
- VoxCPM2 is optional and currently requires at least 32 GiB physical memory. The frozen CPU profile reserves a 16 GiB peak; Metal currently reserves 24 GiB pending target qualification. Admission accounts for the application memory budget and currently available memory. These conservative bounds are qualification gates, not measured claims for every platform.
- Both alignment profiles currently reserve 6 GiB of host memory and qualify GPU capacity before
  worker startup. This is a conservative admission bound; target qualification must raise it if a
  supported input exceeds it. A smaller download is not a claim of measured peak memory or speed.
- Discrete GPU: optional for Knovum's core knowledge runtime.
- Target-platform latency, peak memory and output-quality acceptance remain required on macOS arm64, macOS x64 and Windows x64.

## Model selection and identity

The Rust resource catalog owns model groups, defaults, supported operations, immutable revisions,
file sizes and hashes. Settings groups its entries by knowledge, audio and image capabilities; it
shows effect/hardware guidance, download and installed size, verification, progress, cancellation
and idle removal. The UI does not maintain another model allowlist or package selector.

Each selectable capability has an independent strict v1 preference. A new request reads its default
once and persists the selected model in the durable request/configuration or derivation engine.
Preparation, admission, worker identity, render caches, retries, provenance and package restore use
that frozen identity. Changing defaults affects new work only. Existing voices, projects, queued
requests and accepted outputs retain their model. Removing an idle optional snapshot preserves its
preference; its next use prepares the same snapshot. Obsolete baselines missing required identities
fail explicitly, without conversion or automatic deletion. Qwen vectors are fixed by the stored index.

U²-Net Compact/full snapshots are 4,574,861 / 175,997,641 bytes and share the same ONNX mask and
Rust compositing contract. Spleeter FP16/FP32 snapshots are 39,362,041 / 78,636,679 bytes and share
the exact source timeline and automatic two-node derivation contract. Both separation models execute on CPU;
no qualified GPU path is implemented for this engine. A precision choice changes the model rather
than the operation or its publication semantics.

Speaker variants use the shared snapshot installer, preparation, leases and removal path. Their
public sources need no account or import. The supported clustering controls follow the
[Sherpa diarization API](https://k2-fsa.github.io/sherpa/onnx/speaker-diarization/index.html).
Full precision is not a promise of better automatic speaker counts on every recording.

Reusable voices are application-wide immutable assets, exposed by Settings through application composition.
Every Space can pin their revisions without copying or assigning them to a Space.
Audio/video nodes own transcription, alignment, speaker recognition, separation and reference-voice
creation; text nodes own speech/audiobooks, and video nodes own dubbing. Operation/project and
asset-review contracts belong to [spaces.md](spaces.md). Download presentation belongs to
[ui-system.md](ui-system.md), and task lifetime to [background-tasks.md](background-tasks.md).

## Voice identity and delivery instructions

Text reading, audiobook paragraphs and video dubbing share the same immutable voice revisions.
VoxCPM2 offers the original 4,960,735,429-byte snapshot and a 2,955,019,717-byte Q8_0 snapshot;
cloning and design use the explicitly selected model, frozen before a task is created.
A smaller download does not establish a lower process/GPU peak; the Q8 profile retains the
qualified resource budget until hardware measurement supports a narrower one.

For a VoxCPM2 reference voice without an exact transcript, a paragraph can include an optional
delivery instruction of 1–500 characters. The same instruction is used in audition and final
generation, saved in the project revision, included in the render digest and preserved by Space
export/import. Its effect depends on the reference voice and language; it is not a graded emotion
control. MOSS presets, exact-transcript clones and saved designed voices do not expose this control.
The backend reports instruction support explicitly; the UI does not infer engine compatibility.

First-time voice design passes the description without a reference transcript. Once a designed
candidate is selected, its immutable recording and matching text form the reference for exact
cloning. The design description remains provenance, rather than being reapplied during generation.
This preserves the selected voice identity. See the [pinned official VoxCPM2 model card](https://huggingface.co/openbmb/VoxCPM2/resolve/32279effe8c19989596f05d353d1447f51d9e915/README.md).

## Image inference snapshots

Image resources use the same snapshot installer, verification, preparation task, leases and removal
owner as speech models. The catalog adds independent defaults for `imageUpscaling`, `imageInpainting`,
`imageSelection` and `imageDepth`; background removal keeps `imageSegmentation` and its existing default.
Selections are strict current v1 preferences, not an arbitrary model-import mechanism.

| Capability      | Catalog variants                     | Frozen inference contract                                                                                         |
| --------------- | ------------------------------------ | ----------------------------------------------------------------------------------------------------------------- |
| Upscaling       | Real-ESRGAN General x4v3 / x4plus    | Release v0.62.2 graph plus external weights; CPU float32, fixed 128px tiles, native 4×                            |
| Mask repair     | MI-GAN pipeline v2                   | Author ONNX uint8 RGB and mask; adapter flips selected-mask polarity and protects every unselected source byte    |
| Smart selection | EfficientSAM Ti / S                  | Frozen author encoder + decoder, RGB 0–1, foreground/exclusion points and rectangle; one worker-owned encoding    |
| Relative depth  | Depth Anything V2 Small FP32 / UINT8 | Frozen ONNX conversion and processor; ImageNet-normalized RGB, dimensions divisible by 14, relative inverse depth |

The resource catalog is authoritative for immutable URLs, exact hashes, file sets and bytes. Every
snapshot includes its license; depth also includes its processor and conversion model card.
Real-ESRGAN installation verifies the compressed archive and every extracted graph/data member;
transfer size is distinct from installed size. Complete multi-file snapshots are required before use.
The capability specification incorporates the complete snapshot digest. The sealed native runtime and its
existing ONNX Runtime 1.22.1 are reused; no Torch training dependency or second download service is added.

Initial CPU admission reservations are conservative guards, not measured performance claims.
Upscaling admits at most 4,194,304 rendered source pixels and a 4096px side; inference uses 128px
context tiles and publishes the requested 2K/4K/8K image. Repair/selection/depth admit at most
16,777,216 rendered source pixels. The existing image renderer also bounds original sides at 10,000px.
Selection reduces the encoding input to a 2048px bounding size; depth
uses its pinned processor with at most 1,048,576 normalized pixels. Output tensors are bounded at
256 MiB, canonical PNG assets at 64 MiB. Qualification must measure the actual selected model,
compiled runtime inventory, latency, cancellation and peak memory on every supported target.

BiRefNet Lite, LaMa, MobileSAM and larger noncommercial depth variants are not installed by this
change: a complete qualified and redistributable graph chain has not been established. A missing
second repair/background candidate is an unmet qualification item, not a renamed existing model.
Snapshot presence and adapter tests alone do not establish release qualification.

### Historical Python reference evidence — 2026-10-10

Seven frozen graphs were loaded and inferred through their source adapters with the installed
macOS x64 historical Python LightInference ONNX Runtime 1.22.1, CPU provider, four intra-op threads and
one inter-op thread. Development generation directory:
`ba6b3ec1862d5a16bbfcef32a574ae77f772c9d07d9d6a72a91cf7231b827b1c`.
The runtime manifest digest used by initialization is
`79d2ff3a97602d7ff871c8d1f45c734a90b1066cc83dd970b3c22ff1a40e8567`.
This is source-adapter evidence against the managed native libraries, not rebuilt-worker or
cross-platform release qualification. The 128×96 RGB fixture is the public-domain
[NASA astronaut photograph supplied by scikit-image](https://scikit-image.org/docs/0.25.2/api/skimage.data.html#skimage.data.astronaut),
whose downloaded PNG SHA-256 is
`88431cd9653ccd539741b555fb0a46b61558b301d4110412b5bc28b5e3ea6cb5`.
Depth used a 518×392 normalized input derived from the same photograph. Results from this
single photograph establish execution and tensor behavior, not general visual quality.

| Variant                     | Load (s) | First inference (s) | Warm inference (s) | Process peak RSS (MiB) |
| --------------------------- | -------- | ------------------- | ------------------ | ---------------------- |
| `realEsrganGeneralX4v3`     | 0.36     | 3.63                | 4.12               | 107.7                  |
| `realEsrganX4plus`          | 2.41     | 41.45               | 34.87              | 570.3                  |
| `migan`                     | 1.47     | 3.08                | 2.85               | 981.6                  |
| `efficientSamTi`            | 5.65     | 11.31               | 0.53               | 726.7                  |
| `efficientSamS`             | 6.73     | 20.17               | 0.19               | 1351.5                 |
| `depthAnythingV2Small`      | 3.43     | 3.05                | 3.04               | 436.4                  |
| `depthAnythingV2SmallUint8` | 1.98     | 2.86                | 2.74               | 330.2                  |

Outputs had the expected shapes/dtypes and finite values; repeated inference was byte-identical.
MI-GAN preserved all RGB bytes outside the selected area. Pre-cancelled calls rejected execution;
in-flight cancellation latency still needs compiled-worker qualification. The UINT8 depth output
had a 0.99957 correlation and 0.00649 mean absolute difference against normalized FP32 depth
on this fixture. The signed INT8 graph was rejected by ORT 1.22.1 because its ConvInteger kernel
was unsupported; it is excluded rather than retried through another runtime or model.
The UINT8 graph SHA-256 is
`fcf51f1b230362b28690bb9d1809bf0431f29cad20534e3f589bd7285547f20d`.

Release evidence remains required for macOS arm64 and Windows x64, the multi-category image
corpus, maximum admitted sizes, tile boundaries, in-flight cancellation, packaged-worker identity,
and desktop interaction screenshots. Depth-based background blur is withheld until depth-edge
quality is established. These outstanding checks must not be described as passing.

### Debug Worker protocol evidence — 2026-10-11

All seven variants also ran through the isolated debug-source Worker using the managed interpreter,
real initialization/compute/health/cancel RPC and the manifest identity above. Repeated outputs were
byte-identical, and corrupted input digests were rejected. The following timings were captured while
other development and verification work used the device; they establish protocol behavior, not a
controlled performance profile. RSS is the health observation after inference, not peak memory.
The smoke session specification was test-owned; this is not packaged-worker release qualification.

| Variant                     | Initialize (s) | Warm (s) | Observed RSS (MiB) | Cancel (s) | Cancellation outcome |
| --------------------------- | -------------- | -------- | ------------------ | ---------- | -------------------- |
| `realEsrganGeneralX4v3`     | 3.44           | 2.98     | 105.5              | 0.64       | cooperative          |
| `realEsrganX4plus`          | 2.31           | 66.60    | 579.6              | 5.18       | bounded termination  |
| `migan`                     | 8.71           | 5.81     | 993.3              | 4.56       | cooperative          |
| `efficientSamTi`            | 7.53           | 0.44     | 746.9              | 5.22       | bounded termination  |
| `efficientSamS`             | 8.53           | 0.47     | 1283.7             | 5.39       | bounded termination  |
| `depthAnythingV2Small`      | 5.67           | 4.58     | 446.8              | 4.19       | cooperative          |
| `depthAnythingV2SmallUint8` | 5.57           | 1.22     | 338.3              | 0.83       | cooperative          |

Cancellation was sent during active inference. Native ONNX calls cannot be interrupted directly;
SAM encoding and x4plus exceeded the five-second acknowledgement window in this run and needed
worker termination. The measured termination time includes process exit after that window. No result
from a cancelled invocation is published; the next request starts a fresh worker through the existing
lifecycle. Full maximum-size and supported-platform acceptance remains required.

### Image qualification reports

The existing acceptance command has a separate image suite:
`pnpm inference:acceptance --suite images --corpus <corpus.json> --capture <capture.json> --output <report.json>`.
Its strict contracts live in `python/tests/acceptance/image_corpus.py`; the same hashed-file resolver,
execution identities, device evidence and atomic report writer are reused from the core suite.

Preregister each target's model specification, snapshot digest, execution configuration and measured
load/warm/RSS/cancellation limits before capturing results. Every model profile must cover hair,
product edges, transparency, text, anime, repeated textures, large selections, thin images and EXIF
orientation. Each sample records its source URL, license, original bytes, worker-grid RGB tensor and
binary selected-region tensor with hashes. Worker-grid dimensions include the owning operation's
source reduction; depth evidence uses that grid before ImageNet normalization. Numeric artifacts
are frozen `.npy` files, read without pickle. Upscaling must return native 4× RGB, repair must preserve
unselected RGB exactly, selection must be binary, and depth must be finite, nonconstant and normalized.
Visual quality needs a named review with notes; numeric validity alone cannot establish good edges,
alpha composition or tile seams.

Captures bind the compiled Worker and runtime digests, model/configuration, target, device and
invocation identities. In-flight cancellation timing is distinct from rejecting a pre-cancelled request.
A missing model capture or reference-only execution reports `unavailable`; failed quality or resource
limits report `failed`. Current adapter smoke evidence does not satisfy this release gate. The unit
fixtures are synthetic contract checks and are deliberately not distributed as a qualification corpus.
