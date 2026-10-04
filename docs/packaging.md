# Packaging

Packaging creates desktop application/install artifacts. It is distinct from production release signing and updater publication.

The project author and bundle publisher are Zan. Keep `package.json`'s `author`,
`src-tauri/Cargo.toml`'s `authors`, and `src-tauri/tauri.conf.json`'s `bundle.publisher`
consistent with this attribution.

## Local commands

Compile without bundling:

```bash
pnpm desktop:build
```

Build the platform's default configured bundles:

```bash
pnpm package
```

macOS direct-distribution artifacts:

```bash
pnpm package:macos
```

This builds the `.app` and `.dmg` bundles. Run it on macOS.

Windows installer artifacts:

```bash
pnpm package:windows
```

This builds the NSIS `setup.exe`. Run it on Windows. `src-tauri/tauri.windows.conf.json` explicitly disables installer downgrades, keeps NSIS per-user/no-admin by default, and ships English + Simplified Chinese installer localization.

MSI is an explicit stable/enterprise artifact:

```bash
pnpm package:windows:msi
```

WiX/MSI requires a numeric `major.minor.patch[.build]` installer version, so prerelease SemVer such as `1.0.0-rc.x` must not use the MSI path. Run MSI packaging only for a release version that satisfies that rule. MSI creation relies on the Windows WiX toolchain and may require the Windows VBSCRIPT optional feature. The platform config pins a stable WiX upgrade code so product renaming cannot accidentally create duplicate MSI products.

## Bundled runtime provisioning

Place each platform's runtime binaries under `src-tauri/resources/bundled-runtime/<target>/`.
`pnpm runtime:provision` is the only provisioning entry point. It first stages the controlled
independently pinned FFmpeg archive and the controlled CPython 3.11.17 archive produced by the native runtime workflow. The FFmpeg archive must contain FFmpeg and its notices only; it cannot carry a second Python tree.
The archive is SHA-256 verified before it replaces the target directory. Provisioning
rejects archives containing yt-dlp or Deno because those tools do not belong to the desktop bundle.

`python-runtime/runtime-manifest.json` enumerates every interpreter, extension and package file
with exact size and SHA-256, and binds the package to the exact `python/uv.lock` digest.
The native target build writes the manifest with `pnpm runtime:manifest --target <target>
--directory <signed-python-runtime>` after signing the interpreter and native extensions.
Build-time provisioning rejects links, undeclared files and an
incorrect platform or protocol identity. Rust repeats the complete verification before launching
the private worker. The package scripts include only the selected platform's declared runtime
tree; the user computer never runs pip, uv or another dependency resolver.

Packaged applications resolve these files only from
`<Tauri resource directory>/resources/bundled-runtime/`. `pnpm tauri dev` and `pnpm desktop:dev`
set the documented tool-path environment variables to the provisioned host directory before Tauri
starts; an explicit developer override remains authoritative. The runtime does not probe repository
paths or alternative bundle layouts.

`media-import-manifest.json` pins the official yt-dlp and Deno release assets, archive and executable
hashes, and yt-dlp corresponding-source archive. The Rust host embeds this repository-owned manifest.
After a user starts a supported web media import, the local-resource tool owner downloads missing
tools over HTTPS, verifies the pinned hashes, and publishes versioned directories under
`<app-data>/local-resources/tools/`. Concurrent imports share one installation; cancelling one import
does not cancel another waiter's installation. There is no PATH or bundled-runtime fallback. Explicit
developer path overrides remain the only override boundary.

No font files are included in the desktop package. Native Python wheels are locked and verified for macOS arm64 and Windows x64; release acceptance executes the actual ASR, embedding, image, and speech profiles on both target architectures. Other architectures are outside the approved release scope.

The locked runtime currently contains `sherpa-onnx==1.13.8`. Its package metadata is Apache-2.0,
but its default TTS build includes GPL eSpeak NG and MIT piper-phonemize. Production packaging stays
blocked until the distribution review records the exact target wheels, source archives, build and
installation information, license texts and notices. The corresponding-source bundle must include
the sherpa 1.13.8 source plus its exact pinned eSpeak NG and piper-phonemize sources; the separate
`espeak-ng-data` archive also needs source provenance matching the conveyed bytes. A source URL or
hash is evidence material, not legal approval. `tools/package-desktop.mjs` must continue to fail a
production build while that decision is unresolved.

Background removal uses ONNX Runtime inside the private Python worker. U²-Net is a verified on-demand model resource; Rust owns image decoding, preprocessing, mask composition, and output publication.

The provision command requires `KNOVUM_FFMPEG_<TARGET>_URL` / `_SHA256` and
`KNOVUM_PYTHON_RUNTIME_<TARGET>_URL` / `_SHA256`; the optional
`KNOVUM_BUNDLED_RUNTIME_AUTHORIZATION` secret supplies an Authorization header for a private
artifact host. `tools/provision-bundled-runtime.mjs` owns build-time provisioning for FFmpeg and
the controlled Python runtime. On-demand yt-dlp/Deno installation belongs to
`src-tauri/src/local_resources/tools/`; build scripts do not implement a second installer.

All `pnpm package:*` commands route through `tools/package-desktop.mjs`. It validates the runtime,
rejects on-demand tools in the resource tree, and passes Tauri an explicit file list for exactly one
target plus the media import manifest. `src-tauri/tauri.conf.json` deliberately has no catch-all bundled-runtime glob, so
a macOS package cannot accidentally contain Windows binaries or the other macOS architecture (and
vice versa).

## Controlled Python runtime producer

`pnpm runtime:build --output <fresh-directory> --signing <adhoc|unsigned|production>`
runs only on macOS ARM64 or Windows x64. `python/runtime-source.json` pins the exact
CPython 3.11.17 standalone release assets, size and SHA-256 for both native targets,
independently of uv's managed-interpreter catalog. uv 0.12.23 exports the frozen
runtime and build dependency groups. Registry inputs are installed with required
lock hashes; source distributions build with the frozen build environment and no
isolated dependency resolution. VoxCPM is built from the lock's exact Git commit,
with commit/tree/source-archive/wheel hashes recorded. No developer virtual
environment is copied into the application.

The producer installs into `packages`, materializes only internal interpreter file
aliases, rejects external links, and moves the complete tree to a new path containing
spaces and Chinese characters before checking worker/native imports. Native code is
signed and verified before the runtime manifest is written. macOS QA uses ad-hoc
signing; Windows QA explicitly records unsigned status. Production requires Developer
ID or Authenticode certificate configuration and timestamp verification. Verification binds the actual
macOS leaf certificate name and Team ID or Windows signer certificate thumbprint to approved
configuration. QA artifacts
are not production-signing evidence.

`.github/workflows/python-runtime.yml` emits the immutable target archive, SHA-256,
GitHub build attestation, frozen requirements, actual installed dependency inventory,
CycloneDX SBOM and source/license evidence. The existing provisioner validates the
producer identity against the interpreter source and dependency lock as well as all
runtime bytes. There is no old opaque Python archive input path.

Production packaging verifies every native runtime signature before invoking Tauri;
macOS also requires the runtime and application signing identity to match. Tauri
preserves the pre-signed Resources tree on macOS; on Windows it skips EXE/DLL resources
already accepted by Authenticode and does not sign PYD files itself. The producer owns
all of those signatures. After bundling, packaging rechecks the source tree and the
actual macOS app resource manifest and signatures without rewriting either. Every final
Windows NSIS/MSI package is extracted into an owned temporary directory and must contain
exactly one runtime matching the frozen manifest. Production packages must also pass the
same approved-signer checks. Verification uses the independent frozen build Python rather
than executing the runtime being verified. NSIS extraction requires 7-Zip 26.03; MSI uses
Windows Installer's [administrative image](https://learn.microsoft.com/windows/win32/msi/administrative-installation)
operation, without installing the desktop application. Temporary images are removed on
success and failure. A signature or byte change fails the build; no post-notarization repair path exists. Final `.app` strict
signature, Gatekeeper and notarization acceptance remains mandatory for this resource
layout. See the [Tauri bundler implementation](https://github.com/tauri-apps/tauri/blob/8909f221d1515955fc843808032bdc5d62209c96/crates/tauri-bundler/src/bundle/macos/app.rs)
and [Windows resource signing policy](https://github.com/tauri-apps/tauri/blob/8909f221d1515955fc843808032bdc5d62209c96/crates/tauri-bundler/src/bundle/windows/sign.rs).

Dependency metadata and source hashes do not approve native licenses. The evidence
record deliberately requires review; sherpa's embedded GPL components and the
separate Kokoro model/data provenance remain subject to the production gate above.
The produced tree includes pinned sherpa/eSpeak/piper build-recipe source archives and
VoxCPM source. This closes source collection for those pinned recipes; it does not
prove that a wheel was built exactly with those recipes or that the older Kokoro data
corresponds. Full corresponding-source/installation materials must accompany approved distribution;
the generated SBOM alone does not establish them. Native profile quality/performance,
network privacy, notarization, clean installation and desktop behavior still need
release-target acceptance.

Production workflow configuration additionally supplies `KNOVUM_WINDOWS_SIGNTOOL`
(the installed SDK executable), the approved runtime signer
`KNOVUM_WINDOWS_CERTIFICATE_THUMBPRINT`, `KNOVUM_WINDOWS_TIMESTAMP_URL`, the Windows PFX
certificate/password secrets, and the existing Apple signing identity/certificate
secrets. Certificate files/stores are cleaned after the job.

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

Normal QA packages remain updater-disabled even when updater environment variables are inherited. `pnpm package:release` (or `package:release:macos` / `package:release:windows`) validates aligned versions, production signing and permanent HTTPS discovery, then embeds the public verification configuration before plugin initialization. It merges `src-tauri/tauri.release.conf.json` and builds with the Cargo lockfile. Signed updater artifacts are separate from human-facing DMG installers. The candidate workflow covers both supported targets; promotion, native upgrade acceptance and GitHub/COS migration are documented in `release.md`.
