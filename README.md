# Knovum

Knovum is a Tauri 2 + Vue 3 local-first desktop knowledge space. It combines an infinite canvas,
structured knowledge, rich documents, mixed media, retrieval and traceable AI derivations in one
application.

Author: Zan.

## License

Knovum is source-available under the custom [Knovum Personal Noncommercial License 1.0](LICENSE).
Personal noncommercial use, modification and free sharing are permitted under its terms.
Organizational use, professional work and commercialization require separate prior written authorization.
Third-party components retain their own licenses.

许可说明与使用示例：[个人非商业使用与商业授权](docs/licensing.md)。

## Foundation stack

- Vue 3 + TypeScript + Vite
- Tailwind CSS v4 `@theme` desktop tokens
- Reka UI interaction/accessibility primitives + Vue Sonner
- Vue I18n with system locale support
- Pinia + TanStack Vue Query + TanStack Virtual + VueUse + Zod
- Tauri 2 official desktop plugins plus a dedicated local credential database through Rust
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
pnpm install --frozen-lockfile
pnpm desktop:dev
```

Full workstation and installer guidance: `docs/installation.md`.

Dependency changes must include the corresponding pnpm and Cargo lockfile changes.

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
[scripts/README.md](scripts/README.md). Signing and target-platform acceptance follow
[docs/packaging.md](docs/packaging.md) and [docs/release.md](docs/release.md).

## Gates

```bash
pnpm check:web
pnpm check:rust
pnpm check
```

`check:web` runs Vue/TypeScript type checking, Oxlint, dependency graph rules, Vitest, Knip, jscpd and Oxfmt verification. `check:rust` runs rustfmt, Clippy with warnings denied and Rust tests. The Quality workflow additionally runs production npm advisory/signature checks, Rust advisory checks, signing rejection tests and desktop compile checks on macOS arm64/x64 and Windows x64 before a release commit can be promoted.

Before feature work, read `AGENTS.md` and `docs/README.md`. The documentation map points to the authoritative architecture, UI, platform, OSS-first, packaging and release contracts.

## Release readiness

Production acceptance, signing, managed-runtime qualification and update promotion follow
[docs/release.md](docs/release.md). A successful source build alone does not establish release readiness.
