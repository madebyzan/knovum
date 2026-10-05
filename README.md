# Knovum

Knovum is a Tauri 2 + Vue 3 local-first desktop knowledge space. It combines an infinite canvas,
structured knowledge, rich documents, mixed media, retrieval and traceable AI derivations in one
application.

Author: Zan.

## Foundation stack

- Vue 3 + TypeScript + Vite
- Tailwind CSS v4 `@theme` desktop tokens
- Reka UI interaction/accessibility primitives + Vue Sonner
- Vue I18n with system locale support
- Pinia + TanStack Vue Query + TanStack Virtual + VueUse + Zod
- Tauri 2 official desktop plugins plus OS keyring secrets through Rust
- Oxlint + Oxfmt + dependency-cruiser + Knip + jscpd
- Vitest + Vue Test Utils + Tauri WebdriverIO E2E setup

## Desktop settings already owned by the foundation

- System / Light / Dark, synchronized with the Tauri window theme
- Small / Default / Large semantic desktop type ramp
- Compact / Comfortable control density
- System / Simplified Chinese / English locale
- launch at login
- persisted window/workspace state
- macOS Application Menu + menu-bar status item; Windows Tray/Taskbar without a legacy menu bar

## Start

Knovum is a desktop application. A plain browser is not a supported runtime target.

```bash
corepack enable
pnpm install
pnpm desktop:dev
```

Full workstation and installer guidance: `docs/installation.md`.

After the first successful install/build, commit both `pnpm-lock.yaml` and `src-tauri/Cargo.lock`.

## Daily commands

| Task                                | Command            |
| ----------------------------------- | ------------------ |
| Develop the desktop app             | `pnpm desktop:dev` |
| Verify all code                     | `pnpm check`       |
| Run frontend unit tests             | `pnpm test`        |
| Format frontend/configuration files | `pnpm format`      |
| Build the host's QA installers      | `pnpm package`     |

Use flags for variants: `pnpm lint --fix`, `pnpm format --check`,
`pnpm package --bundles msi`, or `pnpm package --release` with production signing configured.
Focused gates remain `check:web`, `check:rust`, and `check:python`.
`dev` and `build` are Vite hooks used by Tauri, not independent supported product runtimes.
Specialized runtime, E2E and contract commands and every internal tool are described in
[tools/README.md](tools/README.md). Signing and native acceptance follow
[docs/packaging.md](docs/packaging.md) and [docs/release.md](docs/release.md).

## Gates

```bash
pnpm check:web
pnpm check:rust
pnpm check
```

`check:web` runs Vue/TypeScript type checking, Oxlint, dependency graph rules, Vitest, Knip, jscpd and Oxfmt verification. `check:rust` runs rustfmt, Clippy with warnings denied and Rust tests. The Quality workflow additionally runs production npm advisory/signature checks and Rust advisory checks before a release commit can be promoted. Desktop-affecting PRs also compile-check macOS and Windows through `.github/workflows/platform-smoke.yml`.

Before feature work, read `AGENTS.md` and `docs/README.md`. The documentation map points to the authoritative architecture, UI, platform, OSS-first, packaging and release contracts.

## Foundation 1.0 freeze

See `docs/foundation-1.0.md`. This RC includes the hidden-main startup lifecycle, QA packaging workflow and engineering/agent documentation; it is business-development ready; promote it to 1.0 only after lockfiles are committed and the macOS/Windows CI smoke build is green.
