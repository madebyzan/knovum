# Local AI runtime and model distribution

Knovum separates bundled core media runtimes from on-demand capability resources. Core knowledge features do not require Ollama or another model server.

## Shipped with the signed application

Platform-specific runtime binaries belong to the application release and are never fetched from arbitrary URLs at runtime:

- FFmpeg
- trusted platform descriptors for the on-demand CPython 3.11.17 inference and Agent profiles

Debug builds prepare the selected managed runtime from this checkout on first use. CI publishes native resources for all three desktop targets and embeds the resulting immutable descriptors in the application. Models use the same verified application-owned installation locations in development and release; no individual-file environment overrides exist. Release builds package FFmpeg from `src-tauri/resources/bundled-runtime/<platform>/` into `resources/bundled-runtime/<platform>/`. Each separately published Python runtime has its own manifest listing every file, size and SHA-256. Rust verifies the complete runtime before starting its private worker; Vue has no process-launch permission. Missing resources do not block ordinary startup or Space editing; feature execution prepares its required runtime and model through the shared cancellable Task Center.

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

| Capability                     | Supported models                                                                                         |
| ------------------------------ | -------------------------------------------------------------------------------------------------------- |
| Text embedding                 | Fixed Qwen3-Embedding-0.6B, Sentence Transformers                                                        |
| Transcription                  | Whisper Tiny / Small (default) / Large-v3; MLX on Apple Silicon, Faster Whisper on Intel Mac and Windows |
| Speech synthesis and cloning   | MOSS-TTS-Nano (small default); VoxCPM2 Q8_0 / original-precision GGUF                                    |
| Voice design                   | VoxCPM2 Q8_0 / original-precision GGUF                                                                   |
| Transcript alignment           | Qwen3 Forced Aligner Q8_0 (default) / full Transformers checkpoint                                       |
| Speaker recognition            | Sherpa Pyannote 3.0 INT8 (default) / FP32 + ERes2Net embeddings                                          |
| Vocal/accompaniment separation | Spleeter FP16 (default) / FP32, Sherpa CPU engine                                                        |
| Image background removal       | U²-Net Compact (default) / full, ONNX engine                                                             |

Preparation selects the model snapshot's execution package: Intel ASR, background
removal, separation, diarization and MOSS-TTS-Nano use `light-inference`; Windows ASR, Apple Silicon ASR, embeddings
and alignment use `torch-inference`; VoxCPM2 uses `voice-generation`. Hub downloads use
`resources` only when verified snapshots are missing. Ordinary settings do not expose package
profiles or environment preparation controls. First-use preparation inspects required packages and
missing snapshots before downloading and reports their combined download budget through Task Center.
A missing production publication is a build configuration failure, not an unsupported computer.
Diarization, MOSS-TTS-Nano and VoxCPM2 use direct hash-bound downloads without the Hub SDK package.

The Apple Silicon ASR profile uses the ONNX Silero VAD model packaged with the exact-locked
`silero-vad==6.2.3` runtime dependency. It evaluates the prepared 16 kHz PCM audio on its original
timeline before MLX transcription; speech interval filtering does not compress timestamps.

Forced alignment and speaker diarization are separate optional speech stages. Neither is required
to complete plain ASR. Qwen3 Forced Aligner is prepared only for a user-selected range no longer
than eight transcript segments or 300 seconds. Settings offers a smaller Q8_0 native model (default)
and the official full Transformers checkpoint. The native snapshot fixes the audio.cpp GGUF export
at `09fe073ba154561f4474162e8bd4ab233a848eca`; weights, Apache-2.0 license and model card total
1,129,987,258 bytes. It uses the same frozen audio.cpp supplier as native voice generation,
through its public C API, and selects Metal/CUDA/Vulkan when the compiled backend has sufficient
currently free memory. CPU selection happens before inference; failures are reported without
trying another engine or device. The full snapshot is `Qwen/Qwen3-ForcedAligner-0.6B-hf` at
`c07281df297b9905d24a508279258cccf987a064`, matched to the frozen Transformers 5.19 API.
Its six native-layout files total 1,847,225,867 bytes. The obsolete checkpoint layout is not read or
converted. The adapter requests supplier loading evidence and rejects missing, unexpected or
mismatched parameters before inference; no randomly initialized weights are accepted. See the
[official native model documentation](https://huggingface.co/docs/transformers/model_doc/qwen3_asr).
Each durable alignment request freezes a model specification independent of its execution device.
Retries use that specification; transcript provenance retains the selected model and upstream revision.
The native synchronous C API owns one session per worker and does not expose an inference abort hook.
Cancellation is checked before/after inference and between segments; the host's existing bounded
worker shutdown terminates a call that does not settle. No additional cancellation protocol is simulated.
Alignment writes stable word or character units against the selected transcript revision; partial
results remain visible, while a manual edit invalidates timing only for the edited segment.
The locked aligner profile covers Chinese, English, Cantonese, French, German,
Italian, Portuguese, Russian and Spanish. Japanese and Korean require optional
upstream tokenizers absent from this reviewed runtime and return
`unsupported_language`. Prepared PCM samples enter the processor as NumPy arrays;
the worker does not delegate file decoding to Transformers' automatic backend.
See the [frozen 5.19.0 forced-alignment input contract](https://github.com/huggingface/transformers/blob/v5.19.0/src/transformers/models/qwen3_asr/processing_qwen3_asr.py).

Speaker diarization offers the official Sherpa-ONNX Pyannote 3.0 INT8 (default) and FP32 segmentation
exports, both with 3D-Speaker ERes2Net base embeddings. Weights and MIT/Apache-2.0 notices total
41,146,685 bytes for INT8 and 45,599,092 bytes for FP32. The Rust-owned catalog fixes each
upstream revision, path, size and SHA-256;
verified staging is atomically published. Downloads require no token, account, manual model import
or terms form. No external configuration enters the worker. The node action and Settings model
entry share the same cancellable capability preparation request and light inference package.

Diarization is bounded to two hours of mono 16 kHz PCM16 WAV. The user may leave the speaker count
automatic or provide one exact known count from 1 through 32. Sherpa clustering does not expose
arbitrary minimum/maximum constraints; the product does not simulate them. Existing durable job
columns retain the automatic/exact-count invariant. Native acceptance must measure automatic
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
The frozen Sentence Transformers 6.1 adapter uses its current public
[Pooling module](https://www.sbert.net/docs/package_reference/sentence_transformer/modules.html)
and `get_embedding_dimension` API. It requires exactly one `lasttoken` pooling module with prompt
inclusion; there is no legacy-attribute compatibility branch. Real JSON-RPC qualification checks
1,024 finite dimensions, unit norms and relevant-query ranking on the complete verified Qwen snapshot.
Installed weights live under the current `local-resources/models/` application-data namespace; pre-release legacy paths are not read.

## Runtime ownership

Rust verifies resources and freezes the engine, model, device, owner, attempt, audio identity and
deadline before starting a private worker. Workers load verified snapshots offline, return validated
results or request-scoped artifacts, and cannot write the Knowledge database or vector index.
Process admission, resident-memory checks, idle reuse, cancellation and publication are defined in
[system-architecture.md](system-architecture.md); native source and license approval remains a
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

Qwen3 embeddings now probe the verified managed Torch runtime before admission, prefer CUDA then
MPS when supported and available, and otherwise select CPU. Torch 2.13 requires unified-memory
Apple Silicon for MPS allocation; Intel AMD GPU enumeration alone does not qualify that backend.
Detection runs before each execution profile is frozen. CUDA qualification uses current globally free device memory, not total VRAM; previous reports cannot freeze a later job on CPU or reuse GPU capacity occupied by another application. Detection failures remain errors. MPS checks its recommended working-set capacity, while admission separately checks currently available host memory.
The request and worker identity freeze the selected backend, F32 precision and thread budget.
The model revision, last-token pooling, token limit and 1,024-dimensional index remain fixed.
The unified-memory restriction is in the pinned [Torch MPS allocator](https://github.com/pytorch/pytorch/blob/v2.13.0/aten/src/ATen/mps/MPSAllocator.mm).
Apple Silicon transcription already selects MLX/Metal. Windows uses the frozen CUDA 12.8 Tensor closure (Torch 2.13.0, cuDNN 9.10.2.21) for Qwen and Faster Whisper. It includes CPU execution for computers without a qualified CUDA device. Faster Whisper checks CTranslate2 float16 support before selecting its CUDA profile. The closure is application-managed; users do not install a CUDA toolkit. Windows native GPU execution still requires hardware acceptance. Other CPU-only engine packages retain their supplier execution contract.
VoxCPM2's native x64 runtime builds the supplier's CPU instruction-set variants and chooses a supported kernel before loading the model. Its Windows Vulkan backend is a separately loaded module; a machine without a Vulkan driver can initialize the CPU backend. Native Metal/Vulkan availability and free GPU capacity are probed before the request freezes its device. The Windows build path still needs actual Windows native acceptance; the Intel host does not prove that target.
Backend selection follows the official [PyTorch backend availability API](https://docs.pytorch.org/docs/stable/backends)
and [Sentence Transformers device contract](https://sbert.net/docs/package_reference/sentence_transformer/model.html).

## Hardware guidance

- Preset speech and the smaller local models retain the ordinary supported-target acceptance matrix.
- VoxCPM2 is optional and currently requires at least 32 GiB physical memory. The frozen native CPU profile reserves a 16 GiB peak; Metal currently reserves 24 GiB pending native qualification. Admission accounts for the application memory budget and currently available memory. These conservative bounds are qualification gates, not measured claims for every platform.
- Both alignment profiles currently reserve 6 GiB of host memory and qualify GPU capacity before
  worker startup. This is a conservative admission bound; native qualification must raise it if a
  supported input exceeds it. A smaller download is not a claim of measured peak memory or speed.
- Discrete GPU: optional for Knovum's core knowledge runtime.
- Native latency, peak memory and output-quality acceptance remain required on macOS arm64, macOS x64 and Windows x64.

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
the exact source timeline and two-output review contract. Both separation models execute on CPU;
no qualified GPU path is implemented for this engine. A precision choice changes the model rather
than the operation or the user's acceptance responsibility.

Speaker variants use the shared snapshot installer, preparation, leases and removal path. Their
public sources need no account or import. The supported clustering controls follow the
[Sherpa diarization API](https://k2-fsa.github.io/sherpa/onnx/speaker-diarization/index.html).
Full precision is not a promise of better automatic speaker counts on every recording.

Reusable voices are Space-owned immutable assets, exposed by Settings through application composition.
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
