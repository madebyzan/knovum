# Packaging

Packaging creates desktop application/install artifacts. It is distinct from production release signing and updater publication.

The desktop entry point links Knovum's internal Rust library as an `rlib`; the
`[lib]` target follows this topology. Tauri produces the desktop executable and
application/install artifacts.

The project author and bundle publisher are Zan. Keep `package.json`'s `author`,
`src-tauri/Cargo.toml`'s `authors`, and `src-tauri/tauri.conf.json`'s `bundle.publisher`
consistent with this attribution.

## License delivery

Distributions must include the applicable Knovum [LICENSE](../LICENSE) and third-party notices
required by the components actually shipped. Cargo packages reference the root license;
`python/LICENSE` is the standalone Python package's copy and must remain identical to it.
Verify these files in the actual distribution.

## Local commands

Daily development uses `pnpm desktop:dev`. The same project can run through the standard
tools directly, without importing any helper implementation:

```bash
uv sync --project python --locked
cargo build --locked --manifest-path src-tauri/Cargo.toml --workspace
pnpm exec tauri dev
```

The Cargo workspace builds the desktop host and its private Rust worker. The helper only
coordinates these tool invocations. Distribution packaging additionally assembles and verifies
the product's resources, signatures and license evidence before invoking Tauri.

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

The installer contains FFmpeg, the Rust inference worker and its library/notice closure, the
complete frozen Agent onedir distribution, and bundled Agent plugin data. Agent dependencies
are built from the one `python/uv.lock`; users do not download an interpreter or install packages.
Model weights and optional yt-dlp/Deno remain independently prepared resources.

The Tauri host package contains only the product binary. Contract generation belongs to the
separate `tools/contract-exporter` Cargo workspace package and is not distributed with the app.

`inference-contracts` owns neutral Rust worker DTOs; Python generation consumes Agent contracts
only. Obsolete pre-release protocols fail visibly without compatibility readers or alternate engines.

### Cargo and CMake ownership

`inference-libraries` owns pinned ONNX, audio.cpp, Sherpa and Whisper suppliers, CMake projects,
notices and the voice bridge. Independent ExternalProject builds avoid GGML target collisions.
`media-libraries` owns the pinned FFmpeg archive, legal records and qualification. Both are normal
Cargo dependencies and install inside their own `OUT_DIR/runtime`; CMake uses hashed supplier
archives and Cargo tracks CMake sources and build parameters. Window/Tauri configuration does
not participate in a handwritten compiled-library cache key. There is no manual preparation command.

CMake's configure and build processes limit Git discovery to the supplier build directory with
`GIT_CEILING_DIRECTORIES`. Extracted upstream archives must not inherit Knovum's Git commit as
their build version or recompile when application commits change; archive identity remains the
pinned supplier URL and SHA-256 recorded in the build evidence.

The ONNX wheel remains an archive supplier for the previously qualified libraries and notices;
its Python package is never shipped. Windows uses licensed MSVC redistributables selected by
CMake and verifies Microsoft signatures. Vulkan is the pinned supplier, never a System32 copy.
Missing imports outside the packaged/system closure fail the build. Source changes trigger the
nested CMake dependency graph; there is no Cargo invocation inside a Cargo build script.

### One Agent project

`uv sync --project python --locked` installs the editable Agent into `python/.venv`. The desktop
entry runs this standard command before development. The app never invokes uv, pip, Node or a
repository producer. Debug processes run the installed module with `-I -B -X utf8`; distribution
processes execute `knovum-agent` directly. Credentials/configuration from the launcher remain
excluded from the child environment. Required resources fail visibly instead of being repaired
or replaced by system Python.

Debug Cargo builds do not track the frozen Agent manifest as an input; that manifest belongs
only to distribution builds. Editing or qualifying the Python project does not rebuild Rust.

`python/packaging/agent.spec` uses PyInstaller onedir, console stdio and the Windows UTF-8
manifest. The Windows host uses CREATE_NO_WINDOW. macOS uses Tauri's official `bundle.macOS.files`
to preserve the complete directory and POSIX symlinks; ordinary resource iteration dereferences
links. Windows uses resource mapping. No executable is separated from its `_internal` directory.

PyInstaller owns bootloader/extension relocation and macOS signing. The distribution carries
runtime metadata/licenses, CPython/PyInstaller notices, dependency/build inputs and a validated
CycloneDX SBOM. An immutable file/link inventory is sealed after signing and embedded by the host.
Each link must be relative and resolve within the distribution; every file is verified before spawn.
The bundled Agent has no resource-preparation target or interpreter installation task. Its own
process startup validates the distribution once, within the feature's cancellation and time budget.
DeepAgents, LangGraph, provider SDKs, checkpoints, authorization, cancellation and Skills remain
product code. Model provider imports occur at execution, not control initialization.

### Desktop orchestration and verification

The small desktop helper consumes Cargo's JSON artifact records, builds the frozen Agent,
assembles resources, signs compiled libraries, seals inventories and invokes Tauri. Worker profiles
follow Tauri debug/custom profiles. Frozen builds require the matching target host; cross-host
packaging is rejected explicitly. `--no-bundle` assembles resources beside the Cargo executable.

Final macOS app and extracted Windows NSIS/MSI inventories are compared with trusted build
inputs before upload. Production signing/notarization verification remains mandatory. Compiled libraries
inventories include the worker executable after signing, before the host embeds their hashes.

CI installs pinned Protobuf/CMake and Windows libclang through `setup-rust-build`. Project gates
build the same Cargo graph; the desktop matrix also builds the actual frozen Agent. There is no
separate interpreter archive release, publication tag, GitHub downloader or provisioning step.
Real installation, offline Agent use, model acquisition, cancellation, OS certificates and
supported-target behavior are release evidence; changing CI does not claim unavailable hardware.

## Artifact policy

- macOS: `.dmg` is the primary human-facing direct installer; `.app` is retained for validation/debugging.
- Windows: NSIS `setup.exe` is the primary human-facing installer; MSI is opt-in for stable numeric releases and managed deployment needs.
- Never publish a production package that has not passed the same commit's quality gates and platform build.
- Publish required third-party notices and corresponding-source bundles beside every installer and
  updater artifact, or through the approved durable source-offer location bound to that release.
  Mutable branches and unversioned project homepages do not satisfy artifact identity.

## QA packaging workflow

The manual `workflow_dispatch` entry of `.github/workflows/release-candidate.yml` produces internal
QA artifacts from the selected commit, using the same platform target matrix and project
builds as signed candidates:

- macOS uses ad-hoc signing so test artifacts are structurally signed without pretending to be a notarized production release;
- Windows NSIS artifacts are unsigned and may trigger SmartScreen warnings;
- artifacts are uploaded to the workflow run, not published as a production GitHub Release.

Manual QA uses the `runtime-qa` environment. Production signing steps, signing secrets,
notarization and candidate publication execute only for version-tag pushes in their protected
release environments. Sharing build preparation does not grant the QA entry production authority.

This workflow requires committed lockfiles and the configured public distribution repository. It refuses to package if resource verification fails.

## Production packaging

Production distribution requires real organization-owned configuration:

- macOS Developer ID signing + notarization;
- Windows code signing;
- final product icons and bundle metadata;
- updater endpoint/public key/signing private key when auto-update is enabled.

Those values are intentionally not filled with placeholders in source control. See `release.md`.

## Updater artifacts

Normal QA packages remain updater-disabled even when updater environment variables are inherited. `package.json` owns the application version through Tauri's `"version": "../package.json"` setting. `pnpm package --release` validates that version source, production signing and permanent HTTPS discovery, then embeds the public verification configuration before plugin initialization. It merges `src-tauri/tauri.release.conf.json` and builds with the Cargo lockfile. Signed updater artifacts are separate from human-facing DMG installers. The official Tauri action builds and uploads all three targets to a draft and generates the update
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
they are set by the application and are not user configuration. Tests requiring built artifacts
use an explicit ignored-test invocation rather than an environment switch that silently reports success.
