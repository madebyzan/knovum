# Bundled media resources

Only target-specific FFmpeg and its required notices/dynamic libraries belong here. yt-dlp,
Deno, CPython and third-party Python packages are installed on demand outside the application.
`pnpm runtime:provision` verifies build inputs; Python publication/evidence are staged in
`runtime-artifacts/<profile>/<target>/`, never in this bundle tree. Core FFmpeg preserves its
supplier prefix (`bin/ffmpeg` on macOS; `Library/bin/ffmpeg.exe` on Windows), license notices,
compiled dependency evidence and `media-manifest.json`.

The package command copies the current application Worker to `resources/python-worker/` and
its five trusted descriptors to `resources/python-<profile>-descriptor.json`, with an explicit resource
list for exactly one supported platform. See `docs/packaging.md` for producer and release inputs.

Desktop development automatically prepares and verifies the same host FFmpeg resource before
starting Vite. Developer and release builds do not need separately configured FFmpeg URLs.
