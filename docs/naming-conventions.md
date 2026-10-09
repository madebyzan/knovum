# Naming conventions

## Knowledge-work execution language

- `SpaceAction` is the only name for a stable business capability shared by UI, Workflow, Builtin Agent
  and MCP. Do not introduce `KnowledgeAction`, generic `Capability` or `Task` synonyms.
- `Invocation` identifies one authenticated SpaceAction call; `Receipt` identifies its committed
  result.
- `AgentExecutionObservation` reports an executor's checkpoint, execution state and interruptions;
  it does not represent a committed SpaceAction receipt.
- `Run`, `StepInstance` and `Attempt` describe durable execution. `Job` remains local to an engine or
  process.
- `Workflow` is the only product term for a user-owned published program. Do not use `Pipeline` or
  `Flow` for the same object.
- `Conversation` means shared dialogue and history for built-in and external execution.
- `Builtin Agent` means Knovum's built-in executor. `Agent Runtime` means its graph execution mechanism
  and process boundary. `External Agent` means an ACP or equivalent external controller.
- Python uses `knovum_agent_runtime`, runtime profiles use `agent`, and model configuration uses
  `builtinAgent`. The standard model message role `assistant` retains its protocol spelling.
- `Authorization` covers principals, scopes, grants and approvals. Content review is a separate
  concept.
- Space Action IDs use one imperative verb-object phrase (`draft-document`, `generate-video`,
  `render-presentation`). Resource type names stay nouns (`video`, `presentation`); do not prefix
  them with `knowledge-` merely because they live in a Space.
- Workflow definitions are user-owned captures of verified Action derivations. Durable event types use
  the CloudEvents reverse-DNS convention; local Run event names are closed Rust enums serialized as `snake_case`.

Do not use `Card`, `Box`, `Widget`, `Manager`, `Details`, `List`, `Row`, `Entry` or `Record` as project-owned component identity. Name the actual
interaction or information: `SpaceAgentArtifactPicker`, `SpaceVisualAnalysis`,
`ExternalAgentAvailability`. Do not replace a vague suffix with another vague suffix such as `Panel` merely
to pass review. Collections, table rows and enumeration verbs remain valid inside their actual scope.
Name components for their concrete responsibility, such as `ConnectorConnections`,
`AiConnectionConfiguration` or `VoiceCandidateReview`, rather than the container's visual shape.
Use `Summary` only for actual summarized content, not an arbitrary reduced DTO; name read models
after their purpose (`RunObservation`, `Conversation`, `UsageMetrics`).

Avoid `core-*`, `base-*` or `common-*` as catch-all module identities. Name validation boundaries
(`scalar-schema`, `content-schema`) and place reusable policy in its real owner. Third-party package
names and protocol identifiers keep their external spelling. A rename-only wrapper is not an owner.

Do not use generic `result`, `*Data` or `*application` as project-owned file, type, function or local
variable identities. Name the concrete content, observation, receipt, response, publication or
interaction. The spelling must describe the actual responsibility; replacing one vague suffix with
another is insufficient. Standard library types (`Result`, DOM `ImageData`), imported vendor types,
fixed vendor methods and external protocol fields retain their contractual spelling. `metadata`
means actual descriptive attributes and is not a synonym for an arbitrary payload.

Names are part of the architecture contract. Prefer domain language over implementation details and keep one term for one concept.

## Source paths and files

Application-owned Python import packages use `knovum_<capability>` under `python/src/`.
The `knovum-python` distribution names the product's Python project; capability process entries
name their operation boundary, such as `knovum-inference` and `knovum-model-resources`. `knovum_inference/runtime` means
inference Worker lifecycle; managed Python runtime resources mean CPython and frozen third-party
packages. These are separate owners. Python capability tests use `python/tests/<capability>/`,
and Rust-owned wire schemas use `python/schemas/<capability>.schema.json`.

Paths describe ownership; file names describe the responsibility within that owner. Do not
mechanically repeat the complete parent path in a file name. Once a capability owns submodules, use
a directory and move its implementation under that directory instead of retaining prefixed sibling
files.

| Source object              | Rule                                                   | Example                                  |
| -------------------------- | ------------------------------------------------------ | ---------------------------------------- |
| Python directory/module    | `snake_case`, named within its concrete owner          | `speech/synthesis/kokoro.py`             |
| Rust directory/module/file | `snake_case`                                           | `status/parser.rs`                       |
| TypeScript directory/file  | `kebab-case`                                           | `relation-repository.ts`                 |
| Vue SFC                    | `PascalCase.vue`                                       | `SpaceRelationEditor.vue`                |
| Composable file/export     | `use-<capability>.ts` / `useCapability`                | `use-media-player.ts` / `useMediaPlayer` |
| Type/interface/enum        | `PascalCase`                                           | `SpaceRelation`                          |
| Function/variable          | `camelCase` in TypeScript, `snake_case` in Rust/Python | `restoreSpace` / `restore_space`         |
| Constant                   | `SCREAMING_SNAKE_CASE`                                 | `DEFAULT_NODE_WIDTH`                     |

Avoid `index.ts`: it hides responsibility and can become a barrel without changing its path. Import
the named owner directly. Prefer conventional Rust module paths over `#[path]` indirection, and keep
module ownership obvious when choosing between a file module and a directory module.
An unsplit Rust module uses `<name>.rs`. Once `<name>/` exists for its submodules, its entry must
move to `<name>/mod.rs`; do not keep both `<name>.rs` and `<name>/` as siblings. Do not create an
empty directory for a file-only module or use `#[path]` to preserve the old physical layout.

Feature roots contain only the module entry, primary view and genuine top-level owners. Non-route UI
belongs to its capability's `components/` or, for Spaces, `presentation/<capability>/`; secondary routed pages belong in `views/`. Composables stay beside their
owning capability rather than in a catch-all directory. Small features do not create empty layering
directories. Renderer directories name product capabilities or concrete technical ownership;
`model/` is reserved for shared typed contracts and pure transforms. Generic
`application/`, `domain/` and `infrastructure/` directories are not Renderer ownership names.

`native` is not a naming prefix for paths, files, types, commands or variables. Knovum is one desktop
product, so it separates nothing; name the mechanism that actually differs, such as `rust`, `webview`,
`bundled-runtime`, `menu` or `system`. Prose keeps the word only for operating-system provided UI and
services, as in native window controls, native traffic lights or OS-native dialogs.

## Functions and Vue composables

Use `useCapability` for reactive behavior, component-context access, scoped effects, and composition
of other composables. Pure transforms, IO operations and command factories remain ordinary functions;
receiving or updating a caller-owned `Ref` does not by itself make a command factory a composable.
Explicit state and resource factories may use `createCapability` when construction and ownership are
their contract. They must expose cleanup for resources they own instead of hiding scope requirements.
Pinia store accessors retain `useCapabilityStore`. Each reactive instance belongs to its actual session
or view owner; multiple controls for one session consume that instance instead of creating copies.
Scope-owned external subscriptions and async work must detach or invalidate when the owner stops.

## Model and contract ownership

`model/` is an optional owner for pure data models, invariants and projections shared by several
operations within the same capability. It is not the default destination for every interface.
`spaces/model` owns shared Space node, relation and structure models; `knowledge/model` owns the
knowledge sub-capability's models. Keep their separate scopes and dependency restrictions; do not
create duplicate definitions of the same entity in both. No Pinia store, Vue composable, IPC,
persistence or UI lifecycle belongs in these model directories. Pure formatting/projection logic
may belong here; an operation that performs IO may not.

`schema/` holds executable structural validation and inferred types. `contracts/` holds an explicit
shared interface between independent implementations/owners, or generated wire types. Use the plural
directory spelling consistently. A field-bearing interface alone does not justify creating a
contracts directory. Local shapes stay in their concrete owner with specific names; options,
contexts, envelopes and search matches are not generic `types` modules. Do not create `types/`
as a category for TypeScript declarations. A business capability concerning knowledge object types
uses `object-types/` to distinguish the actual entity.

Contract directories do not own preview thresholds, visual budgeting or business state policy.
`canvas/contracts` holds editor/surface interfaces, `canvas/preview` owns pure preview policy,
and `workspace/persistence-state.ts` defines the workspace persistence state. Avoid a global
`models/`, `types/` or `schemas/` container spanning unrelated capabilities. Source ownership remains
business-first; model/schema/contracts describe responsibility inside an existing owner.

## Schema and supporting module placement

Standalone Renderer runtime schema modules live under their owning capability or model's
`schema/<subject>.ts`. Do not repeat `-schema` inside that directory or maintain sibling
`<subject>-schema.ts` modules for the same role. Schema fields and Zod refinements stay with the
schema they define. Public parsing/validation operations use a concrete validation module when
they serve callers independently of schema construction. Inferred types may remain beside their
schema; do not introduce forwarding type barrels. Renderer callers import functions and types
from their defining modules. IPC adapters, stores, schemas and projections do not re-export another
module's declarations. Generated contracts retain their generated names; do not rename them merely
to provide a second frontend vocabulary. Local derived types still belong beside their derivation.

A local schema used solely to validate one IO response may stay private beside that operation.
Importing Zod does not turn an IO adapter, a projection or a type-only consumer into a schema module.
Generated wire contracts remain in the owning `contracts/generated` directory; database DDL remains
in feature-owned SQL baselines. Rust Arrow schemas stay with the retrieval index and query-vector
validation lives in `validation.rs`. These boundaries have different responsibilities.

`contracts` names owned wire/boundary shapes; `schema` names executable structural validation or
storage layout; `validation` names validation policy; `decode` names persistence decoding;
`repository` names persistence operations. Use a concrete business owner such as `object_types`
for a module containing lifecycle operations instead of calling it `types`. Rust file and directory
modules follow the same responsibilities: split into a directory only when there are actual
submodules, and keep the `mod.rs` entry focused on composition where practical.
A sibling file and directory must not share the same stem, including case-only differences;
this applies to repository entry documents and local resource folders as well as source modules.

## Domain verbs

### Command verbs

The table describes semantic verb families, not a closed token allowlist. Select a verb from the
actual operation and use the same term for the same intent across Rust and TypeScript. New domain
verbs are valid when their meaning is documented; do not force unrelated operations into a word
merely because it already appears in the table.

Language-level conventions follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/naming.html)
and [RFC 430](https://rust-lang.github.io/rfcs/0430-finalizing-naming-conventions.html). TypeScript uses
camelCase operations and PascalCase types; Rust uses snake_case operations and UpperCamelCase types.
Rust attribute getters usually omit `get_`; an IPC read operation can retain `get_` because it names
an explicit boundary operation rather than an in-memory attribute getter. Preserve standard trait
methods and vendor API names. Semantic verb selection below is Knovum policy, not a claim that
industry mandates one spelling for every use case.

| Group       | Intent                                                         | Verb         |
| ----------- | -------------------------------------------------------------- | ------------ |
| Lifecycle   | Create a business object                                       | `create`     |
| Lifecycle   | Change fields of a business object                             | `update`     |
| Lifecycle   | Delete a durable business object                               | `delete`     |
| Lifecycle   | Move a business object into the reversible bin                 | `trash`      |
| Lifecycle   | Reconstruct a business object from the bin or a package        | `restore`    |
| Lifecycle   | Delete an owned file, model or temporary resource              | `remove`     |
| Lifecycle   | Withdraw a granted relation or authority                       | `revoke`     |
| Lifecycle   | Take a durable object out of active use, reversibly            | `archive`    |
| Lifecycle   | Return an archived object to active use                        | `unarchive`  |
| Persistence | Write the current state of a record idempotently               | `save`       |
| Persistence | Write renderer-produced bytes under app ownership              | `persist`    |
| Staged work | Create a temporary authorized resource pending completion      | `stage`      |
| Staged work | Finalize a staged resource                                     | `commit`     |
| Staged work | Apply a patch or a prepared command                            | `apply`      |
| Staged work | Activate a downloaded artifact as the running version          | `install`    |
| Staged work | Decline a staged proposal                                      | `reject`     |
| Async work  | Begin a long-lived activity or service                         | `start`      |
| Async work  | End a running activity or service                              | `stop`       |
| Async work  | Abandon an in-flight job                                       | `cancel`     |
| Async work  | Record that a job ended in error                               | `fail`       |
| Async work  | Run one unit of queued Rust work                               | `run`        |
| Async work  | Hand a renderer-produced result to Rust for recording          | `submit`     |
| Async work  | Return a completed generation or response                      | `complete`   |
| Async work  | Close out a multi-step capture                                 | `finalize`   |
| Read        | Read one known identity                                        | `get`        |
| Read        | Return a collection                                            | `list`       |
| Read        | Query by free text or criteria                                 | `search`     |
| Read        | Load a durable aggregate or session projection                 | `load`       |
| Read        | Read from a network boundary                                   | `fetch`      |
| Read        | Assemble a snapshot or aggregate representation                | `collect`    |
| Sync        | Bring derived state back in sync                               | `reconcile`  |
| Sync        | Discard and reconstruct a derived index                        | `rebuild`    |
| Sync        | Re-read from the authoritative source                          | `refresh`    |
| Sync        | Return persisted state to its baseline                         | `reset`      |
| Sync        | Write a copy for safekeeping                                   | `backup`     |
| Assignment  | Assign a scalar or preference directly                         | `set`        |
| Assignment  | Record a state transition or timestamp                         | `mark`       |
| Assignment  | Reserve a managed output path                                  | `allocate`   |
| Assignment  | Move an object between containers                              | `move`       |
| Assignment  | Append to an ordered stream                                    | `append`     |
| Assignment  | Reveal a resource in its owning surface                        | `open`       |
| Transfer    | Bring external data under app ownership                        | `import`     |
| Transfer    | Write app-owned data to a user-selected location               | `export`     |
| Transfer    | Transfer bytes down from a remote source                       | `download`   |
| Transfer    | Store a network response for reuse                             | `cache`      |
| Compare     | Compare local state against an authoritative source and report | `check`      |
| Domain      | Produce vector embeddings for knowledge                        | `embed`      |
| Domain      | Produce derived knowledge from a source asset                  | `enrich`     |
| Domain      | Produce a transcript from media                                | `transcribe` |
| Domain      | Produce the same text in another language                      | `translate`  |
| Domain      | Produce a transformed editor document                          | `transform`  |
| Domain      | Reply to a question with grounded evidence                     | `answer`     |
| Domain      | Produce a rendered artifact                                    | `render`     |
| Domain      | Produce a copy of an object                                    | `duplicate`  |

Model-produced text and artifacts use `generate` (`generate_grounded_document`,
`generate_image_layers`, `generate_video`). `render` names deterministic media/artifact processing
(`render_video_edit`). `create` allocates a durable entity or owned resource;
`derive_image` executes an image transformation. `request` accepts work for later execution;
`await` waits for a defined lifecycle condition; `inspect` examines a boundary resource; `detect`
discovers system installations. These verbs name distinct effects and are not interchangeable.
`capture_web_resource` brings remote content under product ownership; `import_web_image` persists
an image asset rather than promising cache reuse. Read readiness uses `availability`; a supported
operation set can use `capabilities`.

Deletion is four distinct intents, not four synonyms: `delete` for durable entities, `remove` for
owned files/models/temporaries, `trash`/`restore` for the reversible bin, and `revoke` for a granted
relation. Do not use them interchangeably.

Reversible state bits get a domain verb pair, never `set_<thing>(true)`: `archive_space` /
`unarchive_space`, `delete-subtrees` / `restore_deleted_structure`. A private helper may still take the boolean
(`set_archived(app, space_id, true)`) because the flag does not cross a module boundary.

Avoid generic `process`, `manage`, and `do` when they hide the object or effect. `execute` is valid
for a concrete typed command protocol such as `execute_space_edit`; it must not become a universal
string-driven dispatcher. `set` means direct
assignment, not a persisted business mutation. Short verbs such as `start`, `stop`, `cancel`, and
`fail` are acceptable because the owning module already supplies the missing domain object.

A command names one intent, so it carries exactly one verb. `_and_` compounds are not acceptable:
`updater_check_and_install` claimed two intents in a single address, and conflated checking with installation. The command is `install_update`, because re-resolving the update is intrinsic to
installing it rather than a second intent — the official `plugin:updater` shapes the same work as
`check()` then `downloadAndInstall()`. When a second verb really is a separate user-visible intent,
split the command instead of naming both.

Ending async work keeps three verbs apart: `cancel` abandons a job that has not finished, `stop`
terminates a service that is still running, and `fail` records that the work already ended in error.
A command that only writes a failure record is `fail_*`, not `stop_*`.

Plugin and Skill availability exposes `enable_plugin` / `disable_plugin` and `enable_skill` / `disable_skill`; only private persistence helpers accept a boolean.

Voice profiles expose `archive_voice_profile` and `unarchive_voice_profile` as separate public intents. Their shared private persistence operation may accept a boolean; the public request contracts do not. Conversation tool activity uses `context` for invocation inputs or execution context and `output` for the tool response, consistently across storage and presentation. Run attempts and steps use `output` for their produced artifacts. Vendor tool-result payloads retain their protocol fields.

### Persistence and Renderer-local verbs

These name work inside persistence or Renderer-local APIs. Persistence implementation verbs such as
`insert` and `upsert` do not leak into IPC; the read verbs `get`, `load`, `find` and `read` can also
name an IPC operation when they describe its public intent.

| Intent                                                 | Verb      |
| ------------------------------------------------------ | --------- |
| Read one known identity without I/O                    | `get`     |
| Read one known identity through I/O                    | `load`    |
| Locate something that may not exist                    | `find`    |
| Read files or bytes                                    | `read`    |
| Derive a final value                                   | `resolve` |
| Purely assemble a representation                       | `build`   |
| Receive an event or protocol message                   | `handle`  |
| Insert a persistence row                               | `insert`  |
| Idempotent insert/update inside one persistence module | `upsert`  |
| Replace a business object wholesale                    | `replace` |
| Wipe a collection                                      | `clear`   |
| Drop records past a retention window                   | `prune`   |
| Adopt or clean up orphaned resources                   | `reclaim` |
| Transfer bytes up to a remote destination              | `upload`  |

At the IPC boundary, `get_<entity>` reads one known identity; `load_<aggregate>` assembles durable
state such as a Space session projection, edit snapshot or camera state. Both are read operations.
Existing `load_space_session_projection`, `load_space_edit_snapshot` and `load_canvas_view_state`
retain their stable addresses. I/O alone does not determine which verb to use.

## Tauri commands

Tauri resolves custom commands in two separate IPC namespaces, and each one names its commands
differently.

In-process plugin commands resolve as `plugin:<key>|<command>`. The plugin key already carries the
owner, so the command must not repeat it:

```text
<verb>_<noun>                plugin:spaces|list_nodes
```

Omit the noun only when it is the plugin's own aggregate root, which is what the official plugins
do (`plugin:store|reset`, `plugin:updater|check`). Keeping a
`<feature>_` prefix on a plugin command duplicates its namespace; reject Renderer addresses shaped
`<plugin-key>_<rest>` during review.

Names identify the resource or aggregate actually operated on, not the pane displaying it.
Use a plural resource for collection reads (`list_scenes`, `list_knowledge_events`) and an explicit
entity for singleton reads (`get_node`). Bulk mutations can use plural objects (`create_nodes`,
`update_nodes`), and snapshot reads can name their aggregate. A paged or status-bearing collection
wrapper still represents a collection. Preserve idiomatic mass nouns (`list_provenance`,
`list_related_content`) instead of inventing unnatural plurals. Google's
[AIP-132](https://google.aip.dev/132) supports verb-first plural resource names for List RPCs;
its resource API rules are guidance rather than a mandatory REST model for all local commands.

Custom in-process plugins currently registered in `src-tauri/build.rs` are `spaces`, `ai`,
`secrets`, `desktop`, `backup`, `dictation`, `local-resources` and `extensions`. Official plugins such as Store remain vendor-owned;
there are no custom `preferences` or `database` namespaces. A business capability does not
automatically require a separate plugin.

App commands registered through the application `invoke_handler` share one flat namespace, so the
owner has to be part of the name:

```text
<verb>_<entity>             get_update_state, check_for_update, install_update
```

The flat namespace uses explicit entities to prevent collisions. The updater wrappers remain app
commands because `updater` is the official plugin key; they do not need object-first prefixes to
identify their owning module. In-process plugin registration cannot claim that vendor key.

An app command has four touch points, and missing any one of them makes it unreachable or ungranted:

1. the `#[tauri::command]` function itself;
2. the `tauri::generate_handler![...]` list in `src-tauri/src/app_builder/mod.rs`;
3. a manual `commands.allow` entry in `src-tauri/permissions/<feature>.toml` and a capability grant
   naming that permission;
4. a Renderer `invoke("<verb>_<entity>")` literal.

A plugin command has four: the `#[tauri::command]` function; its name in
`tauri_build::InlinedPlugin::commands(&[...])` in `src-tauri/build.rs`, which generates the
`<key>:allow-<command>` and `<key>:deny-<command>` ACL at build time; its path in the
`generate_handler![...]` inside `tauri::plugin::Builder::new("<key>")`; and a capability grant,
either as `"<key>:allow-<command>"` or, where the plugin declares
`DefaultPermissionRule::AllowAllCommands`, as `"<key>:default"`. A plugin command has no entry in
the application invoke handler and no hand-written block in `permissions/*.toml`.

Keep these touch points synchronized through Rust/Tauri compilation and review. Review the generated
plugin permissions, each capability file and the windows named by that capability whenever a command
is added or removed. A grant in one capability says nothing about another window, and a
`<plugin>:default` grant must be reviewed against the plugin's generated default permission. Command
names and Renderer call sites are also review responsibilities; no source scanner infers their
correctness.

## Frontend file suffixes

- `*-client.ts`: an actual network/SDK client with request or connection lifecycle. Ordinary Tauri IPC functions use operation names; a forwarding function is not a client.
- `*-repository.ts`: actual aggregate persistence access or persistence policy; never UI state or projection decoding alone.
- `session/<feature>-store.ts`: a feature's Pinia cross-view session composition root.
- `*-controller.ts`: UI lifecycle and multi-primitive orchestration.
- `*-actions.ts`: user/domain actions over one state owner.
- `schema/<subject>.ts`: standalone runtime schema and its inferred types; private single-operation IO schemas stay with their IO owner.
- Static type-only files use the concrete object or information role (`search-match.ts`, `audio-recorder-options.ts`); avoid generic `*-types.ts` and type containers. The business term `object-types` denotes user-defined knowledge object classifications, not TypeScript declarations.
- `*-presentation.ts`: pure domain-to-view-model transforms.
- `*-runtime.ts`: resources and lifecycle that exist only for the current process.

Do not use `*-service.ts` when a domain noun or one of the boundary suffixes above states the
responsibility. Deciding whether an existing adapter's behavior matches a suffix remains a code
review responsibility; static checks must not infer semantics from implementation text.

Renderer files must not use `*-database.ts`; the WebView does not own SQLite authority.

## Rust module direction

Use explicit imports and re-exports for project-owned business interfaces. Test modules may use
`use super::*`, and vendor preludes may use their documented glob imports. Use `super::` for nearby
module relationships and `crate::` when repeated parent traversal obscures the owning capability.
Scoped `use` declarations, full-path calls, aliases and conditional imports remain valid Rust
mechanisms; choose them for clarity without mechanically rewriting every call. Re-exports expose
an owner's intended interface and must retain its existing visibility boundary.

Use names that reflect ownership:

- `spaces/queries`: read-only Space projections and response mapping.
- `spaces/commands`: atomic state-changing Space application operations.
- `spaces/knowledge`: semantic contracts and knowledge governance.
- `spaces/derivation`: source-grounded artifact generation, rendering, persistence and their read projections; `commands.rs` owns execution, `queries.rs` owns reads.
- `spaces/db.rs`: Space connection/error foundation.
- `local_resources/models/`: locally installed AI model descriptors, downloads, integrity verification and installation lifecycle.
- `local_resources/tools/`: discovery of application-bundled executables and shared compiled libraries.

Shared/foundation modules must never depend on a feature solely to reuse a helper.

The `queries`/`commands` split follows observable business effects, not an annotation or first-word
scan. A Tauri `#[command]` is a transport entry point and may be a query. Reads return projections;
mutations persist state or control execution and may return receipts or accepted identifiers.
Opening an OS preview window is a command; calculating a workflow preview without writes is a query.
Shared execution registries belong to their engine owner, not a query module. This does not require
separate databases, a CQRS bus or new layers.
See [Tauri commands](https://v2.tauri.app/develop/calling-rust/) and
[Fowler CQS](https://martinfowler.com/bliki/CommandQuerySeparation.html). Both sides mirror the same sub-domain names, which is what makes the direction readable —
`queries/relations.rs` reads what `commands/relations.rs` writes, `queries/scenes.rs` reads what
`commands/scenes/` writes.

## Variables and parameters

- Boolean names use `is*`, `has*`, `can*`, or `should*` when they cross a local expression.
- Collections use plural nouns; identifiers use `<entity>Id` / `<entity>_id`, and identifier
  collections use `*Ids` / `*_ids`.
- A command parameter is never bare `id`; the entity owns the name (`threadId`, `patchId`,
  `sceneId`, `objectTypeId`). `id` stays legal as a field of a semantic request object
  (`SpaceSceneInput.id`) and as a loop-local, where the type or the collection already names the
  entity.
- Identifiers name the entity or lifecycle they actually identify: `job_id` for an engine job,
  `run_id` for a durable Run, `session_id` for a session, `derivation_id` for an artifact derivation,
  `generation_id` for a generation producing multiple derivations, and `mutation_id` for an idempotent edit/import. Cancellation does not turn every identifier into
  a job ID. Bare IDs inside entity-shaped request contracts may remain when ownership is explicit.
- A Tauri `Channel` parameter names its payload or purpose: `on_progress` / `onProgress` for progress,
  `on_stage` / `onStage` for stages, `on_event` / `onEvent` for event streams, and `updates` for Run or
  conversation state snapshots. A domain-specific `progress` field is valid when its contract is
  explicit. Serialized parameter names are stable IPC fields; renaming requires both Rust and
  Renderer updates. Internal parameters are not restricted to a closed three-word list.
- Maps use `<value>By<Key>`; timestamps use `*At` / `*_at`; durations include their unit.
- Only DOM/component references use `*Ref`; a Vue `ref()` does not make `Ref` part of the domain name.
- `data`, `value`, and `item` are limited to short, unambiguous local transforms.
- Prefer a semantic options/command object after three parameters when it makes the call easier to
  understand. Rust types and review own this decision; there is no repository-specific parameter
  count gate.
- Complex object-shaped parameters receive a named interface/type near the owning API. Inline object
  types are limited to small, obvious local values such as a two-axis point or a single optional flag.
- Vue components and composables obtain locale context with `useI18n()`. Translation callbacks cross
  into pure policy, command-catalog or IO functions only when keeping those functions independent of
  Vue setup context is intentional; components do not relay `t` through intermediate composables.

## Boolean APIs

A named local boolean is fine when its meaning is obvious. Across module boundaries, avoid calls such as `operation(value, true)` when the flag changes behavior. Prefer explicit operations (`copySelection` / `cutSelection`) or a semantic union/options object (`"active" | "dormant"`, `{ preserveTitle: true }`).

## Local names

Short local names such as `row`, `value`, `save`, `start`, `busy`, and `editing` are acceptable in small scopes when module/component context makes the meaning unambiguous. Do not expand names mechanically just to make them globally unique.

## Tests

Keep Rust unit tests close to the owning module, in dedicated directories. Test suites, fixtures
and helpers belong under the owner's `tests/` directory, with `tests/mod.rs`
as the test module entry. Split suites use concrete subject names inside that directory.
The business module contains only the `#[cfg(test)] mod tests;` declaration; do not embed test
bodies in business files. A module that gains test submodules follows the same `<name>/mod.rs`
layout as other split modules. Leaf modules may declare a test module with an explicit path into
their owner's `tests/` directory; the module remains a child of the tested implementation.
Test-only helpers remain behind `#[cfg(test)]`. Child unit-test
modules can exercise private implementation without widening the production API. Use
`src-tauri/tests/` for integration tests through the crate's public interface; tests requiring
private interfaces remain unit tests within their owning module.

Vue/TypeScript tests live under the nearest common owner's `__tests__/` directory and use
the `.test.ts` suffix. A unit test should make its tested owner obvious, for example `UiButton.vue`
and `UiButton.test.ts`. Name a cross-owner integration scenario for the user capability it covers
rather than pretending it maps to one source file. Shared test fixtures and mocks belong under that
owner's `__tests__/support/`, so they remain outside production typechecking and business imports.
Production source must not import test files, Vitest or Vue Test Utils. Complete Vue SFCs, including template and style, are limited to 180 nonblank lines by the source-layout gate.
Python worker/qualification tests live in `python/tests/`, repository tooling policy tests (Python
and Node) in each tooling owner's `__tests__/` (the standalone generator uses `scripts/__tests__/`), and desktop E2E
scenarios in `tests/e2e/`. Test-specific imports belong in test files, not business modules.

## Automated checks

A repository-specific check earns its maintenance cost only when it reads authoritative structured
configuration, uses an established tool's rule, or verifies an explicit source-layout contract
that existing language tools do not enforce. Source-layout gates have fixture-based regression
coverage and do not attempt to infer business behavior. What remains:

- the capability policy check rejects WebView SQL, `fs:default` on the main window and persistent
  filesystem mutation in the Spaces capability by reading the capability JSON directly;
- Oxlint's Renderer ceilings, including 180 non-blank/non-comment script lines, complexity 10 and four
  parameters;
- Knip reports dead code and unused dependencies;
- `scripts/checks/source-layout.mjs` rejects `index.ts` and checks sibling
  stems, test placement and the Rust/Python/Vue size limits specified in `architecture.md`.
  Descriptive naming, including the `native` convention, remains a code-review responsibility.

Cross-language wire types come from ts-rs generated contracts and compile in TypeScript. Provider
catalog behavior is covered by Rust parse, serialization and `as_str` tests. Design-token merging
is covered by behavioral tests of the configured merge function. These checks exercise the actual
contract instead of scanning source text for matching tokens or nullable field spellings.

Command vocabulary is deliberately not enforced. A verb allowlist parsed from the table above was
tried and removed: it checked the verb position and nothing else, so it passed
`list_knowledge_health` while the command returned `SpaceKnowledgeHealthIssue[]`, and it passed
`updater_check_and_install` because the verb position held the documented `check` and the compound
tail was invisible. Both defects were found by a reader in one pass. A gate that misses every defect
a human spots in a single review costs attention and returns none, so naming is a review
responsibility — this document is the reference, and the two names above are the recorded evidence
for why a token scan cannot replace it.

Ownership duplication, suffix meaning, schema/type duplication, whether an unused command should be
wired up or deleted, and integration-scenario naming also stay in review; inferring them from tokens
produces false positives, and a gate people route around is worse than no gate.

`scripts/` is itself inside Knip's project scope, so scripts that lose their last caller are reported
as dead code.

## Consistent implementation style

Oxfmt and rustfmt own formatting; Oxlint and Clippy own supported static checks. Do not introduce
a second formatter or imitate TypeScript idioms in Rust. Vue SFCs use Composition API with
`<script setup lang="ts">`, followed by template and optional scoped style. Template-owned DOM
references use `useTemplateRef`; scalar state uses `ref` or `shallowRef` according to the required
reactivity, and structured editable state uses `reactive` when appropriate. Derived state uses
`computed`, not a function invoked repeatedly from the template.

Use named function declarations for module operations; arrows remain appropriate for callbacks,
computed projections and returned closures. Import types with `import type` or inline `type`.
Keep pure transforms in model owners so rendering and export can reuse policy. Remove identity-only
forwarders and move invariant classes outside conditional expressions. Prefer explicit control flow
over compressed expressions when async ownership, error handling or cancellation is involved.
Preserve boundary validation and visible failures during simplification. Serialized property names,
IPC command names are owned contracts: semantic renaming updates Rust, Renderer, registration,
permissions, generated contracts, tests and documentation atomically, without aliases or compatibility
readers. Third-party API and fixed external protocol names remain vendor-owned.

Project-owned types also avoid generic `Entry` and `Record` identities. Name the entity directly
(`Conversation`, `ArchivedSpace`, `PackageNode`), or the specific information role
(`SpaceNodeProjection`, `EnrichmentJobState`, `ReversibleSpaceChange`, `ArchiveMember`).
Recording audio, recording history, SQL rows and external archive APIs retain their actual verbs
and fields. Do not change wire property names merely to remove a word from source identifiers.

## Detail identifiers

Do not use `detail`, `details`, `*Detail*` or `*Details*` as project-owned file, type, function,
variable or property identities. Name the actual policy, content or interaction: `CanvasPreviewPolicy`,
`ImageTextureSettings`, `statusMessage`, `setStatusMessage`, `explanation`, or `diagnosticText`.
Task Center `statusMessage` is an in-memory UI field for current stage text and failure reasons,
not a compatibility alias for `detail`. Canvas preview policy expresses preview levels and item
budgets. Image texture adjustment expresses sharpness, clarity and grain processing.

HTML `<details>`, DOM/library event `detail`, vendor message fields and fixed external protocol
spelling remain unchanged. Natural-language copy and user-provided text are not identifiers.
Evaluate a persisted or external field before renaming it; do not add compatibility readers.

## Agent extension vocabulary

`PluginInstallation` identifies an installed package; its `PluginInspection` is reviewed immutable
revision content. `PluginPublication` binds an explicit catalog, publisher ID and package name.
`SkillRevision` identifies personal or plugin-owned Skill content. `McpConnection` is a protocol
instance; `ConnectorDefinition` describes authorization requirements and `ConnectorAccount`
is one selected identity. `AgentExtensionSelection` is user choice; `AgentExtensionSnapshot`
is frozen execution input. `AgentToolBinding` distinguishes real Space Actions, MCP tools,
resource reads and prompt retrieval. `ConversationToolOperation` records durable attempts
for both built-in and ACP runs. Protocol `assistant`, `result`, `data` and `content` fields keep
their upstream meanings; they do not rename a product Agent to an Assistant.
