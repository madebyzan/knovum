# Bundled media resources

Only target-specific FFmpeg and its required notices/dynamic libraries belong here. yt-dlp,
Deno, CPython and third-party Python packages are installed on demand outside the application.
`pnpm runtime:provision` verifies build inputs; Python publication/evidence are staged in
`runtime-artifacts/<target>/`, never in this bundle tree.

The package command copies the current application Worker to `resources/python-worker/` and
its trusted descriptor to `resources/python-runtime-descriptor.json`, with an explicit resource
list for exactly one supported platform. See `docs/packaging.md` for producer and release inputs.
