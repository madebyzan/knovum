# Engineering standards

This is the review baseline. Detailed rules remain in their authoritative documents; do not copy them into feature-specific guides.

## Architecture

- Respect the dependency graph in `architecture.md`.
- Keep feature ownership explicit and feature-to-feature dependencies forbidden by default.
- Prefer direct mature vendor APIs over rename-only wrappers.
- Split files by real responsibility, not by arbitrary folder ceremony.
- Treat ~250 lines of hand-written feature/runtime code as a review trigger: split when more than one responsibility is present. Generated/vendor/schema data are exceptions, not a reason to grow business modules.
- DRY applies to duplicated knowledge and product rules, not to every repeated line of code.
- SOLID is used as a design pressure, not a reason to add interfaces before there are multiple responsibilities or implementations.

## Implementation

- Research stable maintained OSS before non-trivial custom implementation.
- Do not add fallback behavior for static internal invariants.
- Validate real external/persistence boundaries.
- Do not install speculative dependencies.
- Do not widen desktop permissions before the business feature needs them.

## Frontend

- Vue 3 `<script setup lang="ts">`.
- Tailwind CSS v4 semantic tokens from `tokens.css`.
- Reka for interactive/accessibility primitives.
- Pinia for cross-view session state; TanStack Vue Query for async/server state.
- No arbitrary pixel typography or icon sizes for ordinary product UI.
- Verify theme, text size, density, locale, reduced motion, keyboard navigation, and focus state.
- Durable/background/Rust/persistence failures go through `@tauri-apps/plugin-log`; user-action failures also surface through product UI when actionable. Browser `console.*` is reserved for deliberate development-only diagnostics.

## Rust

- Prefer official Tauri plugins.
- macOS and Windows behavior are designed separately where platform conventions differ.
- Long-running Rust work streams through Tauri Channels into the Task Center.
- Credentials use OS credential stores.
- Capabilities are least privilege and window-scoped.

## Testing and gates

```bash
pnpm check:web
pnpm check:rust
pnpm check
```

The repository uses Oxlint/Oxfmt, dependency-cruiser, Knip, jscpd, Vitest, Vue Test Utils, rustfmt, Clippy and Rust tests. Desktop-affecting changes also require real macOS/Windows builds.

## Documentation required with code changes

Update documentation in the same PR for changes to architecture, design tokens, permissions, persistence schema, commands, background task contracts, packaging, or release behavior.
