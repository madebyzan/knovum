# Repository scripts

These tools support the Tauri desktop product: development, cross-language contracts,
controlled resources and signed distribution. The installed application consumes finished
resources; it does not run repository producers or install dependencies with uv/pip.

## Everyday commands

Use `pnpm desktop:dev`, `pnpm test`, `pnpm format`, `pnpm check` and `pnpm package`.
`dev` and `build` are Tauri's Vite hooks, not a separate browser product.
`desktop:dev` prepares core FFmpeg before starting the official Tauri CLI; invoking
`pnpm exec tauri dev` directly bypasses that preparation. Use vendor flags for watch,
fix and bundle variants instead of adding aliases.

| Command                                                      | Purpose                                                     |
| ------------------------------------------------------------ | ----------------------------------------------------------- |
| `check:web`, `check:rust`, `check:python`, `check:scripts`   | Verify the owning source scope and its tests.               |
| `security:web`, `security:python`                            | Audit the committed production dependency graphs.           |
| `contracts:generate`, `contracts:check`                      | Generate Rust-owned TS/Python contracts and verify drift.   |
| `e2e`                                                        | Build and exercise the dedicated desktop E2E flavor.        |
| `runtime:prepare --profile <profile>`                        | Prepare one development Python runtime explicitly.          |
| `runtime:build --profile <profile> …`                        | Produce a controlled Python publication on its target host. |
| `runtime:provision --profile <profile> --publication <file>` | Verify and stage an existing publication.                   |
| `inference:acceptance`                                       | Score retained model acceptance evidence.                   |

## Owners

- `desktop/`: core-media development preflight, Tauri resource assembly, final installer
  verification and Windows application signatures.
- `runtime/`: resource producers, frozen inputs, safe file/archive handling, manifests,
  installed inventories, license evidence and OS signatures. Python production uses the
  locked publication tooling group. FFmpeg preparation and signature verification use only
  the pinned Python interpreter and standard library. `media.mjs --write-manifest` is the
  media producer's manifest entry; `signing.py` also exposes its verification CLI.
- `release/`: version/signing policy, artifact collection, immutable draft candidates and
  stable updater promotion. Publication entries are invoked by their workflows.
- `checks/`: source layout, structured Tauri permission policy and language/test gates.
  Layout checks scope web, Rust, Python and tooling separately; no Rust token scanner or
  filename-prefix scanner claims to prove architectural ownership.
- `contracts.mjs` and `generate_python_contracts.py`: one generated-output inventory and
  Rust-schema-to-Python generation through the existing vendor generator.

`voice_bridge.cpp` and `voice_bridge.cmake` are compiled voice-library build inputs.
Supplier-specific modules install the selected pinned MOSS/audio.cpp resources.
`python/runtime-source.json` owns producer source closures; the existing recipe identity
binds selected target/profile inputs, producer bytes and dependency locks. Development
paths and publication receipts consume the same identity.

Each directory's `__tests__/` owns its regression tests. `pnpm check:scripts` discovers Node
unit tests. Python tooling tests run in `check:python`; OS-signing integration and real model
acceptance remain separate. CI's Quality workflow owns shared source/unit checks; Platform
smoke owns per-platform compilation, and Update policy owns actual signing integration.

Build controlled runtimes separately, then select their immutable `runtime-*` release for
QA or signed desktop packaging. Packaging verifies the selected bytes and rejects stale
producer recipes; it does not rebuild the runtime matrix. Required inputs and target-platform
acceptance are documented in [packaging.md](../docs/packaging.md) and
[release.md](../docs/release.md).
