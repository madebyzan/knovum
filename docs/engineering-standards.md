# Engineering standards

This is the review baseline. Detailed rules remain in their authoritative documents; do not copy them into feature-specific guides.

## Before feature implementation

Define the user outcome, owning capability, state and persistence boundaries, desktop permissions,
and whether execution must continue while the WebView is hidden. Compare maintained solutions
under [oss-first.md](oss-first.md) before adding a non-trivial dependency or custom subsystem.
Identify keyboard/focus behavior, progress and cancellation, and the focused verification needed
to prove the feature's contract. The sections below own the implementation and review baseline.

## Architecture

- Respect the dependency graph in `architecture.md`.
- Keep feature ownership explicit and feature-to-feature dependencies forbidden by default.
- Prefer direct mature vendor APIs over rename-only wrappers.
- Split files by real responsibility, not by arbitrary folder ceremony.
- Authored TS/Vue files have Oxlint's 180-effective-line ceiling. Split all code, including Rust, by real responsibility rather than by adding forwarding wrappers. Generated contracts are checked separately. Review responsibility and complexity before reaching that ceiling.
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
- Keep module and test ownership obvious. Prefer conventional Rust module paths and use `mod.rs` for directory modules. Keep reusable TypeScript fixtures under the owning `__tests__/support/`; executable TypeScript suites use `.test.ts`.
- Path, symbol, IPC and suffix rules are defined only in [naming-conventions.md](naming-conventions.md). Name by the actual operation or owned concept; do not add a service/manager/adapter suffix to a forwarding function.
- Application IPC errors follow the envelope and normalization boundary in `architecture.md`. Cancellation decisions use stable codes, not message substrings. Log at the owner that can add context; cleanup failures do not overwrite the original failure or a successful commit.
- Keep committed Space facts in the single session projection. Derive display values directly; retain drafts only with their Space/node identity and an explicit reset lifetime, as defined in `spaces.md`.
- A suppression requires a concrete limitation. Schema-only types and platform-only fields may explain unused fields; working application commands must not use blanket unused-import or dead-code suppressions.

- Rust owns IPC wire DTOs through ts-rs; neutral generated types live in `shared/contracts/generated`; Spaces business contracts live in `features/spaces/contracts/generated`. Runtime validation remains at external boundaries and is anchored to generated types. UI projections may differ deliberately, but must not copy a wire interface. Runtime enum values and shared limits are generated from the same Rust source. Generated output is excluded from authored-code lint/format rules; it must remain deterministic and pass TypeScript checks.

## Testing and gates

```bash
pnpm check:scripts
pnpm check:web
pnpm check:rust
pnpm check:python
pnpm check
```

Production Renderer and Node-backed test compilation use separate `tsconfig.json` and `tsconfig.tests.json` projects; both run in `pnpm typecheck`. Tests are not hidden from checking to avoid Node type errors.

Dependency-cruiser proves the encoded Renderer dependency rules. Rust visibility and behavioral tests constrain module APIs, while Rust/Tauri compilation resolves handlers and generated ACLs. Capability policy checks cover the explicit permissions they inspect. Rust ownership boundaries remain explicit code-review requirements: `local_resources` must not import AI, Run scheduling must not import concrete capabilities, authorization must not read the Action catalog, and captured Workflow execution must not reclaim generic dispatch. String scans cannot resolve Rust aliases and can mistake comments for dependencies, so they are not architecture gates. Neither compilation nor Knip certifies all cross-language dependency directions or runtime grants; review additions at their owner, including the images/Spaces authorization boundary.

The repository uses Oxlint/Oxfmt, dependency-cruiser, Knip, jscpd, Vitest, Vue Test Utils, rustfmt, Clippy and Rust tests. Desktop-affecting changes also require real macOS/Windows builds.

## Documentation required with code changes

Update documentation in the same PR for changes to architecture, design tokens, permissions, persistence schema, commands, background task contracts, packaging, or release behavior.

Rust compile/test quality runs on supported macOS; the platform smoke matrix covers macOS arm64, macOS x64 and Windows x64, including Python-only build inputs. Dependency audit targets match these three release triples. PR Python audits reuse the production profile inventory and audit all five locked runtimes plus the root project. Script unit tests use one local/CI entry; signing integration, real models and release acceptance remain separate. Generated-contract drift uses the single owner output list, including Model Resources Python output.
