# Native inference implementation plan

> **For agentic workers:** Use `executing-plans` task by task. User explicitly requires one agent and has authorized continuous execution without further process reviews.

**Goal:** Move every non-Agent Knovum Python capability and its development/runtime supply chain into Rust and the existing Node tooling.

**Architecture:** `local_resources` owns acquisition and immutable installation. A private Rust worker owns native inference; a neutral Rust contracts package joins it to the existing host supervisor. Existing business owners retain authorization and publication.

**Tech stack:** Rust, serde, Tokio, reqwest, ort, maintained native speech/embedding engines, existing Node/Oxc scripts.

**Spec:** `docs/superpowers/specs/2026-10-11-native-inference-design.md`.

## Global constraints

- Single agent; no delegation, compatibility readers, fallback engines or speculative dependencies.
- Preserve all 22 catalog model entries and macOS arm64/x64 + Windows x64 contracts.
- Native crashes and hard cancellation remain isolated; capacity stays held through process exit.
- Existing locked model identities and processing establish quality baselines; native execution receives new identities where implementation changes.
- Build serially in the checkout's normal target directory using default profiles. Never clean a target used by the running desktop.
- Generated DTOs have one Rust owner; source-layout, Clippy, fmt, tests and duplication checks include new workspace packages.

## Review focus

- Interrupted or changed server range response: no corrupt append or publication.
- Native inference ignoring cancellation: kill owned process tree and reject late artifacts.
- SAM session reuse with a changed rendered source: recompute encoding and reject old responses.
- TTS trailing frames and delayed ACK: preserve tail and bounded chunk backpressure.
- Engine library/version mismatch and non-ASCII paths: fail visibly before computation.

Implementation checkboxes record code and focused verification. Unchecked model/gate items await the final signed package and fresh full checks; platform/manual release qualifications are recorded separately in `docs/validation/2026-10-11-native-inference.md`.

## Tasks

### 1. Native resource download

Files: `src-tauri/src/local_resources/{download,models,preparation}`, obsolete resources worker/schema/tool references.
Interface: existing `download::to_path(RequestBuilder, Path, DownloadPolicy, CancellationToken, progress)` gains a resumable entry point; `models::download::snapshot` keeps its public signature and no longer spawns Python.

- [x] Test fixed HF/ModelScope URLs against pinned source identities and encoded paths.
- [x] Test range parsing, invalid offsets/lengths, restart responses, cancellation and bounded transfer with a local HTTP fixture.
- [x] Implement strict resumed streaming in the existing download owner and fixed-source URL resolution; preserve final installer SHA verification.
- [x] Remove resources runtime from model preparation and delete its worker, dependency profile and schema callers.
- [x] Run focused Rust tests and script/contracts checks; update evidence and owning documents.

### 2. Shared native wire and worker lifecycle

Files: `src-tauri/crates/inference-contracts`, `src-tauri/crates/inference-worker`, `src-tauri/{Cargo.toml,Cargo.lock}`, host `ai/contracts`, supervisor/process, native resource manifest/producer.
Interface: strict shared runtime/request/event DTOs; worker stdin/stdout keeps bounded JSON-RPC request/notification framing. Native runtime paths carry verified executable/library inventory and leases instead of interpreter/packages.

- [x] Verify strict shared contract tests for unknown fields, identity/epoch, tensor bounds and native-only identities.
- [x] Move existing cross-process DTOs, keeping host-only domain types with the host; regenerate TS.
- [x] Test malformed/oversized frames, concurrent-request rejection, active cancellation, shutdown and child crash.
- [x] Implement reader/compute separation, bounded output, cancellation/ACK state and request directory validation.
- [x] Bind native worker startup to existing host admission, health, process-tree termination and idle reuse.
- [x] Extend normal workspace checks and verify focused contract/lifecycle tests.

### 3. Complete image inference

Files: worker `images/{segmentation,upscaling,inpainting,selection,depth}`, artifact boundary; existing host image execution/resources; remove corresponding Python adapters/tests.
Interface: current `ImageSegmentParams` and `ImageComputeParams` → validated tensor artifacts using the shared native DTOs.

- [x] Port meaningful tensor boundary, mask polarity, tile/rounding, SAM cache and depth tests before adapters.
- [x] Use admitted ORT sessions with strict graph IO and cancellable run options; preserve model-specific transforms and byte rules.
- [x] Run all nine actual model entries against local verified snapshots, comparing baseline tensors and cancellation.
- [x] Switch all image caller/preparation paths and remove their Python runtime use.

### 4. Qwen embedding and new native identity

Files: worker `embedding`, host embedding capability/spec/index consumers, model device admission.
Interface: `EmbeddingEncodeParams` → `EmbeddingEncoding`; role processing/2048 tokens remain explicit.

- [x] Verify maintained model library API with local safetensors/tokenizer and CPU F32.
- [x] Test role prompt, padding/last token, normalization, finite 1024 dimensions and oversize rejection before production adapter.
- [x] Create the Rust-owned execution identity; explicitly reject stale indexes through existing strict ownership.
- [x] Compare actual vectors and new-index retrieval quality; switch callers and remove Torch embedding.

### 5. Native speech engines

Files: worker `speech/{audio,alignment,diarization,separation,synthesis,transcription}`, existing voice ABI/producer; host profiles/resources/catalog/provenance.
Interface: shared speech requests, progress/chunk events and artifact responses consumed by existing speech publication.

- [x] Qualify VoxCPM2/Q8 ABI, high-precision aligner and MOSS candidate before selecting final library paths.
- [x] Test owned FFI lifetime, errors, cancellation, audio timelines and TTS backpressure/tail.
- [x] Implement VoxCPM2 and Q8; validate high-precision resources without dropping that option.
- [x] Use official Sherpa APIs for diarization/separation; retain count and sample-length contracts.
- [x] Integrate qualified MOSS library or minimal demonstrated host gap; preserve ten voices/seed/codec behavior.
- [x] Qualify whisper.cpp model assets for all sizes; replace MLX/faster-whisper with native profiles and original timeline VAD.
- [x] Switch business callers, validate results/provenance and remove obsolete Python adapters/profiles.

### 6. Supply chain, tooling and Agent-only Python

Files: `scripts/{runtime,contracts,checks,release}`, `python/{runtime-source.json,pyproject.toml,conda-locks,runtimes}`, build support, CI and acceptance tooling.

- [x] Port non-Agent producer operations into existing Node owners, consuming maintained archive/provenance primitives.
- [x] Separate general Rust/TS export from Agent Python model generation.
- [x] Move model execution to Rust and offline acceptance metrics to the existing Node tool owner, using strict corpus/provenance and real worker captures (qualified library decision recorded in the spec).
- [x] Delete inference/resources Python packages, schemas, profiles, tensor supplier and obsolete production matrix/state fields.
- [x] Verify ordinary non-Agent desktop dev requires no Python preparation; run retained Agent recovery tests.

### 7. Final review and evidence

Files: authoritative architecture/runtime/model/dependency/packaging/release/platform docs and completion evidence.

- [x] Run `pnpm check:scripts`, `pnpm check:web`, workspace `pnpm check:rust`, contracts reproducibility and retained Agent checks.
- [x] Review the complete changes myself against spec, security boundaries and actual caller inventory.
- [x] Complete real model tests and record unavailable target/manual-review evidence. Desktop packaging was stopped at the user's explicit request; no final app build, bundle signature or desktop E2E pass is claimed.
- [x] Search all executable source/config/tool/CI paths for obsolete non-Agent Python dependencies; fix residual callers.
- [x] Deliver implemented changes and exact verification/qualification gaps; preserve the branch locally and distinguish completed code migration from release qualification.
