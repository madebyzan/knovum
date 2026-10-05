# Packaging

Packaging creates desktop application/install artifacts. It is distinct from production release signing and updater publication.

The project author and bundle publisher are Zan. Keep `package.json`'s `author`,
`src-tauri/Cargo.toml`'s `authors`, and `src-tauri/tauri.conf.json`'s `bundle.publisher`
consistent with this attribution.

## Local commands

Compile without bundling:

```bash
KNOVUM_PYTHON_RUNTIME_DESCRIPTOR_PATH=<verified-descriptor-path> pnpm exec tauri build --no-bundle
```

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

The desktop installer includes FFmpeg, application-owned Python capability source from `python/src` and
`resources/python-runtime-descriptor.json`. It excludes CPython and third-party Python packages.
The small application source payload follows the application version. `knovum_inference` currently
provides the `knovum-inference` process entry; future capability entries have their own lifecycle. One immutable shared Python runtime per target
contains the interpreter, third-party inference dependencies, notices/licenses,
`runtime-requirements.txt` and the complete `runtime-manifest.json` (schema version 2).
Worker source and its private IPC protocol are not part of the third-party runtime identity.

The native runtime producer explicitly selects the `inference` extra and exports frozen
requirements without project, development
or quality dependencies, and records the pinned VoxCPM Git revision separately. Application
packaging compares these actual requirements to the selected publication. Changing the Worker
or unrelated quality dependencies does not require users to download a new runtime.

The trusted descriptor contains target, interpreter version, requirements digest, VoxCPM revision,
immutable HTTPS archive URL, size/SHA-256 and manifest SHA-256. Packaging supplies its path through
`KNOVUM_PYTHON_RUNTIME_DESCRIPTOR_PATH`; `build.rs` embeds its bytes in the executable.
A production build without that input fails. Development and the isolated E2E build may explicitly
have no runtime publication; they do not acquire local inference capabilities until configured.
No fake URLs/hashes, mutable latest selection, system Python, pip or uv run on user computers.

`local_resources/runtimes/python` installs missing components on first capability preparation,
before model preparation and the inference deadline. Downloads share the existing preparation
registry and cancellation semantics. The archive is downloaded into a sibling staging directory,
checked for exact size/digest, extracted without links or unsafe paths, and completely verified
against the trusted manifest digest before atomic publication:

```text
<app-data>/local-resources/runtimes/python/<target>/<archive-sha256>/
```

Worker startup explicitly loads `resources/python-worker` from the application and `packages`
from this verified runtime; initialization passes the actual manifest path. Already prepared
components and models work offline. Bootstrap does not download them. Runtime leases use standard
Rust file locks beside immutable directories; old runtime directories are reclaimed only when
no application/Worker lease holds them. Keeping application data keeps user models.

`KNOVUM_PYTHON_RUNTIME_MANIFEST_PATH` is the explicit development-only override for a verified
third-party runtime. `KNOVUM_PYTHON_WORKER_PATH` explicitly selects the development Worker directory;
`pnpm desktop:dev` sets it to this checkout's `python/src`.
`pnpm desktop:dev` sets the provisioned FFmpeg path and preserves explicit overrides; it does
not discover Python from a legacy bundle layout. Supported inference targets remain macOS ARM64
and Windows x64.

## Controlled build inputs

`pnpm runtime:provision` downloads and verifies FFmpeg into
`src-tauri/resources/bundled-runtime/<target>/`, and the separate Python publication into
`runtime-artifacts/<target>/`. The latter is build evidence/input and is never a bundle resource.
The provisioner requires these real, immutable URLs and SHA-256 values (`TARGET` is
`MACOS_ARM64` or `WINDOWS_X64`):

- `KNOVUM_FFMPEG_<TARGET>_URL` / `_SHA256`;
- `KNOVUM_PYTHON_RUNTIME_<TARGET>_URL` / `_SHA256`;
- `KNOVUM_PYTHON_RUNTIME_DESCRIPTOR_<TARGET>_URL` / `_SHA256`;
- `KNOVUM_PYTHON_RUNTIME_EVIDENCE_<TARGET>_URL` / `_SHA256`.

The optional `KNOVUM_BUNDLED_RUNTIME_AUTHORIZATION` applies only to build-time FFmpeg inputs.
Python publication downloads are anonymous, matching user-side installation. The descriptor's
archive URL/hash/size must match the provisioned source, and its manifest/dependency identities
must match the actual runtime and checkout's frozen runtime requirements.

On-demand yt-dlp/Deno remain in `local_resources/tools`; they are not packaged. Their existing
manifest, verified installation and no-PATH policy remain in force. No font files are packaged.
Background removal continues to use ONNX Runtime with the verified on-demand U²-Net model.

## Controlled Python runtime producer

```bash
pnpm runtime:build --output <fresh-directory> --signing <adhoc|unsigned|production> \
  --archive-url <immutable-public-https-archive-url>
```

The native producer uses the frozen publication group and pinned CPython 3.11.17 standalone
archives from `python/runtime-source.json`. uv 0.12.23 installs hash-bound dependencies in a
frozen build environment. VoxCPM builds from its exact Git commit with source/tree/wheel evidence.
No development environment or application Worker is copied into the runtime. Relocation checks
import the actual third-party inference libraries from a path with spaces and Chinese characters.
Signing precedes file-manifest creation. QA signing is explicitly ad-hoc on macOS/unsigned on
Windows; production signing and signer/timestamp verification remain mandatory.

The producer emits the runtime archive and SHA-256, trusted `runtime-descriptor.json`, plus a
separate target evidence archive containing producer provenance, frozen requirements, actual
installed dependency inventory, CycloneDX 1.6 SBOM and pinned corresponding-source archives.
These evidence/source attachments are offered beside the same immutable runtime version with
equivalent access. Runtime licenses/notices stay in the conveyed runtime; moving evidence is
not distribution approval. The workflow's `archive_base_url` must be the actual immutable public
publication directory. Workflow upload artifacts alone do not publish that URL.

`python/dependency-license-review.json` must approve the runtime requirements digest, VoxCPM
revision and exact per-target installed inventory. The unresolved sherpa/eSpeak/piper native-source
review and separate Kokoro/data review continue to block production. Metadata, source hashes and
QA artifacts are not legal approval. Required complete corresponding-source/build/installation
materials must accompany the reviewed publication.

Packaging verifies the external runtime and its production signatures before Tauri, copies only
current `.py` Worker files and the trusted descriptor, then checks the actual macOS `.app` or
extracted Windows NSIS/MSI contents. Each must contain the exact application source payload/descriptor and no
interpreter/runtime manifest. NSIS extraction uses pinned 7-Zip 26.03; MSI uses Windows Installer's
administrative image. Temporary extraction is removed on success and failure. Runtime imports
are smoke-tested separately with the current application Worker. Qualification binds both
`workerDigest` and the published runtime manifest digest. Final signing/notarization, clean
installation and actual first-download/offline functionality on both platforms remain native
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

Normal QA packages remain updater-disabled even when updater environment variables are inherited. `pnpm package --release` validates aligned versions, production signing and permanent HTTPS discovery, then embeds the public verification configuration before plugin initialization. It merges `src-tauri/tauri.release.conf.json` and builds with the Cargo lockfile. Signed updater artifacts are separate from human-facing DMG installers. The candidate workflow covers both supported targets; promotion, native upgrade acceptance and GitHub/COS migration are documented in `release.md`.
