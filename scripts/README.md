# Repository scripts

These tools support development, verification and desktop publication. Command-line producers and
checks run on development/CI hosts; the installed application consumes their finished resources.
`voice_bridge.cpp` is compiled into the bundled voice library. A tool is retained for a concrete project
policy or workflow; vendor CLIs, standard libraries and existing packages perform the underlying work.

## Commands to remember

Everyday commands invoke vendor CLIs directly where no project policy is needed. They live in the root [README](../README.md): `desktop:dev`, `check`, `test`, `format`,
and `package`. Scripts expose a task once; flags select variants. Use `pnpm run` to list commands.

| Specialized command                                        | When it is needed                                                                           |
| ---------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| `check:web`, `check:rust`, `check:python`, `check:scripts` | Verify the changed language independently.                                                  |
| `typecheck`, `lint`                                        | Run one frontend check; `lint --fix` applies supported fixes.                               |
| `security:python`                                          | Audit the root Python project and all independent production runtime locks.                 |
| `security:web`                                             | Audit production dependencies and registry signatures, especially in CI/release review.     |
| `e2e:build`                                                | Build the dedicated desktop E2E flavor used by `e2e`.                                       |
| `e2e`                                                      | Run desktop interaction acceptance.                                                         |
| `contracts:check`                                          | Regenerate and fail on tracked or untracked drift in the single generated-output inventory. |
| `contracts:generate`                                       | Regenerate Rust-owned TypeScript/schema output and its Python models together.              |
| `runtime:build`                                            | Produce the controlled Python runtime on its supported host.                                |
| `runtime:provision`                                        | Stage and verify controlled FFmpeg/Python artifacts for desktop development/packaging.      |
| `inference:acceptance`                                     | Score retained acceptance corpus evidence, not synthetic unit fixtures.                     |

`dev` and `build` are Tauri's Vite hooks. Browser preview is not a supported product runtime.
For vendor operations such as test watch mode or compile-only Tauri smoke, use `pnpm exec vitest` or
`pnpm exec tauri build --no-bundle`; these do not need another script alias.

## Desktop

| File                                    | Responsibility and caller                                                                   |
| --------------------------------------- | ------------------------------------------------------------------------------------------- |
| `desktop/dev.mjs`                       | Prepare and verify owned core media, then start Vite; Tauri BeforeDevCommand.               |
| `desktop/package.mjs`                   | Validate target resources, production configuration and final packaged runtimes; `package`. |
| `desktop/verify-windows-signatures.ps1` | Check final application/NSIS Authenticode status and timestamps; release CI.                |

## Bundled runtime

`runtime/` owns production, provisioning and verification of the bundled runtime. Python handles
interpreter/dependency packaging and OS signing; Node integrates manifests and verification with
desktop packaging. They share one owner and call each other where the workflow requires it.

| File                                          | Responsibility and caller                                                                         |
| --------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| `runtime/producer.mjs`                        | Bootstrap pinned build tools and invoke the platform producer; `runtime:build`.                   |
| `runtime/development.mjs`                     | Prepare the selected recipe-bound Python environment; on-demand debug host and `runtime:prepare`. |
| `runtime/build_python.py`                     | Build the controlled interpreter/dependency resource and qualify relocation.                      |
| `runtime/media.mjs`, `runtime/build_media.py` | Prepare or publish the core FFmpeg dependency closure.                                            |
| `runtime/verify_signatures.py`                | Isolated OS signature verification entry; `runtime/signatures.mjs`.                               |
| `runtime/artifacts.py`                        | Interpreter download/hash/size, safe extraction and runtime archive rules.                        |
| `runtime/binaries.py`                         | Enumerate compiled binaries for inventory/signing.                                                |
| `runtime/inventory.py`                        | Record actual installed distributions, notices and source evidence; official CycloneDX output.    |
| `runtime/signing.py`                          | Invoke OS signing/verifying commands under approved identity policy.                              |
| `runtime/__init__.py`                         | Regular Python package marker; no exported facade.                                                |
| `runtime/provision.mjs`                       | Download, verify and activate controlled target archives; `runtime:provision`.                    |
| `runtime/write-manifest.mjs`                  | Enumerate and hash the signed runtime; called automatically by its producer.                      |
| `runtime/files.mjs`                           | Strict runtime path/tree rules and file hashing reused by provisioning/manifests.                 |
| `runtime/manifest.mjs`                        | Validate manifest identity, size limits and the exact installed file set.                         |
| `runtime/provenance.mjs`                      | Bind interpreter/worker/source/build identities to frozen inputs.                                 |
| `runtime/license-policy.mjs`                  | Require explicit license approval bound to the exact inventory and lock.                          |
| `runtime/signatures.mjs`                      | Bind runtime signing identity to desktop production identity before isolated verification.        |
| `runtime/windows-package-verification.mjs`    | Extract final Windows installers and verify the exact contained runtime.                          |

Supplier-specific modules `moss_runtime.py` and `voxcpm.py` install the selected engine’s pinned
resource. `voice_bridge.cpp` / `.cmake` provide the bounded voice streaming ABI. `conda_packages.py`,
`media_files.py` and `windows_process.py` own Conda dependency closure installation, FFmpeg dependency
export and Windows UTF-8 process configuration. `toolchain.mjs` / `transfer.mjs` bootstrap and
verify pinned tools; `publication.mjs` assembles the complete target/profile publication.
`media-manifest.mjs` owns the core-media inventory; `write-media-manifest.mjs` is its Python-to-Node
producer entry. Recipe input lists have one owner in `python/runtime-source.json`; unrelated tooling
is excluded from resource cache identities. `profiles.mjs` binds profile and target identities.

These are publication/verification rules. They are not everyday commands. Library or OS
verification does not establish Knovum's exact target, source, lock, inventory or approval policy.

## Release and updater

| File                          | Responsibility and caller                                                                      |
| ----------------------------- | ---------------------------------------------------------------------------------------------- |
| `release/collect.mjs`         | Collect target artifacts/signatures and frozen platform identity; candidate CI.                |
| `release/candidate.mjs`       | Verify all target artifacts and create one immutable draft; candidate CI.                      |
| `release/promote.mjs`         | Verify approved candidate/public bytes and publish the stable channel; protected promotion CI. |
| `release/policy.mjs`          | Version, platform, endpoint, public key and signing requirements.                              |
| `release/artifacts.mjs`       | Artifact size/hash rules, official Minisign verification and anonymous HTTPS downloads.        |
| `release/github.mjs`          | GitHub CLI calls scoped to the approved distribution repository and stable channel.            |
| `release/update-manifest.mjs` | Official updater manifest shape and immutable version/promotion rules.                         |

Publication entries are called by their workflows. They are not added to the local script menu.
The release contract and approvals are owned by [release.md](../docs/release.md).

## Contract generation and checks

| File                           | Responsibility and caller                                                                                          |
| ------------------------------ | ------------------------------------------------------------------------------------------------------------------ |
| `generate_python_contracts.py` | Assign Rust wire definitions to Python owners and invoke datamodel-code-generator; `contracts:generate`.           |
| `checks/source-layout.mjs`     | Enforce explicit path identities, test placement and source size rules absent from vendor linters; language gates. |
| `checks/capabilities.mjs`      | Reject explicitly prohibited WebView permissions in Tauri capability JSON; Rust/CI gates.                          |

Each directory's `__tests__/` owns its regression tests. `pnpm check:scripts` discovers all Node `.test.mjs` files in those directories; `.integration.mjs` signing/platform tests remain explicitly invoked. Local `check` and the CI Script quality job use this same entry. `scripts/__tests__/test_contracts.py` covers the
standalone contract generator. Python model qualification belongs to `python/tests/acceptance/`.

## Removed duplication

- Platform/release package aliases use `package --bundles …` / `package --release`.
- Fix/check/watch aliases use the vendor flags or CLIs; desktop E2E uses one build command.
- Individual internal layout/capability/dead-code/duplication commands are composed into language gates.
- Runtime manifest creation is an internal producer step.
- The Rust source-token scanner was removed: it recognized selected spellings, not resolved module dependencies.
  The architectural boundary remains a Rust module/code-review requirement.
- The empty background-removal smoke entry, its orphan WDIO configuration and stale CI callers were removed.
- The provisioner reuses the existing runtime file hash implementation.

- The pass-through Tauri launcher and its forwarding tests were removed; `desktop:dev` uses `tauri dev`.
- Thin build/prepare entries were folded into their existing producer/development modules.
- Per-model/tool/runtime path overrides and descriptor-path environment variables were removed;
  resource locations are owned by the application and checkout.
- The unused compile-smoke environment flag and duplicate default 7-Zip path override were removed.

No compatibility aliases, generic command dispatcher or extra task-runner framework are retained.
