# Bundled runtime resources

The repository may contain development binaries for multiple targets. A release package contains
only its current target directory plus the shared media import manifest.

```text
bundled-runtime/
  <target>/
    ffmpeg[.exe]
    python-runtime/
      runtime-manifest.json
      bin/python3             # macOS
      python.exe              # Windows
      packages/knovum_inference/
```

Release target names are `macos-arm64` and `windows-x64`.

Development may point to explicit binaries with `KNOVUM_FFMPEG_PATH`, `KNOVUM_YT_DLP_PATH`, and `KNOVUM_DENO_PATH`. These overrides are for development and do not satisfy release packaging or production provenance checks.
The development runner also sets `KNOVUM_PYTHON_RUNTIME_MANIFEST_PATH` to the provisioned host
manifest. Release builds ignore that override and resolve only the signed application resource tree.

`runtime-manifest.json` is schema version 1 and identifies `runtimeId`, exact CPython `3.11.17` `version`,
`protocolVersion`, `target`, `executable`, `packageRoot`, and every runtime file by slash-relative
path, byte size, and lowercase SHA-256. `dependencyLockSha256` binds the package to the repository's
exact `python/uv.lock` bytes. The manifest does not list itself; its exact byte digest is
the runtime identity passed to the Python worker. Archives and packaged trees reject links, special
files, unsafe paths, duplicate entries, undeclared files, and file/hash mismatches. The runtime is
built and signed before this manifest is generated; packaging must not mutate a listed file.
Run `pnpm runtime:manifest --target <target> --directory <signed-python-runtime>` after the
native interpreter, locked wheels, worker package, and native signatures are final.
The command requires the build host to match the target; it cannot label an emulated or foreign tree
as a native runtime.
The Rust resolver fully verifies a runtime once per process. Debug manifests are re-read on use;
changing their exact bytes invalidates the cache, so development file changes must regenerate the manifest.

Every supported target must provide libass `ass`/`subtitles` filters with the native system font provider (Core Text on macOS, DirectWrite on Windows), plus `h264_videotoolbox` on macOS or `h264_mf` on Windows. Video and document export use the fonts installed on the current computer; no typeface is packaged with the runtime.

`media-import-manifest.json` pins the official yt-dlp and Deno release assets, checksums,
and yt-dlp corresponding source. It is compiled into the application and is the trust source for
installing those tools under the application data directory when a user first starts a web media
import. yt-dlp is the sole web media importer; Deno is supplied explicitly for yt-dlp's YouTube
JavaScript challenge support.

`pnpm runtime:provision` stages separately pinned FFmpeg and Python runtime archives and rejects archives
that contain yt-dlp or Deno. `tools/package-desktop.mjs` applies the same boundary and enumerates the
current target's files instead of using a catch-all glob.

The runtime installer verifies the downloaded archive hash and extracted executable hash before
publishing a versioned tool directory. Fixed hashes establish artifact identity; production release
evidence must separately cover platform signing and clean-machine execution.

The Python archive is produced by `pnpm runtime:build` on its exact native target, using
`python/runtime-source.json` and the frozen runtime/build groups. `build-evidence/`
records interpreter origin, installed dependency/source/license inventory, CycloneDX
SBOM, VoxCPM source, pinned sherpa/eSpeak/piper recipe sources, signing and relocation
checks. The provisioner rejects a worker source tree that differs from this checkout.
The sherpa recipe sources do not establish correspondence for the older Kokoro
espeak-ng-data archive; production remains blocked pending that resource review.
