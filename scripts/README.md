# Desktop build helpers

Daily development uses `pnpm desktop:dev`, `pnpm test`, `pnpm format`, `pnpm check` and
`pnpm package`. `dev` and `build` are Tauri's Vite hooks.

`desktop/tauri.mjs` coordinates an ordinary uv project sync, Cargo dependency builds and the
vendor Tauri CLI. Cargo JSON artifacts identify the compiled outputs; there is no independent
runtime recipe, downloader, cache namespace, interpreter installer or publication service.
The remaining modules implement concrete resource assembly, signing, license approval and
final installer verification. Tests are developer verification, not additional entry points.

Compiled suppliers and the voice bridge belong to `src-tauri/crates/inference-libraries`; FFmpeg
belongs to `src-tauri/crates/media-libraries`. CMake owns their supplier builds and hashed downloads.
Agent source, its single uv lock, frozen spec, SBOM and packaging tests belong to `python/`.
Capability checks are Rust tests; Spaces handlers and ACLs consume one Rust declaration.

`contracts.mjs` is the occasional Rust/TypeScript/Python generation and drift-check entry.
Python generation lives in `python/contracts`. `e2e` uses the isolated desktop configuration.
The quality workflow verifies project owners; the desktop workflow builds and verifies complete
installers. There is no independent Python runtime workflow or runtime command family.

Authoritative supply, signing and platform contracts live in [packaging](../docs/packaging.md)
and [release](../docs/release.md).
