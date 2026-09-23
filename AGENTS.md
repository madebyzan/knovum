# Knovum agent instructions

Knovum is a Tauri 2 + Vue 3 local-first desktop knowledge space: one desktop product, not a browser dashboard or a container for peer apps. Product work belongs to the owning capability in `src/features/*`.

## Working in this repository

Understand the affected capability and its contracts, then make the requested change through implementation and relevant verification. Use the authoritative documents below when the change touches their subject; a small, local edit does not require a full repository review.

Before adding a non-trivial capability or custom subsystem, follow `docs/oss-first.md` and `docs/dependency-policy.md`: compare maintained GitHub and npm/crates.io options, prefer Tauri official plugins for OS integration and Reka/TanStack/Vue primitives for frontend behavior, and record meaningful choices in the PR evidence fields. Import vendor APIs directly unless Knovum needs policy, validation, security, domain translation, or orchestration; build custom infrastructure only when mature candidates fail an explicit requirement. Do not add speculative dependencies. Spaces' exact-pinned Konva and vue-konva releases were selected by a canvas-platform benchmark; replacing them requires a fresh benchmark.

## Architecture

`docs/architecture.md` and `.dependency-cruiser.cjs` own dependency rules:

- `app/` composes the product and sets application-wide policy; `foundation/` owns cross-cutting infrastructure; `shell/` owns desktop workbench chrome.
- `features/<name>/` owns business behavior; `shared/contracts/` holds stable neutral contracts; `shared/ui/` holds durable Knovum visual compositions.
- Features do not import other features, `app/`, or `shell/`. Avoid catch-all `services`, `utils`, `helpers`, `composables`, or `platform` directories, barrel `index.ts` files, rename-only vendor wrappers, and universal services/composables spanning unrelated responsibilities.
- Keep files focused. Existing gates cap TS/Vue files at 180 lines and complexity at 10; review function size by responsibility and complexity.

## Vue, data, and security

- Use Vue 3 Composition API with `<script setup lang="ts">`. Keep visual state local, cross-view session state in Pinia, and remote/async cache in TanStack Vue Query.
- Persist non-secret preferences in the strict current v1 Tauri Store envelope. The last workspace route is a disposable localStorage session value. Feature structured data uses feature-owned SQLite v1 baselines, one per database before first public release; do not create a global `DatabaseService`. Credentials and tokens use the OS credential store through foundation secret commands; never put them in Pinia persistence, localStorage, SQLite, logs, or query cache.
- Validate user input and disk, network, IPC, and external-file boundaries. Static data owned and typed by this repository does not need defensive runtime checks.
- Required infrastructure fails visibly. Handle expected cancel, permission denial, unavailable-shortcut, and malformed-input outcomes. Reject or explicitly reset obsolete pre-release data; do not silently reinterpret it or add compatibility readers, migrations, or generic fallback layers.

## UI and desktop behavior

For UI work, use `docs/ui-system.md` and its token source, `src/styles/tokens.css`. Prefer Tailwind v4 semantic utilities; add tokens for repeated roles and reserve arbitrary values for instance-specific geometry. Reka owns primitive behavior; add `shared/ui` only for durable Knovum visual compositions. Keep the compact, low-chroma, keyboard-first desktop language without marketing-page layouts.

- macOS is Dock-first with the native Application Menu. Windows uses Taskbar + Tray and has no legacy File/Edit menu bar by default.
- Preserve native window controls: macOS may use the documented overlay titlebar/material profile, while Windows keeps the native caption. Replacing traffic lights or caption buttons with HTML controls requires an ADR.
- Use OS-native dialogs, notifications, credential stores, menus, taskbar/Dock progress, and shortcuts when Tauri exposes them. Scope Shell, filesystem, HTTP, WebSocket, updater, and custom-command permissions to the feature's needs. A new window needs a use case, lifecycle, routing/context model, and dedicated capability scope.
- The main window starts hidden. Reveal it only after bootstrap reaches a usable state or the recoverable bootstrap-error state. Launch feedback belongs to the Dock/taskbar; do not add a splash WebView.
- `AppCommand` is the command-palette and application-local shortcut contract; features do not add global `keydown` listeners for those commands. Long Rust jobs use Tauri Channels and the shared Task Center bridge; the starting feature/engine owns cancellation. Extend that bridge only for concrete cross-feature needs.

## Verification and documentation

Use Oxc tooling; do not add ESLint or Prettier. Run focused checks for the changed behavior and fix failures caused by the change. For substantial web or Rust changes, use `pnpm check:web` or `pnpm check:rust` respectively; `pnpm check` covers both. Release and desktop-lifecycle changes also need real Tauri builds on macOS and Windows. E2E selectors must not depend on translated visible strings.

Update the authoritative document when its contract changes:

| Change | Document |
| --- | --- |
| System/runtime topology | `docs/system-architecture.md` |
| Dependency boundaries | `docs/architecture.md` |
| UI/design tokens | `docs/ui-system.md` |
| Desktop behavior | `docs/desktop-foundation.md`, `docs/platform-matrix.md` |
| Dependency policy | `docs/oss-first.md`, `docs/dependency-policy.md` |
| Packaging/release | `docs/packaging.md`, `docs/release.md` |

Keep each contract in its owning document rather than creating a second version.
