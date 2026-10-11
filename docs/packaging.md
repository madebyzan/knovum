# Packaging

Packaging creates desktop application/install artifacts. It is distinct from production release signing and updater publication.

The desktop entry point links Knovum's internal Rust library as an `rlib`; the
`[lib]` target follows this topology. Tauri produces the desktop executable and
application/install artifacts.

The project author and bundle publisher are Zan. Keep `package.json`'s `author`,
`src-tauri/Cargo.toml`'s `authors`, and `src-tauri/tauri.conf.json`'s `bundle.publisher`
consistent with this attribution.

## Local commands

Compile development code without bundling:

```bash
pnpm exec tauri build --no-bundle --debug
```

Daily development and tests use Cargo's `dev`/`test` profiles with application line-table
debug information, no dependency debug information, and packed debug information. Incremental
compilation remains enabled. This keeps useful application panic locations while avoiding
macOS's default full-debug, unpacked object-file accumulation. Release profiles are unchanged.
The profile split follows [Cargo's build-performance guidance](https://doc.rust-lang.org/cargo/guide/build-performance.html#reduce-amount-of-generated-debug-information).
Packed symbols use `.dSYM` on macOS and `.pdb` on Windows MSVC; macOS adds a `dsymutil` step
at link time. Measure whole-project disk usage and link time rather than treating symbol packing
alone as a guaranteed size reduction.

For deliberate Rust breakpoint and variable debugging, use the separate full-debug profile:

```bash
pnpm exec tauri dev -- --profile debugging
cargo test --locked --manifest-path src-tauri/Cargo.toml --profile debugging --lib
```

The `debugging` profile has its own `src-tauri/target/debugging` artifacts and includes dependency
debug information. It is not the profile for routine verification. Changing an existing profile
requires rebuilding its affected dependencies; old artifacts are not automatically reclaimed.

After all Cargo/Tauri development processes using the checkout have stopped, reclaim old
Knovum-owned artifacts while retaining dependency caches:

```bash
cargo clean --manifest-path src-tauri/Cargo.toml -p knovum
```

For the one-time transition from older full-debug builds, a complete
`cargo clean --manifest-path src-tauri/Cargo.toml` also removes obsolete dependency artifacts,
but the next build must rebuild dependencies. Do not clean a target used by a running build.
Cargo does not automatically prune target artifacts. Keep verification builds on the checkout's
normal target; serialize conflicting builds instead of copying targets to bypass Cargo's lock.

The web unit suite uses four Vitest workers to bound compiler and subprocess pressure; its
assertions and normal test deadlines remain unchanged. The existing E2E build uses
`com.zand.knovum.e2e` as a dedicated application identifier, keeping test databases and preferences
separate from the product's user data.

Build the platform's default configured bundles:

```bash
pnpm package
```

macOS direct-distribution artifacts:

```bash
pnpm package --bundles app,dmg
```

This builds the `.app` and `.dmg` bundles. Run it on macOS.

Windows installer artifacts:

```bash
pnpm package --bundles nsis
```

This builds the NSIS `setup.exe`. Run it on Windows. `src-tauri/tauri.windows.conf.json` explicitly disables installer downgrades, keeps NSIS per-user/no-admin by default, and ships English + Simplified Chinese installer localization.

MSI is an explicit stable/enterprise artifact:

```bash
pnpm package --bundles msi
```

WiX/MSI requires a numeric `major.minor.patch[.build]` installer version, so prerelease SemVer such as `1.0.0-rc.x` must not use the MSI path. Run MSI packaging only for a release version that satisfies that rule. MSI creation relies on the Windows WiX toolchain and may require the Windows VBSCRIPT optional feature. The platform config pins a stable WiX upgrade code so product renaming cannot accidentally create duplicate MSI products.

## Distribution boundaries

The installer contains FFmpeg, the Rust inference worker, its compiled library/notice closure, the
current Deep Agent `.py` source and exactly one `python-agent-descriptor.json`. The Agent interpreter
and its third-party packages remain an independently published on-demand resource. The installer
never includes Torch, MLX, inference Python packages or a model-download Python worker.

`inference-contracts` owns neutral wire DTOs. Rust exports TypeScript and Rust inference JSON
Schema; Python generation consumes only Agent contracts. Obsolete pre-release request, index and
runtime identities fail strictly, with no compatibility reader or alternate engine.

### Rust inference package

`node scripts/runtime/inference/preparation.mjs` builds the controlled library closure for the actual host target.
Source archives, public ONNX headers, source revisions, sizes and hashes are fixed in the four
`scripts/runtime/inference/*-source.json` supplier records. ONNX Runtime 1.22.1's official wheel is an archive supplier:
only its C library and notices are extracted; no interpreter, Python binding or package is installed.
CMake 4.3.4 builds frozen audio.cpp, Sherpa-ONNX 1.13.8 and whisper.cpp 1.8.3. Sherpa TTS/eSpeak,
Python, PortAudio and server examples are disabled. The callback bridge changes no vendor source.

Audio.cpp's CPU dispatch modules and optional Metal/Vulkan modules live in `bin/`. Whisper has a
separate private GGML closure under `compiled-libraries/whisper` so incompatible supplier GGML
versions cannot share a loader namespace. Installed aliases are materialized as regular files.
Whisper uses upstream CPU dispatch modules, selected from that sealed directory through the public
GGML loader; it preserves portable baseline support while using the host's available instructions.
macOS library references are relocated to verified `@loader_path` dependencies, then signed.
Windows worker DLL search is restricted to System32 and the verified runtime root/bin; ambient
PATH, current directories and `GGML_BACKEND_PATH` cannot authorize a backend.
The host removes inherited and command-local `DYLD_*`, `LD_*`, `GGML_*` and `VK_*` overrides
before worker execution or backend probing, alongside Python path settings and provider tokens.

`pnpm package` builds the Rust worker, copies the verified closure into
`src-tauri/resources/inference-runtime`, signs its compiled resources, preserves verified Microsoft
runtime signatures, and seals the final file inventory.
The worker lives at `bin/knovum-inference-worker[.exe]` inside that inventory. Host compilation embeds
this exact manifest. At runtime, the host checks the manifest against its embedded trusted copy,
rejects extra files/links/reparse directories, checks every hash, then starts the verified executable.
Worker initialization repeats the bounded inventory check before loading compiled libraries. The packaged
manifest does not authorize itself. Final package verification detects any bundler-induced changes
instead of relaxing hashes after signing.

Production uses the application's real Developer ID/Authenticode identity for Knovum resources;
Microsoft runtime files retain their verified vendor signatures. QA uses ad-hoc
macOS signing and unsigned Windows output. Signature verification invokes native OS tools directly
from Node. Rust build inputs, notices and signing receipts accompany the sealed package. Exact
license inventory and target execution evidence remain required before release promotion.

### Retained Deep Agent publication

Only `python/runtimes/agent/uv.lock` and the base Conda-forge CPython 3.11.17 locks remain. Agent
archives contain an independent interpreter/package root, notices, requirements and strict v1
runtime manifest. Worker source remains part of the application; its digest covers the entire
current Agent source tree. Agent environments load no local inference or model-download code.
Agent binary inventory and signing reuse `scripts/runtime/compiled-signing.mjs`, the same OS-native
implementation used by other compiled resources. Local preparation still provisions its managed
uv/micromamba tools automatically; development and packaging commands are unchanged.

```bash
pnpm runtime:build --profile agent --output <fresh-directory> --signing <adhoc|unsigned|production> \
  --archive-url <immutable-https-url>
pnpm runtime:provision --tag runtime-<commit>-<run>-<attempt> \
  --repository <public-distribution-owner/repo> --source-repository <private-source-owner/repo>
```

`python-runtime.yml` produces only Agent on all three OS runners and attests the interpreter
archive, evidence archive and descriptor. It publishes these assets to the initialized public
distribution repository, keeping private source separate. QA and release builds select an explicit
immutable tag. `gh release download` fetches its three target assets; `gh attestation verify` checks
the source repository, owning workflow, `main` ref and exact source commit before any extraction.
Current recipes, requirements, manifests and inventories must still match the checkout.
No custom publication catalog, latest-runtime selection or compatibility reader remains.

CI uses its read-only source-repository `GH_TOKEN` for public asset retrieval and private
attestations. QA needs no distribution write token. Local maintainers need `gh` authenticated for
the same read access. Application clients download the public archive anonymously; they do not need
`gh`, source access or any token. Only the descriptor and owned Worker ship in the installer.

Agent installation uses existing bounded streaming, Task Center progress, complete verification,
atomic activation and OS file leases. Its namespace is
`local-resources/runtimes/python/agent/<target>/<archive-sha256>`. Inference development publications
use `.runtime-development/inference/<target>/<recipe-sha256>` and separate lease anchors. Active
workers retain leases; stale identities are reclaimed only when unleased. Cargo uses the checkout's
ordinary target and default profiles; shared-target builds are serialized.

Completed audio.cpp, Sherpa and Whisper supplier libraries are cached independently under
`.runtime-development/inference-suppliers/<target>/<supplier>/<recipe-sha256>`. Each completed
supplier has a verified file inventory and is activated atomically. A later supplier failure
discards only that supplier's incomplete build; the next preparation reuses earlier verified
suppliers. The final inference bundle is still assembled, relocated, signed and verified as a
complete closure before desktop startup or packaging. Supplier recipe changes invalidate that
supplier; shared download/inventory/library tooling and desktop configuration changes invalidate
all affected suppliers. Downloads retry transient network/server failures at most twice, identify
the URL and underlying error, and remove rejected partial files. Integrity failures are not retried.

### Core FFmpeg and developer startup

FFmpeg 9.0.2 remains one static executable plus notices and owned evidence. The Node producer uses
pinned source archives, yauzl selected-entry extraction, compiled-library dependency checks and OS signing.
FFmpeg is prepared directly on the development/packaging host and included in the app; it has
no independent archive, descriptor, release or provisioning step. Production preparation requires
approved license evidence and signs the actual executable before sealing its manifest.
It rejects non-system dynamic dependencies, nonfree builds and missing subtitles/analysis/loudness/
concatenation/platform encoder capabilities. Real libx264/AAC/loudness, subtitle, concat and PCM
smokes run from relocated paths including Chinese characters. There is no Python media producer.

`media-license-review.json` remains an accountable release input; pending approval permits QA only.
Source/license changes invalidate its receipt. No tooling or QA result constitutes legal approval.

`pnpm desktop:dev` prepares core FFmpeg, inference suppliers and the Rust worker before invoking the
Tauri CLI; `beforeDevCommand` starts only Vite. Rust build tooling requires pinned CMake, the
normal Rust/Clang toolchain and macOS Metal tooling or the frozen Windows Vulkan SDK. The SDK's
`copy_only=1` mode stays in build scratch and does not change system drivers or registry. Python is
required only for retained Agent production and contract/build tools, never local model execution.
CI's single `setup-rust-build` action installs the pinned Protobuf compiler and CMake, and selects
the Windows runner's LLVM `libclang.dll` for bindgen. Missing compiler libraries fail the build
before inference packaging. Quality runs the same `pnpm check:rust` entry as local development
and compiles the desktop on all three supported targets. Final signing/notarization
verification belongs to the package command and runs before draft artifact upload. CI prepares supplier libraries once; Cargo gates or packaging build the worker
in the required profile rather than first building a disposable debug worker.

Packaging verifies the exact Agent source/descriptor payload and forbids bundled interpreters.
NSIS extraction uses pinned 7-Zip; MSI uses a Windows Installer administrative image. macOS verifies
the actual `.app` resource inventory. Inference package checks cover the sealed worker and libraries in
addition to Agent payload validation. Real installation, launch, first-use model download, offline
execution, signing/notarization and Windows/macOS target acceptance are separate mandatory release
evidence. Editing CI does not claim a run on hardware unavailable to the implementer.

## Artifact policy

- macOS: `.dmg` is the primary human-facing direct installer; `.app` is retained for validation/debugging.
- Windows: NSIS `setup.exe` is the primary human-facing installer; MSI is opt-in for stable numeric releases and managed deployment needs.
- Never publish a production package that has not passed the same commit's quality gates and platform build.
- Publish required third-party notices and corresponding-source bundles beside every installer and
  updater artifact, or through the approved durable source-offer location bound to that release.
  Mutable branches and unversioned project homepages do not satisfy artifact identity.

## QA packaging workflow

The manual `workflow_dispatch` entry of `.github/workflows/release-candidate.yml` produces internal
QA artifacts from the selected `runtime_publication_tag`, using the same target matrix and
provisioning as signed candidates:

- macOS uses ad-hoc signing so test artifacts are structurally signed without pretending to be a notarized production release;
- Windows NSIS artifacts are unsigned and may trigger SmartScreen warnings;
- artifacts are uploaded to the workflow run, not published as a production GitHub Release.

Manual QA uses the `runtime-qa` environment. Production signing steps, signing secrets,
notarization and candidate publication execute only for version-tag pushes in their protected
release environments. Sharing build preparation does not grant the QA entry production authority.

This workflow requires committed lockfiles, the selected attested Agent tag and the public
distribution repository. It refuses to package if resource verification fails.

## Production packaging

Production distribution requires real organization-owned configuration:

- macOS Developer ID signing + notarization;
- Windows code signing;
- final product icons and bundle metadata;
- updater endpoint/public key/signing private key when auto-update is enabled.

Those values are intentionally not filled with placeholders in source control. See `release.md`.

## Updater artifacts

Normal QA packages remain updater-disabled even when updater environment variables are inherited. `pnpm package --release` validates aligned versions, production signing and permanent HTTPS discovery, then embeds the public verification configuration before plugin initialization. It merges `src-tauri/tauri.release.conf.json` and builds with the Cargo lockfile. Signed updater artifacts are separate from human-facing DMG installers. The official Tauri action builds and uploads all three targets to a draft and generates the update
manifest. Target-platform acceptance and manual GitHub publication are documented in `release.md`.

## Bundled Agent plugin content

`src-tauri/resources/agent-plugins/knowledge-work` is the single bundled standard package;
Tauri and `scripts/desktop/package.mjs` include this same directory. Debug reads the source
resource; release resolves the packaged Tauri resource. The common installer publishes its
immutable content into application data and preserves the user's enabled state. Application
updates own the bundled revision; it cannot be uninstalled independently.

Plugin installation never downloads executables or runs lifecycle scripts. stdio MCP requires
an explicitly configured installed absolute executable and explicit connection enablement.
Processes use existing process-group/Windows-job ownership and are killed when their task or
account is revoked. External HTTPS package downloads and MCP endpoints pin validated DNS
addresses and do not inherit environment proxies that bypass that policy. Release support remains
macOS arm64/x64 and Windows x64. Agent RPC/graph v1 and profile v1 reject obsolete internal envelopes.

### Windows compiled resource closure

VoxCPM2 and alignment may use the verified Vulkan backend after capacity admission. Embedding and
Whisper use the explicit CPU profile on Windows. No CUDA/Torch/Tensor closure is shipped. Agent
CPython preserves the documented UTF-8 activation manifest; the Rust inference worker and compiled
libraries require actual non-ASCII-path qualification on Windows.

Windows packaging uses CMake's `InstallRequiredSystemLibraries` to select the licensed build
toolchain's x64 MSVC redistributable DLLs. Their valid Microsoft Authenticode signatures are
verified and preserved; Knovum does not re-sign those vendor files. The exact-pinned LunarG
Vulkan loader and its archive license are installed beside the worker in `bin`. This loader
does not supply a GPU driver. The sealed inventory records these dependencies and their source
evidence. `dumpbin /DEPENDENTS` checks every packaged PE binary: imports must resolve beside
the importing binary, in sealed `bin`, or in the Windows system-library/API-set allowlist.
Missing MSVC, Vulkan or other non-system DLLs fail packaging rather than relying on a developer
machine's installed copies. Microsoft redistribution permission remains a release review item.
Windows development requires the MSVC runtime and Vulkan loader installed with its C++
development toolchain; fresh development-machine qualification is separate from installer QA.

## Environment ownership

Normal desktop development and end-user operation require no Knovum resource-path variables.
Model selection and credentials belong to application settings. Release signing identities,
certificate passwords, updater signing keys and distribution endpoints are release-CI inputs;
GitHub job identifiers and artifact addresses are supplied by the workflows. OS variables such
as `PATH`, `SystemRoot` and Windows SDK installation roots retain their system meanings.
Private worker environments configure third-party offline mode, cache ownership and thread counts;
they are set by the application and are not user configuration. Tests requiring provisioned artifacts
use an explicit ignored-test invocation rather than an environment switch that silently reports success.
