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
- `knowledge.md` — current Spaces Knowledge, SQLx migrations, revision/citation/package contracts.
- `document-export.md` — Markdown export subset, backend behavior, limitations and verification.
- [secrets.md](secrets.md) — dedicated local credential database and its security properties.
- `platform-matrix.md` — macOS/Windows behavior matrix.

## Product engineering standards

- `engineering-standards.md` — development/review baseline and links to authoritative detailed rules.
- `ui-system.md` — desktop visual language and Tailwind v4 token contract.
- `oss-first.md` — open-source-first selection requirement.
- `dependency-policy.md` — dependency lifecycle and maintenance policy.
- `feature-checklist.md` — checklist before a feature is considered complete.
- `local-models.md` — signed runtime vs explicit model-download ownership and model inventory.

## Packaging and release

- `packaging.md` — local/CI bundles and installer artifacts.
- `release.md` — production signing, notarization, updater and release procedure.
- `foundation-1.0.md` — foundation freeze criteria.

## Design, naming and baseline decisions

- [naming-conventions.md](naming-conventions.md) — authoritative path, symbol, IPC and suffix naming policy.
- [spaces-resource-presentation.md](spaces-resource-presentation.md) — resource presentation contract.
- [rfcs/knowledge-work-system.md](rfcs/knowledge-work-system.md) — dated execution/domain baseline and implementation-state map; current owner documents take precedence.

The current pre-release data baseline is defined in [data-ownership.md](data-ownership.md) and [knowledge.md](knowledge.md); the freeze criteria are in [foundation-1.0.md](foundation-1.0.md).
