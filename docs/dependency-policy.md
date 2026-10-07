# Dependency policy

## Cross-cutting foundation

| Concern                   | Selected implementation         |
| ------------------------- | ------------------------------- |
| UI/runtime                | Vue 3 + Vue Router              |
| State                     | Pinia                           |
| Async/server cache        | TanStack Vue Query              |
| Common Vue behavior       | VueUse                          |
| Runtime validation        | Zod                             |
| Accessible interactions   | Reka UI                         |
| Toast API                 | Vue Sonner                      |
| Icons                     | Lucide Vue                      |
| Styling                   | Tailwind CSS v4 + Knovum tokens |
| Lint                      | Oxlint                          |
| Format/import/class order | Oxfmt                           |
| Dependency boundaries     | dependency-cruiser              |
| Dead code/dependencies    | Knip                            |
| Duplicate code            | jscpd                           |

Oxfmt owns formatting, import ordering, package.json ordering and Tailwind v4 class sorting. There is no second formatter or linter stack.

Oxlint type-aware linting is intentionally not enabled while this foundation is on TypeScript 6. The current Oxc type-aware path requires TypeScript 7 tooling, so `vue-tsc` remains the strict type gate until the project moves to a compatible stable TypeScript/Oxlint combination. Do not add a parallel ESLint stack to fill that gap.

## Desktop foundation

Official Tauri plugins are registered only when a current product flow calls them. Package/plugin installation does not grant WebView permission; guest permissions stay minimal and are expanded only alongside the feature that needs them.

Current desktop capabilities include lifecycle/single-instance, macOS menu-bar status item / Windows Tray, window state, Store, clipboard, dialogs, filesystem, opener, OS information, notifications, updater and logging. SQLite persistence and embedded migrations are Rust-owned through SQLx; no SQL capability is exposed to the WebView. Provider networking is Rust-owned rather than exposed as a wildcard WebView HTTP capability. Plugins with no current caller are removed rather than pre-provisioned.

Cross-application dictation registers the official Tauri global-shortcut plugin only in Rust.
Its WebView capability exposes Knovum's session state/cancel/copy commands, not the plugin's
register/unregister API. CPAL and platform input bindings are macOS/Windows target dependencies;
microphone bytes stay in a bounded Rust session file and are deleted after recognition or cancel.
The OS adapter must revalidate the captured foreground process/window immediately before paste.
Accessibility/UIPI denial and focus change remain product-visible outcomes, never retry fallbacks.

Tauri remains the IPC transport. Application commands share a small error-normalization
boundary; official plugin APIs are imported directly. Existing ts-rs owns wire DTO
generation, including capability-owned video-editing requests and events. No parallel RPC
framework, handwritten global command map or custom signature generator is provisioned.

Desktop HTTP and filesystem access remain capability-scoped. SQLite access is Spaces-owned and Rust-only through typed Rust commands and SQLx transactions. The WebView has no SQL read or write permission.

Operating-system framework bindings are added only under a `cfg(target_os)` target dependency block, with non-default features narrowed to the classes the product flow actually names. macOS uses QuickLookThumbnailing for cached document thumbnails and QuickLookUI for the system preview panel; Windows uses the Shell image factory for installed thumbnail handlers. Each binding stays invisible to other platform builds. Adopting a binding is not granting a permission: native preview commands still re-authorize every path against the Space-owned asset directory.

Tauri Cargo-managed features must match the configuration on direct Cargo builds as well as CLI builds. The Rust 2.12.1 baseline no longer needs the `macos-private-api` feature or `macOSPrivateApi` configuration flag for transparent macOS windows. Both are removed; native window styling remains in the platform configuration files. `scripts/desktop/__tests__/tauri-config.test.mjs` checks feature/configuration agreement and each platform's native window contract.

The core Rust package, npm API and CLI must use a compatible Tauri release line before an upgrade is accepted. Generate both lockfiles with their package managers, run frozen installations and native builds, and retain version-mismatch checks. A Cargo-only upgrade is not a completed framework migration. Plugins have their own release versions; do not mechanically assign the core version to every plugin. The PR verification record identifies any outstanding lockfile or native-build work.

Linux verification provisions and checks the real `Noto Sans CJK SC` font from `fonts-noto-cjk`; its family is declared alongside the macOS and Windows families in `system_font.rs`. This supports document/media checks on Linux CI, not an additional supported installer target or a generic font fallback layer.

## Feature engines are selected only against real requirements

Spaces uses exact-pinned `konva` and `vue-konva`, confined to `features/spaces/canvas/konva`. Konva owns mounted scene interaction while neutral Knovum contracts own durable canvas facts; no parallel canvas engine is maintained. Spaces rich editing uses Tiptap Vue 3 + StarterKit plus narrowly selected first-party extensions. Do not preinstall additional Git, AI agent/SDK, CRDT or alternative canvas/editor engines before their requirements exist. Starting a feature begins with a fresh GitHub + npm/crates.io benchmark.

Tiptap packages are pinned to one release line as a unit; the current editor integration uses `3.31.4`. Do not mix Tiptap patch lines inside one editor surface.

Tiptap's Markdown extension is currently Beta, so its adoption is deliberately constrained: SQLite Markdown text remains the canonical body contract, only supported round-trippable Markdown structures are enabled, and Knovum does not persist vendor JSON as a parallel compatibility model. Tiptap's Notion-like template, Table of Contents UI and DOCX conversion are commercial/private-registry products and are not dependencies of Knovum's local OSS editor. Knovum derives its compact heading outline from the open editor document model. DOCX viewing fidelity, semantic conversion and editable round-tripping remain different capabilities and must not be treated as one dependency decision. Office ingestion now uses Rust `office_oxide` for plain-text/Markdown extraction so Space search and future RAG indexing do not depend on WebView rendering. Pixel-faithful Office preview remains a separate capability and is not approximated with a custom renderer. Editor search uses Tiptap's first-party Find and Replace extension rather than a parallel ProseMirror search package.

Direct dependencies must have a current caller or satisfy a documented peer/runtime requirement. Knip exceptions are reserved for tooling/runtime cases that static import analysis cannot see; they are not a holding area for speculative packages.

## Private local inference

The desktop package binds a controlled CPython 3.11.17 runtime and the uv-locked
production dependency selections in `python/runtimes/<profile>/uv.lock`. The root application
project has no production libraries; independently locked projects own capability dependencies. Rust invokes that interpreter over private stdio with one selected capability per
session; the WebView has no Python launch permission. Sentence Transformers and PyTorch own the
Qwen embedding model internals. ONNX Runtime Python owns U²-Net inference; Rust `image` still
owns pixel conversion and final output. Candle and Tract are removed when their last respective
callers move. Python libraries are imported only for the selected capability. Users download a verified frozen
runtime archive; dependency resolution and package installation occur only in the controlled producer.
The controlled producer uses the existing `uv==0.12.23` lock workflow and pinned
python-build-standalone `20261003` CPython 3.11.17 target artifacts recorded with their digests in
`python/runtime-source.json`; it does not copy a development virtual environment or infer a
runtime from uv's managed catalog. The temporary runtime-build group pins `setuptools==84.0.0`,
`wheel==0.48.0`, and `setuptools-scm==10.3.4` for required source builds, including VoxCPM, with
build isolation disabled. `python/compiled-source-evidence.json` pins the sherpa-onnx, eSpeak NG,
and piper source archives collected by the producer; this evidence does not claim that the
unversioned 2023 Kokoro data archive corresponds to an approved eSpeak source release.
The `python/uv.lock` quality group pins Ruff, Mypy and pytest separately from the packaged
runtime dependency set. `pnpm check:python` checks worker source, formatting, types and focused
boundary tests without installing model libraries; native capability acceptance still runs against
the complete target runtime.

Development tooling uses the existing Pydantic and NumPy pins rather than a second validation or
numeric implementation. The `acceptance` group adds
[`jiwer==4.0.0`](https://github.com/jitsi/jiwer) (Apache-2.0) for maintained CER/WER computation,
including empty-reference hallucinations. Fixed language normalization and micro-aggregation remain
Knovum's qualification policy; see [JiWER usage](https://jitsi.github.io/jiwer/usage/).
The `publication` group uses
[`cyclonedx-python-lib[json-validation]==11.12.0`](https://cyclonedx-python-library.readthedocs.io/en/stable/examples.html)
(Apache-2.0) and [`packageurl-python==0.17.6`](https://github.com/package-url/packageurl-python)
(MIT) for CycloneDX 1.6 serialization/schema validation and package URLs. The library receives the
actual `runtime/packages` distribution inventory, not the development environment or an inferred
lock-only SBOM. The generic CycloneDX environment CLI would inspect normal interpreter search paths;
Knovum's isolated worker adds `packages` through its Rust bootstrap. Keeping this explicit inventory
adapter preserves exact installed-file and source evidence without implementing another SBOM format.
These groups do not change the production runtime requirements, approve licenses or add dependencies
to end-user machines. `pnpm check:python` checks their source and tests alongside the worker.

The current runtime pins [`torch==2.14.1`](https://pypi.org/project/torch/2.14.1/),
`torchaudio==2.11.0`, and [`torchcodec==0.17.0`](https://pypi.org/project/torchcodec/0.17.0/).
PyTorch 2.13.0 was the first fixed release for
[`GHSA-rrmf-rvhw-rf47`](https://github.com/advisories/GHSA-rrmf-rvhw-rf47); newer pins must
preserve that security floor. The selected releases publish CPython 3.11 wheels for macOS arm64
and Windows x64. The macOS wheels require macOS 14 or later. TorchAudio 2.11 is its
[stable-ABI release](https://github.com/pytorch/audio/blob/main/docs/source/installation.rst)
and supports PyTorch 2.11 and later. TorchCodec's native ABI and complete model execution still
require acceptance on each release target; wheel availability is not execution evidence.
The macOS ASR path pins [`mlx==0.32.3`](https://pypi.org/project/mlx/0.32.3/), preserving the
0.29.4 unsafe-deserialization security floor. Contract generation pins
[`datamodel-code-generator==0.83.0`](https://pypi.org/project/datamodel-code-generator/0.83.0/),
preserving the 0.64.0 audited schema/template injection, local-file disclosure and SSRF fixes.
Do not lower these security floors or split the runtime into security-dependent environments.

The audit is still blocked by VoxCPM's official `datasets>=3,<4` requirement. The
[pinned VoxCPM source](https://github.com/OpenBMB/VoxCPM/blob/f0c787f0937dc1c9a8f4f64d9a332d9c5da2e629/pyproject.toml)
and its current upstream branch retain that bound, while
[`GHSA-379c-qx7v-6h59`](https://github.com/advisories/GHSA-379c-qx7v-6h59) is fixed only in
`datasets==5.0.1`. VoxCPM currently imports Datasets only from its
[training module](https://github.com/OpenBMB/VoxCPM/blob/f0c787f0937dc1c9a8f4f64d9a332d9c5da2e629/src/voxcpm/training/data.py),
but the published package metadata still makes it a mandatory distribution requirement. Knovum does
not delete that requirement, override upstream metadata, fork the package, or suppress the advisory
to make the audit appear clean. Signed distribution remains blocked until VoxCPM publishes a
compatible dependency range or Knovum replaces VoxCPM through a reviewed capability decision and
native acceptance run.

Native wheel availability, model license and actual execution are per-target release gates. A
locked Python version alone does not establish native execution. The product release targets
are macOS arm64/x64 and Windows x64. Windows arm64 remains outside the release scope. Intel macOS
has no qualified heavyweight dependency supplier yet. The unverified full-stack source builder has
been removed. Maintained native distributions and capability-specific engines must be evaluated against
the security baseline and the existing verified runtime publication contract. Supplier selection requires
actual native closure, provenance, relocation and model acceptance evidence; resolver/layout differences
alone do not justify rejecting a maintained distributor. See `packaging.md` for publication gates.

WhisperX 3.8.6 requires the older Torch 2.8 line, which conflicts with the selected secure
runtime baseline. Forced alignment uses the Apache-2.0 Qwen3 Forced Aligner snapshot pinned at
`c7cbfc2048c462b0d63a45797104fc9db3ad62b7` through the Transformers 5.19.0 API. Its complete
verified manifest is prepared only by a selected-segment enrichment job and it is never an ASR
fallback. Pyannote Community-1 requires individual gated model access. Knovum downloads exactly four files after official account authorization from revision
`3533c8cf8e369892e6b79ff1bf80f7b0286a54ee`, verifies their manifest, and supplies its own fixed
pipeline configuration. The access token stays in the Rust-owned secrets database and authenticated download transport; no worker credential, external YAML or unauthorized mirror is used.
The plan's Demucs htdemucs code license does not grant commercial use of its official pretrained
weights, so it is not the production baseline. Knovum instead selects sherpa-onnx 1.13.8 with the
two-stem Spleeter FP16 ONNX snapshot pinned at
`93ba771920ade509f8cbd6825b1a90856c797e08`. The [Spleeter project](https://github.com/deezer/spleeter)
and its [JOSS paper](https://joss.theoj.org/papers/10.21105/joss.02154) state that the code and
pretrained models are MIT licensed. This is a vocals/accompaniment model, so every derived background
requires user audition and explicit acceptance. Release qualification must measure spoken-dialogue
residual and background-sound loss on the fixed video corpus; implementation does not imply that
quality gate has passed.

Kokoro remains blocked for signed distribution. Its model package carries GPL eSpeak NG data, while
the locked `sherpa-onnx==1.13.8` native Python wheel enables TTS by compiling GPL eSpeak NG with MIT
piper-phonemize despite the sherpa source package's Apache-2.0 declaration. Upstream tracks this
conflict in [issue 3731](https://github.com/k2-fsa/sherpa-onnx/issues/3731). Release materials must
bind the exact platform wheels to the sherpa source, build information, pinned eSpeak and piper
sources, their license texts, and the exact source for the distributed `espeak-ng-data`. The current
2023 data archive has no embedded source/version notice and is not proven to correspond to the 2026
eSpeak source used by sherpa 1.13.8. Model, voice, lexicon, cppjieba dictionary and Chinese FST
provenance/notices also remain incomplete. Tooling must keep this review blocked until a maintainer
records an explicit scoped legal decision; it must never infer approval from populated hashes.
VoxCPM2 code and the pinned OpenBMB model revision both declare Apache-2.0, so they do not inherit
the Kokoro/eSpeak release blocker. The model is an on-demand 4,960,722,408-byte resource rather
than a packaged runtime payload. Release acceptance must prove the exact snapshot on MPS macOS
arm64 and CPU Windows x64 and must include a disk-capacity failure path; it must not enable
automatic device selection, optimization, denoising, Hub downloads or a synthesis fallback.

If a future feature needs Shell, WebSocket, updater, notifications, process control, custom window positioning or persisted external file scopes, add the corresponding official Tauri plugin in the same change that defines its capability and product behavior.

## Supply-chain release gates

One narrowly scoped pnpm override addresses the current dependency graph:

- `tauri-plugin-clipboard-api@2.1.11 > valibot` uses `1.4.2`. The plugin uses primitive parsing APIs, verified against the actual plugin's valid and malformed clipboard payloads. Remove the override when the plugin itself selects a supported secure Valibot release and the regression passes.

PptxGenJS and its image-size override are removed: the product presentation capability is Rust-owned, and the former Web worker had no production caller. Do not add a second document serialization engine in the Renderer.

Rust `reqwest` explicitly enables `query`, because provider clients use `RequestBuilder::query`. Do not rely on unrelated SDK dependencies to enable required features transitively. `async-openai`, direct `rig-core` and direct `futures-util` have no current Knovum caller and are not direct dependencies. The Rig harness and its dependencies have been removed.

- CI installs from the committed pnpm/Cargo lockfiles only.
- Project `.npmrc` selects the official npm registry for installation; registry mirror lag must not be repaired with transitive version overrides. WebdriverIO CLI/runner/framework/reporter use exact 9.30.1 and globals uses exact 9.29.1, matching the Tauri service 1.4.0 SDK family. Newer CLI/runner releases use expect-webdriverio 6, conflicting with the service's pinned globals requiring 5; upgrade the family together when the service updates its pins.
- Production npm dependencies are checked with `pnpm audit --prod --audit-level high`; registry package signatures are checked with `pnpm audit signatures`.
- Rust advisories and dependency licenses are checked in CI with `EmbarkStudios/cargo-deny-action@v2`; `deny.toml` is the explicit license allowlist and changes to it require release/legal review.
- Dependabot remains the scheduled update signal; passing advisory checks does not replace the OSS-first maintenance/license review for newly introduced dependencies.

### Dependency refresh reviewed 2026-10-07

The npm, Cargo and both Python dependency selections were refreshed together from official
registry metadata. Exact pins remain exact; each lockfile is generated by its owning package
manager. The Rust MSRV is 1.94 because SQLx 0.9 requires it. SQLx dynamic queries use its public
`QueryBuilder`; user values remain bound. ACP 3 explicitly enables its `process` feature and
RMCP 3.5 uses `ClientConfig` / `ServerConfig`. SHA-2 0.11 returns byte arrays without hexadecimal
formatting; the existing transitive MIT/Apache-2.0 `hex` 0.4.3 crate is now direct where production
calls `hex::encode`. Digest algorithms and persisted lowercase hexadecimal values remain unchanged.
No custom encoding layer or locally patched SDK is added.

[pnpm 11 configuration](https://pnpm.io/settings) reads only registry/authentication options
from `.npmrc`. `engineStrict`, `strictPeerDependencies` and `saveExact` live in
`pnpm-workspace.yaml` so these existing policies continue to apply after the package-manager
upgrade. Peer conflicts are resolved through compatible releases, not ignored or patched.

| Dependency family | Accepted selection | Reason a newer release is retained outside the selection                                                                                                                                                                           |
| ----------------- | ------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| TypeScript        | 6.0.3              | [TypeScript 7](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/) does not ship the compiler API required by current vue-tsc.                                                                                   |
| WebdriverIO       | 9.29.1 / 9.30.1    | Tauri service 1.4.0 pins globals 9.29.1 with expect-webdriverio 5. CLI, runner and adapters use compatible 9.30.1 releases; newer releases require expect-webdriverio 6 or WebdriverIO 10. Strict peer checks pass.                |
| LanceDB           | Exact 0.37.1       | 0.39 fails compilation with the local feature set: job.rs uses `Error::Http` while error.rs gates it behind `remote`. Inspection confirms the same defect in 0.38. No remote feature or vendor patch is added to a local database. |
| Arrow             | 58.4.0             | LanceDB's public Arrow types require major 58; major 60 cannot be mixed at that boundary.                                                                                                                                          |
| sse-stream        | 0.2.6              | RMCP 3.5.1 exposes the 0.2 SSE type in its public client trait; 0.3 changes that type.                                                                                                                                             |
| huggingface-hub   | 1.33.0             | Sentence Transformers 6.1.0 requires `<2`; uv rejects 2.1.1.                                                                                                                                                                       |
| NumPy / PyAV      | 2.4.6 / 18.1.0     | Newer stable releases require CPython 3.12; the controlled runtime is CPython 3.11.17.                                                                                                                                             |

VoxCPM is pinned to upstream commit `f0c787f0937dc1c9a8f4f64d9a332d9c5da2e629`.
Its new streaming `seed` parameter receives the selected seed directly; a preceding
`torch.manual_seed` no longer controls the SDK's own seeding. The regression verifies this
handoff. CPAL 0.18 uses a numeric sample rate, owned stream configuration and typed error kinds;
permission classification no longer searches localized error messages.

The remote resource request boxes its HTTP transport future at the acquisition boundary.
This keeps Reqwest's concrete transport type out of ACP's nested session futures while
preserving cancellation and resource validation; the crate's recursion limit is unchanged.

Knip 6's stricter export analysis removes unused handwritten exports and unreachable functions.
Generated Rust-owned IPC DTO types are excluded only from the unused-type rule because native
and Python consumers are outside Knip's TypeScript graph. Generated file/dependency analysis and
all handwritten export/type checks remain enabled.

Desktop installer builds, hardware microphone capture and full inference model acceptance are
outside this dependency-editing verification. Changed runtime inventories must receive fresh
native acceptance and license review before signed distribution; existing blocked reviews remain
blocked.

### Rust security patch and audit scope

The 2026-10-02 lock refresh upgrades `h2` to `0.4.16` and `rustls` to `0.23.45`, addressing `RUSTSEC-2026-0258` and `RUSTSEC-2026-0285`. It also replaces the yanked `chacha20 0.10.1` with `0.10.2`. These versions are a recorded remediation, not a permanent exemption from later advisories.

`citationberg 0.7.0` is temporarily patched to official upstream commit `06a591e2f237d25e1dfdedac3f3d1494c496c52d` from [upstream PR #44](https://github.com/typst/citationberg/pull/44). That merged change selects `quick-xml 0.41`, eliminating the remaining `0.38.3` instance and addressing `RUSTSEC-2026-0194` and `RUSTSEC-2026-0195`. This is an immutable upstream revision, not a moving branch or locally modified dependency. Remove the patch when a compatible published release includes the fix, regenerate the lockfile with Cargo, verify no vulnerable XML instance remains, and rerun native checks.

The patch was tested in [dependency preparation run 36966374084](https://github.com/madebyzan/knovum/actions/runs/36966374084): all 12 upstream unit tests and one documentation test passed with no ignored or filtered tests. Upstream's full roundtrip tests require sibling `citation-style-language/styles` and `citation-style-language/locales` checkouts; the recorded fixture revisions are `8d129d151adfe68256ceb6e6ea59c6f651ab526a` and `bc0a222b4ca526126faf892c35b2b7b1215c10eb`, respectively. This upstream result does not substitute for Knovum's native compile, tests or installed upgrade acceptance.

The audit graph includes all features across macOS arm64/x64, Windows x64, and the Linux x64 CI target. This evaluates the supported application configurations rather than unrelated custom `cfg(fuzzing)` dependencies such as rav1e's `libfuzzer-sys`. Adding a target or enabling a fuzzing configuration requires an appropriate audit scope for that configuration. No third-party license has been added to the allowlist for this change.

Knovum is an unpublished application crate (`publish = false`), not a public Rust library with an assumed OSS license. The license check exempts only unpublished workspace packages from declaring such a license; their third-party dependencies remain audited. Do not use `exclude-unpublished` to remove the application's dependency graph. Maintenance advisories remain enforced; this remediation adds no advisory-ignore entries and does not represent release approval.

The Tauri Rust upgrade selects `tauri-utils 2.10.1` and `urlpattern 0.6.0`, removing `unic-char-range`, `unic-common`, `unic-char-property`, `unic-ucd-version` and `unic-ucd-ident` through a published upstream migration. The unrelated `unic-langid` family remains in use. Do not classify crates by a shared name prefix when checking a specific advisory's removal.

Remaining maintenance advisories are tracked in [issue #22](https://github.com/madebyzan/knovum/issues/22). A maintenance notice is not itself proof of an exploitable vulnerability, and a maintained replacement is not automatically behavior-compatible. Prefer a tested parent-package migration. Any temporary acceptance must identify the exact advisory, dependency version/path, exposure review, approving release owner, review date and removal condition. It is not implied by opening an issue or approving unrelated code. Do not add broad suppression, rename-only replacement crates or unreviewed native rendering changes to make an audit green.

The 2026-10-04 cargo-deny 0.20.2 check still rejects six maintenance advisories:
`bincode 1.3.3`, `paste 1.0.15`, `proc-macro-error 1.0.4`, `rustybuzz 0.20.1`,
`ttf-parser 0.25.1`, and `yaml-rust 0.4.5`. It also rejects the unapproved
`bzip2-1.0.6` license of `libbz2-rs-sys 0.2.5`, reached through `bzip2 0.6.1`.
That decoder has a real caller in the pinned Kokoro archive installer. The license
allowlist remains unchanged pending the required scoped release/legal review.
Local compile and regression success do not close either audit gate.

### Python installed-distribution license release gate

Each `python/runtimes/<profile>/dependency-license-review.json` starts with `reviewStatus: required`. Production
packaging refuses it until an accountable review approves the frozen runtime requirements SHA-256 and VoxCPM source revision
and each target's exact `build-evidence/dependency-evidence.json` SHA-256. Target records
contain `inventorySha256`, `reviewer`, `reviewedAt` and a durable HTTPS `evidenceLocation`.
The evidence must cover the interpreter and its contained native libraries as well as
every actually installed distribution, its notices, native code
and corresponding-source obligations; package metadata alone is not approval. Changed runtime dependencies, source revision or conveyed inventory invalidate the review;
unrelated development dependencies and application Worker source do not. This gate complements the separately
owned Kokoro and native-source gates; QA artifacts do not represent license approval.

Model resource downloads use `python/runtimes/resources/pyproject.toml`: official
huggingface-hub 1.33.0 (Apache-2.0), modelscope-hub 0.4.5 (Apache-2.0), and the already selected
Pydantic 2.13.5 plus tqdm 4.70.1 for its public progress interface. They belong to the prepared runtime, not the desktop installer. Their public
Hub APIs own resume/transfer metadata; Rust owns product inventory, SHA-256 and publication.
Provider commits can differ only when every required file matches the same recorded identity.
Regional source qualification and the current acceptance gaps are recorded in `oss-first.md`.

## Builtin Agent dependencies

`python/runtimes/agent/uv.lock` owns the dependency-only Agent Runtime. Exact direct pins
are in its `pyproject.toml`; this selection is independent of `python/uv.lock`. Deep Agents and
LangGraph own graph execution through public APIs, and the official LangChain clients own HTTP
protocols. Rust does not duplicate those clients. `truststore` is initialized only at application
startup before SDK clients, using the native OS trust store. Credentials/telemetry configuration
are not inherited through process environment. Production dependencies and native wheels for both
release targets require the same inventory/SBOM/license/relocation gates as inference. The
Builtin Agent review policy lives beside its own lock and must bind each conveyed platform inventory.

## Agent extension dependency record

| Dependency            | Exact version | Responsibility                             | License           |
| --------------------- | ------------- | ------------------------------------------ | ----------------- |
| rmcp                  | 3.5.1         | Existing server + official client/OAuth    | Apache-2.0        |
| gix                   | 0.88.0        | HTTPS isolated Git object acquisition      | MIT OR Apache-2.0 |
| yaml-rust2            | 0.13.0        | Skill frontmatter, duplicate-key rejection | MIT OR Apache-2.0 |
| iri-string            | 0.7.14        | RFC 6570 URI template expansion            | MIT OR Apache-2.0 |
| unicode-normalization | 0.1.25        | NFC portable package identity              | MIT OR Apache-2.0 |
| oauth2                | 5.0.0         | rmcp OAuth HTTP public request types       | MIT OR Apache-2.0 |
| sse-stream            | 0.2.6         | Bounded MCP HTTP event parsing             | MIT OR Apache-2.0 |
| async-trait           | 0.1.92        | SDK CredentialStore implementation         | MIT OR Apache-2.0 |
| url                   | 2.5.8         | Explicit origin/destination validation     | MIT OR Apache-2.0 |
| tokio-util            | 0.7.19        | Cancellation and public SDK framing        | MIT               |

Existing transitive dependencies become direct only where production code imports their public
APIs. Git configuration remains isolated and has no checkout/submodule/credential-helper path.
rmcp client features do not add a second protocol implementation. Frontend controls use existing
Vue, TanStack, Zod, Tauri dialogs and product UI primitives. No frontend plugin framework or
Python MCP dependency is added. Registry crate licenses and target feature graphs must be
reviewed from the refreshed lockfile/SBOM before release.

Pinned Agent Plugins 1.0.0 validation schemas (upstream agent-plugins.org):

- `plugin.schema.json`: SHA-256 `0a4aad95ce337878ad38802ebf0daa3fde76abe3f65400c86bcbb1ec0b3ab883`.
- `mcp.schema.json`: SHA-256 `6539175bfcdf43085855183e86da40ea94b166547a72b47ae9a0a390516d3acb`.
