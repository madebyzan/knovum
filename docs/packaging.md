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

The installer contains FFmpeg, current application-owned `.py` capability source from `python/src`,
and `resources/python-<profile>-descriptor.json` for every profile. It excludes CPython and
third-party Python packages. Application source follows the desktop version. The shipped
`workerDigest` covers the entire application-owned Python Worker tree.

Five immutable runtime profiles per target contain their own interpreter/package root, licenses,
`runtime-requirements.txt` and schema-1 `runtime-manifest.json`: `resources`, `light-inference`,
`torch-inference`, `voice-generation` and `agent`. Each exports production dependencies from its
own `python/runtimes/<profile>/uv.lock`; the root project owns tooling only. Publications contain
neither quality dependencies nor application source. All profiles reuse one producer and installer.
Resources and light inference do not convey Torch or VoxCPM; Agent remains independent.

Schema-1 descriptors bind profile, target, interpreter version, dependency lock/requirements hashes,
one immutable HTTPS `archiveUrl`, size/SHA-256 and manifest SHA-256. Packaging stages them at
`src-tauri/resources/python-<profile>-descriptor.json`; `build.rs` embeds those same fixed files. Release requires all five; debug/E2E can explicitly omit
publications, leaving their capabilities unavailable. Source-only edits do not change runtime
archives. A changed profile lock invalidates only that profile's publication. The runtime is never
resolved or installed with pip or uv on an end-user computer. Old `inference` publications are rejected.

Producer records also use schema version 1. Publications created with earlier development schemas
must be rebuilt with the current producer; renumbering manifest bytes changes their integrity hashes.

`local_resources/runtimes/python` installs a selected profile on demand, using the existing shared
preparation registry, measured progress, cancellation and immutable archive verification. Settings
shows all five profiles independently. Archive download, safe extraction and full file verification
precede atomic publication to:

```text
<app-data>/local-resources/runtimes/python/<profile>/<target>/<archive-sha256>/
```

Workers use current application `resources/python-worker` and the selected verified `packages` root.
Inference initialization passes its manifest; Builtin Agent has a versioned graph/protocol handshake
and a private checkpoint path. Prepared runtime/models work offline; bootstrap does not download
resources. Standard file leases protect in-use publications, including while a process shuts down.
User models keep their existing app-data lifetime.

Development resource locations are owned by the checkout: core FFmpeg uses
`src-tauri/resources/bundled-runtime/<target>`, application Workers use `python/src`, and each
Python profile uses `.runtime-development/<profile>/<target>/<recipe-identity>/runtime`.
`pnpm tauri dev` and `pnpm desktop:dev` share this policy and require no project environment
variables. Released applications use bundle resources and verified immutable app-data paths.
Models and media-import tools are always downloaded into application-owned storage; no per-file
path override or PATH search exists. All profiles target macOS arm64/x64 and Windows x64;
target qualification gaps are explicit below.

## Compiled resource production and consumption

macOS arm64, macOS x64 and Windows x64 each have immutable Conda dependency locks in `python/conda-locks`. The Conda-forge CPython and compiled-library closure and hash-bound Python wheels are installed by the controlled producer; no dependency solver runs in the shipped application. The producer verifies inventory, notices, relocation, imports and signatures before creating a descriptor. The Intel MOSS application adapter has real generation, cloning and cancellation/reuse evidence; that does not qualify other engines or targets automatically.

`runtime-source.json` names that supplier `condaSupplier`; producer evidence uses
`conda-packages.json`, and the core-media descriptor binds `condaLockSha256`. Voice shared libraries
are installed under `compiled-libraries/`, matching the inference loaders and relocation checks.
`dependency-evidence.json` records CPython `version` and `source` under `cpython`; the separate
`runtimeInstallation` inventory covers every non-package compiled binary and notice in the installed
runtime, including audio.cpp libraries and source notices.
Renaming a producer input changes its recipe identity. Previously prepared development resources
must be rebuilt by the normal preparation path; published resources must be produced and verified
against the current recipe rather than relabeled.

The reusable `python-runtime.yml` workflow builds all five profiles and the core media resource on all three target runners, verifies the complete 18-entry publication and publishes archives, descriptors and evidence under a commit/run/attempt-specific GitHub release tag. QA packaging and signed release candidates depend on this workflow and consume its `runtime-publication.json` artifact. A partial matrix cannot publish. Addresses and SHA-256 values are generated from the actual archives; maintainers do not configure separate FFmpeg or Python resource addresses.

`pnpm runtime:provision --profile <profile> --publication <runtime-publication.json>` verifies the selected target/profile archive, descriptor and evidence. The light-inference invocation additionally provisions the core FFmpeg archive from the same publication, verifies its descriptor, frozen compiled closure and complete file inventory, and activates it atomically. All published resources support anonymous download.

Core FFmpeg 9.0.1 is produced independently from the Python environments using the frozen `python/conda-locks/media/<target>.json` Conda-forge closures, the same pinned micromamba installer and platform code-signing policy. Its prefix retains required shared libraries and license notices: macOS resolves `bin/ffmpeg`; Windows resolves `Library/bin/ffmpeg.exe`. The media manifest binds the exact producer recipe, Conda package inventory and every regular file. Production resource signatures and complete corresponding-source/license review remain release requirements.

The producer exports the entry point's transitive compiled dependency graph and runtime data/notices, excluding SDK tools, headers, static archives and unreferenced library aliases. macOS uses the Xcode command-line `otool`; Windows uses the latest installed MSVC inspector located through official `vswhere`. These are producer requirements on target-platform Rust/Tauri builders, never dependencies of the installed application. The signed compiled file count must equal the retained graph, and its paths must be present in the media file manifest. Dependency inspection or qualification failures prevent activation.

`pnpm tauri dev` and `pnpm desktop:dev` run `scripts/desktop/dev.mjs`: it verifies or automatically produces the host core media resource before entering the official Tauri CLI. Tauri's `beforeDevCommand` then starts only Vite through `pnpm dev`; resource production does not consume its 180-second frontend readiness window. Preparation failures stop startup with their actual error. A changed recipe or damaged owned resource is rebuilt before the desktop starts. This requires no developer-supplied FFmpeg path, URL, package manager or system Python. Python feature environments remain prepared on demand by the application. Recipe identity includes the common producer files and the selected resource/profile files declared in `python/runtime-source.json`; Node preparation and Rust resolution use the same declaration. Unrelated distribution scripts and other profiles’ supplier code do not invalidate an unchanged resource. Developer preparation uses the pinned tooling bootstrap; the packaged application contains the finished core media prefix and does not run this producer.

Provisioning checks the archive URL/hash/size against its descriptor and validates manifest/dependency identity against the checkout. Python runtimes remain external on-demand resources; only current worker source and frozen descriptors enter the app. A release build without all five descriptors fails at compile time. Debug `pnpm tauri dev` prepares resources on feature use without a published archive. Publication workflow execution, hosting and target-platform acceptance are required before claiming a distributable release; editing the workflow alone does not publish resources.

On-demand yt-dlp/Deno remain in `local_resources/tools`; they are not packaged. Their existing
manifest, verified installation and no-PATH policy remain in force. No font files are packaged.
Background removal continues to use ONNX Runtime with the verified on-demand U²-Net model.

## Controlled Python runtime producer

```bash
pnpm runtime:build --profile <resources|light-inference|torch-inference|voice-generation|agent> --output <fresh-directory> --signing <adhoc|unsigned|production> \
  --archive-url <immutable-https-archive-url>
```

The CPython producer uses the frozen publication group and pinned Conda-forge CPython 3.11.17/compiled closure from the per-target locks. uv 0.12.23 installs hash-bound dependencies in a
frozen build environment. The audio.cpp package builds the exact source archive for VoxCPM2 and Qwen3 forced alignment. VoxCPM2 uses the application callback bridge; alignment uses the supplier public C API. Both libraries share one verified backend closure; archive, compiler recipe and bridge hashes accompany the compiled-runtime evidence. Metal kernels are compiled during resource production and installed beside the private interpreter.
The x64 producer enables the supplier's CPU instruction-set dispatch modules. Shared backends and their dependencies live beside the private interpreter; the isolated voice worker searches only that verified runtime. Windows builds the optional Vulkan module using LunarG SDK 1.4.363.0, pinned by archive size and SHA-256. The SDK's documented `copy_only=1` mode keeps its files in producer scratch space without modifying system PATH, registry or drivers. It is build tooling and is not shipped to users. Production runtimes and application packages use the same `release-signing` environment for certificate material.
No development environment or application Worker is copied into the runtime. Relocation checks import the selected profile's actual third-party stack from a path containing
spaces and Chinese characters. Resources import both Hub SDKs without Torch; light inference
blocks Torch imports while loading ONNX/Sherpa and compiled CTranslate2 ASR; heavy inference,
voice generation and Agent each import their independently selected stack.
Signing precedes file-manifest creation. QA signing is explicitly ad-hoc on macOS/unsigned on
Windows; production signing and signer/timestamp verification remain mandatory.

The producer emits the runtime archive and SHA-256, trusted `runtime-descriptor.json`, plus a
separate target evidence archive containing producer provenance, frozen requirements, actual
installed dependency inventory, CycloneDX 1.6 SBOM and pinned corresponding-source archives.
These evidence/source attachments are offered beside the same immutable runtime version with
equivalent access. Runtime licenses/notices stay in the conveyed runtime; moving evidence is
not distribution approval. The workflow publishes the generated immutable archive addresses before dependent application packaging. Anonymous reachability in supported networks remains a release acceptance criterion.

`python/runtimes/<profile>/dependency-license-review.json` approves each profile's lock/requirements
digests and exact per-target installed inventory. Voice generation conveys pinned VoxCPM source evidence. The unresolved sherpa/eSpeak/piper corresponding-source
review continues to block production for the conveyed light inference binary; the former Kokoro model/data archive is no longer distributed. Metadata, source hashes and
QA artifacts are not legal approval. Required complete corresponding-source/build/installation
materials must accompany the reviewed publication.

Packaging verifies the external runtime and its production signatures before Tauri, copies only
current `.py` Worker files and both trusted descriptors, then checks the actual macOS `.app` or
extracted Windows NSIS/MSI contents. Each must contain the exact application source payload/descriptor and no
interpreter/runtime manifest. NSIS extraction uses pinned 7-Zip 26.03; MSI uses Windows Installer's
administrative image. Temporary extraction is removed on success and failure. Runtime imports
are smoke-tested separately with the current application Worker. Qualification binds both
`workerDigest` and the published runtime manifest digest. Final signing/notarization, clean
installation and actual first-download/offline functionality on both platforms remain target-platform
release acceptance requirements.

## Artifact policy

- macOS: `.dmg` is the primary human-facing direct installer; `.app` is retained for validation/debugging.
- Windows: NSIS `setup.exe` is the primary human-facing installer; MSI is opt-in for stable numeric releases and managed deployment needs.
- Never publish a production package that has not passed the same commit's quality gates and platform build.
- Publish required third-party notices and corresponding-source bundles beside every installer and
  updater artifact, or through the approved durable source-offer location bound to that release.
  Mutable branches and unversioned project homepages do not satisfy artifact identity.

## QA packaging workflow

`.github/workflows/package.yml` is a manual workflow for internal QA artifacts:

- macOS uses ad-hoc signing so test artifacts are structurally signed without pretending to be a notarized production release;
- Windows NSIS artifacts are unsigned and may trigger SmartScreen warnings;
- artifacts are uploaded to the workflow run, not published as a production GitHub Release.

This workflow requires committed pnpm and Cargo lockfiles plus configured controlled bundled-runtime archive URLs/SHA-256 values for the runner architecture. It refuses to package if provisioning or verification fails.

## Production packaging

Production distribution requires real organization-owned configuration:

- macOS Developer ID signing + notarization;
- Windows code signing;
- final product icons and bundle metadata;
- updater endpoint/public key/signing private key when auto-update is enabled.

Those values are intentionally not filled with placeholders in source control. See `release.md`.

## Updater artifacts

Normal QA packages remain updater-disabled even when updater environment variables are inherited. `pnpm package --release` validates aligned versions, production signing and permanent HTTPS discovery, then embeds the public verification configuration before plugin initialization. It merges `src-tauri/tauri.release.conf.json` and builds with the Cargo lockfile. Signed updater artifacts are separate from human-facing DMG installers. The candidate workflow covers all three supported release targets; promotion, target-platform upgrade acceptance and GitHub/COS migration are documented in `release.md`.

## Bundled Agent plugin content

`src-tauri/resources/agent-plugins/knowledge-workflows` is the single bundled standard package;
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

### Windows GPU resource closure

The Windows Tensor Conda lock freezes one CUDA 12.8 closure for Torch, torchaudio and torchcodec, including cuDNN 9.10.2.21. Its 137 immutable Conda archives total 2,979,077,089 compressed bytes before Python wheels and the application archive. Windows Faster Whisper/CTranslate2 belong to this runtime, while Intel macOS ASR remains in light-inference. The relocated producer verifies these imports and the Torch CUDA ABI. CPU execution remains available within the same verified package; no user-installed CUDA toolkit is required. Windows GPU and CPU operation must be qualified separately on real hardware; this document records the producer contract and does not claim that acceptance.

Windows Python runtime production preserves CPython’s original activation manifest and sets its Win32 process code page to UTF-8 with the installed x64 Windows SDK Manifest Tool before signing. Original execution privileges, DPI, and long-path settings remain in the manifest. The original/configured manifests are build evidence, and relocation smoke verifies code page 65001. This applies to every Python runtime profile; the supported Windows baseline is Windows 10 1903+ x64.

## Environment ownership

Normal desktop development and end-user operation require no Knovum resource-path variables.
Model selection and credentials belong to application settings. Release signing identities,
certificate passwords, updater signing keys and distribution endpoints are release-CI inputs;
GitHub job identifiers and artifact addresses are supplied by the workflows. OS variables such
as `PATH`, `SystemRoot` and Windows SDK installation roots retain their system meanings.
Private worker environments configure third-party offline mode, cache ownership and thread counts;
they are set by the application and are not user configuration. Tests requiring provisioned artifacts
use an explicit ignored-test invocation rather than an environment switch that silently reports success.
