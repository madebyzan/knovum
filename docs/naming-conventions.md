# Naming conventions

Names are part of the architecture contract. Prefer domain language over implementation details and keep one term for one concept.

## Source paths and files

Paths describe ownership; file names describe the responsibility within that owner. Do not
mechanically repeat the complete parent path in a file name. Once a capability owns submodules, use
a directory and move its implementation under that directory instead of retaining prefixed sibling
files.

| Source object              | Rule                                            | Example                                  |
| -------------------------- | ----------------------------------------------- | ---------------------------------------- |
| Rust directory/module/file | `snake_case`                                    | `status/parser.rs`                       |
| TypeScript directory/file  | `kebab-case`                                    | `relation-repository.ts`                 |
| Vue SFC                    | `PascalCase.vue`                                | `SpaceRelationEditor.vue`                |
| Composable file/export     | `use-<capability>.ts` / `useCapability`         | `use-media-player.ts` / `useMediaPlayer` |
| Type/interface/enum        | `PascalCase`                                    | `SpaceRelation`                          |
| Function/variable          | `camelCase` in TypeScript, `snake_case` in Rust | `restoreSpace` / `restore_space`         |
| Constant                   | `SCREAMING_SNAKE_CASE`                          | `DEFAULT_NODE_WIDTH`                     |

`index.ts` is forbidden, including non-barrel files: it hides responsibility and can become a
barrel without changing its path. Import the named owner directly. Do not use `#[path]` to simulate
a different Rust module tree, and do not keep both `capability.rs` and `capability/mod.rs`.

Feature roots contain only the module entry, primary view and genuine top-level owners. Non-route UI
belongs in `components/`; secondary routed pages belong in `views/`. Composables stay beside their
owning capability rather than in a catch-all directory. Small features do not create empty layering
directories. Renderer directories name product capabilities or concrete technical ownership;
`model/` is reserved for shared typed contracts and pure transforms. Generic
`application/`, `domain/` and `infrastructure/` directories are not Renderer ownership names.

`native` is not a naming prefix for paths, files, types, commands or variables. Knovum is one desktop
product, so it separates nothing; name the mechanism that actually differs, such as `rust`, `webview`,
`bundled-runtime`, `menu` or `system`. Prose keeps the word only for operating-system provided UI and
services, as in native window controls, native traffic lights or OS-native dialogs.

## Product concept glossary

Use one term for each creation concept:

- **Creation** is the user-facing umbrella for choosing or starting a way to create an artifact.
- **Workflow** is an ordered, registered execution plan that coordinates multiple named steps.
- **Derivation** is a traceable operation that produces an output from one or more Space sources.

Do not introduce parallel synonyms such as `generation workflow`, `artifact workflow`, or `creation derivation`
unless they represent a genuinely different product concept. UI wording may say “create”, but code ownership
uses `workflow` for orchestration and `derivation` for traceable produced outputs.

## Domain verbs

### Command verbs

The closed vocabulary for Tauri command names. No gate reads this table; it is the reference a
reviewer consults before naming a command, and the reason it stays closed is that a synonym nobody
rejected becomes a second way to say the same thing. One intent owns exactly one verb, and every row
has at least one command.

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

Deletion is four distinct intents, not four synonyms: `delete` for durable entities, `remove` for
owned files/models/temporaries, `trash`/`restore` for the reversible bin, and `revoke` for a granted
relation. Do not use them interchangeably.

Reversible state bits get a domain verb pair, never `set_<thing>(true)`: `archive_space` /
`unarchive_space`, `trash_nodes` / `restore_nodes`. A private helper may still take the boolean
(`set_archived(app, space_id, true)`) because the flag does not cross a module boundary.

Avoid generic `process`, `manage`, `do`, and `execute` unless the word is part of an external
protocol; they are not in the table, so a reviewer should reject them. `set` means direct
assignment, not a persisted business mutation. Short verbs such as `start`, `stop`, `cancel`, and
`fail` are acceptable because the owning module already supplies the missing domain object.

A command names one intent, so it carries exactly one verb. `_and_` compounds are not acceptable:
`updater_check_and_install` claimed two intents in a single address, and its second verb was not in
this table at all. The command is `updater_install`, because re-resolving the update is intrinsic to
installing it rather than a second intent — the official `plugin:updater` shapes the same work as
`check()` then `downloadAndInstall()`. When a second verb really is a separate user-visible intent,
split the command instead of naming both.

Ending async work keeps three verbs apart: `cancel` abandons a job that has not finished, `stop`
terminates a service that is still running, and `fail` records that the work already ended in error.
A command that only writes a failure record is `fail_*`, not `stop_*`.

### Persistence and Renderer-local verbs

These name work inside a persistence module or a Renderer-local API. They never appear in a command
name, because SQL implementation details such as `insert` and `Transaction` do not leak through
Renderer APIs.

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

The `get`/`load` split is a Renderer-local distinction. Every Tauri command performs I/O, so at the
IPC boundary `get_<entity>` is simply "read one known identity" and `load` is not a command verb.

## Tauri commands

Tauri resolves custom commands in two separate IPC namespaces, and each one names its commands
differently.

In-process plugin commands resolve as `plugin:<key>|<command>`. The plugin key already carries the
owner, so the command must not repeat it:

```text
<verb>_<noun>                plugin:spaces|list_nodes
```

Omit the noun only when it is the plugin's own aggregate root, which is what the official plugins
do (`plugin:autostart|enable`, `plugin:store|reset`, `plugin:updater|check`). Keeping a
`<feature>_` prefix on a plugin command duplicates its namespace; `tools/check-tauri-command-coverage.mjs`
rejects any Renderer string literal shaped `<plugin-key>_<rest>`.

The noun is the element type the command actually returns — read the Rust `Vec<T>` or the Renderer
`Promise<T[]>`, not the pane or query name. `list` takes that type's plural and every other verb
takes its singular, which is what keeps a family readable: `list_scenes` / `create_scene` /
`delete_scene`, `create_provenance` / `revoke_provenance`. Google's
[AIP-132](https://google.aip.dev/132) states the same rule for `List`: the name begins with the verb
and the remainder is the plural form of the resource.

A collection therefore may not be named after the subject it describes. `list_knowledge_health`
named a mass noun while returning `SpaceKnowledgeHealthIssue[]`; the command is
`list_knowledge_health_issues`. The pane and its query keep the shorter subject name
(`SpaceKnowledgeHealth.vue`, `useSpaceKnowledgeHealthQuery`) because they present the topic, not the
collection.

One exception is on record: `list_provenance` returns `SpaceProvenance[]`, and the strict plural
`provenances` is not idiomatic English, so the resource keeps its mass-noun form. Do not add a second
exception without recording it here.

This is the default namespace. Every Knovum capability owns one: `spaces`, `ai`, `secrets`,
`preferences`, `desktop`, `database`.

App commands registered through the application `invoke_handler` share one flat namespace, so the
owner has to be part of the name:

```text
<feature>_<verb>[_<noun>]    updater_get_status, updater_check, updater_install
```

Only the updater remains there, and it cannot move: `updater` is the official Tauri updater
plugin's key, so an in-process plugin cannot claim it, and the flat namespace is the only place a
Knovum-owned `updater_*` command can live. Its prefix must equal its owning Rust module.

An app command has four touch points, and missing any one of them makes it unreachable or ungranted:

1. the `#[tauri::command]` function itself;
2. the sorted `tauri::generate_handler![...]` list in `src-tauri/src/app_builder.rs`;
3. a manual `commands.allow` entry in `src-tauri/permissions/<feature>.toml` and a capability grant
   naming that permission;
4. a Renderer `invoke("<feature>_<verb>_<noun>")` literal.

A plugin command has four: the `#[tauri::command]` function; its name in
`tauri_build::InlinedPlugin::commands(&[...])` in `src-tauri/build.rs`, which generates the
`<key>:allow-<command>` and `<key>:deny-<command>` ACL at build time; its path in the
`generate_handler![...]` inside `tauri::plugin::Builder::new("<key>")`; and a capability grant,
either as `"<key>:allow-<command>"` or, where the plugin declares
`DefaultPermissionRule::AllowAllCommands`, as `"<key>:default"`. A plugin command has no entry in
the application invoke handler and no hand-written block in `permissions/*.toml`.

The gates assert the corresponding invariants:

- the application invoke handler is sorted, duplicate-free, resolves to real Rust commands, and
  matches the Rust app commands in both directions;
- every app command has a manual permission and a Renderer literal invoke, and every Renderer
  invoke has both;
- the `build.rs` declaration and the `generate_handler!` list are identical, and every handler path
  resolves to a real `#[tauri::command]`;
- every plugin command is granted to at least one capability, and every Renderer
  `plugin:<key>|<command>` literal resolves to a declared, granted command;
- no Renderer string literal uses the flat `<plugin-key>_*` shape;
- every command reference in `docs/commands.md` resolves to a declared command.

Nothing asserts the names themselves. `## Automated boundary` records what the gates still cover and
why command vocabulary is no longer among them.

## Frontend file suffixes

- `*-client.ts`: remote, Tauri IPC, third-party SDK or other communication adapter.
- `*-repository.ts`: aggregate/domain persistence or query port; never UI state.
- `session/<feature>-store.ts`: a feature's Pinia cross-view session composition root.
- `*-controller.ts`: UI lifecycle and multi-primitive orchestration.
- `*-actions.ts`: user/domain actions over one state owner.
- `*-schema.ts`: runtime validation at a real boundary and types derived from that schema.
- `*-types.ts`: static contracts that cannot be derived from a schema; never a duplicate schema source.
- `*-presentation.ts`: pure domain-to-view-model transforms.
- `*-runtime.ts`: resources and lifecycle that exist only for the current process.

Do not use `*-service.ts` when a domain noun or one of the boundary suffixes above states the
responsibility. Deciding whether an existing adapter's behavior matches a suffix remains a code
review responsibility; static checks must not infer semantics from implementation text.

Renderer files must not use `*-database.ts`; the WebView does not own SQLite authority.

## Rust module direction

Use names that reflect ownership:

- `spaces/queries`: read-only Space projections and response mapping.
- `spaces/commands`: atomic state-changing Space application operations.
- `spaces/knowledge`: semantic contracts and knowledge governance.
- `spaces/derivation`: pure revision, passage and chunk derivation.
- `spaces/db.rs`: Space connection/error foundation.
- `local_resources/models/`: locally installed AI model descriptors, downloads, integrity verification and installation lifecycle.
- `local_resources/tools/`: discovery of application-bundled executables and shared native libraries.

Shared/foundation modules must never depend on a feature solely to reuse a helper.

The `queries`/`commands` split is enforced: a `#[tauri::command]` under a `queries/` path must use a
read verb, so a state-changing operation cannot hide in a projection module. Both sides mirror the
same sub-domain names, which is what makes the direction readable — `queries/relations.rs` reads
what `commands/relations.rs` writes, `queries/scenes.rs` reads what `commands/scenes/` writes.

## Variables and parameters

- Boolean names use `is*`, `has*`, `can*`, or `should*` when they cross a local expression.
- Collections use plural nouns; identifiers use `<entity>Id` / `<entity>_id`, and identifier
  collections use `*Ids` / `*_ids`.
- A command parameter is never bare `id`; the entity owns the name (`threadId`, `patchId`,
  `sceneId`, `objectTypeId`). `id` stays legal as a field of a semantic request object
  (`SpaceSceneInput.id`) and as a loop-local, where the type or the collection already names the
  entity.
- The handle for a cancellable async job is always `job_id` / `jobId`, across every feature that
  produces one. A recording session is a session, not a job, and keeps `session_id`.
- A Tauri `Channel` parameter is named for its payload type, and there are exactly three:
  `on_progress` for a typed progress record, `on_stage` for a stage-name string, `on_event` for a
  completion stream event. A command may carry more than one when it genuinely emits more than one
  payload type.
- Maps use `<value>By<Key>`; timestamps use `*At` / `*_at`; durations include their unit.
- Only DOM/component references use `*Ref`; a Vue `ref()` does not make `Ref` part of the domain name.
- `data`, `value`, `result`, and `item` are limited to short, unambiguous local transforms.
- Prefer a semantic options/command object after three parameters. Seven parameters remains the hard
  Rust gate ceiling.
- Complex object-shaped parameters receive a named interface/type near the owning API. Inline object
  types are limited to small, obvious local values such as a two-axis point or a single optional flag.
- Vue components and composables obtain locale context with `useI18n()`. Translation callbacks cross
  into pure policy, command-catalog or IO functions only when keeping those functions independent of
  Vue setup context is intentional; components do not relay `t` through intermediate composables.

## Boolean APIs

A named local boolean is fine when its meaning is obvious. Across module boundaries, avoid calls such as `operation(value, true)` when the flag changes behavior. Prefer explicit operations (`copySelection` / `cutSelection`) or a semantic union/options object (`"active" | "dormant"`, `{ preserveTitle: true }`).

## Local names

Short local names such as `row`, `result`, `value`, `save`, `start`, `busy`, and `editing` are acceptable in small scopes when module/component context makes the meaning unambiguous. Do not expand names mechanically just to make them globally unique.

## Tests

Rust module unit tests live under the owning module's `tests/` directory, whose `mod.rs` is loaded by
exactly one `#[cfg(test)] mod tests;` declaration. Inline test modules, `tests.rs`, `*_tests.rs`,
`#[path]` indirection, and duplicate `mod tests` declarations are forbidden. `src-tauri/tests/` is
reserved for integration tests of cross-module or externally visible crate behavior.

Vue/TypeScript tests live under the nearest common owner's `__tests__/` directory and use the exact
`.test.ts` suffix. A unit test uses the tested source basename with identical casing and lives directly under that source owner's `__tests__` directory, so `shared/ui/UiButton.vue` maps to `shared/ui/__tests__/UiButton.test.ts`. The naming gate enforces this when a test basename has one unambiguous source owner. A test covering an integration scenario across several owners may live at the bounded-context `__tests__` directory and uses a kebab-case user-capability basename instead of pretending to map to one file. Shared test support may use the explicit `*-fixture.ts` suffix under `__tests__`; fixture files are not executable tests.

## Automated boundary

A gate earns its maintenance cost only when it checks something that crosses a boundary no single
toolchain owns, or states a ceiling cheaply enough that nobody argues with it. What remains:

- Rust `Option<T>` in a `Serialize` struct must meet a required, nullable TypeScript field, because
  `vue-tsc` cannot see Rust and `cargo` cannot see TypeScript;
- the `build.rs` `InlinedPlugin::commands()` declaration and the plugin's `generate_handler!` list
  are identical, and every handler path resolves to a real `#[tauri::command]` — a mismatch means
  the ACL is never generated and the command is unreachable at runtime;
- declaration, grant and invoke coverage for both IPC namespaces, in both directions;
- no Renderer string literal uses the flat `<plugin-key>_*` shape, and every command reference in
  `docs/commands.md` resolves to a declared command;
- no capability grants WebView SQL, `fs:default` to the main window, or persistent filesystem
  mutation to `spaces`;
- `local_resources/tools/mod.rs` does not depend on product capabilities, and `ai/` does not depend on `spaces`;
- 300 effective lines per file and 7 parameters per Rust function; Renderer file size is owned by Oxlint;
- Rust module layout: snake_case paths, no inline `mod tests`, no `#[path]` indirection, no
  file/directory module collision; frontend path casing, forbidden `index.ts`, test location and
  `.test.ts` suffix.

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

`tools/` is itself inside the dead-code gate — `knip.json` lists it under `project`, so a script that
loses its last caller is reported — because an enforcement layer nothing enforces is how orphaned
scripts accumulate.
