# Spaces product model

Outline rows expose native-context-menu Rename and the `spaces.rename-node` AppCommand (`F2`, one selected persisted node). Inline editing trims and validates the existing 1–512 character node title contract; Enter commits, Escape or leaving the field cancels, and IME confirmation does not commit. Failed saves retain the draft and show a localized error. The existing node-data/history pipeline persists the change, updates canvas/outline/search projections and supports undo/redo. Title changes do not modify filenames, URLs or source revisions, and media-enrichment intents synchronize only when their source/preview/analysis/processing inputs change. Website capture records its source `pageTitle` independently of the editable knowledge title; refresh preserves the knowledge title.

**Knovum** is a spatial knowledge application. The knowledge-space capability is owned by
`src/features/spaces/*`; `spaces` is the bounded-context and IPC namespace, not a peer application
inside a multi-product container. Its sidebar, toolbar and selection-driven inspector are composed
into the stable desktop shell, while Settings is a separate application route.

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

Canvas tools are transient Space session state, separate from persisted canvas geometry. The top-center Reka toolbar exposes Select (V), Pan (H), Note (N) and Text (T). Outline belongs to the current-Space region of the sidebar; Space management belongs to native context menus, not the tab strip. Tool shortcuts enter through AppCommand, not feature-level keyboard listeners. Opening Outline reveals the sidebar through its neutral presentation contract. Note and Text place empty knowledge at the next canvas click without changing the viewport, then return to Select and activate inline editing. Pan routes left drags through the existing viewport controller and disables node manipulation, relation hit testing and DOM content interaction. Held Space and middle-button drags use that same controller; trackpad scrolling remains unchanged. Switching Spaces resets the tool.

Space management is available through native context menus on Space cards, tabs and Inbox. These entries share one feature-owned menu composition and always bind to the selected Space. Knowledge Health opens the actual inspection capability, not a misleading Space Details page; entering the library closes the contextual Inspector. Space package import belongs to the library toolbar and creates a new Space. Space package export commits the active session before collecting its transactional snapshot. JSON Canvas export is an advanced exchange option for the active canvas, not a knowledge backup; JSON Canvas import remains available through the command palette. The incomplete automatic backup capability is removed; existing backup files are not deleted and remain importable Space packages.

JSON Canvas export writes through one Rust command authorized by the user-selected `.canvas` destination. The command validates the document and its exact managed-resource set, prepares a uniquely owned sibling resource directory, rewrites relative references, flushes a complete staging document and replaces the selected file through a same-directory rename. Failed preparation cleans up only that publication's files. Existing exported resource directories are not modified or deleted: they may still be used by external documents. No frontend filesystem scope is broadened. Space package archives likewise replace their destination directly from complete staging files, without moving the old destination aside first.

Space packages export active knowledge, not the Recently Deleted collection. Derivations are included only for active outputs. Each input preserves its historical `sourceIdentity`, revision and immutable `sourceSnapshot` (title, kind, original URL and platform); its nullable `nodeId` is a current-space link, not the historical identity. Permanent source deletion sets that link to null rather than cascading away the input history. Export detaches links to deleted or out-of-space sources; restore preserves these snapshots without restoring deleted nodes. Workflow output links include only active nodes. Knowledge events are selected against the actual exported node, relation, provenance and custom-type identities. This is the strict pre-release baseline contract; obsolete development databases/packages are not read through compatibility adapters.

The sidebar keeps Space navigation fixed above a directly visible Outline and Recently Deleted fixed below it. Inbox comes first as the capture destination for knowledge saved before choosing an organizational Space, followed by All Spaces, Recently Opened and Favorites. These four entries use consistent row spacing without an extra divider or gap after Inbox. Only the Outline list scrolls. A subdued Outline heading identifies the content region without repeating the current Space title already shown in its tab. Whitespace separates navigation from the Outline; there are no decorative divider rules or whole-Outline disclosure state. The `spaces.focus-outline` command (`$mod+Shift+O`) reveals the sidebar and focuses its content region on every invocation. Section disclosures remain because they represent actual document structure.

Truncated Outline titles scroll in one direction after a short hover delay and remain at the end until the pointer leaves. They never reverse or loop. Titles that fit stay still. Measurement and resize observation are limited to the hovered row; unchanged geometry does not restart the animation. Leaving or unmounting cancels the native animation and restores truncation. Reduced-motion preferences disable scrolling, while the original full title remains available to accessibility and the native title hint.

Recently Deleted has one workbench dialog and one entry in the persistent sidebar footer. It opens with all Spaces; Space filtering belongs inside the dialog. The query returns deleted Structure roots with deletion-batch identity, kind, title, Space and time. Deleting several effective roots creates one batch. A row can restore one root, while the batch action restores the complete user operation; both preserve StructureNodeIds, hierarchy, local transforms, content identities, internal relations and history semantics. Permanent deletion validates retained Portal targets and removes complete subtrees plus empty deleted containers in one Rust transaction. Filesystem cleanup receives only post-commit candidates and removes a file only after the asset command confirms it is unreferenced. The list remains virtualized and does not hide an arbitrary result limit.

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

The canvas is a projection over durable knowledge, not the data model itself. Resources preserve source material; `object` nodes carry strict typed Knowledge Objects; Portal nodes are navigation projections. Relations, Provenance and temporal Knowledge Events are first-class Space-local records. Agent output enters long-term state only through a validated atomic KnowledgeChangeSet. The loopback-only MCP server exposes reads and the same Rust-owned commit use case; it never exposes SQLite or partial mutations. See `knowledge.md` for the current contract.

## Knowledge Derivation

Implemented action families currently include image background removal and video subtitle burn-in. The video action consumes the selected video's current token-timed subtitle knowledge, creates a separate MP4 resource with subtitles rendered into the image beside the source, and records both the durable derivation row and `derives-from` relation. It never replaces the source video or consumes stale or pre-v2 transcript knowledge.

Spaces treats generation and media transformation as **Knowledge Derivation**: selected knowledge produces a new durable resource without losing its source context. The product entry is **Create from selection**, followed by an intent such as Summarize, Compare, Visualize, Remove background, Create subtitles or Turn into a document. Provider connections remain infrastructure configuration. Compatible model selection is available at the creation surface, defaults by capability, and can be captured with bounded generation parameters as a reusable **Generation Preset**. Models are execution targets, not ordinary knowledge nodes and not the canvas information architecture.

Editing an existing node keeps its identity; Create from selection publishes a separate resource.
Duplicate copies a node without claiming derivation provenance, while Download writes an external
file. The exact effect and revision rules for every node kind live in `docs/architecture.md` under
“Space operation effects and source identity”.

A **Workflow** is an editable, published and version-pinned dependency graph over the stable Space Action catalog. Knovum ships five production recipes: presentation production, video production, social content package, subtitled-video production and meeting content package. These are common knowledge-work starting points, not samples or private executors. Each recipe creates the same editable Workflow definition used by user-authored graphs and places a content-review gate before an expensive or irreversible rendering step. Runs, step inputs and outputs, per-item batch lineage, attempts and events are durable, exported with terminal history, and restored without replaying side effects. Batch retries retain stable item keys, reuse successful items and create new attempts only for failed or unavailable items.

The Rust Run scheduler is the only execution authority. It compiles immutable Workflow versions,
leases ready steps, fences stale workers, records checkpoints and events, and delegates business
effects to their owning Space Actions. Vue edits one structured draft through graph and outline views
and observes Run state; it never advances a step or commits a result because a component reported success.

A **Creation Intent** is the user-facing catalog entry. It owns selection availability, localized presentation, and whether execution dispatches one atomic derivation or a versioned Workflow. The canvas uses one intent catalog and one creation-dialog session, so adding a Workflow does not add another event chain or independent boolean state through the component tree.

Published Workflow versions are immutable. Subworkflows bind exact versions, while drafts remain
incomplete and autosaved. Execution bindings, credentials, approvals and trigger grants are not part
of the portable definition. Imported triggers and external-tool bindings are disabled until the user
reviews their destination, current schema fingerprint and authorization.

Personal Workflow templates carry the complete fixed-version subworkflow closure. Copying one into a
Space assigns new Workflow identities, publishes dependency versions in child-first order and rewrites
the root definition atomically. Space-owned external-tool bindings are deliberately rejected from
personal templates instead of being copied as ambient authority; the user must bind and review those
tools in the destination Space.

External Agent conversations are distinct from one-shot artifact creation and from the built-in
Assistant. A connected conversation owns one ACP runtime session and accepts multiple turns through
that session; each turn can include user-selected local attachments, and the UI derives mode, model
and other configuration controls from the actual ACP session response. Local conversation, turn
and message identities remain durable. Tool activity is recorded
as reported Run events, while Space changes still require Knovum MCP receipts. An application or runtime
disconnect marks the conversation disconnected and preserves history; only an explicit new conversation
starts another runtime session. Closing a conversation exposes any workspace artifacts through the
existing reviewed artifact-import path.

DOCX and PDF export consume one explicit current Markdown resource. They do not silently synthesize a hidden document from an arbitrary selection; users create or choose the editable Markdown knowledge first, then export it. Multi-stage outcomes remain Workflows and keep their independently useful intermediate resources visible.

AI configuration has four explicit levels:

1. an **AI Connection** owns provider, endpoint and one credential reference;
2. models registered under that connection declare their supported capabilities;
3. each capability has an application default model target;
4. an individual creation may override the default or apply a reusable Creation Preset.

The resolved provider and model are recorded with completed derivations. Changing a default never changes the identity of an existing generated resource.

A successful derivation:

- preserves its input resource IDs and exact source revisions;
- records the action, bounded parameters, engine/model identity, actor, timing and outcome in a structured Derivation record;
- creates the output as an ordinary Space resource that can be edited, searched, exported and used as a later input;
- creates `derives-from` Relations from the output to its inputs;
- places the result beside its source or selection bounds without moving the inputs;
- distinguishes generated artifacts from accepted Knowledge claims.

Transformation lineage and epistemic Provenance are different contracts. A Derivation explains how an artifact was produced. Provenance records evidence supporting a Knowledge claim or Relation. Generated factual or semantic changes enter durable Knowledge only after complete KnowledgeChangeSet validation and one atomic transaction.

Long derivations create a pending output node immediately. The pending node reserves the eventual result position and mirrors the feature-owned Rust job's meaningful stage and progress through Task Center. Success replaces it in place; failure remains inspectable and retryable but does not create a false completed resource or semantic lineage.

Workflow control and data dependencies are edited in the dedicated Workflow Studio rather than on the
knowledge canvas. Studio provides one graph/outline draft, compiler diagnostics, readiness preflight,
version history, isolated whole-flow or selected-step tests, persistent breakpoints and Run Center
inspection. Model samplers and provider plumbing remain execution bindings rather than canvas nodes.

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

Audio/video knowledge is a first-class Space record. Analysis accepts only Knovum-owned Space assets and uses bundled FFmpeg once to create a 16 kHz mono PCM working track plus a compact waveform. One accelerated `whisper-cli` process handles the complete track through its official full JSON contract and built-in Silero VAD, skipping long silence without repeatedly loading the model or creating intermediate chunk files. CLI progress is mapped into the shared Task Center, and segment plus token timestamps remain on the original media timeline so citations and subtitle cues align with actual speech. Media bytes never enter a generative-AI provider path.

`mediaKnowledge` v3 persists Transcript + token-timed subtitle cues + waveform as a complete local state. Subtitle cues are separated only by detected speech pauses, sentence endings, or elapsed audio time; FFmpeg/libass measures glyphs and performs Unicode wrapping against the actual video frame. Summary and Chapters are optional `insights` generated only after the user explicitly invokes that action with Assistant AI enabled; they are not required for search or playback. Chapter starts are validated against real Transcript segment boundaries, so every Chapter and Transcript row can seek the canvas player precisely. Transcript and generated-insight chunks are indexed separately, and long transcripts use `@tanstack/vue-virtual` rather than mounting every segment in the WebView DOM.

## Sensitive information

A Sensitive Information card is either an API Key or an Account & password record. SQLite stores only non-secret metadata (`title`, `credentialKind`, `credentialConfigured`). Secret values live exclusively in the OS credential store. The card never exposes the value in its collapsed state, duplication does not copy a secret, and permanent Space deletion cleans the corresponding credential entries.

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

A single imported resource file is limited to 8 GiB. Directory imports are not supported. Archive files are preserved as opaque binary file assets without container expansion or semantic indexing. The node contract rejects obsolete development `folder` and `archive` data rather than interpreting either as a file. Space package ZIP transfer remains a separate workspace operation, not a canvas-resource import.
Known credential-bearing dotfiles such as `.env` and `.npmrc` remain opaque file resources instead of being copied into SQLite/FTS text previews, and are equally excluded from Quick Look thumbnails: a canvas thumbnail must not become a second, less protected copy of a secret. Users can still open the private copy in their associated editor.
On macOS and Windows, images, audio, video and renderable documents request the system's own thumbnail, cached beside the other private previews. macOS uses Quick Look and Windows uses the registered Shell thumbnail provider. Preview generation is a strict native capability: failure is reported instead of activating an alternate renderer. Source/code formats retain their product-owned text presentation.

Opening an owned resource launches the Space-private copy. When Knovum starts, regains focus or becomes visible again, it re-inspects private files so edits performed in Word, VS Code or another specialist application update source previews and media/PDF version stamps. Refresh failures are surfaced rather than silently replacing the resource.

PDF/Office **full embedded editing is not a product goal**. `system_preview` is the only canvas renderer for native thumbnails: Quick Look owns the macOS adapter and registered Shell providers own the Windows adapter. `pdf-inspector` owns PDF page-level text extraction; `office_oxide` owns semantic Office extraction. The evidence inspector shows extracted text and opens the original file; it no longer renders PDF pages. PDF coverage distinguishes trusted text from pages requiring recognition, and explicitly reports that document visual analysis is not yet implemented. Import, native visual preview and semantic extraction are separate ordered phases: the pending node appears immediately, the native preview is published through the import channel as soon as it exists, and semantic extraction continues in the background. The original private bytes remain authoritative and professional applications own high-fidelity editing.

Current Markdown knowledge can be exported as DOCX or PDF. DOCX remains an editable delivery artifact; PDF is a fixed-layout delivery artifact. Both are managed Space resources with exact source revision, derivation metadata and `derives-from` lineage.

## Canvas cards and Sections

Native website layers do not take initial keyboard focus from the canvas. During canvas motion or direct manipulation, the child WebView is suspended while the node shows its saved site identity or preview image; after the viewport settles, the browser is placed at the final bounds and shown again. Native placement requests coalesce to the latest frame rather than queuing obsolete coordinates. Browser content bounds are projected from the same scene frame and viewport that paint the canvas, using the host rectangle only as the origin; they are not sampled from CSS-transformed node DOM. The main WebView origin is then converted to the parent window's client-area logical pixels at the Rust boundary. Rotated or partially clipped website content stays on its saved preview because a native child WebView cannot rotate or clip with the canvas. Resizable nodes have semantic minimum sizes but no arbitrary presentation maximum; persisted frames must remain finite and positive.

Canvas items are **content-first spatial representations**, not a grid of generic file cards. Notes and Markdown remain bounded summaries because their full body has a Knovum editor. Images and document snapshots expose content directly; media stays an inert cover until activated. A video cover carries its duration; a separate, fixed-height caption shows at most two title lines and then platform and creator, without letting a long title push the source out of view. Code shows source. Saved Websites persist the URL immediately and enrich metadata in a bounded background queue. Website nodes render a stable local preview or site identity; the original URL remains available for opening in the system browser. The canvas does not host independent browser surfaces. Opaque specialist formats keep a restrained resource identity until a maintained renderer is selected.

Image and video editing deliberately use different product surfaces. One selected image exposes
non-destructive crop presets, quarter-turn rotation, horizontal/vertical flip, tonal adjustments
and an eraser directly in the canvas selection bar. The persisted `imageEdit` contract keeps the
source asset immutable; the canvas renderer projects the crop, transform, filter and normalized
eraser strokes. System preview and PNG export render the same saved edit through Rust; opening
an image never hands the unedited source to the system preview. Image comments are normalized
positions on the displayed image, saved in the image node payload with their text and creation
time. A comment may target a point, a freehand
stroke or a rectangular region; all three modes render a numbered pin, while strokes and regions
retain their image-relative geometry. Adding a comment to the built-in conversation stages its
text in the composer rather than starting an assistant task. Multi-angle generation accepts
subject or camera movement, yaw from -90 to 90 degrees, pitch from -30 to 60 degrees, and
close-up, medium or wide-angle framing. Its preview is an interactive orientation guide; the
model generates the actual new view. Video editing opens a same-window workbench because
transport, playhead, ordered clips and timed text require persistent timeline context. A strict
v1 `videoEditProject` stores one ordered video track, one text track and output transform. Saving
records the project on the source
node through normal Space history. The editor opens from the managed source immediately while
Rust prepares a revision-keyed, short-GOP 960×540 editing proxy in the app cache. Once ready, the
WebView plays that proxy; video frames never cross IPC. Proxy preparation is cancellable when the
workbench closes, and its failure is shown rather than hidden. The displayed playhead follows
presented frames instead of the media element's coarse timeupdate event. Rendering reads the
managed source, seeks to the first retained source interval, reports measured FFmpeg progress and
creates a new derived video node. The source bytes and edit project remain available, and render
cancellation stays in Task Center.

The light editor keeps one continuous, ripple-joined video track and one timed-text track. The
time ruler supports pointer scrubbing; clip-edge handles trim against the original source with
100 ms snapping, and Alt temporarily enables millisecond precision. Focused trim handles also
accept arrow keys. No inspector is shown until a clip or timed-text item is selected. Its floating
panel sits over the preview stage while transport and timeline keep the full workbench width; Escape
or the close control clears selection and returns focus to the ruler. Timed text is selectable from
the preview and timeline. The panel offers a searchable subtitle list, live text edits, placement,
size, color and four export-backed styles; clip selection exposes source in/out times, volume,
rotation and both flips. Text size is expressed in 720p-relative pixels, and rendering uses the
bundled Noto Sans CJK font. A local edit history owns undo and redo until Save commits one
project snapshot through Space history. The workbench shows unsaved state and requires an explicit
Save or Discard choice on close. Application commands provide the editor's keyboard shortcuts;
there are no feature-level global key listeners. The preview places timed text within the visible
video frame and uses the same relative vertical placement as final rendering.

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
- Preview and export share clip ranges, per-clip volume, orientation, and timed-text placement;
  a removed or shortened interval ripples the text track by the same duration.

URL paste reads clipboard HTML, text and, on macOS, the native `public.url` format in one typed Rust operation before applying an explicit product-owned content-URL policy for selected media platforms: YouTube, Bilibili, Douyin, TikTok, Instagram, Vimeo, X/Twitter, and Facebook. Direct media resources and recognized platform content links enter yt-dlp acquisition. The capture projection appears at the paste location as compact desktop transfer chrome rather than an empty media cover; its screen size is capped when zooming in so temporary feedback does not balloon over the canvas. Resolving shows indeterminate activity; downloading shows measured bytes; finalizing shows measured media time when FFmpeg prepares a playable asset. yt-dlp prefers H.264/AAC streams, and the import stage converts other selected codecs to H.264/AAC in MP4 or AAC in M4A before the asset is committed. The node persists the canonical local asset size and its media origin (platform/extractor, original URL, and source ID); the canvas shows platform identity, while the inspector exposes origin and file details. The projection transitions into the persisted node on success and remains in place with retry/dismiss actions on failure. Platform homepages, profiles, and all other HTTP(S) URLs become Saved Websites immediately. The automatic clipboard path neither enumerates yt-dlp extractors nor probes arbitrary webpages for embedded video; this avoids blocking ordinary website capture on media-tool startup or network timeouts. A recognized media URL that cannot be acquired reports the media failure instead of silently changing node type. Clipboard file paste is a separate trusted import source: Rust verifies each requested path against the current native clipboard file list before staging it; dialog and drop imports continue to require their explicit filesystem scope.

A document the system could render takes that page's own proportions, so a portrait report and a landscape deck read differently at a glance. A near-square render is a generic icon plate rather than a page, and keeps the default file card.

Resources whose bytes cannot speak for themselves — files and code — close with a caption strip naming the resource type and its size. The strip replaces metadata scattered through the card header, stays at a fixed height across every canvas detail level so zooming never resizes the card, and is withheld from media: an image, a video or a player already says what it is.

Group and Section are Structure nodes rather than content-node variants:

- Structure owns `StructureNodeId`, parent, sibling order, lock and container title; Layout owns local transforms, Section frames, appearance and relation-style overrides; content rows own knowledge payloads.
- Group creation uses effective selected roots and preserves world geometry while converting children to local transforms. Entering a Group establishes the explicit editing scope; ordinary internal movement does not infer migration out of that Group.
- Section creation supports an empty drawn frame, fitting selected roots and drawing around fully contained roots. Creation decides membership once; later Section movement and resize never capture nearby structure.
- Dragging content or a whole Group across a Section boundary creates one explicit move-to-container command. A collapsed Section must be hovered for about 600 ms before it temporarily expands as a drop preview. Leaving, cancelling, Escape, pointer cancellation or window blur clears the preview; a successful drop commits movement, expansion and reparenting in one Rust transaction and one history entry.
- `Fit Section to content` changes its frame around existing descendants. `Remove Section only` reparents descendants while preserving world geometry. Deleting a Section deletes its subtree into one recoverable batch.
- Clipboard, duplicate, cut, cross-Space move, package transfer and Recently Deleted operate on complete effective-root subtrees. No path reconstructs membership from visual overlap after the user action.

Sections never nest. Groups may nest and may belong to a Section. A Structure node has at most one parent; overlapping Section geometry never creates ambiguous ownership.

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

Zoom, fit, reveal and selection clearing cross one small editor port and delegate to the mounted engine. Pinia holds the current aggregate projection and session selection but does not mirror vendor records. Rust owns revision-checked structural history; Renderer history groups related native edits so one physical action remains one Undo/Redo step. Camera movement stays outside structural history. System preview is a workbench command for one selected managed asset and opens the platform preview directly; it is not a canvas input-mode protocol.

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

Spaces owns `spaces.db` and its single pre-release SQLx v1 baseline at
`src-tauri/migrations/spaces/0001_spaces.sql`. The embedded migrator installs that complete schema
before repositories receive the pool. During pre-release development, schema changes are folded
back into this baseline and obsolete development databases are deleted instead of receiving
compatibility migrations. New derivations capture immutable source metadata when created.
Rebuildable projections remain derived from their owning nodes and relations.

Structured state lives in SQLite. Imported binary assets and cached website assets live under Knovum's app-data `spaces/assets` directory. Markdown imports are read as UTF-8 text into SQLite instead of being copied as opaque attachments.

WebView access to private assets is limited to that app-data subtree through Tauri's asset protocol. Source files selected through the native dialog or dropped from the OS are authorized only for the import operation; Spaces keeps its own private copy and does not retain arbitrary filesystem scope.

## OSS-first implementation choices

The feature follows `AGENTS.md` and `docs/oss-first.md`:

- The selected canvas engine owns rendering and transient interaction behind `CanvasEditorPort`; Knovum owns the engine-neutral Structure/Layout persistence contract. **JSON Canvas 1.0** is an explicitly lossy exchange boundary: visual groups import as Sections only when membership is unambiguous, while unsupported Groups, transforms, locks, folding and relation semantics are rejected instead of flattened. Imported edges become the same semantic `SpaceRelation` records used everywhere else.
- **Tiptap** (`@tiptap/vue-3`, StarterKit, Markdown, task-list, table, placeholder and character-count extensions) owns the shared editing engine. The official Vue `BubbleMenu` handles selection-local formatting while Knovum keeps compact desktop chrome. Knovum Notes persist Knovum's validated structured document tree plus derived plain text; Markdown persists Markdown text and only enable round-trippable capabilities.
- **MediaRecorder** owns local audio capture while a typed Rust command validates and persists the resulting container; a media framework is not introduced for a capability already covered by stable platform APIs.
- **Tauri Menu** owns right-click context menus on non-editable desktop workspace surfaces so macOS/Windows render native menu typography, separators, highlighting, keyboard interaction and placement. Canvas-background Paste uses Tauri's predefined native Paste role; the keyboard shortcut uses the existing application command and clipboard pipeline. Editable text deliberately keeps the platform/WebView text-editing menu. **Reka UI** remains appropriate for in-app left-click Dropdown/Dialog/Listbox behavior.
- **SQLite FTS5** provides the deterministic lexical index. Spaces uses the built-in trigram tokenizer for Chinese/English/code substring retrieval and a bounded short-term query for one- and two-character searches; Qwen3 embeddings remain a separate rebuildable semantic index, not a second source of truth.
- **VueUse** provides the existing bounded debounce primitive for Note/Markdown autosave.
- **Tauri official plugins + reqwest** provide the desktop integration surface. The website fetch command is custom only because arbitrary user-saved public websites require a dynamic, security-validated network boundary that cannot be expressed as a least-privilege static frontend origin allowlist.

The official Rust **RMCP** SDK owns the local MCP protocol/server implementation. Knovum does not maintain a custom MCP protocol stack.

The current Knowledge contract, SQLx migration policy, Provenance/History/Patch model and Work/MCP boundaries are documented in `knowledge.md`. The feature should continue to prefer stable maintained OSS before adding custom infrastructure.
