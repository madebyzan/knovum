# Local AI runtime and model distribution

Knovum separates bundled core media runtimes from on-demand capability resources. Core knowledge features do not require Ollama or another model server.

## Shipped with the signed application

Platform-specific runtime binaries belong to the application release and are never fetched from arbitrary URLs at runtime:

- FFmpeg
- a controlled CPython 3.11.13 runtime for local embedding, image segmentation, and speech inference

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

| Capability                       | Model                                                                                           | Ownership | Product action |
| -------------------------------- | ----------------------------------------------------------------------------------------------- | --------- | -------------- |
| Local text embedding             | `Qwen3-Embedding-0.6B` (Sentence Transformers)                                                  | Knovum    | Prepare on use |
| Multilingual media transcription | `mlx-community/whisper-small-mlx` on macOS arm64; `Systran/faster-whisper-small` on Windows x64 | Knovum    | Prepare on use |
| Preset speech synthesis          | `kokoro-int8-multi-lang-v1_0`                                                                   | Knovum    | Prepare on use |

The Apple Silicon ASR profile uses the ONNX Silero VAD model packaged with the exact-locked
`silero-vad==6.2.3` runtime dependency. It evaluates the prepared 16 kHz PCM audio on its original
timeline before MLX transcription; speech interval filtering does not compress timestamps.

Forced alignment and speaker diarization are separate optional speech stages. Neither is
required to complete plain ASR. The candidate Qwen3 Forced Aligner resource still needs a
verified complete file manifest and an on-demand caller; Pyannote Community-1 needs the user's
model access authorization. Video background separation has no qualified distributable weight.

Every local model file has a pinned revision, size and SHA-256. The embedding snapshot includes the Sentence Transformers module and last-token pooling configuration, not just weights and tokenizer. Installation uses a temporary file, streams real byte progress into the shared Task Center, verifies the checksum, and only then atomically activates the model.
Installed weights live under the current `local-resources/models/` application-data namespace; pre-release legacy paths are not read.

## Runtime ownership

- Rust verifies each platform's ASR files before starting the private Python worker. The request fixes engine, device, model specification, audio hash, owner, attempt, and deadline; no engine is silently selected after a failure.
- Local text embedding runs in the application-managed Python worker through JSON-RPC 2.0 over private stdio. Rust owns resource verification, request identity, cancellation and index publication. The worker loads the verified Qwen snapshot offline, uses the selected CPU profile, and returns 1,024-dimensional normalized vectors. It cannot write the Knowledge database or LanceDB index.
- Ollama remains an optional **generative AI provider**. It is not required by media transcription or semantic vector search.

FFmpeg preparation remains in the media feature. Embedding, image segmentation, and speech inference use the single Rust-owned local runtime supervisor. None of these capabilities switch to a remote model after failure. Kokoro's bundled espeak-ng-data has an unresolved GPL-3.0 distribution review; release packaging is blocked until the manifest records an approved decision.

## Generative AI and privacy

Knowledge indexing runs locally using the private Qwen3 worker. Vector embeddings and extracted chunk texts do not leave the machine. Grounded answering sends only the explicit prompt plus bounded relevant text context when cloud generative AI is configured. Source files and media binaries are not uploaded by these paths.

Summary and Chapters are optional generative media insights. A transcript is persisted locally before any generative step, so transcription remains useful when Assistant AI is disabled.

## Hardware guidance

- The 16 GB Apple Silicon and Windows CPU devices in the remediation plan are the required baseline acceptance devices for embedding and the planned speech capabilities. Their latency and memory targets still require measured results.
- Discrete GPU: optional for Knovum's core knowledge runtime.
- 24/32 GB memory is recommended when the user also runs a separate local generative LLM.
