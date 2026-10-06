# Local AI runtime and model distribution

Knovum separates bundled core media runtimes from on-demand capability resources. Core knowledge features do not require Ollama or another model server.

## Shipped with the signed application

Platform-specific runtime binaries belong to the application release and are never fetched from arbitrary URLs at runtime:

- FFmpeg
- a controlled CPython 3.11.17 runtime for local embedding, image segmentation, and speech inference

Development and CI point to controlled runtime paths through the documented environment variables. Release builds package the runtime from `src-tauri/resources/bundled-runtime/<platform>/` into `resources/bundled-runtime/<platform>/`. The Python runtime has its own manifest listing every file, size and SHA-256. Rust verifies the complete runtime before starting its private worker; Vue has no process-launch permission. Missing Python does not block ordinary startup or Space editing, but embedding requests fail with `runtime_missing`.

## Prepared on first use

A supported web-media import prepares the exact-pinned yt-dlp executable and Deno JavaScript runtime
from the repository-owned manifest. Both are installed under the versioned
`local-resources/tools/` application-data namespace after SHA-256 verification. Concurrent imports
share the same installation, and the Task Center shows preparation progress. Settings reports their
status, but users do not need to understand or manually install them before using the feature.

## Local model preparation

Knovum does not download model weights during application installation or first launch. A capability
that needs a missing model prepares it after the user starts that capability and reports the work in
the shared Task Center. The provider settings page manages generative model connections, not these resources.

| Capability                         | Model                                                                                           | Ownership | Product action             |
| ---------------------------------- | ----------------------------------------------------------------------------------------------- | --------- | -------------------------- |
| Local text embedding               | `Qwen3-Embedding-0.6B` (Sentence Transformers)                                                  | Knovum    | Prepare on use             |
| Multilingual media transcription   | `mlx-community/whisper-small-mlx` on macOS arm64; `Systran/faster-whisper-small` on Windows x64 | Knovum    | Prepare on use             |
| Preset speech synthesis            | `kokoro-int8-multi-lang-v1_0`                                                                   | Knovum    | Prepare on use             |
| Designed-voice synthesis           | `openbmb/VoxCPM2`                                                                               | Knovum    | Prepare on use             |
| Selected transcript alignment      | `Qwen3-ForcedAligner-0.6B`                                                                      | Knovum    | Prepare on use             |
| Optional speaker diarization       | `pyannote/speaker-diarization-community-1`                                                      | User      | Import after authorization |
| Video vocals/background separation | `csukuangfj/sherpa-onnx-spleeter-2stems-fp16` at `93ba771920ade509f8cbd6825b1a90856c797e08`     | Knovum    | Prepare on use             |

The Apple Silicon ASR profile uses the ONNX Silero VAD model packaged with the exact-locked
`silero-vad==6.2.3` runtime dependency. It evaluates the prepared 16 kHz PCM audio on its original
timeline before MLX transcription; speech interval filtering does not compress timestamps.

Forced alignment and speaker diarization are separate optional speech stages. Neither is required
to complete plain ASR. Qwen3 Forced Aligner is prepared only for a user-selected range no longer
than eight transcript segments or 300 seconds and runs on CPU from its complete verified snapshot.
Alignment writes stable word or character units against the selected transcript revision; partial
results remain visible, while a manual edit invalidates timing only for the edited segment.
The locked aligner profile covers Chinese, English, Cantonese, French, German,
Italian, Portuguese, Russian and Spanish. Japanese and Korean require optional
upstream tokenizers absent from this reviewed runtime and return
`unsupported_language`. Prepared PCM samples enter the processor as NumPy arrays;
the worker does not delegate file decoding to Transformers' automatic backend.
See the [5.13.1 forced-alignment input contract](https://github.com/huggingface/transformers/blob/v5.13.1/src/transformers/models/qwen3_asr/processing_qwen3_asr.py).

Pyannote Community-1 is never downloaded by Knovum. The user accepts the Hugging Face terms in the
browser and selects a directory containing the four files from revision
`3533c8cf8e369892e6b79ff1bf80f7b0286a54ee`. Rust verifies their fixed names, sizes and SHA-256 values,
copies only those files, and writes Knovum's reviewed pipeline configuration. External YAML, Hugging
Face tokens and network fallback never enter the worker. Diarization is an optional CPU job bounded
to two hours and speaker bounds from 1 through 32. Rust supplies mono 16 kHz PCM16 WAV;
Python validates its actual frame count and passes the decoded tensor through
pyannote's official `waveform` input. This path does not use TorchCodec file decoding
or require FFmpeg shared libraries; the bundled FFmpeg executable owns transcoding.
Pyannote can report its upstream optional-decoder warning during import. See its
[4.0.7 audio input contract](https://github.com/pyannote/pyannote-audio/blob/4.0.7/src/pyannote/audio/core/io.py).
Video source separation uses the two fixed
Spleeter FP16 ONNX files through sherpa-onnx on CPU. Rust converts the source to 44.1 kHz stereo
PCM, verifies both stems on the same sample timeline and persists only the accompaniment candidate.
Because sherpa's Spleeter implementation materializes the full STFT and masks, one request is
limited to ten minutes; longer videos must be split explicitly before separation and are never
silently truncated.
The model separates vocals and accompaniment; it does not establish that spoken dialogue was fully
removed or that every environmental sound was preserved. The candidate therefore remains unusable
as a video background until the user auditions and explicitly accepts it.

Every local model file has a pinned revision, size and SHA-256. The embedding snapshot includes the Sentence Transformers module and last-token pooling configuration, not just weights and tokenizer. Installation uses a temporary file, streams real byte progress into the shared Task Center, verifies the checksum, and only then atomically activates the model.
Installed weights live under the current `local-resources/models/` application-data namespace; pre-release legacy paths are not read.

## Runtime ownership

- Rust verifies each platform's ASR and speech files before starting the private Python worker. The request fixes engine, device, model specification, audio hash, owner, attempt, and deadline; no engine is silently selected after a failure.
- Source separation is a dedicated durable Run. Its source revision, attempt and output publication are Rust-owned; Python can write only request-scoped artifacts. Long stems are hashed and copied as streams rather than accumulated in Rust memory.
- Designed and cloned voice candidates run through the durable Run/Attempt scheduler. Rust freezes clone input into a managed hash-bound snapshot, publishes the normalized 16 kHz reference and 48 kHz audition through the shared asset journal, and persists audition plus selected/rejected identity. Voice creation requires explicit authorization and a completed audition, then publishes one immutable revision. VoxCPM2 consumes that revision and does not substitute a preset voice.
- Idle-worker admission verifies `runtime.health` against the current session and empty request slot, including real resident bytes from Mach task information or the Windows process working set. An unavailable or mismatched health result fails visibly. Memory pressure reclaims the idle worker before admission; interactive requests also reclaim an incompatible idle worker. In-flight stdio is never read by the health path. The existing single-worker reservation, priority fairness and 60-second idle TTL remain the lifecycle owners.
- Local text embedding runs in the application-managed Python worker through JSON-RPC 2.0 over private stdio. Rust owns resource verification, request identity, cancellation and index publication. The worker loads the verified Qwen snapshot offline, uses the selected CPU profile, and returns 1,024-dimensional normalized vectors. It cannot write the Knowledge database or LanceDB index.
- Ollama remains an optional **generative AI provider**. It is not required by media transcription or semantic vector search.

FFmpeg preparation remains in the media feature. Embedding, image segmentation, and speech inference use the single Rust-owned local runtime supervisor. None of these capabilities switch to a remote model after failure. Kokoro remains release-blocked: its model archive contains GPL eSpeak NG data, and the packaged `sherpa-onnx==1.13.8` Python binary enables TTS with GPL eSpeak NG and MIT piper-phonemize. Exact corresponding-source provenance for the data archive and complete notices for all model-package materials are not yet established. A release tool may verify those materials, but it cannot create a legal approval decision.

## Generative AI and privacy

Knowledge indexing runs locally using the private Qwen3 worker. Vector embeddings and extracted chunk texts do not leave the machine. Grounded answering sends only the explicit prompt plus bounded relevant text context when cloud generative AI is configured. Source files and media binaries are not uploaded by these paths.

Summary and Chapters are optional generative media insights. A transcript is persisted locally before any generative step, so transcription remains useful when Builtin Agent AI is disabled.

## Hardware guidance

- Preset speech and the smaller local models retain the ordinary supported-target acceptance matrix.
- VoxCPM2 admission is deliberately conservative: macOS arm64 uses MPS float32 only on systems with
  at least 48 GiB physical memory; Windows x64 uses CPU bfloat16 only with at least 32 GiB. Runtime
  admission also accounts for currently available memory and active model reservations.
  These thresholds are product safety gates, not measured performance claims.
- Discrete GPU: optional for Knovum's core knowledge runtime.
- Native latency, peak memory and output-quality acceptance remain required on both release targets.
