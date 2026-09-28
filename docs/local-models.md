# Local AI runtime and model distribution

Knovum separates bundled core media runtimes from on-demand capability resources. Core knowledge features do not require Ollama or another model server.

## Shipped with the signed application

Platform-specific runtime binaries belong to the application release and are never fetched from arbitrary URLs at runtime:

- FFmpeg
- whisper.cpp `whisper-cli`

Development and CI point to explicit runtime paths through the documented environment variables. Release builds package the controlled binaries from `src-tauri/resources/bundled-runtime/<platform>/` into the single application-resource location `resources/bundled-runtime/<platform>/`.

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

| Capability                       | Model                                           | Ownership | Product action              |
| -------------------------------- | ----------------------------------------------- | --------- | --------------------------- |
| Local text embedding             | `Qwen3-Embedding-0.6B` (`fastembed` + `candle`) | Knovum    | Prepare on use              |
| Multilingual media transcription | `ggml-small-q8_0.bin`                           | Knovum    | Prepare on use              |
| Speech-region detection          | `ggml-silero-v6.2.0.bin`                        | Knovum    | Prepared with transcription |

Every local model descriptor has a pinned source and SHA-256. Installation uses a temporary file, streams real byte progress into the shared Task Center, verifies the checksum, and only then atomically activates the model.
Installed weights live under the current `local-resources/models/` application-data namespace; pre-release legacy paths are not read.

## Runtime ownership

- Whisper's built-in Silero VAD skips silence while one `whisper-cli` process owns the complete media transcription, so the model is loaded once and timestamps remain on the original timeline.
- Local text embedding runs entirely in-process using `Qwen3-Embedding-0.6B` via Candle and fastembed, producing 1,024-dimensional normalized vectors. No cloud API or API keys are required for indexing and search.
- Ollama remains an optional **generative AI provider**. It is not required by media transcription or semantic vector search.

Whisper, FFmpeg preparation, and local text embedding run as feature-owned operations with explicit cancellation. There is no remote fallback path.

## Generative AI and privacy

Knowledge indexing runs entirely locally on device using the in-process Qwen3 embedding model. Vector embeddings and extracted chunk texts never leave the machine. Grounded answering sends only the explicit prompt plus bounded relevant text context when cloud generative AI is configured. Source files and media binaries are not uploaded by these paths.

Summary and Chapters are optional generative media insights. A transcript is persisted locally before any generative step, so transcription remains useful when Assistant AI is disabled.

## Hardware guidance

- Minimum usable: Apple M1 or modern six-core Windows CPU, 8 GB RAM, SSD.
- Recommended: Apple Silicon with 16 GB unified memory, or a modern Core i5/Ryzen 5 class Windows machine with 16 GB RAM and SSD.
- Discrete GPU: optional for Knovum's core knowledge runtime.
- 24/32 GB memory is recommended when the user also runs a separate local generative LLM.
