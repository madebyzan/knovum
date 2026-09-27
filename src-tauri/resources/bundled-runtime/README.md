# Bundled runtime resources

The repository may contain development binaries for multiple targets. A release package contains
only its current target directory plus the shared media import manifest.

```text
bundled-runtime/
  <target>/
    ffmpeg[.exe]
    whisper-cli[.exe]
    platform libraries required by whisper-cli
```

Release target names are `macos-arm64`, `macos-x64`, `windows-x64`, and `windows-arm64`.

Development may point to explicit binaries with `KNOVUM_FFMPEG_PATH`, `KNOVUM_YT_DLP_PATH`, `KNOVUM_DENO_PATH`, and `KNOVUM_WHISPER_PATH`. These overrides are for development and do not satisfy release packaging or production provenance checks.

Every supported target must provide libass `ass`/`subtitles` filters with the native system font provider (Core Text on macOS, DirectWrite on Windows), plus `h264_videotoolbox` on macOS or `h264_mf` on Windows. Video and document export use the fonts installed on the current computer; no typeface is packaged with the runtime.

`media-import-manifest.json` pins the official yt-dlp and Deno release assets, checksums,
and yt-dlp corresponding source. It is compiled into the application and is the trust source for
installing those tools under the application data directory when a user first starts a web media
import. yt-dlp is the sole web media importer; Deno is supplied explicitly for yt-dlp's YouTube
JavaScript challenge support.

`pnpm runtime:provision` stages only the controlled FFmpeg/whisper.cpp archive and rejects archives
that contain yt-dlp or Deno. `tools/package-desktop.mjs` applies the same boundary and enumerates the
current target's files instead of using a catch-all glob.

The runtime installer verifies the downloaded archive hash and extracted executable hash before
publishing a versioned tool directory. Fixed hashes establish artifact identity; production release
evidence must separately cover platform signing and clean-machine execution.
