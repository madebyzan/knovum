# Knovum documentation map

Use one authoritative document per concern.

## Start and contribute

- `../README.md` — project overview and quick start.
- `../CONTRIBUTING.md` — team contribution workflow.
- `../AGENTS.md` — repository contract for coding agents and AI-assisted work.
- `../DESIGN.md` — canonical product, interaction and Knowledge Derivation design contract.
- `installation.md` — developer setup and end-user installation expectations.
- `troubleshooting.md` — common local/build/runtime failures.

## Architecture

- `system-architecture.md` — process/runtime topology, startup, storage, IPC, security and platform lifecycle.
- `architecture.md` — source dependency direction and ownership boundaries.
- `desktop-foundation.md` — Tauri/desktop capability inventory.
- `background-tasks.md` — long-running work and Channel bridge.
- `commands.md` — command palette and shortcut contract.
- `data-ownership.md` — Store/SQLite/session/secret ownership.
- `spaces.md` — Space/Space product model, capture behavior and persistence boundary.
- `knowledge.md` — current Spaces Knowledge, SQLx baseline, revision/citation/package contracts.
- `document-conversion.md` — Document conversion routes, backend behavior, limitations and verification.
- [secrets.md](secrets.md) — dedicated local credential database and its security properties.
- `platform-matrix.md` — macOS/Windows behavior matrix.

## Product engineering standards

- `engineering-standards.md` — development/review baseline and links to authoritative detailed rules.
- `ui-system.md` — desktop visual language and Tailwind v4 token contract.
- `oss-first.md` — open-source-first selection requirement.
- `dependency-policy.md` — dependency lifecycle and maintenance policy.
- `local-models.md` — signed runtime vs explicit model-download ownership and model inventory.

## Packaging and release

- `packaging.md` — local/CI bundles and installer artifacts.
- `release.md` — production signing, notarization, updater and release procedure.

## Naming

- [naming-conventions.md](naming-conventions.md) — authoritative path, symbol, IPC and suffix naming policy.

The current pre-release data baseline is defined in [data-ownership.md](data-ownership.md) and [knowledge.md](knowledge.md). Release prerequisites and acceptance belong to [release.md](release.md). Candidate-specific test reports, target-platform probes and dependency audits belong to the corresponding CI or release artifacts, rather than a second documentation tree.
