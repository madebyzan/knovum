# Packaging

Packaging creates desktop application/install artifacts. It is distinct from production release signing and updater publication.

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
target archive containing FFmpeg, `whisper-cli`, and their native
dependencies, then adds the exact official yt-dlp standalone executable and application-owned Deno
runtime from `media-import-manifest.json`. Every downloaded file is SHA-256 verified before it
enters the target directory.

Packaged applications resolve these files only from
`<Tauri resource directory>/resources/bundled-runtime/`. `pnpm tauri dev` and `pnpm desktop:dev`
set the documented tool-path environment variables to the provisioned host directory before Tauri
starts; an explicit developer override remains authoritative. The runtime does not probe repository
paths or alternative bundle layouts.

`media-import-manifest.json` pins the official yt-dlp and Deno release assets, executable hashes, and yt-dlp corresponding-source archive. No font files are included in the desktop package. `whisper-cli` must be built from the pinned whisper.cpp v1.9.3 release with built-in VAD. Apple Silicon uses the embedded Metal backend. Intel macOS uses the Accelerate CPU backend because the v1.9.3 Metal backend fails buffer allocation on supported Intel hardware; inference is invoked with GPU disabled on that architecture. The provisioned-runtime smoke enforces these platform contracts, including the exact yt-dlp and Deno versions.

Background removal is linked into the Rust host through exact-pinned `tract-onnx`; it is not a separately provisioned user runtime. The embedded U²-Net model checksum and provenance live beside the model asset under `src-tauri/src/images/models/`. Release validation must build and run a real inference smoke for each supported macOS and Windows target.

The provision command uses `KNOVUM_BUNDLED_RUNTIME_<TARGET>_URL` and
`KNOVUM_BUNDLED_RUNTIME_<TARGET>_SHA256`; the optional
`KNOVUM_BUNDLED_RUNTIME_AUTHORIZATION` secret supplies an Authorization header for a private
artifact host. `tools/provision-bundled-runtime.mjs` owns the complete download, integrity
verification, safe archive extraction and media-runtime installation flow in one file. Each
downloaded archive is hashed once; the extracted Deno executable is verified separately. One staging
directory is cleaned up when the command finishes or fails.

All `pnpm package:*` commands route through `tools/package-desktop.mjs`. It validates the runtime
and passes Tauri an explicit resource list for exactly one target plus shared fonts and the media import manifest. `src-tauri/tauri.conf.json` deliberately has no catch-all bundled-runtime glob, so
a macOS package cannot accidentally contain Windows binaries or the other macOS architecture (and
vice versa).

## Artifact policy

- macOS: `.dmg` is the primary human-facing direct installer; `.app` is retained for validation/debugging.
- Windows: NSIS `setup.exe` is the primary human-facing installer; MSI is opt-in for stable numeric releases and managed deployment needs.
- Never publish a production package that has not passed the same commit's quality gates and platform build.

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

The official updater plugin is part of the bundled runtime. Normal QA packages remain updater-disabled unless `KNOVUM_UPDATER_ENDPOINT` and `KNOVUM_UPDATER_PUBKEY` are embedded at compile time. Production updater packages use `src-tauri/tauri.release.conf.json` to generate signed update artifacts; see `release.md` for channel, signing and publication requirements.
