# Design

## Source of truth

- Status: Active
- Last refreshed: 2026-09-27
- Primary product surfaces: desktop knowledge-space shell, infinite canvas, object actions, editors, Inspector, one conversation surface, canvas-derived reusable workflows, contextual task feedback and Settings.
- Evidence reviewed: `AGENTS.md`, `docs/spaces.md`, `docs/knowledge.md`, `docs/ui-system.md`, `docs/background-tasks.md`, `docs/system-architecture.md`, `docs/architecture.md`, `docs/naming-conventions.md`, `docs/oss-first.md`, `src/features/spaces/*`, `src-tauri/src/spaces/*`, `src-tauri/src/ai/*`, and `src-tauri/src/external_agents/*`.

This file owns the product and interaction goals specified below. The dated domain/execution baseline and its implementation-state map are recorded in [docs/rfcs/knowledge-work-system.md](docs/rfcs/knowledge-work-system.md). Current runtime, dependency and data contracts remain in their owning documents until implementation changes them.

The target conversation/workflow direction is described in this file's Information architecture and Knowledge Derivation sections. It does not declare an unverified redesign complete or supersede current persistence contracts. Separate conversation/workflow and canvas redesign specifications previously referenced here are not present in this repository; no missing document is authoritative. Visual rules currently available to implementation are in this file and [docs/ui-system.md](docs/ui-system.md).

Changes to conversation history, contextual workflow editing and external-tool configuration need implementation evidence against the RFC state map. Product goals are distinct from completed platform acceptance.

## Brand

- Personality: thoughtful, capable, calm, spatial and quietly intelligent.
- Trust signals: visible source boundaries, reversible actions, explicit progress, traceable AI output, local-first processing where available, and stable desktop behavior.
- Avoid: AI spectacle, chat-first product framing, model marketplaces as primary navigation, dashboard-card layouts, unexplained automation, decorative graph complexity and generated content presented as established fact.

## Product goals

- Goals:
  - Make a Space a durable environment in which people capture, understand, connect, transform and reuse knowledge.
  - Let existing knowledge produce new knowledge through explicit, traceable derivations.
  - Preserve the relationship between an output, its source material and the operation that produced it.
  - Keep the canvas useful for thinking and recall rather than turning it into a generic workflow builder.
- Non-goals:
  - Recreating ComfyUI or exposing provider execution graphs as the default product experience.
  - Becoming a generic prompt playground, media editor or provider chat client.
  - Treating every generated artifact as verified knowledge or evidence.
  - Hiding long-running work, silently replacing source material or creating uncontrolled canvas clutter.
- Success signals:
  - Users can create a useful derivative without leaving the knowledge context that motivated it.
  - Every derivative can answer “what did this come from?” and “how was it produced?”.
  - Long work gives immediate, continuous and cancel-aware feedback through the canvas and Task Center.
  - Derived resources remain editable, searchable, movable, exportable and usable as inputs to later work.

## Personas and jobs

- Primary personas: knowledge workers, researchers, creators, product teams and individuals building a long-lived body of mixed-media knowledge.
- User jobs:
  - Capture source material without deciding its final structure up front.
  - Understand and synthesize selected material.
  - Turn knowledge into a better representation for the next task: document, diagram, image, audio, video, transcript or structured object.
  - Inspect sources and revise AI-assisted artifacts without blocking their use behind ceremonial approval.
  - Trace a result back to the exact resources and revisions that shaped it.
- Key contexts of use: focused desktop work, large mixed-media Spaces, offline/local processing, provider-backed generation, keyboard-driven inspection and direct canvas manipulation.

## Information architecture

- Primary navigation: the Spaces sidebar selects a Space or its internal navigator; the unified toolbar owns canvas and application-level actions. Settings is a temporary route, not a peer product module.
- Core routes/screens: Space canvas, full Note/Markdown editor, selection Inspector, Knowledge history, Task Center and Settings.
- Conversations have one history, message timeline and composer. Grounded retrieval is a conversation
  capability; API models and desktop Agents are execution targets with distinct native session behavior.
- Reusable workflows are captured from actual canvas derivations and applied to new selections. Their
  library and contextual editing belong to the canvas work area. External-tool connections belong in Settings.
- Task progress and required decisions appear at the initiating message or canvas result, with the existing
  Task Center providing a shared projection. There is no separate Run Center or Automation destination.
- Content hierarchy:
  1. A Space is the durable knowledge boundary.
  2. Resources and Knowledge Objects are the durable semantic material.
  3. Relations express meaning between items.
  4. Derivations record transformations that create new resources from existing material.
  5. Sections, Groups, Saved Views and projections organize or reveal the same knowledge without owning it.

## Design principles

### Knowledge is the product; AI is a capability

Start from the user’s selected knowledge and intended outcome. Do not make provider, model or prompt selection the primary interaction.

### Create from context

Generation begins from one or more selected resources, a Section or an explicit Space scope. Blank-prompt creation may exist for authored content, but it is not the default entry to Knowledge Derivation.

### Ask for an intent, not a modality

Present meaningful actions such as **Summarize**, **Explain**, **Compare**, **Visualize**, **Extract**, **Remove background**, **Create subtitles** or **Turn into a presentation**. The output format follows the intent and remains visible before execution.

Reusable generation parameters are **Generation Presets**. Multi-step knowledge production is expressed as a **Creation Workflow**: a small versioned graph with explicit step inputs and outputs. Workflows must keep intermediate resources visible when they remain useful knowledge, expose one coherent task state, and preserve Provenance for every produced resource.

### Every result returns to the Space

A successful derivation creates a normal durable resource near its inputs, connects it with semantic lineage and makes it available to search, editing, export and future derivations.

### Lineage is visible, detail is progressive

The canvas shows a quiet `derives-from` relation. The Inspector reveals source revisions, operation, parameters, engine/model, timestamps and status. Execution details do not permanently occupy canvas chrome.

### Generated does not mean true

Generated media may be an illustration, simulation or transformation rather than evidence. Generated identity and provenance stay visible; durable factual or semantic mutations must pass the strict KnowledgeChangeSet contract and commit atomically.

### Progress is part of the result

Long work must acknowledge immediately. A pending derivative reserves the eventual output position, shows a human-readable stage and remains synchronized with Task Center. The application must not freeze while leaving the user without feedback.

### A derivation must strengthen the knowledge space

A new output is justified only when it improves understanding, expression, reuse or transformation of knowledge already present in the Space. Generation must preserve source scope, lineage and source revisions. A result that cannot be searched, edited, related, inspected or used as a later input is an export utility, not a Knowledge Derivation.

### The canvas shows knowledge, not execution plumbing

The canvas may reveal source-to-result lineage and a compact pending state, but it does not expose model nodes, samplers, codecs or provider request graphs as ordinary knowledge objects. Reusable sequences become named Workflows captured from recorded derivations. The existing canvas previews their resources and steps; contextual panels edit input bindings, parameters and outputs. Template editing has separate draft and undo state and does not create ordinary Space nodes or a second canvas engine.

### Tradeoffs

- Prefer discoverable, typed `SpaceAction` families with focused contextual surfaces over a single
  universal “AI process” action.
- Prefer durable lineage and inspectability over instant but disposable generation.
- Prefer a quiet canvas with collapsible lineage over displaying the complete execution graph.
- Keep Workflow as a reusable, typed derivation definition while keeping one-click object operations direct;
  not every click creates a Run or Workflow instance. Arbitrary scripts, scheduling and general programming
  controls are outside the target workflow product.

## Visual language

- Color: semantic low-chroma roles from `src/styles/tokens.css`; accent is reserved for selection, focus, active progress and primary actions.
- Typography: compact desktop roles from `docs/ui-system.md`; readable content remains visually stronger than operation metadata.
- Spacing/layout rhythm: dense but calm; derivative outputs use the same type-specific canvas geometry as ordinary resources.
- Shape/radius/elevation: flat canvas objects, stable hairlines and restrained overlays; pending and completed derivatives must not appear as dashboard cards.
- Motion: short positional continuity from pending result to completed resource; no celebratory or decorative generation animation.
- Imagery/iconography: use the existing icon library and content-first resource previews. AI identity may appear in metadata, never as a competing visual theme.

## Components

- Existing components to reuse: Space selection bar and menus, canvas node frames, relation rendering, Inspector sections, editor surfaces, Knowledge history and shared Task Center.
- New/changed components:
  - **Create from selection** contextual action surface.
  - **Derivation action picker** grouped by user intent, filtered by selected input capabilities.
  - **Pending derivative node** occupying the output’s future canvas position.
  - **Derivation details** Inspector section for lineage, execution and source-freshness information.
  - **Lineage controls** for reveal, collapse, compare, rerun and inspect sources.
  - **Canvas Workflow capture and reuse** with typed inputs, explicit binding and contextual parameter editing.
  - **Conversation surface** with one history and composer, visible resource context, citations and task messages.
  - **Contextual task feedback** with cancellation, required input and selective retry, projected into Task Center.
  - **Connection diagnostics** for model, external Agent and external tool bindings.
- Variants and states: ready, configuring, queued, running, cancelling, failed, completed and superseded.
- Token/component ownership: Spaces owns derivation orchestration and canvas presentation; Foundation owns only existing cross-feature task and integration policy; reusable processing engines live behind explicit neutral desktop capabilities rather than feature-to-feature imports.

## Accessibility

- Target standard: WCAG 2.2 AA for WebView interfaces while preserving desktop conventions.
- Keyboard/focus behavior: every derivation action, configuration field, cancel/retry control and lineage inspection path is keyboard reachable with visible focus. Application commands use `AppCommand`, not feature-level global key listeners.
- Contrast/readability: progress, generated identity and relation meaning must not depend on color alone.
- Screen-reader semantics: pending resources expose action name, stage and progress; completed resources announce type, title and source count; lineage controls describe direction explicitly.
- Reduced motion and sensory considerations: positional continuity becomes an immediate state replacement when reduced motion is enabled; indefinite spinners always include text that identifies the stage.

## Responsive behavior

- Supported breakpoints/devices: macOS and Windows desktop windows; the product is not designed as a mobile web application.
- Layout adaptations: narrow windows prioritize canvas/editor content and move derivation detail into the existing Inspector or modal configuration surface without duplicating the workflow.
- Touch/hover differences: hover may reveal affordances, but every action remains available through selection and keyboard paths; no essential action depends on a narrow hover target.

## Interaction states

- Loading: create the pending derivative immediately, keep the source available and report a meaningful stage such as “Preparing sources”, “Removing background” or “Rendering subtitles”.
- Empty: explain which selection or source type is required; do not show an empty model playground.
- Error: retain the failed pending derivative with a concise reason, inspectable details and retry/remove actions. Never create a false completed resource.
- Success: replace the pending node in place with the durable result, preserve spatial continuity and reveal a quiet lineage relation.
- Disabled: explain the unmet capability, configuration or source requirement at the action boundary.
- Offline/slow network: local transformations remain available; provider-backed actions state that a connection is required before starting and preserve configuration if interrupted.

## Content voice

- Tone: direct, calm and outcome-oriented.
- Terminology:
  - Use **Create from selection** for the entry point.
  - Use **Derivation** in details and product documentation.
  - Use the concrete action name during execution.
  - Use **Source** and **Derived resource** for lineage endpoints.
  - Use **Workflow** only for a reusable, user-named derivation configuration or sequence.
- Microcopy rules:
  - Describe the result before the engine: “Create a subtitled copy”, not “Run FFmpeg”.
  - Name where the result will go: “A new image will be added beside the original”.
  - Distinguish **Generated**, **Source changed** and **Failed** without implying factual confidence.
  - Never use a generic “Processing…” when a real stage is known.

## Implementation constraints

- Framework/styling system: Tauri 2, Vue 3 Composition API with `<script setup lang="ts">`, a feature-local Konva canvas adapter, Tailwind CSS v4 semantic utilities, Reka primitives and Oxc tooling.
- Design-token constraints: `src/styles/tokens.css` remains the only token source; derivation UI extends existing Space and Task Center roles.
- Performance constraints: opening configuration and creating pending feedback must not wait for heavy parsing, model loading or Rust processing. Long jobs run outside the WebView main thread and stream progress through Tauri Channels.
- Compatibility constraints: no generic fallback, dual-write path or old-format runtime reader.
  Unsupported inputs or unavailable required infrastructure fail visibly at the action boundary.
- Test/screenshot expectations: capability filtering, immediate pending feedback, cancellation, failure retention, successful in-place replacement, lineage creation, keyboard operation and large-canvas behavior require regression coverage. Visual changes follow the existing desktop language and need representative macOS/Windows verification.

## Knowledge Derivation contract

Knowledge Derivation is the product capability by which one or more Space resources produce a new durable resource.

### Conceptual model

- **Resource**: source material or a generated/transformed output that can participate in later knowledge work.
- **Derivation**: a structured immutable record of one completed or attempted transformation. It records input resources and revisions, typed input roles, action identity and version, bounded parameters, engine/model identity, actor, timestamps, status and output resource IDs.
- **Relation**: the semantic `derives-from` edge connecting each output to its inputs.
- **Provenance**: evidence supporting a Knowledge claim or relation. Provenance is not reused as the transformation audit log.
- **KnowledgeChangeSet**: a temporary, validated set of factual or semantic operations committed atomically by Rust.
- **Workflow**: a user-owned, versioned definition extracted from replayable derivations over registered
  SpaceActions, with explicit resource inputs, step dependencies, parameters and outputs. Runtime execution
  binds fresh resources and exact targets; ordinary canvas relations do not become executable steps.

### Action families

| Intent          | Representative actions                                 | Typical output                               |
| --------------- | ------------------------------------------------------ | -------------------------------------------- |
| Understand      | summarize, explain, compare, extract structure         | Note, Markdown, Knowledge Objects            |
| Visualize       | diagram, concept image, storyboard                     | Image or presentation resource               |
| Transform media | remove background, resize, transcode, create subtitles | Derived image, audio, video or subtitle file |
| Re-express      | turn into document, presentation, narration or video   | Document or media resource                   |
| Synthesize      | combine selected sources into a report, brief or plan  | Note, Markdown or exported document          |

Every action declares one stable capability contract:

- accepted source kinds and minimum source count;
- required knowledge state and source revisions;
- user intent and bounded configuration;
- local or provider-backed execution ownership;
- output resource kind and generated identity;
- progress stages and cancellation ownership;
- persisted engine/action version and non-secret reproducibility metadata.

The UI derives action availability from this contract. Components must not duplicate capability rules with unrelated condition chains.

### Knowledge trust states

The resource lifecycle and the epistemic lifecycle are related but distinct:

| State                   | Meaning                                                                 | Canvas treatment                                       |
| ----------------------- | ----------------------------------------------------------------------- | ------------------------------------------------------ |
| Pending                 | An output identity and position are reserved; no completed resource yet | Type-aware placeholder with named stage and progress   |
| Completed deterministic | A local transformation finished without introducing semantic claims     | Normal resource with lineage                           |
| Generated               | Provider-generated artifact exists and records its exact inputs         | Normal editable resource with quiet generated marker   |
| Superseded              | A newer derivative replaces this result for the same purpose            | Preserved history, de-emphasized lineage               |
| Failed                  | The attempt ended without a valid output                                | Inspectable failed placeholder with retry/remove paths |

Editing a generated document creates a new current revision while preserving its derivation lineage. Resource use is immediate; epistemic acceptance exists only when content proposes durable Knowledge-claim mutations.

### Cross-modality product model

Knowledge Derivation is modality-independent at the product level and modality-specific at the engine boundary:

| Source knowledge                         | Intent                     | Durable result                            |
| ---------------------------------------- | -------------------------- | ----------------------------------------- |
| One or more text/media/document sources  | Summarize or synthesize    | Cited Markdown draft                      |
| Structured concepts or selected passages | Visualize                  | Explanatory image with generated identity |
| Image                                    | Remove background          | Transparent PNG                           |
| Video + current token-timed transcript   | Create subtitles           | Subtitle resource and/or burned-in video  |
| Text, images and storyboard knowledge    | Create video               | Generated video plus production metadata  |
| Document or transcript                   | Re-express as presentation | Presentation resource with source lineage |

The shared product contract stops at intent, source identity, result identity, lineage and progress. Prompt construction, media processing and document rendering remain separate action-owned modules.

### Canvas behavior

1. The user selects compatible resources and chooses **Create from selection**.
2. The action picker shows only actions whose input contract is satisfied.
3. Configuration explains the intended output, source scope and whether processing is local or provider-backed.
4. Starting creates a pending derivative beside the primary source or adjacent to the selection bounds.
5. Progress updates the pending node and Task Center from the same feature-owned job state.
6. Success replaces the pending node in place and creates structured derivation records plus `derives-from` relations atomically.
7. Failure retains an inspectable failed state without creating semantic lineage to a nonexistent output.
8. The user may inspect, edit, compare, rerun, save as Workflow or use the result as a new derivation input.

Placement uses the measured bounds of the full source selection. A single-source result appears beside that source; a multi-source result appears beside the selection bounds. Pending and completed forms share one node identity and geometry so success never jumps to a different position. Lineage edges are created only when the output transaction completes.

### Configuration behavior

- Immediate actions with one obvious outcome may start directly from the contextual menu.
- Provider-backed or meaningfully branching actions open a compact configuration dialog before creating the pending output.
- The first screen describes the outcome and source scope; provider/model controls are secondary advanced settings.
- Required infrastructure is checked before the job starts. Missing AI configuration, unavailable bundled runtime or stale source knowledge produces a specific disabled reason.
- Closing configuration makes no persistent changes. Cancelling a running job is owned by the action engine and leaves an inspectable failed/cancelled operational record according to that action contract.

### Lineage presentation

- `derives-from` points from the output resource to every source resource.
- One primary source may own default spatial placement, but it does not replace the complete input set.
- Dense multi-source lineage collapses visually into a quiet bundle while preserving individual relations in the Inspector.
- Selecting a derivative provides **Reveal sources**, **Compare with source**, **Rerun** and **Inspect derivation** when those actions are meaningful.
- Generated content never cites the derivative itself as evidence for claims inherited from its sources.

### Initial vertical slices

1. Image background removal: image → transparent PNG beside the source.
2. Subtitle production: video → transcript knowledge → editable subtitle resource → optional subtitled video copy.
3. Grounded document synthesis: selected sources → cited editable Markdown/Note → durable document.
4. Knowledge visualization: selected concepts and sources → explicitly labelled explanatory image or diagram.
5. Knowledge deliverables: selected semantic knowledge → editable outline/script → DOCX, PPTX or generated video, with optional image references.

## Open questions

- [ ] Audit each derivation action against the durable execution and Workflow contracts in `docs/rfcs/knowledge-work-system.md`: exact normalized parameters, named inputs/outputs, revision semantics and non-secret execution evidence / Spaces + Foundation / implementation evidence.
- [x] Generated documents remain directly usable artifacts; durable Knowledge-claim mutations use strict KnowledgeChangeSet validation, atomic commit and provenance presentation.
- [ ] Decide when multiple source relations collapse into one lineage bundle on dense canvases / Spaces design / large-canvas readability.
- [x] DOCX uses `docx-rs`, PDF uses the official Typst compiler/PDF exporter through a pinned embedding adapter, and PPTX uses `PptxGenJS` in a worker. All three remain derived delivery artifacts rather than editable knowledge sources.
- [x] Published Workflow definitions are immutable and versioned; saved bindings and durable Runs are
      included in the `.knovumspace` contract.
