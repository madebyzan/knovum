# Space Editor Tabs Implementation Plan

> **For agentic workers:** Use executing-plans inline. User requires a single agent, implementation first, verification afterwards, `git add`, and no commits.

**Goal:** Preserve independent document and video work sessions inside each Space.

**Architecture:** Workspace owns typed work-item identities and activation. Document/video owners hold session state in separate Pinia stores. Existing ordered participants distinguish navigation from scoped close; all writes retain explicit Space identity.

**Tech Stack:** Vue 3 Composition API, Pinia, existing Reka Tabs and Tiptap, existing Tauri/Space mutation operations.

**Spec:** `docs/superpowers/specs/2026-10-11-space-editor-tabs-design.md`

## Global Constraints

- Single agent; preserve other working-tree edits; stage only this task's files and do not commit.
- Implement all changes before running tests/checks, per the user's explicit ordering.
- No dependencies, generic editor registry, compatibility state, migrations, windows or database tables.
- Files remain focused, at most 180 TS/Vue lines, complexity at most 10; existing ownership boundaries apply.
- Update authoritative Spaces, architecture and UI documents with the final implemented contract.

## Review Focus

- Cross-Space late writes must update their captured owner only.
- Closing multiple dirty editors and cancelling later must retain every session.
- Hidden editor commands, players and asynchronous preparation must not affect the active editor.
- Resource refresh/removal must preserve dirty drafts and expose conflicts rather than overwrite them.
- Render completion must not close, navigate or overwrite a newer editor session.

## Task 1: Work-item state and lifecycle

**Files:** `workspace/work-items.ts`, `workspace/editor-surface-actions.ts`, `workspace/session-participants.ts`, `workspace/space-transition.ts`, `workspace/space-session-context.ts`, `session/{session-state,workbench-state,space-store}.ts`.

**Interfaces:** `SpaceEditorTab { id, kind, spaceId, nodeId }`; work-item state holds ordered tabs and per-Space active tab; surface is readonly derived state. Participants carry an optional editor owner and switch participation; scoped preparation returns a deferred finalizer.

- [x] Replace writable surface with work-item state and a derived projection.
- [x] Implement idempotent open, activation, adjacent close selection and scoped close preparation.
- [x] Separate editor-preserving navigation from commit/close decisions.

## Task 2: Concrete editor sessions

**Files:** `editor/document-session-store.ts`, document persistence/composables; `video-editing/video-session-store.ts`, video history/actions/media composables; `workspace/editor-sessions.ts`.

**Interfaces:** Concrete session maps keyed by tab lifecycle ID; captured `spaceId`/node reference, dirty/error state and owner models. Workspace composition initializes, synchronizes and releases the two concrete session kinds.

- [x] Preserve document draft, Tiptap history, title and position through tab/Space activation.
- [x] Preserve video project/history/selection/playhead/zoom; gate hidden commands and media work.
- [x] Bind saves/render/proxy operations to explicit immutable Space and node identities.
- [x] Reconcile clean content changes; expose dirty conflicts and missing owners without dropping drafts.

## Task 3: Presentation, navigation and mutations

**Files:** `SpaceView.vue`, `presentation/workspace/SpaceWorkTabs.vue`, editor views; workspace navigation/close/archive modules; command composition; node deletion/transfer ownership; locales.

**Interfaces:** Work tab list takes readonly identities; editor views take identity and active visibility. Only active views register AppCommand ports. Scoped closes run decisions before removing owners.

- [x] Compose compact content tabs and retained owner views with explicit active visibility.
- [x] Restore selected work items across Space navigation and library visits.
- [x] Prepare all affected editor decisions before Space close/archive or node removal/transfer.
- [x] Remove render-driven editor close/navigation; retain Task Center cancellation and source-bound publication.
- [x] Update owning contracts and translated labels.

## Task 4: Verification and staging

**Files:** work-item/session lifecycle tests, document/video integration tests and affected existing tests; Rust node payload validation and the video owner's project validator.

- [x] Test duplicate open, adjacency, retained state, inactive command/media behavior, failed saves and cancelled batch close.
- [x] Test cross-Space writes, refresh/deletion conflicts and late render results against their original owners.
- [x] Run focused Vitest suites and relevant gates; record full-check limitations separately, with no repeated full-suite runs per the user's updated instruction.
- [x] Review diff/dependency direction/file budgets and stage this task's changes with `git add`; do not commit.

## Execution record

User approval authorizes immediate inline implementation without another plan approval pause.
Tests run after the implementation phase as requested. Final review is performed by this agent;
the single-agent instruction supersedes the skill's reviewer delegation. No commits are permitted.

Verification evidence: focused lifecycle, navigation, save, close, render, locale and
command-scope suites passed. Web lint, dependency boundaries, duplication and unused-code
checks passed; the Vite production build passed. The initial full web test run found
two failures, which were fixed and verified with focused tests. It is not recorded as
a passing full web check. The latest full-check rerun was stopped following the user's
instruction to avoid repeated expensive checks.

`pnpm check:rust` is blocked before compilation by the unrelated missing module
`src-tauri/crates/inference-native/build_support/tests.rs`. The task's Rust files pass
format checking. The targeted Rust test compilation was stopped to limit further
verification time; Rust execution and native macOS/Windows builds remain unverified.
Only task-owned changes are added to the index; existing unrelated staged changes are
preserved. No commit was made during implementation.
