# Knovum agent instructions

This file is the repository-level contract for coding agents and AI-assisted changes. It applies to the entire repository unless a deeper `AGENTS.md` explicitly narrows a rule for a real subsystem.

## Product

Knovum is a Tauri 2 + Vue 3 local-first desktop knowledge space. It is one desktop product,
not a browser dashboard or a container for peer applications. Product work belongs to the owning
capability under `src/features/*`.

## Required workflow

1. Understand the feature requirements and the owning module before changing code.
2. For every non-trivial capability, research stable, actively maintained solutions on GitHub and npm/crates.io first.
3. Prefer Tauri official plugins for OS integration and Reka/TanStack/Vue ecosystem primitives for frontend behavior.
4. Record meaningful dependency choices or custom implementations in the PR using the OSS-first evidence fields.
5. Add only the minimum Tauri capability/scope required by the feature.
6. Run the relevant quality gates before handoff.

Do not implement custom infrastructure because it is "small" or because a library API name is inconvenient. Custom code is justified only when mature candidates fail an explicit requirement.

## Architecture boundaries

Authoritative dependency rules live in `docs/architecture.md` and `.dependency-cruiser.cjs`.

- `app/`: composition root and application-wide policy.
- `foundation/`: cross-cutting product infrastructure.
- `shell/`: desktop workbench chrome.
- `features/<name>/`: business ownership.
- `shared/contracts/`: stable neutral contracts.
- `shared/ui/`: durable Knovum visual compositions only.

Forbidden by default:

- feature-to-feature imports;
- feature imports from `app/` or `shell/`;
- catch-all `services`, `utils`, `helpers`, `composables`, or `platform` directories;
- barrel `index.ts` files;
- rename-only wrappers around vendor APIs;
- a universal service/composable that owns unrelated responsibilities.

Keep files small and responsibilities explicit. Current gates are ceilings, not targets: 300 lines per TS/Vue file and complexity 10. Prefer cohesive ownership over splitting a responsibility only to satisfy a line count. Function length is reviewed by responsibility and complexity rather than a physical line-count limit.

## Vue and TypeScript

- Use Vue 3 Composition API with `<script setup lang="ts">`.
- Use direct vendor imports unless Knovum owns real policy or orchestration.
- Component-local visual state stays local.
- Cross-view session state uses Pinia.
- Remote/async cache uses TanStack Vue Query.
- Persisted non-secret preferences use the strict current v1 Tauri Store schema during pre-release development.
- Secrets never enter Pinia persistence, localStorage, SQLite, logs, or query cache.
- Validate real boundaries: user input, disk, network, IPC, and external files.
- Do not add defensive runtime checks for static data entirely owned and typed by this repository.

## UI and Tailwind CSS v4

`src/styles/tokens.css` is the design-token source. Follow `docs/ui-system.md`.

- Use Tailwind v4 semantic utilities generated from `@theme` tokens.
- Prefer `bg-canvas`, `text-body`, `text-foreground-secondary`, `rounded-control` and `shadow-popover`.
- Do not use raw `var()` utilities when a token exists.
- Do not introduce arbitrary pixel typography or icon sizes.
- Add a semantic token when a size/color/elevation is a repeated desktop role.
- Arbitrary values are only for genuinely instance-specific geometry or runtime values owned by a primitive.
- Reka owns focus, keyboard behavior, ARIA, portals, collision, and primitive state. Do not reimplement these behaviors.
- Do not wrap Reka only to rename props/events. Add `shared/ui` only for a durable Knovum visual composition.
- Preserve the Apple/Linear-like desktop language: compact, low-chroma, low-radius, restrained elevation, keyboard-first, no marketing-page layouts.

## Desktop platform behavior

- macOS is Dock-first and uses the native Application Menu.
- Windows uses Taskbar + Tray and does not show a legacy File/Edit menu bar by default.
- Preserve native window controls. macOS may use the documented overlay titlebar/material profile; Windows keeps the native caption. Do not replace traffic lights/caption buttons with HTML controls without a new ADR.
- Use OS-native dialogs, notifications, credential stores, menus, taskbar/Dock progress, and shortcuts where Tauri already exposes them.
- Do not broaden Shell, filesystem, HTTP, WebSocket, updater, or custom-command permissions preemptively.
- New windows require a real use case, explicit lifecycle, routing/context model, and dedicated capability scope.

## Startup

The main window starts hidden. Bootstrap must always reach either a usable application state or the recoverable bootstrap-error state before revealing it. Do not add an intermediate splash WebView; Dock/taskbar launch feedback remains platform-owned.

## Commands and background work

- `AppCommand` is the single command contract for command palette and application-local shortcuts.
- Do not add feature-level global `keydown` listeners for application commands.
- Long Rust jobs use Tauri Channels and the shared Task Center bridge.
- Cancellation remains owned by the feature/engine that started the job.
- Do not create a universal background-job framework unless concrete cross-feature requirements prove the current bridge insufficient.

## Persistence and data

- Preferences: strict current v1 Tauri Store envelope; no pre-release compatibility readers/migrations.
- Last workspace route: disposable localStorage session value.
- Feature structured data: feature-owned SQLite v1 baseline; one baseline per database before first public release.
- Credentials/tokens: OS credential store through the foundation secret commands.
- Never create a global `DatabaseService` for all product domains.

## Dependencies

Follow `docs/oss-first.md` and `docs/dependency-policy.md`.

Do not install speculative Git, editor, AI, CRDT, search, or alternative canvas engines before feature requirements are known. Spaces uses exact-pinned Konva and vue-konva releases selected by the canvas-platform benchmark; replacing them still requires a fresh benchmark. "May use later" is not a dependency requirement.

## Errors and compatibility

- No generic compatibility/fallback layer.
- Required infrastructure fails visibly.
- Expected platform/user outcomes are handled normally: cancel, permission denial, unavailable shortcut, malformed external input.
- Never silently reinterpret incompatible persisted data. Before first public release, reject/reset obsolete development data explicitly; do not add compatibility migrations or readers.

## Quality gates

Use Oxc tooling only; do not add ESLint or Prettier.

```bash
pnpm check:web
pnpm check:rust
pnpm check
```

Before release or desktop-lifecycle changes, also validate a real Tauri build on macOS and Windows. E2E selectors must not depend on translated visible strings.

## Documentation

Update the authoritative document when a contract changes:

- system/runtime topology: `docs/system-architecture.md`;
- dependency rules: `docs/architecture.md`;
- UI/design tokens: `docs/ui-system.md`;
- desktop behavior: `docs/desktop-foundation.md` and `docs/platform-matrix.md`;
- dependency policy: `docs/oss-first.md` / `docs/dependency-policy.md`;
- packaging/release: `docs/packaging.md` / `docs/release.md`.

Do not create a second document that restates the same rules with different terminology.
