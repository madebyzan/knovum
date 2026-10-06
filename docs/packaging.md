# Packaging

Packaging creates desktop application/install artifacts. It is distinct from production release signing and updater publication.

The desktop entry point links Knovum's internal Rust library as an `rlib`; the
`[lib]` target follows this topology. Tauri produces the desktop executable and
application/install artifacts.

The project author and bundle publisher are Zan. Keep `package.json`'s `author`,
`src-tauri/Cargo.toml`'s `authors`, and `src-tauri/tauri.conf.json`'s `bundle.publisher`
consistent with this attribution.

## Local commands

Compile without bundling:

```bash
KNOVUM_PYTHON_INFERENCE_DESCRIPTOR_PATH=<verified-inference-descriptor> \
KNOVUM_PYTHON_AGENT_DESCRIPTOR_PATH=<verified-builtin-agent-descriptor> \
pnpm exec tauri build --no-bundle
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

The installer contains FFmpeg, current application-owned `.py` capability source from `python/src`,
`resources/python-inference-descriptor.json` and `resources/python-agent-descriptor.json`.
It excludes CPython and third-party Python packages. Application source follows the desktop version.
Two immutable runtime profiles per target contain their own interpreter/package root, licenses,
`runtime-requirements.txt` and schema-3 `runtime-manifest.json`. Inference selects the frozen
`inference`/`model-resources` extras from `python/uv.lock`; Builtin Agent selects production dependencies
from `python/runtimes/agent/uv.lock`. Neither publication contains quality dependencies or
application source. The profiles reuse one producer and installer; their incompatible Pydantic
constraints require separate dependency environments. No Torch or inference models enter Builtin Agent.

Schema-2 descriptors bind profile, target, interpreter version, dependency lock/requirements hashes,
regional immutable HTTPS archive URLs, size/SHA-256 and manifest SHA-256. Packaging passes their paths
through `KNOVUM_PYTHON_INFERENCE_DESCRIPTOR_PATH` and `KNOVUM_PYTHON_AGENT_DESCRIPTOR_PATH`;
`build.rs` embeds both. Release requires both; debug/E2E can explicitly omit publications, leaving
these capabilities unavailable. Source-only edits do not change the runtime archive. A changed
lock invalidates that profile's frozen publication, including a changed quality constraint in the
same lock. The runtime is never resolved/installed with pip or uv on an end-user computer.

`local_resources/runtimes/python` installs a selected profile on demand, using the existing shared
preparation registry, measured progress, cancellation and immutable archive verification. Settings
shows both profiles independently. Archive download, safe extraction and full file verification
precede atomic publication to:

```text
<app-data>/local-resources/runtimes/python/<profile>/<target>/<archive-sha256>/
```

Workers use current application `resources/python-worker` and the selected verified `packages` root.
Inference initialization passes its manifest; Builtin Agent has a versioned graph/protocol handshake
and a private checkpoint path. Prepared runtime/models work offline; bootstrap does not download
resources. Standard file leases protect in-use publications, including while a process shuts down.
User models keep their existing app-data lifetime.

Development-only verified-runtime overrides are `KNOVUM_PYTHON_INFERENCE_MANIFEST_PATH` and
`KNOVUM_PYTHON_AGENT_MANIFEST_PATH`. `KNOVUM_PYTHON_WORKER_PATH` selects source explicitly;
`pnpm desktop:dev` sets this checkout's `python/src` and the provisioned FFmpeg path. There is no
legacy bundle discovery or system-Python fallback. Both profiles support macOS ARM64 and Windows x64.

## Controlled build inputs

`pnpm runtime:provision --profile inference` downloads and verifies FFmpeg into
`src-tauri/resources/bundled-runtime/<target>/`, and the separate Python publication into
`runtime-artifacts/inference/<target>/`. Run `pnpm runtime:provision --profile agent` for
`runtime-artifacts/agent/<target>/`. The latter is build evidence/input and is never a bundle resource.
The provisioner requires these real, immutable URLs and SHA-256 values (`TARGET` is
`MACOS_ARM64` or `WINDOWS_X64`):

- `KNOVUM_FFMPEG_<TARGET>_URL` / `_SHA256`;
- `KNOVUM_PYTHON_<PROFILE>_<TARGET>_URL` / `_SHA256`;
- `KNOVUM_PYTHON_<PROFILE>_DESCRIPTOR_<TARGET>_URL` / `_SHA256`;
- `KNOVUM_PYTHON_<PROFILE>_EVIDENCE_<TARGET>_URL` / `_SHA256`.

`PROFILE` is `INFERENCE` or `AGENT`.

The optional `KNOVUM_BUNDLED_RUNTIME_AUTHORIZATION` applies only to build-time FFmpeg inputs.
Python publication downloads are anonymous, matching user-side installation. The descriptor's
selected regional archive URL/hash/size must match the provisioned source, and its manifest/dependency identities
must match the actual runtime and checkout's frozen runtime requirements.

On-demand yt-dlp/Deno remain in `local_resources/tools`; they are not packaged. Their existing
manifest, verified installation and no-PATH policy remain in force. No font files are packaged.
Background removal continues to use ONNX Runtime with the verified on-demand U²-Net model.

## Controlled Python runtime producer

```bash
pnpm runtime:build --profile <inference|agent> --output <fresh-directory> --signing <adhoc|unsigned|production> \
  --global-archive-url <immutable-global-https-archive-url> \
  --china-archive-url <immutable-china-https-archive-url>
```

The native producer uses the frozen publication group and pinned CPython 3.11.17 standalone
archives from `python/runtime-source.json`. uv 0.12.23 installs hash-bound dependencies in a
frozen build environment. VoxCPM builds from its exact Git commit with source/tree/wheel evidence.
No development environment or application Worker is copied into the runtime. Relocation checks import the selected profile's actual third-party stack from a path containing
spaces and Chinese characters: inference and both Hub SDKs, or Deep Agents/SQLite saver and all
official model clients.
Signing precedes file-manifest creation. QA signing is explicitly ad-hoc on macOS/unsigned on
Windows; production signing and signer/timestamp verification remain mandatory.

The producer emits the runtime archive and SHA-256, trusted `runtime-descriptor.json`, plus a
separate target evidence archive containing producer provenance, frozen requirements, actual
installed dependency inventory, CycloneDX 1.6 SBOM and pinned corresponding-source archives.
These evidence/source attachments are offered beside the same immutable runtime version with
equivalent access. Runtime licenses/notices stay in the conveyed runtime; moving evidence is
not distribution approval. The workflow's `global_archive_base_url` and `china_archive_base_url` must be actual immutable
public publication directories serving identical archive bytes. Workflow upload artifacts alone do not publish that URL.

`python/dependency-license-review.json` and
`python/runtimes/agent/dependency-license-review.json` approve each profile's lock/requirements
digests and exact per-target installed inventory. Inference also conveys pinned VoxCPM source evidence. The unresolved sherpa/eSpeak/piper native-source
review and separate Kokoro/data review continue to block production. Metadata, source hashes and
QA artifacts are not legal approval. Required complete corresponding-source/build/installation
materials must accompany the reviewed publication.

Packaging verifies the external runtime and its production signatures before Tauri, copies only
current `.py` Worker files and both trusted descriptors, then checks the actual macOS `.app` or
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
