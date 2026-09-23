# Commands and shortcuts

`AppCommand` is the single command contract consumed by the command palette and shortcut layer.

- Space commands live with `features/spaces/workspace.ts` as a command factory that receives the current translation function; titles/descriptions therefore follow the app locale without a second locale listener.
- Window-local shortcuts use `tinykeys` syntax such as `$mod+Shift+P`.
- Commands exposed in the macOS Application Menu set `appMenuShortcut: true`. macOS lets the menu own those accelerators; Windows/Linux bind the same shortcut through tinykeys because no classic application menu bar is installed there.
- The macOS Edit submenu carries only the responder-chain Cut/Copy/Paste. Undo, Redo and Select All are never menu-owned on any platform: their accelerator belongs to the focused control — Tiptap history in the editor, native text editing in a field, canvas history/selection otherwise — and a native key equivalent would consume the press before the WebView saw it.
- The tinykeys binding passes an `ignore` predicate that allows every event, so editable targets are decided per command by the shortcut layer's own guards (`allowInEditable`, bare-key yielding) instead of being dropped before they run.
- Canvas zoom accelerators (`$mod+=`, `$mod+-`, `$mod+0`) stay tinykeys-owned on every platform: the macOS View submenu carries only Fullscreen, so marking them `appMenuShortcut` would leave them unbound on macOS. Each one publishes a typed zoom request that the mounted Canvas applies through its single viewport funnel (`docs/spaces.md`).
- Canvas workbench shortcuts are registered once through `AppCommand` and call the mounted `CanvasEditorPort`; feature-level global key listeners are forbidden. Space/middle-button panning remains local to the focused Konva surface.
- System Preview remains available through its explicit workbench command and does not compete with Space-drag panning.
- Do not add feature-level `window.addEventListener("keydown", ...)`.
- Do not create a second command registry for menus, context menus, or toolbars; adapt those surfaces to command ids.

On macOS, Settings follows the conventional application submenu and Command Palette is exposed under Navigate; the application menu emits `knovum-run-command` with an existing command id. The Shell resolves that id against the same command list used by the palette. Windows uses the same command ids through WebView shortcuts and keeps desktop integration in Tray/Taskbar rather than adding a classic menu bar.

## Application command namespaces

Every Knovum capability except the updater owns an in-process Tauri plugin, so its commands are
addressed as `plugin:<key>|<command>` and named without a feature prefix. The grammar, the touch
points each namespace requires and the invariants the gates assert live in
`docs/naming-conventions.md`; this document records only what a name cannot say.

## Space persistence commands

Space persistence is entirely Rust-owned. Reads and writes cross typed Rust commands; multi-statement invariants execute on one SQLx transaction. These commands accept domain-shaped payloads and none accepts arbitrary SQL from the WebView.

- `plugin:spaces|load_space_edit_snapshot` — load the authoritative Structure/Layout/Edit-State projection and edit revision;
- `plugin:spaces|execute_space_edit` — validate and commit one typed structure command with optimistic revision and idempotent mutation receipt;
- `plugin:spaces|undo_space_edit` and `plugin:spaces|redo_space_edit` — replay Rust-owned aggregate history with revision checks;
- `plugin:spaces|search_structure_global` — search active Group and Section titles across non-archived Spaces without treating containers as knowledge content;
- `plugin:spaces|revoke_relations` / `plugin:spaces|restore_relations` — revoke or restore one relation selection in a single database transaction;
- `plugin:spaces|get_space_edit_receipt_result` — resolve an ambiguous IPC result by mutation identity without retrying a business operation;
- `plugin:spaces|create_nodes` — atomically insert content rows and their Structure nodes and layouts at an explicit parent;
- `plugin:spaces|update_nodes` — update node data and any affected source/derived/media knowledge atomically;
- `plugin:spaces|discard_pending_content` — abandon one failed pending creation by deleting its content and Structure/Layout atomically without creating a Recently Deleted entry;
- `plugin:spaces|load_canvas_view_state` and `plugin:spaces|save_canvas_view_state` — persist only the disposable camera;
- `plugin:spaces|import_structure_clipboard` — validate and commit a remapped structure subtree, content and internal relations as one mutation;
- `plugin:spaces|index_knowledge` — index current knowledge for a Space or a selected set of nodes;
- `plugin:spaces|answer_grounded_question` — run one grounded answer over the hybrid retrieval result and stream its completion;
- `plugin:spaces|move_structure_selection` — move explicit Structure roots, content and affected relations between two revision-checked Spaces atomically;
- `plugin:spaces|list_recently_deleted_structure`, `plugin:spaces|restore_deleted_structure` and `plugin:spaces|permanently_delete_structure` — list, restore or purge deletion batches and selected structure roots while preserving subtree identity and retained-reference rules;
- `plugin:spaces|delete_space_permanently` — require an archived user Space, collect all owned asset paths and cascade-delete the Space atomically;
- `plugin:spaces|restore_space_package` — restore one validated/remapped Space package as one transaction.

Agent artifact commands never accept an app-owned absolute source path from the Renderer. `plugin:spaces|stage_generated_artifact_import` resolves a validated Space/session/relative artifact identity in Rust and enters the same prepared Space import pipeline used by user-selected sources; `plugin:spaces|consume_agent_artifact` removes that source only after import succeeds. Preview, discard, recovery listing and creation use the same identity boundary.

## Governed execution commands

- `plugin:spaces|list_space_actions` projects the single versioned Action catalog; Action execution is internal to Assistant, Workflow and MCP adapters rather than a second renderer dispatcher.
- Workflow commands save one optimistic-revision draft, compile/preflight it, publish immutable versions, instantiate built-in templates, copy personal templates with their fixed-version subworkflow closure, and start whole-flow or isolated selected-step Runs.
- External conversation commands start one explicit ACP runtime session, expose and update only its actual ACP mode/configuration options, send additional turns with explicitly selected attachments only while that exact session remains connected, list durable history and close the session without silent reconnection.
- Run commands list Runs, steps, per-item batch facts and ordered events; request pause/resume/cancel; continue a persisted debug breakpoint; provide human input/review; reconcile unknown external outcomes; retry selected steps into a new Run; and produce a content-redacted diagnostic bundle.
- Assistant commands own conversations, branching, archive/search/export and durable tasks. Closing a view does not own task lifetime.
- Automation commands own schedule preview, persisted triggers, occurrences, explicit renewal and enable/disable. Import never enables a trigger or restores its grant.
- External-tool commands test a connection without persistence, save a reviewed schema fingerprint, disable a binding, or delete it only when no Workflow definition references it.
- The Space MCP surface composes independent read, Action, Workflow and Run routers. Mutations still enter SpaceAction authorization; Run control additionally requires the same external principal that created the Run.
- `plugin:spaces|decide_external_agent_permission` resolves one actual ACP permission request using only an option advertised by that request; absence of a decision is cancellation, never implicit approval.

Saved Website networking is Rust-owned: the crate-level web capture module performs HTTP fetching and returns a neutral document/image result; `plugin:spaces|fetch_webpage` maps it into a Space response and persists direct images. `plugin:spaces|cache_web_asset` persists optional favicon/preview-image URLs discovered inside a saved document.

## System preview command

- `plugin:spaces|preview_asset` — authorizes one managed Space asset and opens the platform system preview. The engine is internal Rust code and never accepts an untrusted Renderer path directly.
- Canvas surfaces omit the action when the platform has no system preview panel; media retains its own spacebar playback behavior.

## Commands without a Renderer consumer

These declared commands have no production Renderer consumer. The list is documentation, not a gate
registry: nothing enforces it, so it is only as true as the last review that read it. Treat each
entry as authorized debt and prefer deleting an unused command — an unreachable command still
widens the WebView ACL, which `AGENTS.md` forbids granting preemptively.
