# Workbench context remediation implementation plan

**Goal:** Keep the active editor, contextual panels, commands and canvas feedback in one consistent context.

**Architecture:** Spaces owns one mutually exclusive work surface. Existing session participants prepare content/layout/persistence before a surface or Space changes. Shell consumes capability projections and retains user layout intent; each editor retains its own selection and history.

**Tech stack:** Existing Vue 3, Pinia, Konva, Tiptap, Reka and Vitest. No new dependencies.

**Spec:** The accepted analysis in this session; authoritative contracts are `docs/spaces.md`, `docs/ui-system.md` and `docs/architecture.md`.

## Constraints

- Execute with one agent on the current branch. Stage only this task; no commits or packaging.
- Remove the independent library/document/video mode fields rather than maintaining aliases.
- Reuse the content/layout/persistence leave barrier; cancellation and failed saves preserve the original surface.
- Keep the canvas mounted for camera/selection restoration, but revoke interaction and overlays while inactive.
- Document citation source viewing remains available; canvas inspection and conversation do not leak into editors.
- Authored TS/Vue files stay within 180 effective lines and complexity 10.

## Task 1: Work surface and lifecycle

**Files:** `session/workbench-state.ts`, `session/content-state.ts`, `session/space-store.ts`, `workspace/*actions.ts`, `workspace/space-session-context.ts`, `session/actions/*space-actions.ts`, editor entry points and their tests.

**Interface:** `SpaceSurface` is canvas/library/document/video; document/video contain their node ID. `openDocumentEditor`, `openVideoEditor`, `returnToCanvas` and `showSpaceLibrary` complete the existing leave barrier before changing surface.

- [x] Add regression tests for library navigation while editing, cancelled/failed leave, same-Space return and editor replacement.
- [x] Observe failures, implement surface state and transitions, update all callers without legacy setters.
- [x] Verify navigation/session tests and typecheck.

## Task 2: Contextual UI and commands

**Files:** `SpaceView.vue`, workspace/outline presentation, `use-spaces-workspace.ts`, right-sidebar actions, `commands/*`, editor commands and source viewing.

**Interface:** Derived workbench capabilities govern canvas activity, outline and supported right-pane views. Document commands dispatch to the mounted document editor; canvas commands are registered only for the active canvas.

- [x] Test that document/video contexts cannot open stale canvas panels or dispatch canvas commands.
- [x] Implement capability consumers, inactive canvas suspension, document command routing and source return behavior.
- [x] Verify panel/command tests, including restoration of layout intent.

## Task 3: Canvas presence and inspection

**Files:** canvas presentation, canvas surface/viewport contracts, selection inspector and focused tests.

**Interface:** Authored Structure determines navigation; current-Space capture/drawing state additionally determines initial empty feedback. Hidden/offscreen objects and empty Sections are not initial empty documents. Relation selections receive explicit inspection.

- [x] Test Section-only, hidden-node and other-Space capture scenarios and empty viewport input.
- [x] Correct presence projection and viewport admission while preserving creation and Undo camera state.
- [x] Add relation inspection without falling through to Space health.

## Task 4: Contracts, verification and staging

- [x] Update owning documents; no duplicated architecture specification.
- [x] Run focused tests followed by `pnpm check:web`; inspect and resolve task-caused failures.
- [x] Review all changes and stage only task-owned code and document hunks.
- [x] Confirm HEAD is unchanged and no packaging command ran.

## Verification evidence

Final web-gate components were verified individually after the iterative `pnpm check:web` runs:

- Source, test, Node tooling and E2E TypeScript checks passed.
- Repository Oxc lint and Oxfmt checks passed; task files satisfy the configured size/complexity rules.
- Vitest: 356 files and 1406 tests passed. Focused surface, command, viewport, panel and video ownership regressions also passed.
- Desktop script unit tests: 29 passed; no desktop build or packaging was run.
- Dependency Cruiser: no violations across 2031 modules and 6651 dependencies.
- Knip passed; frontend duplication was 0.16%, below the 1% gate.
- Staged diff whitespace checks passed. Only task-owned files and document hunks were staged; unrelated work remained unstaged and HEAD was unchanged.

Validation used automated component/session/viewport tests. Native desktop interaction and packaged macOS/Windows acceptance were not exercised in this task.
