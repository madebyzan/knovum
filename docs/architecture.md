# Architecture

This document owns source dependency direction and code ownership. Runtime topology belongs to
[system-architecture.md](system-architecture.md), product behavior to [spaces.md](spaces.md), local
model policy to [local-models.md](local-models.md), interface behavior to
[ui-system.md](ui-system.md), and path and identifier rules to
[naming-conventions.md](naming-conventions.md).

## Dependency direction

```text
main.ts / KnovumApp.vue    composition root
        |
        +--> app/          application composition and app-wide policy
        |     +--> features/*
        |     +--> foundation/*
        |     +--> shared/*
        |
        +--> shell/        stable desktop workspace chrome
        |     +--> foundation/*
        |     +--> shared/*
        |
        +--> features/*    product capabilities
        |     +--> foundation/*
        |     +--> shared/*
        |
        +--> foundation/*  cross-cutting product infrastructure
              +--> shared/*

shared/* -----------------> shared/* only
```

Features do not import other features, `app/`, or `shell/`. `shared/` is neutral and cannot import
product owners. Application composition may connect isolated capabilities through explicit props,
contracts, and injected operations; it must not create a global registry that bypasses ownership.

`dependency-cruiser` enforces the frontend graph and presentation boundaries. Oxlint owns source
quality and restricted imports; it is not a dependency-graph engine.

The Spaces core/presentation rule exempts tests in any capability-owned `__tests__/` directory.
All other dependency rules remain in force, including the prohibition on production imports of tests.

## Ownership

- `app/` owns the router, application-wide policy, workspace-session wiring, diagnostics, bootstrap
  orchestration, and composition between otherwise isolated owners.
- `foundation/` owns cross-cutting infrastructure used by several product capabilities: bootstrap,
  application commands, i18n, preferences, secrets, AI connection settings, runtime-resource
  presentation, and the Task Center bridge.
- `shell/` owns stable desktop workbench chrome: workspace layout, sidebar, inspector, command
  surface, Task Center, and application/feature error boundaries.
- `features/<name>/` owns one product capability and its presentation, state, operations, validation,
  and persistence adapters.
- `shared/contracts/` owns stable neutral contracts with several consumers.
- `shared/ui/` owns durable Knovum visual compositions. Reka remains the primitive behavior owner.
- `src-tauri/src/desktop/` owns desktop lifecycle that must remain correct while the WebView is hidden
  or unavailable.
- `src-tauri/src/app_builder/` composes producer task observation. Desktop task presentation consumes
  neutral snapshots and does not open feature databases or enumerate their work.
- `src-tauri/src/job_registry/` owns job leases and atomic maintenance admission. Engines expose
  reservations for their own registries; `backup/admission.rs` holds them while composing
  reconciliation, durable checks and restart scheduling. Resource preparation retains its own
  registry because it also owns completed task snapshots.

Application routes and the remembered workspace destination belong to `app/`. The shell receives
workspace visibility and emits navigation intent; feature views receive explicit navigation
operations from application composition. Neither shell nor foundation chooses product route names.
Shared AI language labels belong to the AI locale namespace rather than a Space editor namespace.
Workspace command composables receive explicit shell presentation operations during shell setup;
they do not resolve descendant providers from the application root. The application router commits
the resident workspace before leaving its destination; the resident view owns window-close commits.

Directories express stable ownership, not temporary implementation stages. Start a capability flat
and split it only when distinct responsibilities exist. Once split, move its implementation under
the owner instead of retaining prefixed siblings.

Do not create catch-all feature directories such as `application`, `domain`, `infrastructure`,
`services`, `utils`, `helpers`, `composables`, `common`, `misc`, or `platform`. A feature's
`session/` directory is reserved for its Pinia composition root and cross-capability actions;
business policy remains with the capability that owns it. `model/` holds shared pure models and
invariants within one owner, `schema/` holds executable structural validation, and `contracts/`
holds explicit implementation boundaries or generated wire types. These names do not create an
independent domain layer.

Direct imports preserve the real owner. Barrel files, rename-only wrappers, and generic type
containers do not create architecture boundaries. Use Vue, Reka, VueUse, TanStack, official Tauri
plugins, and other selected vendor APIs directly unless a local boundary owns Knovum policy,
validation, security, domain translation, or real multi-API orchestration.

Use `cloneDeep` from the existing `es-toolkit` dependency for independent editable copies of
repository-owned data, including Vue reactive data. Do not add a clone wrapper or use serialization
as cloning. Built-in `structuredClone` remains appropriate when platform structured-clone semantics
are the actual requirement. Validation remains the responsibility of the owning boundary.

## Capability boundaries

### Spaces

Spaces is the product bounded context, and `Space` is its aggregate root. The Inbox is the system
Space. The Spaces Browser is application navigation rather than a second domain model. Source paths,
database tables, commands, types, and UI use `spaces` for the bounded context and `space` for one
aggregate; aliases and dual command registrations are forbidden.

Renderer `workspace/session-participants.ts` owns registration and the ordered content, layout and
persistence leave barrier. Editor owners prepare saves or decisions and defer draft cleanup until
that barrier accepts all participants. `canvas/editor-session.ts` owns canvas editor state and
suspension, consuming a readonly participant snapshot rather than hosting workspace policy.

The Rust boundary is divided by business responsibility:

- `actions/` owns the versioned action contract and catalog; the domain owner validates and commits
  each action.
- `authorization/` owns principals, explicit Space/Application Run scopes, grants, and effect-specific approval.
- `runs/` owns durable runs, steps, attempts, scheduling, recovery, and events. Application voice
  generation shares this engine with Space work and has no implicit Space or Space permissions.
- `workflows/` owns capture and replay of verified action chains.
- `conversations/` owns shared conversation history and execution admission; built-in and external
  executors remain separate implementations.
- `knowledge/` owns semantic contracts, provenance, indexing, retrieval, grounding, patches, and
  governance.
- `derivation/` owns use cases that transform pinned Space resources into traceable outputs.
- `nodes/` owns content mutation and node lifecycle. `structure/` owns layout, membership, edit
  history, clipboard, deletion, and structural integrity. Disposable camera state is separate.
- `assets/` owns asset authorization, import, enrichment, references, and lifecycle.
- `speech/` owns the global voice library and Application voice-generation Runs; Space speech
  projects own their content and pin global immutable voice revisions.
- `queries/` owns read-only projections. `commands/` owns Space mutations without a more specific
  capability owner.
- `package/` owns package validation, snapshots, restore, and failed-import cleanup.
- `agent_artifacts/` owns agent workspaces, previews, import, recovery, and consumption.
- `run_execution.rs` composes finite Action, captured Workflow and speech step routing and capability-owned terminal maintenance. `run_control.rs` composes durable Run control with capability interruption. The scheduler receives three concrete hooks at startup; it never selects capabilities itself.
- `recovery.rs` owns ordered reconciliation of interrupted Space workflows during bootstrap.

Space agent workspace roots and session paths are defined in `spaces/agent_artifacts/workspace`.
`external_agents/artifacts` owns safe artifact collection, output projection and resolution shared by
agent execution consumers. External Agent task workspace initialization belongs to that executor's
lifecycle, rather than AI settings or usage initialization.

`queries/` never mutates durable state or controls execution. `knowledge/` does not depend on
`derivation/`, `assets/`, or `queries/`; governance must remain independent from the operations that
produce resources. `commands/` and `derivation/` may depend on `knowledge/`, indexing, and
infrastructure adapters, but cannot duplicate semantic policy. A command/query split for one
aggregate is intentional, not duplicate behavior. Files are duplicates only when both decide the
same policy, validate the same invariant, translate the same boundary, or own the same side effect.

Within `knowledge/`, `contracts/` holds shared actor, relation and external-source shapes;
`model/` owns canonical semantic revisions and shared document-coverage models, invariants and
projections. Neither owns persistence. `validation/` composes node boundary validation with those
models, while `relations/`, `provenance/` and `events/` own their respective policies. Indexing owns
chunk generation and index operations rather than shared semantic revision rules.
`change_commit/contracts.rs` defines proposed changes; `change_commit/validation/` owns their
structural and database-reference checks before execution.

`spaces/structure/planner/` is private except for the structure-scoped clone, deletion, bounds, and
spatial/tree-policy boundaries consumed by transactions and transfer. Neutral image engines accept
caller-authorized paths; Spaces performs authorization before invocation, and the engine never
imports Spaces.

The Renderer presentation root may compose capability operations and query contracts, but it never
imports persistence adapters. Canvas geometry and engine state stay in the canvas adapter; rich
content stays in its product presentation owner. Detailed Space behavior, operation effects,
revision semantics, node ownership, and persistence are specified in [spaces.md](spaces.md).

### Engines and local resources

`documents/`, `media/`, `images/`, `file_inspection/`, `web_capture/`, `system_preview/`, and
`local_resources/` are independent crate-level capabilities. They do not depend on Spaces or emit
Space domain types. A Space use case authorizes inputs, calls an engine through its typed boundary,
and owns durable publication.

`local_resources/models` owns model identity, installation, and integrity;
`local_resources/tools` owns verified tool installation and bundled executable discovery;
`local_resources/runtimes` owns trusted runtime publications and leases; and
`local_resources/preparation` coordinates preparation and execution leases. `app_resources` may
compose resource removal with consumers that must release an active model, while
`local_resources` remains independent from AI and Spaces. System memory is a resource fact; device eligibility and execution admission remain AI policy. `app_resources` composes catalog facts with that policy for product availability.

Runtime selection, Python processes, model/device policy, downloads, packaging, and GPU behavior are
specified in [system-architecture.md](system-architecture.md) and
[local-models.md](local-models.md). The WebView never starts Python directly, chooses unverified
resource paths, or publishes engine results to Space storage.

### Python capabilities

`python/` is the application-owned `knovum-python` project. Production packages use the standard
`src/` layout and capability names `knovum_<capability>`. Capability packages do not import peer
implementation internals; composition belongs to the process entry or Rust orchestration. A shared
module requires real consumers and one concrete responsibility. Package `__init__.py` files do not
re-export implementation symbols.

Within `knovum_inference`, `runtime/` owns request admission and capability dispatch,
`transport/` owns bounded framing and JSON-RPC validation, and capability directories own their
computation. Capability implementations depend on their modules, generated contracts, and neutral
errors; they do not depend on worker lifecycle or transport.

`knovum_model_resources` owns one bounded resource request and invokes vendor SDKs directly. It does
not import inference engines. Rust remains the authority for inventory, region policy, integrity,
and installation publication. Python cannot authorize Space data, create durable runs, choose a
replacement model after failure, or publish results.

### Agents and extensions

Spaces owns conversation history, Space-scoped grants, execution admission, run state, tool ledgers,
and publication receipts. The built-in runtime and desktop-agent protocols remain separate
executors behind that contract.

Rust `extensions/{plugins,skills,connectors,mcp,snapshots}` owns installed extension content,
metadata, authorization, and outbound protocol access and has no dependency on Spaces. Spaces
composes immutable execution snapshots; extension reclamation never queries a Space database.
Extension startup independently retries account credential cleanup after backup readiness. Space
recovery supplies retained snapshot IDs to extension-owned lease reconciliation; the two operations
share the extension content lock without making account cleanup depend on Space recovery.
`features/extensions` owns management UI, while foundation exposes neutral IPC/query contracts.
Settings and Spaces do not import extension business UI. Connector definitions, connection
instances, accounts, and protocol execution retain distinct identities and owners; MCP operation
does not become service-definition policy.

The complete agent host, security, and extension runtime contracts belong to
[system-architecture.md](system-architecture.md).

## One source of truth

`app/router.ts` owns the fixed Spaces and Settings routes.
`features/spaces/use-spaces-workspace.ts` exposes the single composition contract for Spaces-owned
shell surfaces and commands. Knovum is one knowledge-space product, not a container for peer apps.

`AppCommand` is the single source of truth for command-palette entries and application-local
shortcuts. Native menus emit the same command IDs rather than implementing actions again.

`file_inspection/classification` is the authority for imported-resource classification. Import,
inspection, and preview eligibility consume its typed result instead of maintaining extension lists
or inferring categories from persisted node-kind strings.

Each in-memory Rust job family uses a typed registry over the crate-private `job_registry`
lifecycle primitive. Long-lived sessions, singleton engines, credentials, and serialization locks
are state owners rather than jobs and do not use that primitive.
Durable Action handlers receive the Run Attempt's cancellation token through execution and
publication. They do not register another job identity or dispatch cancellation by Action kind.

## State and persistence ownership

- Component-local refs own local visual state.
- Pinia owns cross-view session state.
- TanStack Vue Query owns async/IPC query cache.
- Tauri Store owns durable non-secret preferences through the strict current v1 schema.
- Feature-owned SQLite databases own structured local product data and indexes.
- The foundation-owned `secrets.db` owns credentials and tokens.
- A feature-owned Rust implementation owns long work and projects progress through Tauri Channels
  and the shared Task Center bridge.

Composable APIs expose their owned state as refs and computed refs, with explicit operations.
Consumers directly destructure plain composable returns or consistently use a coherent aggregate
object, including when passing it as an input contract; do not retain parallel member aliases. Pinia and reactive objects require `storeToRefs`
or `toRefs` when destructuring state. Forward existing refs directly rather than wrapping them in
identity computed getters. Writable model refs are explicit input/output contracts, not copies of
a second state.

Use callbacks for reactive input getters, injected operations or policies, and real event or lifecycle
contracts. Operation results return through their promises; presentation owns the resulting visual
changes. Do not inject callbacks or writable error refs merely to mirror composable state into a
caller, and do not replace that coupling with watcher-based state copies. The operation owner exposes
its error; a composition owner derives combined errors when several operations feed one view.

Do not mirror one state across stores without an explicit synchronization design. Presentation emits
intents; the owning capability validates, commits, and projects results.

Before the first public release, Store envelopes, SQLite schemas, packages, and internal protocols
accept only their current v1 form. Incompatible development data is reset explicitly. Do not add
compatibility readers or migration chains before persisted compatibility becomes a supported product
contract.

Optional preference fields define absence in the owning schema. Only absence receives that declared
default; malformed values, unknown fields, missing required records, and unsupported versions remain
errors. Reads do not rewrite storage. Normal saves write one complete canonical value through the
serialized Store mutation path. The TypeScript and Rust decoders must agree where both consume the
schema.

The preference defaults are owned by `src-tauri/src/preferences/mod.rs` and exported as `preferenceDefaults` for
the frontend. `backgroundTaskNotifications`, `workbench`, and individual workbench layout fields
are optional on disk; absence uses their declared product defaults while existing choices remain
unchanged. The theme, density, text size, and locale remain required. Adding a nonessential preference
must declare its absence semantics in both decoders; removing or changing a field's meaning is a
contract change and must not be hidden behind defaults. Persisted optionality does not make fields
optional in the normalized application state or canonical saved value.

The non-sensitive last-workspace route may use one disposable `localStorage` value so optional
session restoration cannot make Store a bootstrap dependency. Secrets never enter Store,
`localStorage`, Pinia persistence, feature databases, logs, or query cache.

## IPC and schema ownership

Rust owns serialized request, response, and event types. `ts-rs` exports neutral contracts to
`src/shared/contracts/generated` and feature contracts directly to their owning feature. Foundation
must not import feature implementations to obtain wire types. Use generated request types for IPC
arguments and generated response/event types for wire data; validate consumed wire values before
entering the frontend model.

Expose one command per user or application intent so the WebView cannot compose a second domain
implementation from low-level primitives. Vue components consume capability operations and query
options, never Tauri application-command transport directly. Application composition may invoke
startup operations; the owning capability still owns its command contract.

Frontend model schemas may impose stricter product policy than a serialized DTO. That refinement is
not a second wire protocol. `ts-rs` generates DTOs, not command bindings: capability operations
expose concrete call contracts, and the repository does not maintain a handwritten global command
map or signature parser.

`pnpm contracts:generate` exports Rust contracts and then generates strict Python models. Generated
files are never edited independently. Contract regression tests generate into a temporary directory
and compare committed output. Each schema definition has one owner; independent Renderer schemas
live under the owning capability's `schema/<subject>.ts` layout.

### Error boundary

`app_error::AppError` is the product error envelope `{ code, message }`, and `AppResult<T>` is its
result alias. Codes are explicit snake_case values assigned by the source owner. Plain engine errors
become `operation_failed` while preserving their message. Business policy never derives a code from
message text or converts a classified error back to a string before acting on its code.

Renderer application calls use `foundation/ipc/invoke-command` and
`foundation/errors/app-error`. The boundary preserves an existing `Error` and invoke arguments,
normalizes valid product envelopes, and retains malformed/transport causes. It owns normalization only;
Tauri owns transport, serialization, and Channels. It does not own notifications, caching, retries,
cancellation, or business recovery.

`foundation/errors/error-format` owns user-facing messages and diagnostic formatting. An operation
has one notification owner. Built-in `AggregateError` is reserved for independent failures such as an
operation and failed rollback, with the original failure retained as its cause. Cleanup or Renderer
projection failure must not replace an original failure or undo an already committed result.

## Validation and failure policy

Validate data crossing a trust or persistence boundary: user input, Store/SQLite records,
filesystem and network data, external process results, and IPC payloads. Static registries and
repository-owned invariants do not need defensive runtime checks when the type system already
guarantees them.

Required desktop infrastructure fails explicitly. Expected outcomes such as cancelled dialogs,
denied permissions, unavailable shortcuts, or unsupported taskbar decoration are modeled at their
own boundary. Do not hide required failures behind generic fallbacks, PATH discovery, alternate
models, command aliases, obsolete envelope readers, swallowed errors, or string-prefix recovery.

Knovum targets Tauri desktop. Browser compatibility layers and parallel Web implementations of Rust
domain behavior are forbidden.

## Source layout and checks

Source directories cannot contain a file and sibling directory with the same basename. A file-only
Rust module uses `<name>.rs`; a module with children uses `<name>/mod.rs` and never a sibling
`<name>.rs`. [naming-conventions.md](naming-conventions.md) owns the detailed path, identifier,
suffix, and test-layout rules.

Rust and Python source files are limited to 350 nonblank lines, including generated Python
contracts. TypeScript files and Vue scripts are limited to 180 nonblank, non-comment lines; complete
Vue SFCs are limited to 180 nonblank lines. Oxlint complexity is capped at 10. These are ceilings,
not targets; split by responsibility. Declarative schemas, static fixtures, and documentation follow
their own contracts.

No source path alias is configured. Relative imports intentionally expose ownership and dependency
direction.

The repository enforces these boundaries through:

- `pnpm check:web`: TypeScript checks, Oxlint, source-layout policy tests, dependency-cruiser,
  frontend tests, Knip, frontend duplication checks, and Oxfmt;
- `pnpm check:rust`: Rust source-layout, structured capability policy, rustfmt, Clippy, Rust tests and Rust duplication checks;
- `pnpm check:python`: Python source-layout, lint/type checks and application/tooling tests;
- `pnpm check:scripts`: tooling/documentation source-layout and all owned Node script unit tests, discovered from `scripts/**/__tests__/*.test.mjs`; signing/platform acceptance stays separate;
- `pnpm check`: all four groups;
- `pnpm contracts:generate`: reproducible Rust-to-TypeScript/Python wire generation;
- `pnpm contracts:check`: regeneration plus committed/untracked drift checks over the generator-owned output list, including Model Resources Python constraints.

Add or change an owner only with matching dependency rules and focused tests. Do not weaken a gate
to admit a boundary violation; repair the ownership or record a narrow, explicit architectural
exception here.
