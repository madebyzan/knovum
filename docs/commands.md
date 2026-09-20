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

- `plugin:spaces|create_nodes` — insert one user action's node batch, supplemental knowledge and Space timestamp atomically;
- `plugin:spaces|persist_canvas_thumbnail` — validate Renderer-produced JPEG bytes and persist the generated canvas thumbnail under the Space-owned asset directory;
- `plugin:spaces|update_nodes` — update node data and any affected source/derived/media knowledge atomically;
- `plugin:spaces|trash_nodes` and `plugin:spaces|restore_nodes` — move a node batch into or out of the reversible bin together with the Space timestamp;
- `plugin:spaces|load_canvas_state` and `plugin:spaces|save_canvas_state` — load and compare-and-swap strict `CanvasDocumentV1`; node rows do not duplicate current canvas geometry;
- `plugin:spaces|index_knowledge` — index current knowledge for a Space or a selected set of nodes;
- `plugin:spaces|answer_grounded_question` — run one grounded answer over the hybrid retrieval result and stream its completion;
- `plugin:spaces|move_space_selection` — move nodes, reconcile affected relations/knowledge and touch both Spaces atomically;
- `plugin:spaces|delete_deleted_nodes` — validate exact deleted-node selections across Spaces, check retained references and delete their associated data in one transaction; collect only unreferenced owned assets for subsequent filesystem cleanup;
- `plugin:spaces|delete_space_permanently` — require an archived user Space, collect all owned asset paths and cascade-delete the Space atomically;
- `plugin:spaces|restore_space_package` — restore one validated/remapped Space package as one transaction.

Agent artifact commands never accept an app-owned absolute source path from the Renderer. `plugin:spaces|stage_generated_artifact_import` resolves a validated Space/session/relative artifact identity in Rust and enters the same prepared Space import pipeline used by user-selected sources; `plugin:spaces|consume_agent_artifact` removes that source only after import succeeds. Preview, discard, recovery listing and creation use the same identity boundary.

Saved Website networking is Rust-owned: the crate-level web capture module performs HTTP fetching and returns a neutral document/image result; `plugin:spaces|fetch_webpage` maps it into a Space response and persists direct images. `plugin:spaces|cache_web_asset` persists optional favicon/preview-image URLs discovered inside a saved document.

## System preview command

- `plugin:spaces|preview_asset` — authorizes one managed Space asset and opens the platform system preview. The engine is internal Rust code and never accepts an untrusted Renderer path directly.
- Canvas surfaces omit the action when the platform has no system preview panel; media retains its own spacebar playback behavior.
