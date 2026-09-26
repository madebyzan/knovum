# Engineering standards

This is the review baseline. Detailed rules remain in their authoritative documents; do not copy them into feature-specific guides.

## Architecture

- Respect the dependency graph in `architecture.md`.
- Keep feature ownership explicit and feature-to-feature dependencies forbidden by default.
- Prefer direct mature vendor APIs over rename-only wrappers.
- Split files by real responsibility, not by arbitrary folder ceremony.
- Authored TS/Vue and Rust files have a 180-effective-line ceiling. Split by real responsibility, never by adding forwarding wrappers. Generated contracts are checked separately. Review responsibility and complexity before reaching that ceiling.
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
- Credentials use the dedicated local SQLite database through Rust secret commands.
- Capabilities are least privilege and window-scoped.

## Names and boundaries

- Rust uses snake_case modules/functions, PascalCase types and explicit serde names for stable external IDs. Web uses kebab-case TS files, PascalCase Vue components and `use` only for reactive lifecycle APIs.
- A source file must not coexist with a same-stem sibling directory; Rust directory entry points are `mod.rs`. Keep fixtures under `test-support`; only executable test suites use `.test.ts`.
- Name by the actual operation or owned concept. `client` denotes an actual SDK/network client with request or connection lifecycle (for example QueryClient or reqwest Client). Ordinary IPC functions use operation names; projection decoding does not make a Web module a repository. A repository owns actual persistence access or policy. Do not add a service/manager/adapter suffix to a forwarding function.
- Application IPC errors follow the envelope and normalization boundary in `architecture.md`. Cancellation decisions use stable codes, not message substrings. Log at the owner that can add context; cleanup failures do not overwrite the original failure or a successful commit.
- Keep committed Space facts in the single session projection. Derive display values directly; retain drafts only with their Space/node identity and an explicit reset lifetime, as defined in `spaces.md`.
- A suppression requires a concrete limitation. Schema-only types and platform-only fields may explain unused fields; working application commands must not use blanket unused-import or dead-code suppressions.

- Rust owns IPC wire DTOs through ts-rs; neutral generated types live in `shared/contracts/generated`; Spaces business contracts live in `features/spaces/contracts/generated`. Runtime validation remains at external boundaries and is anchored to generated types. UI projections may differ deliberately, but must not copy a wire interface. Runtime enum values and shared limits are generated from the same Rust source. Generated output is excluded from authored-code lint/format rules; it must remain deterministic and pass TypeScript checks.
- Production reachability starts at `main.ts` and actual workers; tests alone do not justify keeping product code.

## Testing and gates

```bash
pnpm check:web
pnpm check:rust
pnpm check
```

The repository uses Oxlint/Oxfmt, dependency-cruiser, Knip, jscpd, Vitest, Vue Test Utils, rustfmt, Clippy and Rust tests. Desktop-affecting changes also require real macOS/Windows builds.

## Documentation required with code changes

Update documentation in the same PR for changes to architecture, design tokens, permissions, persistence schema, commands, background task contracts, packaging, or release behavior.
