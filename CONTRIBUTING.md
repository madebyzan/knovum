# Contributing to Knovum

## Start here

Read these before the first product change:

1. `AGENTS.md`
2. `docs/architecture.md`
3. `docs/system-architecture.md`
4. `docs/ui-system.md`
5. `docs/engineering-standards.md`
6. `docs/oss-first.md`

## Local setup

Follow `docs/installation.md`. The supported workflow is the Tauri desktop runtime; browser-only development is not a product runtime.

```bash
corepack enable
pnpm install --frozen-lockfile
pnpm desktop:dev
```

Dependency changes must include the corresponding committed lockfile changes.

## Feature workflow

A product feature starts in `src/features/<feature>`.

Before adding a non-trivial dependency or custom implementation:

- write the exact requirements;
- check stable, maintained GitHub/npm/crates.io candidates;
- prefer an existing solution that satisfies the requirements;
- record rejected candidates and the exact gap if custom code is required;
- review license, release cadence, security surface, Tauri compatibility, bundle cost, and ownership.

Do not add speculative engines or capabilities.

## Pull requests

Use `.github/PULL_REQUEST_TEMPLATE.md`. Keep one PR focused on one coherent change. Architecture, permission, schema, persistence, or dependency changes require the relevant documentation update in the same PR.

Required before review:

```bash
pnpm check
```

Desktop lifecycle and capability changes also require macOS and Windows smoke builds.

## UI review

Product UI must follow `docs/ui-system.md`:

- semantic Tailwind v4 tokens;
- Reka behavior primitives;
- no arbitrary typography/icon sizes for normal UI;
- keyboard interaction and visible focus;
- light/dark, text-size, density, and locale behavior verified;
- macOS and Windows platform conventions checked separately.

## Database changes

Feature-owned SQLite migrations are append-only once shipped. Never edit an already released migration. Data ownership rules are in `docs/data-ownership.md`.

## Security-sensitive changes

Treat these as security review triggers:

- new Tauri permissions/capabilities;
- Shell execution;
- filesystem scope expansion;
- HTTP/WebSocket origin expansion;
- secrets/credential changes;
- file-association parsing;
- updater/signing changes;
- new webview/window types.

See `SECURITY.md`.
