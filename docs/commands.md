# Commands and shortcuts

`AppCommand` is the single command contract consumed by the command palette and shortcut layer.

- Space command composables live in `features/spaces/commands/`. `features/spaces/use-spaces-workspace.ts` exposes their composition contract; Shell supplies presentation operations during setup, and `useI18n` keeps command titles/descriptions reactive to the app locale.
- Window-local shortcuts use `tinykeys` syntax such as `$mod+Shift+P`.
- Commands exposed in the macOS Application Menu set `appMenuShortcut: true`. macOS lets the menu own those accelerators; Windows/Linux bind the same shortcut through tinykeys because no classic application menu bar is installed there.
- The macOS Edit submenu dispatches palette-hidden `editing.cut`, `editing.copy` and `editing.paste` without native accelerators. When a WebView input or editor is focused, the desktop command sends AppKit's fixed Cut/Copy/Paste selector through the first-responder chain, preserving system rich-text clipboard and native undo behavior. Outside an editable target, the same commands dispatch `spaces.cut`, `spaces.copy` or `spaces.paste`; those Space commands own the canvas shortcuts on every platform. Undo, Redo and Select All remain with the focused control or canvas command.
- The tinykeys binding passes an `ignore` predicate that rejects repeat and composing events, so editable targets are decided per command by the shortcut layer's own guards (`allowInEditable`, bare-key yielding) instead of being dropped before they run.
- Canvas zoom accelerators (`$mod+=`, `$mod+-`, `$mod+0`) stay tinykeys-owned on every platform: the macOS View submenu carries only Fullscreen, so marking them `appMenuShortcut` would leave them unbound on macOS. Each one publishes a typed zoom request that the mounted Canvas applies through its single viewport funnel (`docs/spaces.md`).
- Canvas workbench shortcuts are registered once through `AppCommand` and call the mounted `CanvasEditorPort`; feature-level global key listeners are forbidden. Space/middle-button panning remains local to the focused Konva surface.
- System Preview remains available through its explicit workbench command and does not compete with Space-drag panning.
- Do not add feature-level `window.addEventListener("keydown", ...)`.
- Do not create a second command registry for menus, context menus, or toolbars; adapt those surfaces to command ids.
- `runAppCommand` is the single execution decision for palette, tinykeys, native application-menu and native context-menu entry points. An open modal dialog suppresses commands unless that dialog explicitly lists allowed command ids; closing the command palette completes before its selected command enters the dispatcher.
- Canvas right-click and Shift+F10/Menu-key invocation use a native Tauri Menu composed from the registered `AppCommand` objects. Invoking a menu on an unselected hit target selects that target first, while invoking it inside an existing multi-selection preserves the selection.

On macOS, Settings follows the conventional application submenu and Command Palette is exposed under Navigate; the application menu emits `knovum-run-command` with an existing command id. The Shell resolves that id against the same command list used by the palette. Windows uses the same command ids through WebView shortcuts and keeps desktop integration in Tray/Taskbar rather than adding a classic menu bar.

## Application command namespaces

Every Knovum capability except the updater owns an in-process Tauri plugin, so its commands are
addressed as `plugin:<key>|<command>` and named without a feature prefix. The grammar, the touch
points each namespace requires and the registration review checklist live in
`docs/naming-conventions.md`; Rust/Tauri compilation and review keep those touch points aligned.

## Local resource preparation commands

`plugin:local-resources|prepare_resources` accepts one finite preparation target: transcription, text embedding
or media-import tools. It accepts a job identity and progress Channel, never a download URL
or destination path. `cancel_resource_preparation` releases that request's demand;
`list_resource_preparations` returns active requests and bounded terminal history for the same
Task Center identities after a WebView reload. A disconnected Channel does not cancel installation.
There are no legacy model-install command aliases.

`plugin:spaces|request_media_transcription` accepts a Space and media node identity. Rust resolves
the managed source and current analysis, enqueues the shared durable transcription job, and verifies
the source revision before resource preparation and again before processing. Knowledge
index commands require verified models; the explicit semantic-search and grounded-generation
use cases own resource preparation before indexing. A normal keyword query never downloads models. Search commands carry a job identity; TanStack
query cancellation calls `cancel_knowledge_search`, which cancels queued work and discards stale
results while any already-running blocking computation retains its resource leases.

## Space persistence commands

Space persistence is entirely Rust-owned. Reads and writes cross typed Rust commands; multi-statement invariants execute on one SQLx transaction. These commands accept domain-shaped payloads and none accepts arbitrary SQL from the WebView.

- `plugin:spaces|load_space_edit_snapshot` — load the authoritative Structure/Layout/Edit-State projection and edit revision;
- `plugin:spaces|execute_space_edit` — validate and commit one typed structure command with optimistic revision and idempotent mutation receipt;
- `plugin:spaces|undo_space_edit` and `plugin:spaces|redo_space_edit` — replay Rust-owned aggregate history with revision checks;
- `plugin:spaces|search_structure_global` — search active Group and Section titles across non-archived Spaces without treating containers as knowledge content;
- `plugin:spaces|revoke_relations` / `plugin:spaces|restore_relations` — revoke or restore one relation selection in a single database transaction;
- `plugin:spaces|get_space_edit_receipt` — resolve an ambiguous IPC result by mutation identity without retrying a business operation;
- `plugin:spaces|create_nodes` — atomically insert content rows and their Structure nodes and layouts at an explicit parent, appending to the structure loaded in that write transaction instead of a caller revision;
- `plugin:spaces|update_nodes` — normalize authoritative knowledge revisions, update affected knowledge atomically, and return the committed node payloads;
- `plugin:spaces|discard_pending_content` — abandon one failed pending creation by deleting its content and Structure/Layout atomically without leaving soft-deleted state for undo, against the structure loaded in that write transaction;
- `plugin:spaces|load_canvas_view_state` and `plugin:spaces|save_canvas_view_state` — persist only the disposable camera;
- `plugin:spaces|import_structure_clipboard` — validate and commit a remapped structure subtree, content and internal relations as one mutation;
- `plugin:spaces|index_knowledge` — index current knowledge for a Space or a selected set of nodes;
- `plugin:spaces|move_structure_selection` — move explicit Structure roots, content and affected relations between two revision-checked Spaces atomically;
- `plugin:spaces|delete_space_permanently` — require an archived user Space, collect all owned asset paths and cascade-delete the Space atomically;
- `plugin:spaces|import_space_package` — read a selected current-version archive, validate/remap its references and assets, restore it transactionally, and return the new Space;
- `plugin:spaces|export_space_package` — collect the Space snapshot and write its archive inside one Rust use case;
- `plugin:spaces|import_json_canvas_file` — read the selected file, acquire bounded relative resources, map domain content and commit against the expected Space edit revision;
- `plugin:spaces|cancel_json_canvas_import` — request cancellation of the complete import by mutation ID; copied assets are cleaned up before the operation settles; a committed receipt wins a late cancellation;
- `plugin:spaces|export_json_canvas_file` — validate the projected visual document and publish the document and its exact managed resources;
- `plugin:spaces|request_media_transcription`, `plugin:spaces|retry_enrichment_job`, `plugin:spaces|cancel_enrichment_job` — submit control intents to the Rust processing owner;
- `plugin:spaces|generate_media_insights` — generate and publish insights against the captured current media knowledge;
- `plugin:spaces|await_space_runtime` — await the Rust-owned startup recovery gate without rerunning recovery.

Agent artifact commands never accept an app-owned absolute source path from the Renderer. `plugin:spaces|stage_generated_artifact_import` resolves a validated Space/session/relative artifact identity in Rust and enters the same prepared Space import pipeline used by user-selected sources; `plugin:spaces|consume_agent_artifact` removes that source only after import succeeds. Preview, discard, recovery listing and creation use the same identity boundary.

## Governed execution commands

- `plugin:spaces|list_space_actions` projects the single versioned Action catalog. `plugin:spaces|start_space_action` accepts canvas resource creation with source IDs/revisions and an optional explicit target; Rust owns authorization, target resolution and the Action Run. Builtin Agent, Workflow and MCP reuse the same invocation/execution boundary.
- Workflow commands save and revise personal workflows captured from canvas derivations, start a selected personal workflow, and inspect its Run.
- External conversation commands start one explicit ACP runtime session, send additional turns with explicitly selected attachments only while that exact session remains connected, list durable history and close the session without silent reconnection.
- Run commands list Runs, steps and ordered events; request pause/resume/cancel; resolve a resource review; reconcile unknown external outcomes; retry failed workflow steps into a new Run; and produce a content-redacted diagnostic bundle.
- Conversation drafts use `load_conversation_draft`, `save_conversation_draft`,
  `remove_conversation_draft` and `move_conversation_draft`; the Spaces baseline owns unsent text
  and managed attachment references. External-file display names require picker/drop authorization
  again after restart.
- Builtin Agent conversation reads are `list_conversations`, `list_conversation_messages` and
  `load_conversation_branch_messages`; the last returns the complete branch for Renderer-owned file export.
  `archive_conversation` and `unarchive_conversation` are separate intents accepting
  only a conversation identity. Task state reads live in conversations/queries; closing a view does not own task lifetime.
- The Space MCP surface exposes explicit read, Action and Run routes. Mutations enter SpaceAction authorization; Run control additionally requires the same external principal that created the Run.
- `plugin:spaces|decide_external_agent_permission` resolves one actual ACP permission request using only an option advertised by that request; absence of a decision is cancellation, never implicit approval.

Saved Website networking is Rust-owned: the crate-level web capture module performs HTTP fetching and returns a neutral document/image result; `plugin:spaces|capture_web_resource` maps it into a Space response and persists direct images. `plugin:spaces|import_web_image` persists optional favicon/preview-image URLs discovered inside a saved document.

## System preview command

- `plugin:spaces|preview_asset` — authorizes one managed Space asset and opens the platform system preview. The engine is internal Rust code and never accepts an untrusted Renderer path directly.
- Canvas surfaces omit the action when the platform has no system preview panel; media retains its own spacebar playback behavior.

## Command registration review

When adding or removing an IPC command, review the Rust handler, build-time plugin registration, generated permissions, window capability and production caller together. This document does not maintain an unverified list of unused commands. Knip checks Renderer reachability within its configured entries; it does not prove Rust handler reachability or grant correctness. The exact namespace and synchronization policy is in [naming-conventions.md](naming-conventions.md).

## Generation, rendering and desktop lifecycle names

Editor text transformations use `plugin:ai|transform_editor_text_stream`; translation uses
`plugin:ai|translate_text_stream`. Both operations stream progress and retain explicit cancellation.
Internal provider completion APIs keep their model terminology. Spaces generation enters `start_space_action` with a concrete versioned Action ID;
specialized image-layer generation uses `generate_image_layers`; transformations use `derive_image`, and deterministic
video edits use `render_video_edit`; subtitle burn-in uses the `add-video-subtitles` Action. Cancellation names the ongoing
operation (`cancel_image_layer_generation`, `cancel_run`). Image-layer generation
uses `generationId` for its multi-output request and cancellation; each output has its own derivation ID.
No legacy `id` field is accepted by that strict request. Video generation and committed video rendering cancellation use `derivationId`; disposable proxy processing retains `jobId`.

`get_image_layer_availability` reads Ready/Blocked environment readiness;
`get_image_operation_capabilities`, `check_image_cutout_source` and `list_video_fonts` are queries.
They share the Spaces transport namespace with state-changing commands.

Desktop startup calls `plugin:desktop|mark_exit_coordinator_ready` to announce that exit decisions can
be handled, and `reveal_main_window` to show either the usable product or recoverable error surface.
The latter does not claim bootstrap succeeded. The flat updater commands are
`get_update_configuration`, `check_for_update` and `install_update`; their manual permission grants
are synchronized with the app handler and Renderer callers.
