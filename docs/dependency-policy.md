# Dependency policy

## Cross-cutting foundation

| Concern                   | Selected implementation         |
| ------------------------- | ------------------------------- |
| UI/runtime                | Vue 3 + Vue Router              |
| State                     | Pinia                           |
| Async/server cache        | TanStack Vue Query              |
| Common Vue behavior       | VueUse                          |
| Runtime validation        | Zod                             |
| Accessible interactions   | Reka UI                         |
| Toast API                 | Vue Sonner                      |
| Icons                     | Lucide Vue                      |
| Styling                   | Tailwind CSS v4 + Knovum tokens |
| Lint                      | Oxlint                          |
| Format/import/class order | Oxfmt                           |
| Dependency boundaries     | dependency-cruiser              |
| Dead code/dependencies    | Knip                            |
| Duplicate code            | jscpd                           |

Oxfmt owns formatting, import ordering, package.json ordering and Tailwind v4 class sorting. There is no second formatter or linter stack.

Oxlint type-aware linting is intentionally not enabled while this foundation is on TypeScript 6. The current Oxc type-aware path requires TypeScript 7 tooling, so `vue-tsc` remains the strict type gate until the project moves to a compatible stable TypeScript/Oxlint combination. Do not add a parallel ESLint stack to fill that gap.

## Desktop foundation

Official Tauri plugins are registered only when a current product flow calls them. Package/plugin installation does not grant WebView permission; guest permissions stay minimal and are expanded only alongside the feature that needs them.

Current desktop capabilities include lifecycle/single-instance, macOS menu-bar status item / Windows Tray, window state, Store, autostart, clipboard, dialogs, filesystem, opener, OS information, notifications, updater and logging. SQLite persistence and embedded migrations are Rust-owned through SQLx; no SQL capability is exposed to the WebView. Provider networking is Rust-owned rather than exposed as a wildcard WebView HTTP capability. Plugins with no current caller are removed rather than pre-provisioned.

Tauri remains the IPC transport. Application commands share a small error-normalization
boundary; official plugin APIs are imported directly. Existing ts-rs owns wire DTO
generation, including capability-owned video-editing requests and events. No parallel RPC
framework, handwritten global command map or custom signature generator is provisioned.

Desktop HTTP and filesystem access remain capability-scoped. SQLite access is Spaces-owned and Rust-only through typed Rust commands and SQLx transactions. The WebView has no SQL read or write permission.

Operating-system framework bindings are added only under a `cfg(target_os)` target dependency block, with non-default features narrowed to the classes the product flow actually names. macOS uses QuickLookThumbnailing for cached document thumbnails and QuickLookUI for the system preview panel; Windows uses the Shell image factory for installed thumbnail handlers. Each binding stays invisible to other platform builds. Adopting a binding is not granting a permission: native preview commands still re-authorize every path against the Space-owned asset directory.

## Feature engines are selected only against real requirements

Spaces uses exact-pinned `konva` and `vue-konva`, confined to `features/spaces/canvas/konva`. Konva owns mounted scene interaction while neutral Knovum contracts own durable canvas facts; no parallel canvas engine is maintained. Spaces rich editing uses Tiptap Vue 3 + StarterKit plus narrowly selected first-party extensions. Do not preinstall additional Git, AI agent/SDK, CRDT or alternative canvas/editor engines before their requirements exist. Starting a feature begins with a fresh GitHub + npm/crates.io benchmark.

Tiptap packages are pinned to one release line as a unit; the current editor integration uses `3.31.1`. Do not mix Tiptap patch lines inside one editor surface.

Tiptap's Markdown extension is currently Beta, so its adoption is deliberately constrained: SQLite Markdown text remains the canonical body contract, only supported round-trippable Markdown structures are enabled, and Knovum does not persist vendor JSON as a parallel compatibility model. Tiptap's Notion-like template, Table of Contents UI and DOCX conversion are commercial/private-registry products and are not dependencies of Knovum's local OSS editor. Knovum derives its compact heading outline from the open editor document model. DOCX viewing fidelity, semantic conversion and editable round-tripping remain different capabilities and must not be treated as one dependency decision. Office ingestion now uses Rust `office_oxide` for plain-text/Markdown extraction so Space search and future RAG indexing do not depend on WebView rendering. Pixel-faithful Office preview remains a separate capability and is not approximated with a custom renderer. Editor search uses Tiptap's first-party Find and Replace extension rather than a parallel ProseMirror search package.

Direct dependencies must have a current caller or satisfy a documented peer/runtime requirement. Knip exceptions are reserved for tooling/runtime cases that static import analysis cannot see; they are not a holding area for speculative packages.

If a future feature needs Shell, WebSocket, updater, notifications, process control, custom window positioning or persisted external file scopes, add the corresponding official Tauri plugin in the same change that defines its capability and product behavior.

## Supply-chain release gates

One narrowly scoped pnpm override addresses the current dependency graph:

- `tauri-plugin-clipboard-api@2.1.11 > valibot` uses `1.4.2`. The plugin uses primitive parsing APIs, verified against the actual plugin's valid and malformed clipboard payloads. Remove the override when the plugin itself selects a supported secure Valibot release and the regression passes.

PptxGenJS and its image-size override are removed: the actual product presentation workflow is Rust-owned, and the former Web worker had no production caller. Do not add a second document serialization engine in the Renderer.

Rust `reqwest` explicitly enables `query`, because provider clients use `RequestBuilder::query`. Do not rely on unrelated SDK dependencies to enable required features transitively. `async-openai`, direct `rig-core` and direct `futures-util` have no current Knovum caller and are not direct dependencies; `rig-core` remains a dependency of the actively used `rig-agent` SDK.

- CI installs from the committed pnpm/Cargo lockfiles only.
- Production npm dependencies are checked with `pnpm audit --prod --audit-level high`; registry package signatures are checked with `pnpm audit signatures`.
- Rust advisories and dependency licenses are checked in CI with `EmbarkStudios/cargo-deny-action@v2`; `deny.toml` is the explicit license allowlist and changes to it require release/legal review.
- Dependabot remains the scheduled update signal; passing advisory checks does not replace the OSS-first maintenance/license review for newly introduced dependencies.
