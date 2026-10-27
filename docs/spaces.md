# Spaces product model

Outline rows expose OS-native context-menu Rename and the `spaces.rename-node` AppCommand (`F2`, one selected persisted node). Inline editing trims and validates the existing 1–512 character node title contract; Enter commits, Escape or leaving the field cancels, and IME confirmation does not commit. Rename reveals and expands every container ancestor, including Sections, before scrolling to the target.
The input uses shared IME-safe commit/cancel events; losing focus while a save is pending does not release
the edit lease. Failed saves retain the draft, restore input focus and show a localized error. The existing node-data/history pipeline persists the change, updates canvas/outline/search projections and supports undo/redo. Title changes do not modify filenames, URLs or source revisions, and media-enrichment intents synchronize only when their source, visual input, analysis or processing state changes. Website capture records its source `pageTitle` independently of the editable knowledge title; refresh preserves the knowledge title.

**Knovum** is a spatial knowledge application. The knowledge-space capability is owned by
`src/features/spaces/*`; `spaces` is the bounded-context and IPC namespace, not a peer application
inside a multi-product container. Its sidebar, toolbar and selection-driven inspector are composed
into the stable desktop shell, while Settings is a separate application route.

## Work surfaces and session ownership

The workbench owns one ordered list of tabs and one active tab. A tab is a Space canvas, a document
editor or a video editor; editor tabs identify a concrete Space-owned node and editor kind, and
reopening that target activates its existing tab instead of creating a second one. The active
surface is a readonly projection of the active tab and the navigation destination, never a second
writable editor selector. A Space session stays loaded while at least one tab references it, and is
released once the last one closes. The library is an application navigation destination and
preserves the active tab while hidden.

Tabs preserve drafts and editor history when switching tabs, Spaces, the library or Settings.
Selecting a Space in the sidebar focuses that Space's canvas tab, opening one when it has none.
Closing a tab is requested from the tab strip alone; editor surfaces expose no close or back action.
Closing an editor prepares only its owner, then releases the tab and its resources. Archiving or
deleting a Space prepares all its editor owners; cancelling a later decision never drops earlier
drafts or executes a prepared discard. Actual application exit prepares every retained editor
through the existing window close coordinator. Successful durable writes need not be rolled back
after a later cancellation. The Inbox canvas tab is permanent, so the workbench always retains one
tab. Tabs and unsaved memory drafts do not survive application restart. Document auto-save and
saved video projects retain their existing durable storage owners.

Document sessions own their drafts; retained Tiptap instances own document history and selection.
Video sessions own project drafts, history, the selected item, playback position and the rejection
state of a project whose stored payload fails validation.
Only the active editor registers commands. Hidden video views stop their render loop, silence
scheduled audio and suspend thumbnail work; preview preparation is cancelled while hidden. Document timers pause while inactive, and navigation
flushes pending document writes before changing the active Space. Save failures retain the draft and
block the requested transition. Saves and task callbacks bind to explicit Space/node identities.
Markdown writes its text and rich document together; Rust validates that document and the saved video
project at the node boundary. Project saving and rendering share the video owner's structural rules.

The retained canvas preserves camera, selection and Group scope during same-Space handoffs. Transient
gestures and inline input suspend; inactive canvas navigation requests wait until canvas is active.
Canvas Outline and contextual side panes belong only to the canvas; editors own their local controls.
Document citation previews close on content handoff without discarding the document session.

An open document/video tab protects its live resource's existence and Space ownership, even while
inactive or while the library is shown. Deletion, deletion of containing Groups/Sections, and
cross-Space transfer require closing every affected tab first. A mixed selection is rejected as a
whole, never partially deleted. Layout, grouping, selection and user-authored locks remain separate.
Protection is derived from existing tab identities; it is not a persisted lock or another lease store.
Structural edit, transfer and Undo/Redo IPC carry the current editor resource identities. Rust captures
their live ownership inside the write transaction and rejects a result that loses any identity before
commit, including coordinated history in another Space. The entire transaction rolls back, preserving
its revision and history position. Resources already missing before the transaction are error-state
sessions, not a reason to block unrelated work. Tab opening waits for admitted mutations before it
creates an editor owner, so its protection cannot appear halfway through a structural transaction.
Closing a tab releases protection only after the existing close/save decision succeeds. Space close
and archive explicitly end their owned sessions through the same scoped close protocol.
Clean editors reload changed resources with a fresh history. Dirty conflicts or missing owners keep
the draft, expose an error and require an explicit discard before closing; they never overwrite new
content or write back a missing node. Video rendering captures the source Space and project snapshot,
publishes into that Space through the existing Task Center bridge and offers an explicit result action.
Completion never closes an editor or changes the current navigation.

## Product model

```text
Spaces application
└── Space
    ├── Resources       source material
    ├── Knowledge Objects
    │   ├── Claim / Evidence / Question / Decision
    │   └── strict custom Object Types
    ├── Projections
    │   └── Portal
    ├── Relations + Provenance + Knowledge History
    ├── Derivations     traceable resource transformations
    ├── Groups          operation units
    ├── Sections        visible spatial regions
    ├── Saved Views     viewport + focus + dynamic filters
    ├── Tags            semantic facets
    └── Find / Ask / Work / MCP
```

A **Space** is an infinite spatial surface and the durable boundary for Relations, Provenance, atomic Knowledge changes and local Knowledge history. The system **Inbox** is a stable Space kind used for low-friction capture and is also a valid Move target. Global Search and All-Spaces Ask may read across Spaces, but they do not create cross-Space Relations or Provenance.

Space catalog covers and Saved View previews are rendered from the neutral canvas scene through one non-interactive Konva thumbnail renderer. Whole-Space covers fit all knowledge content into the catalog frame; Saved Views project the current canvas viewport into the same frame. Product chrome, selection state, resize handles, toolbars, zoom controls and the editing grid never enter the render input. The renderer uses the same canvas geometry, canvas background and bounded local preview resources as the active scene without cloning the mounted DOM.

Canvas tools are transient Space session state, separate from persisted canvas geometry. The top-center Reka toolbar exposes Select (V), Pan (H), Note (N) and Text (T). Outline belongs to the current-Space region of the sidebar; Space management belongs to native context menus, not the tab strip. Tool shortcuts enter through AppCommand, not feature-level keyboard listeners. Opening Outline reveals the sidebar through its neutral presentation contract. Note and Text place empty knowledge at the next canvas click without changing the viewport, then return to Select and activate inline editing. An inline editing session lasts while the user is on its surface — the node, its selection bar and the menus that bar opens (`canvas-text-surface.ts`) — and ends on a press or focus off it; editor blur is not leaving, because the bar's controls take focus while they are used. The bar's surface is the floating menu's own root, not its content: WebKit never focuses a pressed button and moves focus to the nearest focusable ancestor, which is that root. Pan routes left drags through the existing viewport controller and disables node manipulation, relation hit testing and DOM content interaction. Held Space and middle-button drags use that same controller; trackpad scrolling uses the same authored-content navigation admission. Switching Spaces resets the tool.

Space management is available through native context menus on Space cards, tabs and Inbox. These entries share one feature-owned menu composition and always bind to the selected Space. Knowledge Health opens the actual inspection capability, not a misleading Space Details page; entering the library closes the contextual Inspector. Space package import belongs to the library toolbar and creates a new Space. Space package export commits the active session before collecting its transactional snapshot. JSON Canvas export is an advanced exchange option for the active canvas, not a knowledge backup; JSON Canvas import remains available through the command palette. The incomplete automatic backup capability is removed; existing backup files are not deleted and remain importable Space packages.

Space package import accepts only the current package version. Rust owns all identity/reference
remapping, validates original ownership and canonical citation revisions before remapping, and
removes the import's asset namespace after a failed restore. Interrupted package namespaces are
reconciled against committed Space identities at startup and before backup admission. Cleanup failures
preserve the original error code and report the remaining staging work. Package transfers emit an
admission acknowledgement on their existing progress Channel after acquiring the engine lease;
cancellation waits for this boundary before invoking the owner. Old development database checksums and
package versions fail explicitly; they are not read through compatibility paths.

JSON Canvas import submits the selected file path and expected edit revision to Rust. Rust validates
section membership, resolves relative files within the canonical source directory, prepares managed
assets and commits nodes, structure, relations and history together. Vue loads that committed
projection and controls selection/reveal. Import failure cleanup belongs to the same Rust use case.

JSON Canvas export writes through one Rust command authorized by the user-selected `.canvas` destination. The command validates the document and its exact managed-resource set, prepares a uniquely owned sibling resource directory, rewrites relative references, flushes a complete staging document and replaces the selected file through a same-directory rename. Failed preparation cleans up only that publication's files. Existing exported resource directories are not modified or deleted: they may still be used by external documents. No frontend filesystem scope is broadened. Space package archives likewise replace their destination directly from complete staging files, without moving the old destination aside first.

Space packages export active knowledge, not soft-deleted content retained for undo. Derivations are included only for active outputs. Each input preserves its historical `sourceIdentity`, revision and immutable `sourceSnapshot` (title, kind, original URL and platform); its nullable `nodeId` is a current-space link, not the historical identity. Permanent source deletion sets that link to null rather than cascading away the input history. Export detaches links to deleted or out-of-space sources; restore preserves these snapshots without restoring deleted nodes. Workflow output links include only active nodes. Knowledge events are selected against the actual exported node, relation, provenance and custom-type identities. Space packages are v1 and shipped: a later package revision raises the package version and the importer keeps reading each shipped version, so an archive somebody exported from Knovum stays importable.

The sidebar keeps Space navigation fixed above a directly visible Outline and the Settings entry fixed below it. Inbox comes first as the capture destination for knowledge saved before choosing an organizational Space, followed by All Spaces, Recently Opened and Favorites. These four entries use consistent row spacing without an extra divider or gap after Inbox. Only the Outline list scrolls. A subdued Outline heading identifies the content region without repeating the current Space title already shown in its tab. Whitespace separates navigation from the Outline; there are no decorative divider rules or whole-Outline disclosure state. The `spaces.focus-outline` command (`$mod+Shift+O`) reveals the sidebar and focuses its content region on every invocation. Section disclosures remain because they represent actual document structure.

The Outline's Locate action enters the target Group scope, expands its owning collapsed Section and
focuses its canvas frame. Ordinary row selection only ensures visibility.

Truncated Outline titles scroll in one direction after a short hover delay and remain at the end until the pointer leaves. They never reverse or loop. Titles that fit stay still. Measurement and resize observation are limited to the hovered row; unchanged geometry does not restart the animation. Leaving or unmounting cancels the Web Animations API animation and restores truncation. Reduced-motion preferences disable scrolling, while the original full title remains available to accessibility and the browser title hint.

Deleting canvas content soft-deletes it so undo and redo can restore it exactly: Structure rows become tombstones that keep their StructureNodeIds and layouts, content rows keep their payload and assets, and relations keep their lifecycle. There is no separate recovery surface; undo is the only way back, so soft-deleted state lives exactly as long as undo can reach it. History keeps the last 100 edits per Space and discards its redo branch on a new edit. When the Space runtime starts, Rust erases every soft-deleted Structure row that no retained history snapshot contains and every soft-deleted content row that no retained history entry lists as active, unless a live Portal or object field still targets it — erasing that would leave the reference dangling, so it waits until the reference is gone. Erasure removes the content's relations, provenance, knowledge events and grounding references in one transaction; filesystem cleanup receives only post-commit candidates and removes a file only after the asset command confirms it is unreferenced.

The Add menu complements, rather than duplicates, Note and Text tools. Separators distinguish direct creation (Markdown and Structured Knowledge), content acquisition (Add Website, Import Files and Audio Note), Agent creation, and association/configuration (Space Portal and Credentials). This order describes entry purpose, not an assumed ranking of usage frequency. Acquisition actions remain directly visible rather than being nested under a content type. Structured Knowledge contains Concept, Question, Claim, Evidence, Experiment and Decision, followed by custom object creation and type management. Credentials contains API Key and Account Password. Only these meaningful type categories use a second level; there is no miscellaneous More submenu. Entries requiring a dialog or picker carry an ellipsis; immediate creation entries and submenu triggers do not. Agent creation retains its actual desktop Agent identity and shares the same translated name between entry and dialog. Structured system labels use the dedicated `spaces.knowledgeObjectTypes` translation namespace, also used when creating their titles. Presentation emits explicit actions to the existing creation/dialog owner; menu ordering does not introduce a separate command or persistence layer.

The viewport controller owns the live camera. Pointer panning applies and publishes the latest screen-space displacement once per animation frame through the existing document save queue. Navigation, scrolling and external camera changes take ownership by consuming the final pending pan frame, clearing the old pointer anchor and cancelling older animation/publication work. Pointer cancellation, tool changes, Escape and window blur finish panning at the current position; they do not roll the camera back. Pointer capture remains owned by pointer interactions and is released when panning ends or its scope is disposed. Navigation transitions explicitly interpolate the same camera; there is no background transition target watching and rewriting pointer coordinates. Document synchronization ignores the model's own echoed document, applies genuinely changed external cameras and merges content/layout-only updates with the live camera. The canvas host synchronously flushes the surface camera before switching its persistence session or disposing the save queue; the surface also flushes on teardown without publishing an already-published camera again.

The organizational concepts are deliberately orthogonal:

- **Group** is a real Structure node and transform boundary. It can nest, move, rotate, scale, rename and define an editing scope; selection operations act on effective roots.
- **Section** is a non-nesting Structure container with a title and explicit spatial frame. Membership changes only through explicit creation, reparenting or a completed drop.
- **Saved View (Scene)** is a named projection over the same Space. It owns no knowledge and stores viewport plus focus StructureNodeIds, relation filters and an optional dynamic query filter.
- **Tag** describes meaning independently of location.
- **Find** retrieves content when the user remembers what it said but not where it lives.

Do not collapse these concepts into a folder tree or a single generic Group abstraction.

## Knowledge State

The canvas is a projection over durable knowledge, not the data model itself. Resources preserve source material; `object` nodes carry strict typed Knowledge Objects; Portal nodes are navigation projections. Relations, Provenance and temporal Knowledge Events are first-class Space-local records. Agent output enters long-term state only through a validated atomic KnowledgeChangeSet. The loopback-only MCP server exposes reads and the same Rust-owned commit use case; it never exposes SQLite or partial mutations. Every published tool schema is a described object down to each property: MCP clients built on the reference SDK validate `tools/list` as a whole, so one free-form (boolean) property schema makes the client drop the entire Knovum tool list and report the tools as missing. Tool arguments therefore use typed contracts, and schemas borrowed from an external MCP server are conformed before they are re-published. See `knowledge.md` for the current contract.

## Knowledge Derivation

Implemented action families include image background removal and the video-node **Add subtitles** derivation described in [Video subtitles](#video-subtitles). Both publish a separate resource and preserve the source.

Spaces treats generation and media transformation as **Knowledge Derivation**: selected knowledge produces a new durable resource without losing its source context. Generations (image, video, document) run as **in-canvas generators**: a frame sized to the output with a composer beneath it, opened from the add menu's _Generate_ group or beside a selection with _Create from selection_; references are optional (see [Creation sources](creation-sources.md#what-the-user-sees)). Transformations (Remove background, Split layers, Add subtitles) start from the resource they transform. Provider connections remain infrastructure configuration. Compatible model selection is available at the creation surface and defaults by capability. Models are execution targets, not ordinary knowledge nodes and not the canvas information architecture.

Operations are classified by their durable effect, independently of their toolbar, engine or AI use:

| Effect          | Durable result                                                                             |
| --------------- | ------------------------------------------------------------------------------------------ |
| Inspect         | Read current state; disposable caches do not create history or lineage                     |
| Edit            | Keep the node/Relation/Structure identity and commit its validated change/history          |
| Create primary  | New authored or acquired resource, without transformation lineage                          |
| Clone           | New identity preserving copied state, without claiming a Derivation                        |
| Derive          | New resource from pinned inputs, with a Derivation and successful `derives-from` Relations |
| Export external | Explicit external destination or system handoff, without creating a Space node             |

Interactive drafts hold a separate occupancy gate until Apply, Cancel or gesture completion.
Inspection and navigation remain available; competing edits, structure mutations, creation, cloning,
export and derivation are blocked at both the originating control and submit entry. The gate projects
the owning session, rather than copying its draft. Submitted background work releases this gate and
retains its pinned input. An operation must not both edit its source and publish a derivative under
one ambiguous intent. Indexing, thumbnails, media proxies and node-owned enrichment are maintained
projections, not user-facing Derivations; `creates_resource` authorization alone does not claim lineage.

Revision identity follows the consumed data: `sourceRevision` identifies managed source bytes;
`rendered` identifies bytes plus the normalized image/video edit snapshot; `knowledgeRevision`
identifies semantic content; Structure's `editRevision` fences layout and membership only.
Derivation inputs record their `asset`, `rendered` or `knowledge` basis and consumed snapshot.
The basis is resolved once, where a selection is pinned — pixels for an image, indexed content for
text — and then travels with the input, so no handler assumes a basis of its own and a resource
nothing has analyzed yet is still something to create from. Rust rechecks that basis and edit/project
before publication. A later source edit marks the old
output historical without rewriting or regenerating it. Image background removal/upscale consume
`rendered`; subtitle burn-in pins video bytes and transcript knowledge; viewport movement consumes
neither. Ordinary content edits compare the complete expected payload atomically, so stale edits
cannot overwrite newer content or comments. Preview and external export use current rendered content.

A **Capability** executes one product operation: document/image/video generation, document export or a media transformation. Selecting several inputs grounds that one operation in several sources; it never becomes a hidden multi-step program. Reuse comes from the derivation record a result already carries, not from a separately stored recipe. Asking a model to change an attached image is not a separate kind of request: pixels travel with the prompt, and the prompt says what to do with them. See `creation-sources.md`.

The canvas starts registered generative capabilities and document exports through `start_space_action`. Rust pins the selected inputs, resolves and freezes the execution target, and creates an authorized Action Run in one transaction. Vue observes the shared Task Center; it does not sequence hidden generation steps or publish output bytes.

The Rust Run scheduler is the only execution authority. Every capability uses the same Action executors, leases, cancellation, result validation and publication rules. Each Run freezes the data digests of its pinned inputs, so later edits or settings changes cannot silently change a running task.

A **Creation Intent** is a localized capability entry with selection availability. Every Action is named once, in `spaces.creation.actions`, so creation menus and Agent activity cannot drift apart. Each Run has its own execution grant; retry reuses only successful predecessor results whose recorded resources remain valid.

Image transformations are ordinary Actions: `upscale-image`, `generate-image-angle`, `remove-image-object`, `remove-image-background`, `extract-image-selection` and `generate-image-depth` use the same authenticated Invocation, durable Run and committed Receipt as generation. Transient masks and selection previews do not produce Action receipts.

The Builtin Agent has one visible conversation and message store for API models and desktop Agents.
The input chooses an enabled target for each turn. Every desktop Agent turn records its own Run,
receives the selected Space resources through validated asset paths, and writes its streamed reply
into the shared conversation. The turns of one conversation share one **runtime** — one Agent
process, one ACP session, one Knovum MCP server and one workspace
(`spaces/external_conversations/runtime/pool.rs`) — so a follow-up is a single `session/prompt`
carrying only the new request: no launch, no initialisation, no replayed history. A runtime is
continued only while it holds exactly the conversation the turn continues: same Agent, same
branch, same extension selection, and the branch's last message is the one its own previous turn
left. An edited or regenerated message, another branch, a built-in Agent reply in between, a
changed extension selection or a lost process all close it, and a new runtime is given the stored
history instead. Cancelling a turn sends `session/cancel` and waits for the Agent to stop; the
runtime stays. Idle runtimes close after 15 minutes and all of them when the app exits. Tool
activity is recorded as Run events, while Space changes still require Knovum MCP receipts; the
MCP server answers only during a turn, under that turn's grant and Run. A desktop Agent reaches the Space
through that server but reads source files itself: attachments arrive as local paths and
`get_node_asset` resolves a node's current file, and when indexed text is missing or thin — slides or
scans kept as pictures — it opens the file with its own tools rather than reporting metadata. The
one composition of these instructions (`prompt_policy::desktop_agent`) feeds both the prompt and the
MCP server's instructions, and asks for Skill discovery only when Skills are selected. The former
grounded-answer threads and messages are moved into the Builtin Agent store by a one-time local
database migration; their verified source snapshots remain attached to the corresponding reply.
A turn publishes what it left in the conversation's workspace through the existing reviewed import
path, and publication empties the workspace for the next turn; a new runtime starts from an empty
one. Each published deliverable `derives-from` every canvas node attached to the turn, written with
the node in the publication transaction: the turn's attachments are its inputs exactly as a
selection is an Action's, so the source shows its derived outputs and the lineage line like any
other derivation. Attached local files are not canvas nodes and have no line. In a tool call's content only what the tool embedded (image, audio, resource) is an output: a
`resource_link` there names a file that lives elsewhere — the source the Agent read, the image it
looked at — and is shown, never fetched, copied or published.

Saving a resource in another format is an export rather than a derivation: the same content in a second file is not new knowledge, so it belongs to the destination the person picked instead of to the canvas. Admitted formats per resource kind and fidelity limits belong to [resource-export.md](resource-export.md).

AI execution selection follows the task's explicit target, then its capability override, then the user's default execution method. An enabled desktop Agent uses its own authentication and tools; a selected user model uses its registered connection and credential reference. Local deterministic Actions do not require either. Missing or failed execution paths remain visible and do not switch to another account.

Each Run freezes the resolved execution selection. Completed derivations record the actual engine or model identity; desktop Agent execution does not invent a model, token count or fee. Changing a default never changes an existing Run or generated resource.

A successful derivation:

- preserves its input resource IDs and exact source revisions;
- records the action, bounded parameters, engine/model identity, actor, timing and outcome in a structured Derivation record;
- creates the output as an ordinary Space resource that can be edited, searched, exported and used as a later input;
- creates `derives-from` Relations from the output to its inputs in the same transaction that reserves the output, so lineage exists while the work runs instead of appearing when it finishes;
- places the result in the derivation field beside its sources without moving the inputs;
- distinguishes generated artifacts from accepted Knowledge claims.

Transformation lineage and epistemic Provenance are different contracts. A Derivation explains how an artifact was produced. Provenance records evidence supporting a Knowledge claim or Relation. Generated factual or semantic changes enter durable Knowledge only after complete KnowledgeChangeSet validation and one atomic transaction.

Long derivations create a pending output node immediately. The pending node reserves the eventual result position and mirrors the feature-owned Rust job's meaningful stage and progress through Task Center. Success replaces it in place; failure remains inspectable and retryable but does not create a false completed resource or semantic lineage.

Layer splitting is the one derivation whose output count is discovered rather than requested. It
still reserves one pending node before the handler runs, like every other derivation, so the click
lands on the canvas immediately instead of only in Task Center. Discovery then reports how many layers the image carries: the first layer publishes
into the node the Run reserved, the rest are reserved in the same transaction, and the whole set is
laid out as one grid beside the source. It is one Action invocation and therefore one Run, one Task
Center task and one cancellation; a split that fails or is cancelled settles its reserved nodes —
the node reserved up front included, whether or not the derivations ever opened — and deletes the
layer files it had written. Each layer is cropped to the pixels it carries, so a subject arrives as a
node at its own size rather than inside a canvas-sized transparent frame, and a layer that carries no
content of its own is never published: an empty cut-out, or the flat fill a sticker sheet was drawn
over. A backdrop becomes a layer only when it is content in its own right — the inpainted
background of a photograph, or the source with the subject holes left open when no pass could
complete it.

Outputs published together are laid out as equal cells in reading order, as close to square as the
count allows, each output centred in its cell at the size its content reports. That set is then
placed as one block into its sources' derivation field: the area beside the sources where
everything they produce accumulates, filled in reading order with as many columns as keep the field
close to square. One rule covers every derivation — an Action's outputs, separated speech, an
Agent's files — so repeated derivations of one source gather into a tidy block instead of
scattering, and a set stays a compact block instead of a row that runs off the side of the canvas.
Occupied cells are skipped rather than reclaimed, so nothing already on the canvas moves: an output
someone dragged elsewhere stays where it was put, and because a reserved node already carries the
pixel size it will publish, nothing moves when the content lands.

There is no separate program-authoring surface. A recorded chain that can only be replayed with the
same settings on new material earned neither its storage nor its binding ceremony: it required
running the chain by hand first, could not vary a parameter for one Run, and scattered one node per
step across the canvas. Reuse therefore reads the derivation record a result already carries — its
Action, pinned parameters and execution identity — instead of a separately stored definition.
Execution dependencies come from validated Action records, never from visual proximity or
hand-drawn relations. Model samplers and provider plumbing remain execution bindings rather than
canvas nodes.
The built-in Agent calls Knovum Space Actions. Desktop Agents manage external tools in their
own runtime; the Space package does not export Agent tool configuration.
Conversation has one Agent entry point, without a mode field or mode selector. Users choose an
execution target and describe the task; answering, planning and execution can continue in the same
conversation. Built-in and desktop Agents act under authenticated Space-scoped task grants.
The built-in Agent exposes registered Space Actions and explicitly selected, authorized extensions.
It runs Deep Agents in the private Builtin Agent runtime with Todo and read-only research available
as standard capabilities. SDK todos project into Run activities only when used; maintaining a task
list does not itself publish business changes. User requests for discussion or a proposal guide Agent
behavior; host authorization, not natural-language classification, enforces tool and Space boundaries.
A model is enabled for the Builtin Agent with a single switch; the wire protocol belongs to the
provider (OpenAI uses Responses, OpenAI-compatible vendors Chat Completions, the rest their native
API) and is never a user choice. Context window and per-request output limit are properties of the
model, not settings: they are resolved from the published model catalogue, cached with the
application and refreshed in the background. An unlisted model simply has unknown limits, which is a
legitimate state — the runtime then lets the harness compact history on overflow instead of
pre-computing a budget, and sends no output-limit parameter at all. A task is bounded by model turns,
tool calls and duration; there is no separate token budget, since turns × context window already
bounds it and reserving every call's full output limit against a fixed total rejected legitimate
answers. The Space snapshot is part of the system prompt, not Agent memory: it is authoritative
state the host regenerates for every turn, while the SDK's memory middleware presents its sources as
possibly stale files the Agent is invited to rewrite with `edit_file`, which would let output reach
state outside a KnowledgeChangeSet.

The built-in Agent works in a directory of its conversation
(`agent_artifacts::workspace::builtin_agent_directory`): its file tools (`ls`, `read_file`, `grep`,
`glob`, `write_file`, `edit_file`, `delete`) address it as `/` through deepagents'
`FilesystemBackend` in virtual mode, so no path leaves it, and it keeps its files across the
conversation's turns. It is the Agent's own: the user cannot open its files, so the Agent is told
never to link them and publishes results through Actions. Branching or regenerating a turn does not
roll the directory back — checkpoints hold the conversation, not the files.

Commands run through `run_command`, a host tool: Knovum, not the Agent process, runs them
(`conversations/shell`), so each command is a conversation tool operation like an MCP call —
recorded before it runs, settled with a receipt that a replay returns instead of running the command
again, `outcome_unknown` when the app stopped mid-command, and killed with every process it started
when its Run stops. deepagents' own `execute` is never offered: it would run unrecorded and
unapproved inside the Agent process. A command sees an allowlisted environment (deepagents'
`inherit_env=False`, never provider keys) plus the login shell's `PATH`, gets no input, and every
process it starts ends with it. On macOS it runs under Seatbelt — it reads anything, writes only
the working directory and temporary directories, and reaches no network — so it runs as the Agent
asks. Elsewhere it runs with the user's permissions, so its operation is created unapproved and the
conversation asks the user, showing the command line, before it runs. The system prompt names the
computer's system and shell — PowerShell on Windows, never bash — and whether commands are
confined. The research subagent gets neither `delete` nor `run_command`.
Answers cite what the Agent actually read. Every indexed passage a search or node read returns —
through Space Actions for the built-in Agent or the `knovum` MCP server for desktop Agents — carries
the Run's `cite` id (`C1`, `C2`, …), numbered once per Run in `spaces_run_citations` from the chunk
row itself. When an answer is stored, its `[Cn]` markers resolve to Grounded citations on the
message: the marker renders as a link that opens the exact chunk and locator, and the sources the
answer cited are listed beneath it. A marker the Run never issued stays plain text; a source edited
after the answer is shown as detached, never silently re-pointed. An Agent learns this shape in one
place, the conversation citation policy, and `grounding::evidence::tests` checks the markers that
policy teaches against the markers the reader resolves: a policy that teaches a different shape
costs every citation in every answer, silently.
Generation tools use their configured business routes, independently of the conversation model.
Only complete checkpoint-proven main messages enter persistent history; in-flight text is a
presentation stream. Public history remains available when an unsupported or unsuccessful graph
segment is retired. Root pause is acknowledged only at a durable boundary. Unknown paid calls
appear as actionable waiting in the conversation and Task Center, with a state-version checked
explicit new-attempt command and possible duplicate-charge explanation. External Agent turns
continue to support execution/cancellation through ACP, through the same mode-free product request, with their own runtime controls.
Builtin Agent failures retain their reason with the task record; interrupted presentation text is
never promoted to a completed answer. See `system-architecture.md` for graph/host ownership.

## Item identity and titles

Every durable item has a semantic `title`. Search, accessibility, the Outline, commands, AI context and broken-content fallbacks need a stable human-readable label.

A semantic title does **not** imply visible title chrome on every canvas item:

| Item           | Semantic title             | Default canvas treatment         |
| -------------- | -------------------------- | -------------------------------- |
| Note           | first meaningful text line | no separate title bar            |
| Saved Website  | webpage title              | visible                          |
| Markdown       | explicit Markdown title    | visible                          |
| Image          | imported filename          | hidden unless needed as fallback |
| Video          | imported filename          | quiet metadata                   |
| Audio          | imported/recording name    | compact built-in player          |
| File           | imported filename          | visible                          |
| Code / Text    | imported filename          | source content                   |
| Sensitive Info | explicit label             | type + stored status only        |
| Section        | explicit section title     | visible                          |

The canvas stays visually quiet while every item remains identifiable outside its visual representation.

## Saved Websites, not raw links

Spaces does not expose a generic `link` item. A URL becomes a **Saved Website**.

Capture accepts HTTP/HTTPS URLs and domain-like input such as `github.com/openai`. The item is created immediately, then a narrow Rust command enriches it with webpage metadata when available:

- final URL;
- page title;
- site name;
- description;
- canonical URL;
- a locally cached site icon;
- a locally cached Open Graph or Twitter preview image when the page exposes one;
- readable article text and sections extracted by Mozilla Readability when available.

The URL remains saved even when preview enrichment fails. Website cards show the source identity (site icon, site name and domain), page title, a short description or readable excerpt, and a bounded preview image when one exists. The inspector exposes the original URL, site name, description and canonical URL. A failed refresh retains the last saved knowledge; a successful refresh replaces source-derived metadata rather than carrying stale fields forward.

Remote preview fetching is intentionally not a general-purpose WebView network permission. The Rust `web_capture` boundary accepts HTTP/HTTPS, follows normal HTTP redirects, requires supported content types and enforces response-size limits. It returns neutral bytes and metadata; the Space adapter stores selected images under Knovum's private asset directory.

## Sticky Notes, Text and Markdown

Sticky Notes, Text and Markdown are intentionally different products. Sticky Notes and Text share a
bounded Tiptap canvas-text schema; Markdown keeps its own interoperable source contract.

A **Sticky Note** is quick canvas writing on a colored paper surface. It has no title bar: its first
line is derived as object identity for search and accessibility but is never rendered twice. Selection
exposes paper fill plus object-wide size, weight, emphasis and two-axis alignment; activation edits its validated Tiptap
tree directly in place. The canvas mounts an editor only for the active edit session and destroys it
when editing ends; inactive annotations remain lightweight static HTML generated from the same Tiptap
tree. A non-empty text selection exposes contextual inline color, size, emphasis, list and paragraph
alignment controls; these are stored as Tiptap marks or node attributes rather than object appearance.

**Text** is a transparent canvas annotation for headings, labels and captions. It uses the same
validated lightweight Tiptap tree and derived plain text as Sticky Notes, but it has no paper surface,
shadow or semantic-relation role. Text additionally exposes foreground color. A new Text object contains an empty document and presents instructional placeholder copy until editing begins; the prompt is never persisted as user content. Its initial frame is a compact 280 by 40 canvas unit label. Left and right handles change wrapping width while the bottom-right handle scales the complete text frame proportionally. No other corner or independent height handle is exposed. Text preserves its current frame width while its frame height continuously fits the rendered content, growing or shrinking after text, typography or wrapping changes. Direct manipulation temporarily owns geometry, so content measurement is coalesced and applied once the resize ends rather than competing with Konva's transform. Content-derived height is persisted as layout state but is not recorded as a user resize history entry.

**Markdown** is the interoperable `.md` model. Its canonical body is Markdown text in SQLite. Tiptap parses that text into an editing state and serializes it back through `@tiptap/markdown`; Markdown-only capabilities must round-trip through that representation. Editor-only structured blocks are not silently written into Markdown.

The full Note/Markdown editor continues to use maintained Tiptap extensions for document structure.
Canvas Sticky Note/Text editing deliberately uses a smaller extension set—paragraphs, inline marks,
lists, alignment, typography, history and placeholder—so a canvas annotation cannot silently acquire
document-only tables, headings or embedded assets.

Audio files can be imported or recorded through the WebView `MediaRecorder` API. Recorded bytes cross a typed Tauri command and Rust validates the container and writes the file into the same private managed-asset directory as other media; the WebView has no Space filesystem-write permission. macOS receives an explicit microphone usage description. Audio and video stay decoder-free until the user activates playback. Once loaded, Knovum renders its own desktop media controls around the underlying media element rather than exposing WebView browser controls. Low-detail canvas levels remove the player again, and explicit unload releases it.

Audio/video knowledge is a first-class Space record. First-use transcription automatically prepares
its fixed model group after asset authorization, displays resource transfer progress in the initiating
task, and revalidates the source revision before execution. Users do not have to download models from
Settings. Cancelling preparation cancels that transcription demand, not another task's shared install. Analysis accepts only Knovum-owned Space assets and uses bundled FFmpeg to create a 16 kHz mono PCM working track plus a compact waveform. The verified local Python ASR profile runs under shared compute admission; Rust verifies its transcript and source revision before publishing. Stage progress is mapped into the shared Task Center, and segment plus word timestamps stay on the original media timeline for citations and subtitle cues. Media bytes never enter a generative-AI provider path.

`mediaKnowledge` v1 persists a source-bound, revisioned Transcript with stable segment/unit identities, 16 kHz source sample offsets, explicit timing provenance, speaker tracks, and timed subtitle cues. Subtitle cues are separated by detected speech pauses, sentence endings, or elapsed audio time; FFmpeg/libass measures glyphs and performs Unicode wrapping against the actual video frame. Editing a transcript segment preserves the artifact identity, advances transcript and segment revisions, invalidates that segment's word timing, rebuilds subtitle cues from the current segment texts, and clears source-derived insights. Ordinary ASR requests and failed-job retries cannot replace an existing current-source transcript, including its manual edits; execution rechecks that rule before recognition and publication rejects concurrent edits. Summary and Chapters are optional `insights` generated only after the user explicitly invokes that action with Builtin Agent AI enabled; they are not required for search or playback. Chapter starts are validated against real Transcript segment boundaries, so every Chapter and Transcript row can seek the canvas player precisely. Transcript and generated-insight chunks are indexed separately, and long transcripts use `@tanstack/vue-virtual` rather than mounting every segment in the WebView DOM.

Selected alignment is an explicit enrichment action over at most eight segments and 300 seconds.
It preserves stable word or character unit identities and may publish partial progress; a manual edit
invalidates the edited segment's timing rather than silently retaining stale alignment. Optional
speaker diarization runs separately for media up to two hours. The speaker count may be automatic
or one known value from 1 through 32. Its node action prepares the public Sherpa-ONNX model and
matching dependencies automatically. Download progress and cancellation belong to the same feature
request and are also available in the Settings model list. No account token or folder import is required.

## Speech projects

Speech actions begin on the selected canvas node and follow the image tools’ contextual bar system.
Reference intervals, voice settings and auditions use button-anchored controls. Source separation
starts with one click; completion publishes two audio nodes beside the source with active
`derives-from` relations. Playback and export belong to those audio nodes; there is no review or
manual insertion stage. Dubbing opens the project editor directly, resuming the most recently updated
project belonging to the selected node and operation. Creation is a separate stage. Manuscripts and
timed scripts come first; independent expandable settings retain drafts and tasks when collapsed.
Ending the operation uses the existing save/discard leave contract.

Speech projects publish audiobook audio, short dubbing audio or video dubbing from durable Runs.
WAV, MP3 and M4B are supported audio outputs. Chapter boundaries and mix placement use integer sample
positions. Audiobook Runs checkpoint each chapter's PCM and its verified input digest, measure
loudness in a separate step, then encode the final output using that measurement. Recovery verifies
checkpoint hashes and sample counts before reuse. Temporary normalization files belong to one
attempt; chapter and measurement checkpoints are cleared after final publication.

Audio and video nodes start a durable source-separation Run bound to their exact node identity,
managed source asset and source revision. Rust decodes the audio track to a fixed 44.1 kHz stereo
PCM timeline and runs the selected pinned Spleeter two-stem CPU profile. Its one publication
transaction creates vocals and background audio nodes, two active system `derives-from` relations
to the original node, the result receipt and the shared canvas layout/history change. Source bytes
and node content are unchanged. A missing output, stale source, cancelled Run or expired attempt
rolls back the complete publication. Retrying an already committed attempt reuses its result and
never inserts duplicate tracks or relations. Publication belongs to the durable Run and continues
when the initiating view closes or another Space opens.

A synthesized line is published as the speech it contains: the silence a model leaves before the
first word and after the last is trimmed, keeping a short guard so no onset or tail is clipped, and
a line that is silent throughout is invalid output. A line's stored duration therefore means spoken
time, which is what every consumer places. Separation between lines is then stated rather than
inherited from a model: an audiobook keeps its configured paragraph and chapter pauses, and a plain
read separates its lines by a sentence pause. Trimming and pausing are identified in the assembly
digest, so an output published under earlier rules is never reused as if it matched.

Dubbed dialogue is heard in script order and never on top of itself. A line starts at its subtitle
time or as soon as the line before it has stopped speaking, whichever is later; a line that would
run into the next one is compressed only as far as the ear tolerates, and past that the next line
waits. Lateness is paid back by the silence between lines, and nothing is heard past the end of the
picture. One dialogue track therefore carries every line, so the mix never sums speech onto itself,
and every placed line ends with a short fade, so a line the picture cuts short never clicks.

A translation Run derives the script from the revision it was created for, so a translation request
cannot carry a script that has already gone stale. Translating commits the editor's script first
and returns the translated lines for review; the original stays with each line as its source text.

Video dubbing may select the published background through its mix configuration. This is a choice
of input, not a second approval of the derivation. Selecting a background preserves its frozen
source/model provenance. The product does not claim that spoken dialogue was completely removed
or that environmental sound was preserved. Space packages preserve the Run identity, source node
identity, both managed stems, their relations and pinned provenance.

A segment's state reports its latest render attempt; it is not a lock and never a dead end. The
executing attempt is a segment's only writer — the Run lease, the execution epoch and the active
revision are verified in the claim transaction — so claiming a segment overwrites whatever an earlier
interrupted attempt left behind. The lifecycle is `ready → rendering → rendered | failed`, and
generation is always available again on the active revision. Space packages carry only settled
states: `ready`, `rendered` or `failed`.

The verified picture is the only authority on how long a published dubbed video is. Every audio
track it carries — the dubbed mix and the retained original — is padded and then trimmed to the
sample count the picture was measured at, so the published length is stated rather than left for the
muxer to infer from whichever stream happens to end first. The length of the source's own audio
track never reaches the published file. The mix is verified sample-exactly against the picture
before it is encoded, and the published track is then verified to end with the picture within one
audio frame, which is as precisely as a frame-granular codec can land.

The voice library is application-wide: profiles and immutable revisions have no Space owner, and
each Space speech project pins a global revision. Settings owns library presentation through app
composition; the speech capability owns its data, generation and asset lifecycle.

A voice design or clone is a durable single-step Application Run with no Space permissions.
Rust decodes only the selected 5–30 seconds
from an authorized audio or video reference into a hash-bound, 16 kHz mono PCM16 input before scheduling. The original
source revision and selected interval remain in the request; the managed snapshot records its own
digest, byte length and duration. Recovered attempts use this exact clip rather than reopening the
original media. Generic workflow retry is limited to Space workflows; generating another voice
candidate creates a new request rather than cloning an identity-bound candidate Run.
Preparation reserves an asset journal entry before creating files and seals it with the clip digest
before the Run transaction takes ownership. Rust recovers interrupted attempts through the shared Run
lease policy, and publishes the normalized reference plus 48 kHz audition as one candidate record.
Audition completion and the selected profile/revision identity are persisted. Discarding an unused
ready candidate removes its terminal Application Run, candidate receipt, unshared grant and
unreferenced audio; selected candidates remain until their archived, unreferenced profile is
permanently deleted. Creating a
reusable voice requires a completed audition; generation and saving do not ask for a separate voice-use declaration. Application Run grants and managed-file access remain enforced by their respective authorization boundaries. Selecting a voice copies the
candidate's verified 5–30 second 16 kHz mono reference into an immutable voice revision. Global
voice profiles may be archived without losing candidate history or breaking existing project pins.
Permanent deletion requires an archived profile with no speech-segment or audiobook-role references
in any Space project revision. Its succeeded, selected creation Runs and candidate receipts are
removed in the same transaction, along with Application grants no surviving Run references;
malformed or nonterminal receipts block deletion. Managed assets
are reclaimed only when the global reference graph confirms that nothing else uses them.
Space packages include only voices referenced by exported projects, their pinned and active
revisions, and their managed reference assets. They exclude application candidate jobs and audition
history, unrelated revisions and the rest of the global library. Request, result and selected voice
revision must agree on the model; MOSS audio-only references retain their own controls.
Cancellation during creation belongs to the preparation request. If creation commits concurrently,
the initiating view carries the cancellation intent to the durable Run, which remains in Task Center
even after the view closes.

Cloning references must contain at least 60% VAD-detected speech. If a confirmed reference transcript
is supplied, its normalized character edit distance from recognition must not exceed 20%; comparison
is bounded to 3,000 characters. Audio-only references remain supported, and inferred ASR text is
never saved as a confirmed cloning condition.

A subtitle interval is the target a dubbed line aims at, not a condition it must meet. A line is
spoken at its own pace and fitted when the mix is planned: it is compressed by up to 1.25× to land
inside its interval, and a line that needs more than that keeps speaking into the room before the
next one rather than being cut or refused. Nothing is slowed down to fill an interval, no line is
heard past the end of the video, and overlapping dialogue is represented by explicit lanes. The
editor offers source-voice mute or a separately imported background track. Knovum does not claim to
retain environmental sound because no legally distributable, qualified dialogue-separation model is
selected.

Video translation owns restore, submission, observation and result loading. Editing and synthesis
remain disabled until the matching result is applied or the job settles; revision changes invalidate
late results. Audiobook output drafts validate loudness, peaks, pauses and metadata before submission.
Dirty speech revisions use the same Space/application leave participant boundary as document and
video editors. Project switching, Space navigation, route changes, update install and quit offer
Save, Discard and Cancel; failed revision saves keep the editor draft available. Leaving also waits
for an already admitted revision save before evaluating the current draft.

## Sensitive information

A Sensitive Information card is either an API Key or an Account & password record. Spaces SQLite stores only non-secret metadata (`title`, `credentialKind`, `credentialConfigured`). Secret values live exclusively in the separate local credential database. The card never exposes the value in its collapsed state, duplication does not copy a secret, and permanent Space deletion cleans the corresponding credential entries.

## Import, preview/edit and export matrix

Spaces follows **content-first canvas, professional-editor handoff**. A resource should look like itself on the canvas whenever a safe local renderer is available. Knovum owns capture, spatial organization, search, lightweight interaction and AI context; specialist applications continue to own high-fidelity specialist editing.

| Format                                                                                        | Import                               | In-canvas behavior                                                                                              | Edit path                               | Export         |
| --------------------------------------------------------------------------------------------- | ------------------------------------ | --------------------------------------------------------------------------------------------------------------- | --------------------------------------- | -------------- |
| Markdown (`.md`, `.markdown`)                                                                 | text becomes canonical Space content | in-place Tiptap editor                                                                                          | in Knovum                               | Markdown       |
| Images / SVG                                                                                  | private copied asset                 | image content                                                                                                   | system image app when opened            | original bytes |
| Audio / Video                                                                                 | private copied asset / recording     | audio stays activation-only; video uses a persistent FFmpeg poster and reveals playback controls on hover/focus | system media app when opened            | original bytes |
| PDF                                                                                           | private copied asset                 | system page thumbnail when available + pdf-inspector page-level text extraction                                 | default desktop PDF app for heavy edits | original bytes |
| Source/config (`js`, `ts`, `html`, `vue`, `rs`, `py`, JSON/YAML/TOML, project dotfiles, etc.) | private copied asset                 | bounded source preview; HTML uses a sandboxed asset-protocol preview                                            | VS Code / associated editor             | original bytes |
| CSV/TSV and plain text records (`txt`, `log`, `rst`, `adoc`, `ics`, `vcf`)                    | private copied asset                 | system thumbnail when available + bounded readable text used for search and knowledge ingestion                 | associated spreadsheet/text application | original bytes |
| DOC/DOCX                                                                                      | private copied asset                 | system page thumbnail + Rust plain-text/Markdown extraction for search and knowledge ingestion                  | Word/LibreOffice/associated app         | original bytes |
| PPT/PPTX                                                                                      | private copied asset                 | system page thumbnail + Rust plain-text/Markdown extraction for search and knowledge ingestion                  | PowerPoint/Keynote/associated app       | original bytes |
| XLS/XLSX                                                                                      | private copied asset                 | system page thumbnail + Rust plain-text/Markdown extraction for search and knowledge ingestion                  | Excel/Numbers/associated app            | original bytes |
| ODT/RTF/ODP/KEY/ODS and other specialist formats                                              | private copied asset                 | system page thumbnail when the OS has a registered handler; otherwise resource identity                         | associated professional app             | original bytes |
| Other regular files                                                                           | private copied asset                 | resource identity                                                                                               | associated desktop app                  | original bytes |

A single imported resource file is limited to 8 GiB. Directory imports are not supported. Archive files are preserved as opaque binary file assets without container expansion or semantic indexing. The node contract has no `folder` or `archive` kind. Space package ZIP transfer remains a separate workspace operation, not a canvas-resource import.
Known credential-bearing dotfiles such as `.env` and `.npmrc` remain opaque file resources instead of being copied into SQLite/FTS text previews, and are equally excluded from Quick Look thumbnails: a canvas thumbnail must not become a second, less protected copy of a secret. Users can still open the private copy in their associated editor.
On macOS and Windows, images, audio, video and renderable documents request the system's own thumbnail, cached beside the other private previews. macOS uses Quick Look and Windows uses the registered Shell thumbnail provider. Preview generation is a strict native capability: failure is reported instead of activating an alternate renderer. Source/code formats retain their product-owned text presentation.

Opening an owned resource launches the Space-private copy. When Knovum starts, regains focus or becomes visible again, it re-inspects private files so edits performed in Word, VS Code or another specialist application update source previews and media/PDF version stamps. Refresh failures are surfaced rather than silently replacing the resource.

PDF/Office **full embedded editing is not a product goal**. `system_preview` is the only canvas renderer for native thumbnails: Quick Look owns the macOS adapter and registered Shell providers own the Windows adapter. `pdf-inspector` owns PDF page-level text extraction; `office_oxide` owns semantic Office extraction. The evidence inspector shows extracted text and opens the original file; it no longer renders PDF pages. PDF coverage distinguishes trusted text from pages requiring recognition. Optional background document visual analysis is implemented through the enrichment job owner, with per-occurrence checkpoints and revision-guarded publication; it requires a configured compatible visual target. This implementation is not a claim that every format and platform has completed acceptance. Extraction, available resource browsing and native thumbnails remain usable without AI. See `knowledge.md` for coverage, limits and cancellation. Import, native visual preview and semantic extraction are separate ordered phases: the pending node appears immediately, the native preview is published through the import channel as soon as it exists, and semantic extraction continues in the background. The original private bytes remain authoritative and professional applications own high-fidelity editing.

Notes, Markdown and supported PDF/Office files can be exported in an admitted target format. The export reads the authorized original asset rather than cached extracted text, verifies its digest before parsing and again before the destination is written, and applies the format policy owned by [resource-export.md](resource-export.md).

## Canvas cards and Sections

Child WebViews do not take initial keyboard focus from the canvas. During canvas motion or direct manipulation, the child WebView is suspended while the node shows its saved site identity or preview image; after the viewport settles, the browser is placed at the final bounds and shown again. Child-window placement requests coalesce to the latest frame rather than queuing obsolete coordinates. Browser content bounds are projected from the same scene frame and viewport that paint the canvas, using the host rectangle only as the origin; they are not sampled from CSS-transformed node DOM. The main WebView origin is then converted to the parent window's client-area logical pixels at the Rust boundary. Rotated or partially clipped website content stays on its saved preview because a child WebView cannot rotate or clip with the canvas. Resizable nodes have semantic minimum sizes but no arbitrary presentation maximum; persisted frames must remain finite and positive.

Canvas items are **content-first spatial representations**, not a grid of generic file cards. Notes and Markdown remain bounded summaries because their full body has a Knovum editor. Images and document snapshots expose content directly; media stays an inert cover until activated. A video cover carries its duration; a separate, fixed-height caption shows at most two title lines and then platform and creator, without letting a long title push the source out of view. Code shows source. Saved Websites persist the URL immediately and enrich metadata in a bounded background queue. Website nodes render a stable local preview or site identity; the original URL remains available for opening in the system browser. The canvas does not host independent browser surfaces. Opaque specialist formats keep a restrained resource identity until a maintained renderer is selected.

Image and video editing deliberately use different product surfaces. One selected image exposes
non-destructive crop presets, quarter-turn rotation, horizontal/vertical flip, tonal adjustments
and AI object removal directly in the canvas selection bar. The active control panel sits beside the
selected image when the canvas has room on either side. Otherwise, the panel uses the larger free
area above or below and scrolls within that area. The selection surface owns this placement rule
for other long node controls as well. The persisted `imageEdit` v1 contract keeps the
source asset immutable; the canvas renderer projects crop, transform and filter edits. The old
transparency eraser is not a same-node edit, and a later `imageEdit` revision carries stored recipes
forward instead of refusing an image somebody already edited. Object removal publishes a new image with `derives-from` lineage.
Changing canvas selection does not discard an image edit draft. The draft remains owned by its
`(spaceId, nodeId)` session until the user returns to it, applies it or discards it. Leaving the Space,
route or application with a dirty image draft offers Apply, Discard and Cancel through the shared
leave boundary; a failed Apply retains the draft.
With a compatible Qwen or GPT Image model, brush selections stay transient and Rust sends the rendered source with a
provider-specific mask. With a Gemini, Grok Imagine, Doubao Seedream, SiliconFlow Qwen edit,
Kling Omni or supported OpenRouter image model (including FLUX.2), the user describes the object
and Rust sends the rendered source with that instruction. Rust uploads the source to Gemini or xAI
before those edit requests; inline encoding happens only for providers whose image edit API needs it.
Text-to-image support alone does not enable object removal. System
preview and PNG export render the same saved edit through Rust; opening
an image never hands the unedited source to the system preview. Image comments are normalized
positions on the displayed image, saved in the image node payload with their text and creation
time. A comment may target a point, a freehand
stroke or a rectangular region; all three modes render a numbered pin, while strokes and regions
retain their image-relative geometry. Adding a comment to the built-in conversation stages its
text and attaches the source image in the composer rather than starting a conversation task.
Multi-angle generation accepts
subject or camera movement, yaw from -90 to 90 degrees, pitch from -30 to 60 degrees, and
close-up, medium or wide-angle framing. Its preview is an interactive orientation guide; the
model generates the actual new view. Video editing opens a same-window workbench because
transport, playhead, placed clips and timed text require persistent timeline context. A strict
v1 `videoEditProject` stores revision-bound managed asset references, explicit output dimensions,
frame rate and background colour, one visual track of absolutely placed clips, a transition
collection anchored to cuts, independent audio clips on lanes and a timed-text track. Clips carry
their own `timelineStartMs`, so a project may expose intentional gaps; a gap shows the project
background in preview and in export. The first asset
retains the owning video identity. Local videos, still images and audio are selected from the owning
Space; the native multiple-file dialog uses the existing managed import pipeline. Asset references
contain no file-path authority. A later `videoEditProject` revision carries stored projects forward;
a payload that fails validation is damaged, and the workbench states that the stored project cannot
be opened and offers a new edit from the unchanged source rather than silently resetting it.

The visual track supports insert, reorder, duplicate, split, dragging a clip to a new timeline
position, closing a gap, source in/out trimming, per-clip volume and fades, and a free transform of
normalized offset, scale, rotation, opacity, contain/cover framing and flips. Transform values are
manipulated directly on the stage: dragging moves the layer, corner handles scale it and a top handle
rotates it, with snapping to the frame centre and quarter turns that Alt disables. A still image has
an editable display duration and starts at four seconds. No clip is shorter than 100 ms.

A transition is its own record anchored to the cut after one clip. Its duration is exactly the
overlap it creates, bounded by 100 ms, five seconds and the media both neighbours can spare;
applying, resizing or removing it retimes the following clips, text and audio in one operation.
The eleven kinds are a dissolve, fades through black and white, four directional wipes and four
directional slides, each mapping one-to-one onto the FFmpeg `xfade` transition of the same name so
that preview and export cannot diverge. Removing, splitting or detaching a connected clip pair
retires that pair's transition. Ripple edits retime text and audio starts in one operation; music
that extends past the shortened project is trimmed to the timeline rather than discarded, and text
that no longer fits is removed. Audio has independent source in/out, timeline start, volume, fades
and up to four lanes; a lane change that would overlap existing audio is refused.

A project is measured from its own source file: Mediabunny parses the container, so admitting media
needs neither a transcode nor playback, and the output starts at the resolution and the frame cadence
the source itself states instead of resampling it to a house default. Rust prepares revision-keyed,
short-GOP H.264 preview files for playback and thumbnails, capped to a 1920-pixel long edge so the
stage shows the picture the render will produce; the recipe is part of the cache identity, so an
improved recipe replaces what it supersedes. Export always reads the original. Video frames never
cross IPC. Preparation is cancellable and failures remain visible.

Playback authority is split on one boundary: Rust owns normalization and rendering, and the WebView
owns interactive presentation of what Rust prepared. No media element takes part. Mediabunny demuxes
the preview file and decodes its video through WebCodecs; a clip's audio track is copied out of that
file without re-encoding and decoded by Web Audio, which every supported target implements. Sound is
scheduled once per voice on the audio timeline with its own gain envelope, and that same audio
timeline is the clock, so the position can never drift from what is audible and playback costs no
per-frame audio work. Each animation frame presents the newest decoded frame for the clock's
position: decoding is pulled one frame at a time, a backwards scrub or a long jump restarts the
decoder at the new position instead of decoding through, and a frame that has not arrived repeats the
last picture rather than stalling the transport or flashing the background. While the editor is
paused the preview repaints only when something changed. Konva owns the stage scene graph, hit areas
and the transformer that moves, scales and rotates layers and overlays; the project stays the single
source of truth for every transform, and handle metrics are compensated for the stage zoom. Preview
uses the same geometry, framing, opacity, gain, fade and transition math as export. Pointer scrubbing
and exact output frame stepping are separate from integer-millisecond persisted edit intervals; trim
handles and dragged clips snap to clip edges, the playhead and the output frame grid, with Alt
enabling millisecond precision.

A tabbed library of materials, text and transitions, the stage and the selected-item inspector sit
above the resizable timeline. The inspector is absent until a clip, transition, audio clip or text
item is selected; closing it returns focus to the ruler. The timeline is scaled in pixels per second:
zooming keeps the time under the pointer fixed, fitting shows the whole project with trailing room,
ruler steps coarsen as the scale shrinks, and the view follows the playhead by writing the scroll
position and the playhead directly, so a moving playhead never re-renders the lanes. Filmstrip
thumbnails and audio waveforms are decoded in bounded background work keyed by asset revision: only
what the viewport shows is requested, outstanding work is replaced rather than queued as the view
moves, extraction yields to playback, and both are released with the materials that produced them. Timed text is
selectable from the stage and the timeline and is positioned freely by dragging it on the frame, with
searchable subtitles, installed font family and face weight, size, colour, alignment and four
export-backed styles. Text size remains
720p-relative; ASS resolution follows the explicit output frame. A missing selected font blocks
export. Each overlay stores its full style and preserves its text whitespace across saving and rendering. Application commands own local shortcuts without
feature-level global keyboard listeners.

Saving records one project snapshot through normal Space history. A local edit history provides
undo and redo before Save. Close, Space navigation, route changes, update installation and quit
require an explicit Save, Discard or Cancel decision for unsaved work. Failed writes retain the draft.
Draft structure is validated on saving; live source admission occurs at rendering so deleted material
can still be inspected and removed from a draft, and package restore does not depend on insertion
order. Missing or replaced material is unavailable in preview and prevents rendering until corrected.

Rendering resolves every used asset in the same Space, checks current revision, readiness and managed
file authorization, and verifies the exact saved project snapshot through the same strict typed contract. JSON integer/float
spelling does not create a false edit conflict. FFmpeg normalizes each source to
the output frame and sample clock, supplies explicit silence for images or silent videos, applies
transitions and subtitle composition, and mixes independent audio. Every used source revision is
captured and revalidated before atomic publication. Measured progress and cancellation use Task
Center. Admission, rendering and publication failures settle the pending output visibly; rejected
publication removes its rendered file. Output is a separate derived video node; original source bytes and the saved edit project
remain available.

Desktop acceptance for this workbench follows Apple's guidance on
[undo and redo](https://developer.apple.com/design/human-interface-guidelines/undo-and-redo),
[focus and selection](https://developer.apple.com/design/human-interface-guidelines/focus-and-selection/),
and [designing for macOS](https://developer.apple.com/design/human-interface-guidelines/designing-for-macos/):

- Every edit has an immediately visible selection, preview, and reversible result; a no-op such as
  deleting the final clip is disabled rather than silently accepted.
- Pointer and keyboard can seek and trim; focused controls have names and visible focus. Timecode
  uses millisecond precision and never calls an arbitrary 100 ms step a video frame.
- Save is explicit, asynchronous work cannot be started twice, and closing with a dirty project
  offers Save, Discard, and Cancel. Failed preview preparation remains visible and retryable.
- Preview and export share clip placement, per-clip volume, transform, transition weighting and
  timed-text position; a removed or shortened interval ripples the text and audio tracks by the same
  duration.
- Direct manipulation on the stage and on the timeline is reversible, snaps to meaningful anchors,
  and reports the resulting value in the inspector rather than only in the gesture.

URL paste reads clipboard HTML, text and, on macOS, the native `public.url` format in one typed Rust operation before applying an explicit product-owned content-URL policy for selected media platforms: YouTube, Bilibili, Douyin, TikTok, Instagram, Vimeo, X/Twitter, and Facebook. Direct media resources and recognized platform content links enter yt-dlp acquisition. The capture projection appears at the paste location as compact desktop transfer chrome rather than an empty media cover; its screen size is capped when zooming in so temporary feedback does not balloon over the canvas. First-use tool preparation and media downloading show measured bytes; resolving shows indeterminate activity; finalizing shows measured media time when FFmpeg prepares a playable asset. yt-dlp prefers H.264/AAC streams, and the import stage converts other selected codecs to H.264/AAC in MP4 or AAC in M4A before the asset is committed. The node persists the canonical local asset size and its media origin (platform/extractor, original URL, and source ID); the canvas shows platform identity, while the inspector exposes origin and file details. The projection transitions into the persisted node on success and remains in place with retry/dismiss actions on failure. Platform homepages, profiles, and all other HTTP(S) URLs become Saved Websites immediately. The automatic clipboard path neither enumerates yt-dlp extractors nor probes arbitrary webpages for embedded video; this avoids blocking ordinary website capture on media-tool startup or network timeouts. A recognized media URL that cannot be acquired reports the media failure instead of silently changing node type. Clipboard file paste is a separate trusted import source: Rust verifies each requested path against the current native clipboard file list before staging it; dialog and drop imports continue to require their explicit filesystem scope.

A document the system could render takes that page's own proportions, so a portrait report and a landscape deck read differently at a glance. A near-square render is a generic icon plate rather than a page, and keeps the default file card.

Resources whose bytes cannot speak for themselves — files and code — close with a caption strip naming the resource type and its size. The strip replaces metadata scattered through the card header, stays at a fixed height across every canvas detail level so zooming never resizes the card, and is withheld from media: an image, a video or a player already says what it is.

Group and Section are Structure nodes rather than content-node variants:

- Structure owns `StructureNodeId`, parent, sibling order, lock and container title; Layout owns local transforms, Section frames, appearance and relation-style overrides; content rows own knowledge payloads.
- Group creation uses effective selected roots and preserves world geometry while converting children to local transforms. Entering a Group establishes the explicit editing scope; ordinary internal movement does not infer migration out of that Group.
- Section creation supports an empty drawn frame, fitting selected roots and drawing around fully contained roots. Creation decides membership once; later Section movement and resize never capture nearby structure.
- Dragging content or a whole Group across a Section boundary creates one explicit move-to-container command. A collapsed Section must be hovered for about 600 ms before it temporarily expands as a drop preview. Leaving, cancelling, Escape, pointer cancellation or window blur clears the preview; a successful drop commits movement, expansion and reparenting in one Rust transaction and one history entry.
- `Fit Section to content` changes its frame around existing descendants. `Remove Section only` reparents descendants while preserving world geometry. Deleting a Section deletes its subtree into one recoverable batch.
- Clipboard, duplicate, cut, delete, cross-Space move and package transfer operate on complete effective-root subtrees. No path reconstructs membership from visual overlap after the user action.

Sections never nest. Groups may nest and may belong to a Section. A Structure node has at most one parent; overlapping Section geometry never creates ambiguous ownership.

Relation lines follow what each relation says. Semantic relations (`supports`, `contradicts`,
`depends-on`, …) are stated on purpose and stay drawn. Lineage — the `derives-from` relations every
derivation writes — explains where content came from, and one source can carry many outputs, so
it is drawn on demand: selecting content draws its whole chain, up to the content it ultimately came
from and down to everything derived from it, while siblings stay out; a selected relation stays
drawn. At rest a source carries the number of outputs derived from it, so hidden lineage remains
discoverable, and the canvas overview leaves lineage out too. Relations of one kind between the same
visible endpoints share a line labelled with their count — a collapsed Section stands in for
everything inside it — and selecting that line selects every relation it stands for. Derived
outputs need no Section to read as a set: one derivation's outputs are placed together beside
their source (`spaces/derivation/placement`).

## Large-canvas retrieval

An infinite canvas must not require users to remember coordinates.

Spaces currently provides four complementary retrieval paths:

1. **Find in Space** — Knowledge Chunk retrieval combines deterministic SQLite FTS5/BM25 with local Qwen3 semantic search. The FTS index uses the built-in trigram tokenizer so Chinese, English and code fragments share one literal substring-capable index; one- and two-character terms use a bounded current-Space substring query. Results carry snippets and canonical source locators, so selecting a hit reveals the exact page, slide, sheet context, block, code position or media timestamp.
2. **Outline** — a structural list of Sections, their members and unsectioned structure. Selecting a row reveals the item on the canvas.
3. **Saved Views** — named projections that restore viewport/focus/relation filters and recompute optional dynamic Smart Queries against current knowledge.
4. **Tags** — semantic facets independent of visual placement.

Space library covers and Saved View previews are disposable local JPEG resources. SQLite stores only their managed file paths; image bytes and Base64 Data URLs do not enter the Space or Saved View rows. A dirty Space cover is committed before opening or creating another Space and when entering the library, while Saved View preview updates follow explicit view captures. Space export carries the view projection but omits these derived thumbnails; restored views start without a preview. Thumbnail paths participate in the managed-asset reference graph, so replacing or deleting a view or Space removes a file only when no remaining record references it. Orphan reconciliation runs once during Space database bootstrap, never during an active library query or thumbnail write.

Each Space persists one strict current Structure/Layout/Edit-State aggregate plus a separate disposable camera. Structure/Layout owns hierarchy, local transforms, frame size authority, order, lock, appearance and relation-style overrides. Imported visual content remains content-sized while intrinsic geometry is resolved; the first authored resize transfers authority to the frame. `SpaceRelation` remains the semantic relation authority. The Konva document is an ephemeral projection adapter generated from Structure/Layout and never becomes a second persistence authority.

Text is a content-height object with two distinct resize gestures. Its side handles change only the reflow width, after which measured content owns the exact height in either direction. Its single bottom-right handle scales the complete typography plane—default and inline type, line spacing, list indentation and text insets—from one immutable gesture baseline; frame and typography scale preview and commit atomically, and Undo restores both. Empty Text stores an empty document and presents its localized prompt only as transient UI. Text edits commit through node-data history, while derived height fitting remains history-neutral. Each batch carries the content revision, measured width, size authority and typography scale; Rust revalidates that read set in the write transaction and rejects stale measurements.

Canvas appearance stores semantic Knovum fill presets or one mutually exclusive `#RRGGBB` custom
fill. JSON Canvas preset and hexadecimal colors therefore cross the exchange boundary without being
silently discarded or converted to an unrelated preset.

The mounted canvas editor is the sole owner of transient camera interaction and selection. Outline, Find and Saved View actions publish typed workbench intents consumed through `CanvasEditorPort`; product components never acquire or retain a vendor editor instance.

The camera is live state, not reactive state. A gesture writes it directly and it projects itself once per
animation frame: the stage transform, the content overlay transform and the screen-space dot grid are written
imperatively in that one frame, and the camera owns the vendor draw loop while it moves so the canvases never
trail the overlay by a frame. Interaction and gesture math always read the live camera. The reactive surface
sees only the zoom, which governs screen-space chrome metrics and preview level, so panning re-renders no
component. The dot grid is an SVG pattern written imperatively per frame rather than a CSS gradient, because
the engine pixel-snaps background geometry: a gradient grid steps a whole device pixel at a time while
subpixel-rendered content moves smoothly, which reads as the grid jittering against the nodes. Viewport
culling samples the live camera on a fixed interval, and its overscan runs ahead of the
camera rather than symmetrically around it, so content enters the visible set before it reaches the viewport.
Scene geometry follows that result immediately while node content mounts under a per-frame budget, which the
settled camera clears at once; a pan into a dense region therefore mounts a few cards per frame instead of a
batch inside one frame. The adapter document adopts the camera when it stops moving, which keeps camera
motion out of document identity, projection and view-state persistence.

Settlement is the expensive edge of camera motion - it hands drawing back to the vendor, redraws every layer
including the hit graph, and lets the document adopt the camera - so a gesture holds the camera open from its
first event until it ends, and a pointer that rests mid-gesture never pays for it. A pan gesture additionally
takes the stage out of hit testing: the vendor otherwise reads pixels back from each listening layer on every
pointer move and dispatches enter and leave events into product state, which is waste while the pointer is
dragging the canvas itself, and it makes each frame draw a second time per layer for the hit graph. The stage
still receives the pointer events the gesture needs, and hit graphs are restored and redrawn on settlement.

A panel animates the work area's width, so the canvas host moves and resizes for the length of the motion,
and the stage is built so that it does not react. Two extents have two jobs: the host is the viewport that
fitting, centring and wheel normalization mean, while the stage surface - every Konva layer's raster, the
dot grid and the region culling admits - covers the largest extent the host can take, the window. A moving
work-area edge is therefore a clip: no layer is reallocated or repainted, no grid is re-rasterized, and no
card is mounted or unmounted as the edge sweeps across it. Only the window resizes the surface, growing
immediately and shrinking once the window settles, quantized to 256px steps. The price is a culled region
up to the sidebars' width wider than the visible host while sidebars are open; cards there are mounted and
clipped, which is what keeps them from mounting inside the next panel motion. Measured on the real canvas
with 2,500 notes and both sidebars open, that is about twice the mounted cards (49 instead of 24 at 100%,
121 instead of 63 zoomed out), and it buys toggles with no layer draw, no canvas reallocation and no card
mount; culling against the host instead mounted 30-63 cards and redrew every layer inside the motion. A
pan pays nothing for the extra cards, because the world layer moves them by transform.

The camera lives in the host's own coordinate space and is never compensated for host movement: a trailing
edge leaves content alone because the origin stays put, and a leading edge carries the content along with the
work area, which the layout moves on its own without a single canvas draw. Compensating the leading edge
would have to read the host's new origin, which no observer can report earlier than the layout that created
it, so the correction always lands one frame after the pixels it corrects - and a one-frame correction of the
whole scene is the jump it was meant to prevent.

Zoom, fit, reveal and selection clearing cross one small editor port and delegate to the mounted engine. Pinia holds the current aggregate projection and session selection but does not mirror vendor records. Rust owns revision-checked structural history; Renderer history groups related Rust edits so one physical action remains one Undo/Redo step. Camera movement stays outside structural history. System preview is a workbench command for one selected managed asset and opens the platform preview directly; it is not a canvas input-mode protocol.

Each still-open Space tab retains its own Renderer Undo/Redo stacks. Switching tabs saves the current
stacks and restores the destination tab's stacks only after the Space leave boundary completes;
closing a tab retires its stacks. History never crosses Space identity and is not persisted across an
application restart.

On macOS one selected managed resource can open in the system Quick Look panel — the same non-modal preview Finder shows for a file. Content without private file bytes has nothing to hand to the platform and therefore does not expose the command. Space remains the focused canvas pan modifier; preview never delays or intercepts it.

## Desktop editing semantics

Spaces follows desktop content-editing expectations:

- Copy/Cut/Paste work on selected structure nodes.
- Paste lands where the pointer last was inside the canvas, converted through the viewport at paste time rather than at pointer-move time, and falls back to the canvas center when the pointer never entered the canvas.
- Duplicate preserves relative layout.
- Copying a Group expands to the whole operation unit.
- Copying a Section includes its contained structure.
- Binary assets are duplicated so pasted items do not share a file whose lifecycle belongs to another item.
- Knovum-to-Knovum clipboard fidelity uses one private structured marker embedded in `text/html`, plus useful `text/plain` for interoperability. There is no second compatibility protocol or silent downgrade path.
- Pasting an OS clipboard image reads pixels through Tauri's clipboard API, then a typed Rust command validates and persists the PNG as a managed Space asset; pasted binary assets enter the same lifecycle as dragged/imported files.
- Undo/Redo consumes one command timeline that coordinates `CanvasDocumentV1` history with reversible SQLite domain mutations; there is no second user entry point.
- Item deletes are soft deletes so an in-session Undo can restore content and private assets safely.
- Delete acts on the whole canvas selection: selected relations are removed before selected items, a relation selected on its own is deleted by the same command, and both fold into one Undo step that restores items before their relations.
- Deleting a user-created Space is a separate destructive operation: it requires confirmation, cannot target the system Inbox, removes its database records and then cleans its private assets.

Application commands go through the shared `AppCommand` registry. Spaces does not install a parallel global `keydown` command system; clipboard event handlers are limited to the browser/desktop copy, cut and paste semantics.

## Storage

Choosing Record Audio starts microphone capture directly. System/WebView authorization owns
permission prompts; Knovum does not present a separate permission confirmation screen or persist
its own granted flag. The recording control dialog shows elapsed time, pause/resume, finish and
cancel, with explicit starting, saving and capture-error states. Closing cancels unfinished
recordings; finalization disables dismissal. Pending starts own their temporary stream/session
until they can be published to the current recording lifecycle.

The Knowledge Types editor keeps one local draft, with type navigation on the leading side and
properties plus compact field rows on the trailing side. The type list and editor own independent
scroll areas within a viewport-bounded dialog; its title, footer and New Type action remain fixed.
Field row identity is independent of
editable field keys. Switching or closing confirms unsaved changes; saving locks editing and
retains failed drafts. Type deletion is confirmed, and the database refuses deletion of types
still referenced by knowledge objects. This editor does not introduce a second type model.

Spaces owns `spaces.db` and its SQLx migration history in
`src-tauri/src/spaces/db/migrations/`, whose shipped `0001_spaces.sql` baseline is frozen. The
embedded migrator brings a database up to the current schema before repositories receive the pool.
A schema change ships as the next numbered migration that carries existing rows forward
([data-ownership.md](data-ownership.md) §Migration policy); the application must never delete or
reset a person's Space data. New derivations capture immutable source metadata when created.
Rebuildable projections remain derived from their owning nodes and relations.

Rust Run changes stream to the Task Center through a Tauri Channel. Active Runs are reattached on
startup; closing a panel does not cancel the Run. The selected model target is checked when a Run
starts. Durable
Action outputs reserve canvas space in their sources' derivation field through the shared structure
placement rule, taking the first clear cell of that field before publishing the placeholder.

Structured state lives in SQLite. Imported binary assets and cached website assets live under Knovum's app-data `spaces/assets` directory. Markdown imports are read as UTF-8 text into SQLite instead of being copied as opaque attachments.

WebView access to private assets is limited to that app-data subtree through Tauri's asset protocol. Source files selected through the native dialog or dropped from the OS are authorized only for the import operation; Spaces keeps its own private copy and does not retain arbitrary filesystem scope.

Dependency selection reasons belong to [oss-first.md](oss-first.md), while Knowledge revision,
citation, package and retrieval rules belong to [knowledge.md](knowledge.md).

## Database upgrade boundary

Spaces upgrades a library in place. SQLx applies pending migrations before schema access and refuses
a database it cannot explain: a checksum that no longer matches an applied migration, or a recorded
version this binary does not carry (a library written by a newer release). The startup errors
`spaces-database-migration-edited` and `spaces-database-newer` name those two cases separately; the
application does not delete, rewrite or downgrade that
database, and it never asks the person using it to reset their library. Space packages and media
knowledge both start at version 1, but remain independent contracts from each other and the database
schema.

Conversation tool operations persist invocation `context`, its `context_digest`, and response `output`.
Run attempts persist `output`, matching step outputs and package `runAttempts[].output`. These are the
current v1 fields; edit receipts persist their `committed_projection`. Changing any of them reaches
existing libraries and packages as a forward migration, not as a rejection. External model and MCP
protocol payloads retain their own field names.

Content layout JSON uses `sizingMode` (`content` or `frame`), which every shipped database carries.

## Session state ownership

The single `space-store` composes existing owners rather than maintaining parallel committed projections. `spaceSessionBindings` explicitly exposes cross-view bindings and hides persistence/coalescing internals. Do not replace this boundary with an unrestricted spread of private state or create a second nodes cache.

| State                                             | Owner                                               | Lifetime                                                              |
| ------------------------------------------------- | --------------------------------------------------- | --------------------------------------------------------------------- |
| Committed nodes, scenes, relations, edit snapshot | Rust SQLite, loaded as one typed session projection | Active Space; replaced after successful load/commit                   |
| Navigation history, cross-view workbench requests | Existing session/workspace owners in Pinia          | Desktop session; reset by Space lifecycle                             |
| Structural Undo/Redo stacks                       | Space session history owner                         | Per still-open Space tab; discarded when that tab closes              |
| Image edit/comment draft and dimensions           | Image editor/comment sessions                       | `(spaceId, nodeId)`; explicit Apply/Discard/Cancel before Space leave |
| Component focus, hover and primitive interaction  | Owning component or editor session                  | Component/editor lifetime                                             |
| Async query results                               | Existing Vue Query keys                             | Space and actual request parameters                                   |
| Run transport                                     | `runs/run-observation`                              | One Channel/invoke per run ID; removed on terminal completion/failure |
| Conversation task mapping                         | Owning capability                                   | Attached run; released when shared observation finishes               |
| Unsent conversation composer draft                | Spaces SQLite `spaces_conversation_drafts`          | `(spaceId, conversationId)`; restored after restart                   |

Navigation flush failure preserves the current Space and reports the failure. Successful website binding publication is independent of best-effort old-file cleanup. Asset refresh generates a new preview, publishes with a revision guard, then reclaims only explicitly owned candidates. The reference graph is read in one SQLite snapshot and retains soft-deleted references for undo. Filesystem deletion is not a database transaction; it is never reported as an atomic commit of both systems.

Mutation IDs bind the entire request digest. A retry of the identical request returns its committed receipt; a different request using that ID fails with `mutation_id_conflict`. Renderer receipt confirmation is reserved for ambiguous transport errors without a returned failure payload. Command failures (including `operation_failed`), invalid arguments and permission rejections retain their normalized `cause` and are never replaced with an older receipt. Revision conflicts use `space_edit_revision_conflict` and preserve the current committed projection; they report a structural edit raised against a structure that has moved on, so node-scoped creation and pending-content discard, which append or remove against the structure their own write transaction loaded, cannot raise one.

## Document delivery

Rust `documents/generation` serializes DOCX, PDF, TXT and PPTX from one revision-checked Markdown
source. Spaces owns authorization, pending output placement and publication; Vue submits intent and
observes the same Action Run. PPTX produces text slides through `knovum-pptx:1`; more than 100 slides
fails visibly. The supported Markdown subset and rendering limits belong to
[resource-export.md](resource-export.md).

### Semantic search activation

Find starts with keyword and structure results. Selecting semantic search prepares the default local
embedding capability and indexes the current Space; Settings is optional management, not an activation
prerequisite. Resource preparation and content indexing have separate visible phases. Only the current
query is selected for display when preparation finishes, and switching Space prevents continuation on
the old scope. The Rust hybrid request explicitly states whether semantic retrieval is requested.
Semantic embedding/vector/database errors fail visibly rather than becoming empty matches or a
successful lexical-only response. Background index updates require already verified models and do
not trigger large downloads on note save. Grounded semantic use prepares resources at the retrieval
use-case boundary; the AI engine itself does not download.

Document exports submit source identity/revision to the same canvas Action acceptance boundary. Rust owns
output path reservation, pending-node placement and source validation;
render completion and failure update that same durable output. Vue does not allocate document
paths or write a second pending output state. Image preview remains an interactive browser renderer,
while image preview/export use Rust; shared tone fixtures constrain both environments without
claiming full pixel equality across browser interpolation and Rust rendering.

## Visual enrichment revisions

Images and video covers share the `analyze_visual` task and `visualKnowledge` result contract. Image revisions include image edits; video-cover revisions include the media source, immutable preview path and preview origin. Node writes compare this visual revision directly, so changing or resetting image edits and replacing a cover reschedule visual work. Titles do not change either revision. Evidence retains its image or video-cover locator so retrieval can open the correct source.

The shipped Spaces schema admits only the current job types; the former `analyze_image` / `analyze_video_cover` job names are gone. There are no compatibility aliases or conversion readers, and nothing in a person's library is deleted to make that true: queued jobs are carried forward or retired by a migration.

### Video subtitles

**Add subtitles** is an atomic video-node derivation (`add-video-subtitles`), not a built-in Workflow. It takes one current video and publishes one new video with burned-in subtitles and a `derives-from` relation to that video. The original video is unchanged. Current timed transcript cues are reused; when no transcript exists, the same local Whisper runtime recognizes speech inside the derivation. Neither step requires a user model/API key. Subtitle text is an internal temporary render input, not an intermediate canvas node or review gate.

Local recognition and FFmpeg share the derivation cancellation token. Input asset and knowledge revisions are captured before execution and revalidated before atomic publication; cancelled or stale results cannot be published. Local resource/tool preparation failures remain visible. There is no standalone `subtitle` node kind or subtitle-resource Action. Subtitle tracks in the video editor and transcript cues remain media-owned editing/recognition data.

### Derivation and source editing

Canvas operation semantics are independent of bar position or panel shape. Image crop, rotation,
flip and adjustment change the original node through the existing Apply/Cancel edit session and
one history entry. Transcription enriches the original node's knowledge. Background/object removal,
upscale, layer splitting, multi-angle generation, burned-in subtitles, rendered video editing,
speech-project generation and source separation publish derived nodes with active `derives-from`
relations. Choosing parameters or invoking Generate is the operation's execution decision; a
successfully validated derived result requires no later acceptance or “add node” click. Cloning
creates a reusable voice-library resource, and retains its own audition and managed-reference validation contract.

Speech-project generation and source separation reserve their output nodes in the same write
transaction that creates their Run, through the reservation contract every derivation uses
([creation-sources.md](creation-sources.md)): content insertion, source-adjacent collision-aware
placement, history and `derives-from` relations. Both separated stems are reserved in one history
operation, and publication only settles content onto nodes the canvas has been showing all along.
A Run that ends without publishing settles its reserved outputs as failed, so nothing waits on
finished work. Each stem is an ordinary audio node with its own playback/export actions; the
original retains its normal action bar. No duplicate result-review bar or insertion state is
maintained in the UI.
The shared resource-run observer invalidates the owning space's resource-publication queries on
completion, including when its editor is closed or another space is selected. Published-result
selectors use that query namespace; canvas refresh and query-cache refresh remain separate.
Refreshing the Space a Run publishes into is a reload, not a navigation: the canvas session stays
mounted, edits stay admitted while the Run reports, and the view changes only where the reloaded
projection differs from the one the session already holds.

The shipped v1 database/package contract records the source node and both stem paths
and digests, without review states. A later revision reaches existing libraries as a forward
migration; no compatibility reader beside the old shape and no silent reset is introduced. Snapshot, restore,
remapping and deletion reference graphs preserve the source identity and both outputs. Interrupted
attempts retain publication journals until both assets, nodes and relations can be committed or
cleaned up.

Audio and video node toolbars expose transcription as a separate operation. A current edited transcript remains authoritative: the action shows the existing transcript state and cannot silently replace it. A queued, running or resource-waiting enrichment job disables a second submission, including jobs restored from Rust state. Transcript and inspector components are keyed by node identity so local editing state cannot leak to another selection.

### Local image transformations

Image operations pin the current asset revision and saved `imageEdit`; the rendered source and all
normalized mask/point/box coordinates refer to that same image. Original managed bytes are verified
before and after rendering. Publication checks the active Run authority, current source/edit and
cancellation in the existing write transaction. Successful operations automatically publish one new
node and `derives-from` relation. Crop, rotation, flips and deterministic adjustments remain same-node
edits. AI background removal, upscaling, repair, selection extraction and depth estimation are derivations.

The explicit execution parameter freezes either a local model with its snapshot-bound specification
or a configured remote target for multi-angle generation and text-described removal only. Upscaling,
brush/selection repair, background removal, selection extraction and depth analysis use local models;
their former provider and external-Agent execution code is removed. Changing preferences affects only
new invocations; a failed execution does not switch models or engines. Local MI-GAN handles a binary
selected-region mask, not text-guided localization. These rules own the image Actions and canvas tools;
conversations retain the built-in/external Agents’ own tool selection without interception.

Smart selection holds points, exclusion points, a rectangle and optional brush additions in a transient
node session. Six decoder prompts include the rectangle's two corners. Prompt updates serialize on
one worker and ignore obsolete masks while retaining the current encoding; model/source changes and
closing the session cancel its work. Only Extract or Repair publishes a derived resource.
Relative depth is saved as a 16-bit PNG with `imageDepth.encoding = relative-inverse-depth-u16-v1`
and its frozen model specification. Near regions are brighter; the values are relative, not distances.
Selection extraction and depth generation are included in both the strict SQLite derivation schema
and package restore validation. Cargo watches the migration directory so embedded SQL follows that
history.
The original PNG is the delivery asset: image and managed-asset exports copy it without photo rendering
or 8-bit conversion. Depth nodes expose no photo-edit tools and reject `imageEdit`; their values are
analysis data rather than photo colors. Depth-based blur remains unavailable until compositing quality
has passed its own foreground-edge and alpha acceptance.

### Selection navigation and smart spacing

Camera projection owns rendering while navigating. `CanvasRenderLayer`, registered through vue-konva's
public custom-node API, delegates scheduling to that owner during motion and to Konva during editing.
Konva's Transformer explicitly invokes `batchDraw()` independently of `autoDrawEnabled`; allowing both
schedulers redraws the selection layer and its hit graph after the camera already painted it. The layer
contract suppresses those redundant schedules, not the Transformer or selection UI. Settlement restores
normal scheduling and hit testing. Floating selection surfaces move with `translate3d`, not layout offsets;
content size and avoidance remain governed by the existing selection-placement model.

Translation snapping uses the captured gesture index and moving selection's union bounds. Each axis
chooses the smallest correction among edge/center alignment, centering between neighbors and continuation
of an existing neighboring pair's positive gap. Exact ties prefer spacing. Spacing participants must share
an actual cross-axis overlap: another row's gap is never borrowed. Distances are axis-aligned outer bounds,
including rotated frames, not distances between arbitrary points on rotated edges. A translation gesture owns its magnetic locks: acquire within 6 screen pixels and release beyond 10,
independently for each axis. The target and reference guides do not get re-elected while held; a spacing
lock is also released when its measurement cross-axis leaves the moving object. New guide coordinates are
captured from the final two-axis placement. Candidate lookup runs only on acquisition, not for a held axis.
Raw movement comes from the pointer in the draggable node's parent coordinate system, minus the initial
grab offset, never from the node's snapped position. Edge auto-pan calls the same pointer projection rather
than accumulating viewport deltas onto a snapped node. This keeps input, magnetic output and rendered
geometry separate, including during auto-pan.

Equal-gap feedback shows both reference and proposed gaps, numeric canvas-unit distances rounded to one
decimal, and endpoint ticks. Text, ticks and strokes stay fixed in screen pixels through zoom. Distance text always sits outside the
measurement line (above a horizontal gap, right of a vertical gap); there is no rounded badge, opaque fill,
or short-gap placement-mode threshold that can flip during dragging. Rendering stays
inside the existing Konva feedback layer; there is no second draggable DOM overlay or external snap engine.
The existing indexed geometry is retained because introducing a DOM-transform controller would add another
gesture owner around Konva's node and section gestures rather than replace them.
