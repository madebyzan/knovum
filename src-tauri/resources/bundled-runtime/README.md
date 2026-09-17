# Bundled runtime resources

The repository may contain development binaries for multiple targets. A release package contains
only its current target directory plus the shared media import manifest and fonts.

```text
bundled-runtime/
  <target>/
    ffmpeg[.exe]
    yt-dlp[.exe]
    deno[.exe]
    whisper-cli[.exe]
```

Release target names are `macos-arm64`, `macos-x64`, `windows-x64`, and `windows-arm64`.

Development may point to explicit binaries with `KNOVUM_FFMPEG_PATH`, `KNOVUM_YT_DLP_PATH`, `KNOVUM_DENO_PATH`, and `KNOVUM_WHISPER_PATH`. These overrides are for development and do not satisfy release packaging.

Every supported target must provide libass `ass`/`subtitles` filters plus `h264_videotoolbox` on macOS or `h264_mf` on Windows. `fonts/NotoSansCJKsc-Regular.otf` is the single cross-platform subtitle typeface; derivation jobs copy it into their isolated render workspace so output never depends on host-installed fonts.

`media-import-manifest.json` pins the official yt-dlp and Deno release assets, checksums,
and yt-dlp corresponding source. `pnpm runtime:provision` is the only provisioning
entry point: it stages the controlled target archive and then adds the exact official yt-dlp and
Deno executables. yt-dlp is the sole web media importer; Deno is supplied explicitly for
yt-dlp's YouTube JavaScript challenge support.

`tools/package-desktop.mjs` is the only package resource selector. It rejects an incomplete target
runtime and supplies Tauri with an explicit resource list containing the current target and shared
resources. The base Tauri configuration intentionally contains no broad runtime glob.
