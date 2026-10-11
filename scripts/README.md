# Repository scripts

These tools support the Tauri desktop product: development, cross-language contracts,
controlled resources and signed distribution. The installed application consumes finished
resources; it does not run repository producers or install dependencies with uv/pip.

## Everyday commands

Use `pnpm desktop:dev`, `pnpm test`, `pnpm format`, `pnpm check` and `pnpm package`.
`dev` and `build` are Tauri's Vite hooks, not a separate browser product.
`desktop:dev` prepares core FFmpeg and the native inference worker before starting the official Tauri CLI; invoking
`pnpm exec tauri dev` directly bypasses that preparation. Use vendor flags for watch,
fix and bundle variants instead of adding aliases.

| Command                                                    | Purpose                                                    |
| ---------------------------------------------------------- | ---------------------------------------------------------- |
| `check:web`, `check:rust`, `check:python`, `check:scripts` | Verify the owning source scope and its tests.              |
| `security:web`, `security:python`                          | Audit the committed production dependency graphs.          |
| `contracts:generate`, `contracts:check`                    | Generate Rust-owned TS/Python contracts and verify drift.  |
| `e2e`                                                      | Build and exercise the dedicated desktop E2E flavor.       |
| `runtime:prepare --profile agent`                          | Prepare the retained Deep Agent Python runtime explicitly. |
| `runtime:build --profile agent …`                          | Produce a controlled Agent publication on its target host. |
| `runtime:provision --profile agent --publication <file>`   | Verify and stage an existing Agent publication.            |
| `runtime:native`                                           | Build the frozen native inference supplier libraries.      |
| `inference:acceptance`                                     | Score retained model acceptance evidence.                  |

## Owners

- `desktop/`: core-media and native inference development preflight, Tauri resource assembly, final installer
  verification and Windows application signatures.
- `runtime/`: resource producers, frozen inputs, safe file/archive handling, manifests,
  installed inventories, license evidence and OS signatures. Agent Python production uses the
  locked publication tooling group. FFmpeg and native inference preparation use Node and
  supplier native build tools. `media.mjs --write-manifest` is the media producer's manifest
  entry; compiled native resources use the Node signing owner.
- `inference/`: strict corpus/capture validation and offline acceptance scoring through maintained
  numerical and edit-distance libraries. Actual inference belongs to the Rust worker.
- `release/`: version/signing policy, artifact collection, immutable draft candidates and
  stable updater promotion. Publication entries are invoked by their workflows.
- `checks/`: source layout, structured Tauri permission policy and language/test gates.
  Layout checks scope web, Rust, Python and tooling separately; no Rust token scanner or
  filename-prefix scanner claims to prove architectural ownership.
- `contracts.mjs` and `generate_python_contracts.py`: one generated-output inventory and
  Rust-to-TypeScript export and Agent-only schema-to-Python generation through the existing vendor generator.

`voice_bridge.cpp` and `voice_bridge.cmake` are compiled voice-library build inputs.
Supplier-specific Node modules build the selected pinned audio.cpp, Sherpa and whisper.cpp resources;
the worker loads catalog-owned MOSS ONNX models directly.
Agent, media and native source configurations own their respective producer source closures; each recipe identity
binds selected target/profile inputs, producer bytes and dependency locks. Development
paths and publication receipts consume the same identity.

Each directory's `__tests__/` owns its regression tests. `pnpm check:scripts` discovers Node
unit tests. Python tooling tests run in `check:python`; OS-signing integration and real model
acceptance remain separate. CI's Quality workflow owns shared source/unit checks; Platform
smoke owns per-platform compilation, and Update policy owns actual signing integration.

Build controlled runtimes separately, then select their immutable `runtime-*` release for
QA or signed desktop packaging. Packaging verifies the selected bytes and rejects stale
producer recipes and builds the current Rust worker with its native supply closure. Required inputs and target-platform
acceptance are documented in [packaging.md](../docs/packaging.md) and
[release.md](../docs/release.md).
