"""Complete the renderer-to-native editor command boundary on the existing branch.

This script is development tooling, not an application compatibility path. Changes
are committed as one reviewed source edit by the branch-restricted workflow.
"""
from pathlib import Path

root = Path.cwd()
marker = root / "src/features/spaces/canvas/contract/editor-command.ts"
if marker.exists():
    raise SystemExit(0)


def write(path, text):
    destination = root / path
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(text.strip() + "\n")


def replace(path, before, after):
    destination = root / path
    source = destination.read_text()
    if before not in source:
        raise RuntimeError(f"Reviewed source precondition failed: {path}: {before[:100]}")
    destination.write_text(source.replace(before, after))


write("src/features/spaces/canvas/contract/editor-command.ts", r'''
import type { StructureCommand } from "../../organization/structure/commands/contract";
import type { CanvasNodeAppearance, CanvasNodeFrameUpdate, CanvasRelationOverrideColor, SpaceEditorDocument } from "./canvas-document";
import type { CanvasLayerMove } from "./canvas-editor-port";

export interface EditorFrameUpdate extends CanvasNodeFrameUpdate { typographyScale?: number }
export interface EditorTextMeasurement {
  itemId: string;
  width: number;
  height: number;
  typographyScale: number;
  contentRevision: string;
}
export type SpaceEditorCommand =
  | { kind: "structure"; command: StructureCommand; selection: readonly string[] }
  | { kind: "transform-frames" | "arrange-frames"; updates: readonly EditorFrameUpdate[] }
  | { kind: "set-appearance"; itemId: string; changes: Partial<{ [K in keyof CanvasNodeAppearance]: CanvasNodeAppearance[K] | null }> }
  | { kind: "set-relation-color"; relationId: string; color: CanvasRelationOverrideColor | null }
  | { kind: "move-layer"; ids: readonly string[]; mode: CanvasLayerMove }
  | { kind: "delete-selection"; ids: readonly string[]; relationIds: readonly string[] }
  | { kind: "measure-text"; measurements: readonly EditorTextMeasurement[] };

export interface ExecuteSpaceEditRequest {
  baseRevision: number;
  mutationId: string;
  command: SpaceEditorCommand;
}
export interface ExecuteSpaceEditDraft {
  baseDocument: SpaceEditorDocument;
  mutationId: string;
  command: SpaceEditorCommand;
}
export interface ReplaySpaceEditRequest {
  baseRevision: number;
  mutationId: string;
  receiptId: string;
  direction: "undo" | "redo";
}
export type ReplaySpaceEditDraft = Omit<ReplaySpaceEditRequest, "baseRevision">;
''')
write("src/features/spaces/canvas/contract/editor-result.ts", r'''
import { z } from "zod";

import { SPACE_RELATION_KINDS, SPACE_RELATION_STATUSES } from "../../model/relations/relation";
import { spaceNodePayloadSchema } from "../../model/nodes/schema/node-variants";
import { spaceEditorDocumentSchema } from "./canvas-document";

const identity = z.string().min(1).max(128);
const revision = z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER);
const relationSchema = z.object({
  id: identity, spaceId: identity, sourceId: identity, targetId: identity,
  kind: z.enum(SPACE_RELATION_KINDS), rationale: z.string(), status: z.enum(SPACE_RELATION_STATUSES),
  statusBeforeInvalidation: z.enum(["active", "contested"]).nullable().optional(),
  confidence: z.number().finite().min(0).max(1).optional(),
  actorKind: z.enum(["human", "assistant", "agent", "system"]), actorId: identity.optional(),
  createdAt: z.number().int(), updatedAt: z.number().int(),
}).strict();
export const editorContentSchema = z.object({
  nodes: z.array(z.object({
    id: identity, initialX: z.number().finite(), initialY: z.number().finite(), data: spaceNodePayloadSchema,
  }).strict()),
  relations: z.array(relationSchema),
}).strict();
export const spaceEditResultSchema = z.object({
  document: spaceEditorDocumentSchema, revision, appliedAtRevision: revision,
  mutationId: identity, selection: z.array(identity), historyReceiptId: identity.nullable(),
  content: editorContentSchema.nullable(),
}).strict().refine((result) => result.appliedAtRevision <= result.revision, "A receipt cannot be ahead of the current editor state");
export const editorWorkspaceSchema = z.object({
  document: spaceEditorDocumentSchema, revision, content: editorContentSchema,
}).strict();
export type SpaceEditResult = z.infer<typeof spaceEditResultSchema>;
export type EditorWorkspace = z.infer<typeof editorWorkspaceSchema>;
export type EditorWorkspaceContent = z.infer<typeof editorContentSchema>;
''')
write("src/features/spaces/canvas/persistence/editor-command-repository.ts", r'''
import { invoke } from "@tauri-apps/api/core";

import type { ExecuteSpaceEditRequest, ReplaySpaceEditRequest } from "../contract/editor-command";
import { editorWorkspaceSchema, spaceEditResultSchema, type SpaceEditResult } from "../contract/editor-result";

export class EditorOutcomeUnknownError extends Error {
  constructor(readonly mutationId: string, cause: unknown) {
    super(`The outcome of edit ${mutationId} could not be confirmed. Do not repeat the operation with a new identity.`, { cause });
    this.name = "EditorOutcomeUnknownError";
  }
}

export async function loadEditorWorkspace(spaceId: string) {
  return editorWorkspaceSchema.parse(await invoke<unknown>("plugin:spaces|load_editor_workspace", { spaceId }));
}
export function executeSpaceEdit(spaceId: string, request: ExecuteSpaceEditRequest) {
  return submit(spaceId, request.mutationId, () => invoke<unknown>("plugin:spaces|execute_space_edit", {
    request: { spaceId, mutationId: request.mutationId, baseRevision: request.baseRevision, command: request.command },
  }));
}
export function replaySpaceEdit(spaceId: string, request: ReplaySpaceEditRequest) {
  return submit(spaceId, request.mutationId, () => invoke<unknown>("plugin:spaces|replay_space_edit", {
    request: { spaceId, mutationId: request.mutationId, baseRevision: request.baseRevision, receiptId: request.receiptId, direction: request.direction },
  }));
}
async function submit(spaceId: string, mutationId: string, operation: () => Promise<unknown>): Promise<SpaceEditResult> {
  let received = false;
  try {
    const result = await operation();
    received = true;
    return spaceEditResultSchema.parse(result);
  } catch (cause) {
    let receipt: unknown;
    try {
      receipt = await invoke<unknown>("plugin:spaces|query_space_edit_receipt", { request: { spaceId, mutationId } });
    } catch (queryError) {
      throw new EditorOutcomeUnknownError(mutationId, new AggregateError([cause, queryError]));
    }
    if (receipt !== null) {
      try { return spaceEditResultSchema.parse(receipt); }
      catch (error) { throw new EditorOutcomeUnknownError(mutationId, error); }
    }
    // Rust's rejected Result is a string. A transport or decode failure is not proof of rollback.
    if (!received && typeof cause === "string") throw new Error(cause);
    throw new EditorOutcomeUnknownError(mutationId, cause);
  }
}
''')
write("src/features/spaces/session/actions/editor-command-actions.ts", r'''
import { createSpaceNode } from "../../canvas/node";
import type { ExecuteSpaceEditRequest, ReplaySpaceEditRequest } from "../../canvas/contract/editor-command";
import type { SpaceEditResult } from "../../canvas/contract/editor-result";
import { executeSpaceEdit, replaySpaceEdit } from "../../canvas/persistence/editor-command-repository";
import { scheduleKnowledgeIndex } from "../../knowledge/indexing/index-scheduler";
import type { SpaceState } from "../session-state";

export function createEditorCommandActions(state: SpaceState) {
  async function submit(operation: (spaceId: string) => Promise<SpaceEditResult>) {
    const spaceId = state.activeSpaceId.value;
    if (!spaceId) throw new Error("An active Space is required for editing");
    const result = await operation(spaceId);
    if (state.activeSpaceId.value === spaceId && result.content) {
      const previous = new Map(state.nodes.value.map((node) => [node.id, node]));
      state.nodes.value = result.content.nodes.map((record) => {
        const node = createSpaceNode(record.data, { x: record.initialX, y: record.initialY }, record.id);
        const old = previous.get(record.id);
        return old?.hidden === undefined ? node : { ...node, hidden: old.hidden };
      });
      state.relations.value = result.content.relations;
      const added = result.content.nodes.filter((node) => !previous.has(node.id)).map((node) => node.id);
      if (added.length) scheduleKnowledgeIndex(spaceId, added);
    }
    return result;
  }
  return {
    executeEditorCommand: (request: ExecuteSpaceEditRequest) => submit((spaceId) => executeSpaceEdit(spaceId, request)),
    replayEditorCommand: (request: ReplaySpaceEditRequest) => submit((spaceId) => replaySpaceEdit(spaceId, request)),
  };
}
''')
replace("src/features/spaces/session/actions/create-space-actions.ts", "import { createCanvasGestureActions }", "import { createEditorCommandActions } from \"./editor-command-actions\";\nimport { createCanvasGestureActions }")
replace("src/features/spaces/session/actions/create-space-actions.ts", "...createCanvasGestureActions(state),", "...createCanvasGestureActions(state),\n    ...createEditorCommandActions(state),")
replace("src/features/spaces/canvas/contract/canvas-editor.ts", "import type { SpaceCanvasNode }", "import type { ExecuteSpaceEditRequest, ExecuteSpaceEditDraft, ReplaySpaceEditRequest, ReplaySpaceEditDraft } from \"./editor-command\";\nimport type { SpaceEditResult } from \"./editor-result\";\nimport type { SpaceCanvasNode }")
replace("src/features/spaces/canvas/contract/canvas-editor.ts", "export interface CanvasHostEvents {", "export interface CanvasHostEvents {\n  onExecuteSpaceEdit(request: ExecuteSpaceEditRequest): Promise<SpaceEditResult>;\n  onReplaySpaceEdit(request: ReplaySpaceEditRequest): Promise<SpaceEditResult>;")
replace("src/features/spaces/canvas/contract/canvas-editor.ts", '"onCommitGesture" | "onCutSelection" | "onDeleteSelection"', '"onCommitGesture" | "onCutSelection" | "onDeleteSelection" | "onExecuteSpaceEdit" | "onReplaySpaceEdit"')
replace("src/features/spaces/canvas/contract/canvas-editor.ts", '  onCommitGesture(request: CanvasGestureCommitDraft)', '  onExecuteSpaceEdit(request: ExecuteSpaceEditDraft): Promise<SpaceEditResult>;\n  onReplaySpaceEdit(request: ReplaySpaceEditDraft): Promise<SpaceEditResult>;\n  onCommitGesture(request: CanvasGestureCommitDraft)')
replace("src/features/spaces/presentation/canvas/SpaceCanvas.vue", "const canvasEvents: CanvasHostEvents = {", "const canvasEvents: CanvasHostEvents = {\n  onExecuteSpaceEdit: store.executeEditorCommand,\n  onReplaySpaceEdit: store.replayEditorCommand,")
replace("src/features/spaces/canvas/host/CanvasSurfaceHost.vue", '<script setup lang="ts">', '<script setup lang="ts">\nimport { isEqual } from "es-toolkit";')
replace("src/features/spaces/canvas/host/CanvasSurfaceHost.vue", '  ...props.events,', '''  ...props.events,
  onExecuteSpaceEdit(request) {
    return session.commitTransaction((baseRevision) => {
      const current = session.document.value!;
      if (request.command.kind !== "measure-text" && (
        !isEqual(request.baseDocument.structure, current.structure) ||
        !isEqual(request.baseDocument.layout, current.layout) ||
        !isEqual(request.baseDocument.relationStyles, current.relationStyles)
      )) throw new Error("The Space changed during this edit. Please retry the operation.");
      return props.events.onExecuteSpaceEdit({ mutationId: request.mutationId, command: request.command, baseRevision });
    });
  },
  onReplaySpaceEdit(request) {
    return session.commitTransaction((baseRevision) => props.events.onReplaySpaceEdit({ ...request, baseRevision }));
  },''')
write("src/features/spaces/canvas/konva/document/canvas-document-model.ts", r'''
import { shallowRef, type ShallowRef } from "vue";

import type { StructureCommand } from "../../../organization/structure/commands/contract";
import type { CanvasNodeAppearance, CanvasRelationColor, SpaceEditorDocument } from "../../contract/canvas-document";
import type { CanvasHistoryEntry, CanvasLayerMove, CanvasViewport } from "../../contract/canvas-editor";
import type { ExecuteSpaceEditDraft, ReplaySpaceEditDraft, SpaceEditorCommand, EditorTextMeasurement } from "../../contract/editor-command";
import type { SpaceEditResult } from "../../contract/editor-result";
import { projectEditorLayouts } from "../../document/editor-layout-projection";
import { sameCanvasViewport } from "./canvas-document-mutations";

interface CanvasDocumentModelEvents {
  executeCommand(request: ExecuteSpaceEditDraft): Promise<SpaceEditResult>;
  replayCommand(request: ReplaySpaceEditDraft): Promise<SpaceEditResult>;
  viewportChanged(viewport: CanvasViewport): void;
  historyEntry(entry: CanvasHistoryEntry): void;
  reportError(error: unknown): void;
}
/** A committed renderer projection. Rust owns edit execution and reversible receipts. */
export class CanvasDocumentModel {
  private readonly current: ShallowRef<SpaceEditorDocument>;
  private readonly sceneCurrent: ShallowRef<SpaceEditorDocument>;
  private readonly observers = new Set<(document: SpaceEditorDocument) => void>();
  private readonly pending = new Set<Promise<unknown>>();
  private tail = Promise.resolve();
  constructor(document: SpaceEditorDocument, private readonly events: CanvasDocumentModelEvents) {
    this.current = shallowRef(document);
    this.sceneCurrent = shallowRef(document);
  }
  get document() { return this.current.value; }
  get documentRef(): Readonly<ShallowRef<SpaceEditorDocument>> { return this.current; }
  get sceneDocumentRef(): Readonly<ShallowRef<SpaceEditorDocument>> { return this.sceneCurrent; }
  replace(document: SpaceEditorDocument) { this.assign(document); }
  applyLocalDocument(document: SpaceEditorDocument) { this.assign(document); }
  layout(id: string) { return projectEditorLayouts(this.document).find((layout) => layout.nodeId === id); }
  async flush() { await Promise.all([...this.pending]); }

  executeNative(command: SpaceEditorCommand, baseDocument?: SpaceEditorDocument, record = true) {
    return this.enqueue(async () => {
      const result = await this.events.executeCommand({
        command, mutationId: crypto.randomUUID(), baseDocument: baseDocument ?? this.document,
      });
      this.adopt(result);
      const receiptId = result.historyReceiptId;
      if (record && receiptId) this.events.historyEntry({
        undo: () => this.replay(receiptId, "undo"),
        redo: () => this.replay(receiptId, "redo"),
      });
      return result;
    });
  }
  async execute(command: StructureCommand, selection: readonly string[] = [], record = true) {
    return (await this.executeNative({ kind: "structure", command, selection }, undefined, record)).selection;
  }
  async moveNodes(ids: ReadonlySet<string>, mode: CanvasLayerMove) {
    await this.executeNative({ kind: "move-layer", ids: [...ids], mode });
  }
  async setRelationColor(relationId: string, color: CanvasRelationColor) {
    await this.executeNative({ kind: "set-relation-color", relationId, color: color === "neutral" ? null : color });
  }
  async setNodeAppearance(itemId: string, patch: Partial<CanvasNodeAppearance>) {
    const changes = Object.fromEntries(Object.entries(patch).map(([field, value]) => [field, value ?? null]));
    await this.executeNative({ kind: "set-appearance", itemId, changes });
  }
  fitNodeHeights(measurements: readonly EditorTextMeasurement[]) {
    if (!measurements.length) return Promise.resolve();
    return this.executeNative({ kind: "measure-text", measurements }, undefined, false)
      .then(() => undefined).catch(this.events.reportError);
  }
  setViewport(viewport: CanvasViewport) {
    if (sameCanvasViewport(this.document.viewport, viewport)) return;
    this.assign({ ...this.document, viewport: { ...viewport } });
    this.events.viewportChanged(viewport);
  }
  observe(observer: (document: SpaceEditorDocument) => void) {
    this.observers.add(observer); observer(this.document);
    return () => { this.observers.delete(observer); };
  }
  private replay(receiptId: string, direction: "undo" | "redo") {
    return this.enqueue(async () => {
      this.adopt(await this.events.replayCommand({ receiptId, direction, mutationId: crypto.randomUUID() }));
    });
  }
  private adopt(result: SpaceEditResult) {
    this.assign({ ...result.document, viewport: this.document.viewport });
  }
  private enqueue<T>(operation: () => Promise<T>): Promise<T> {
    const pending = this.tail.then(operation);
    this.pending.add(pending);
    this.tail = pending.then(() => undefined, () => undefined);
    void pending.then(() => this.pending.delete(pending), () => this.pending.delete(pending));
    return pending;
  }
  private assign(document: SpaceEditorDocument) {
    const previous = this.document;
    this.current.value = document;
    if (previous.structure !== document.structure || previous.layout !== document.layout || previous.relationStyles !== document.relationStyles) this.sceneCurrent.value = document;
    for (const observer of this.observers) observer(document);
  }
}
''')
replace("src/features/spaces/canvas/konva/use-konva-canvas-runtime.ts", '''    commitDocument: async (document, before) => {
      await props.events.onPrepareStructureEdit();
      return props.events.onCommitGesture({ document, baseDocument: before });
    },''', '''    executeCommand: async (request) => {
      await props.events.onPrepareStructureEdit();
      return props.events.onExecuteSpaceEdit(request);
    },
    replayCommand: props.events.onReplaySpaceEdit,''')
# Preserve the existing preview controller; replace only commit planning with native typed intents.
path = "src/features/spaces/canvas/konva/interaction/canvas-gesture-session.ts"
source = (root / path).read_text()
source = source.replace('import { planStructureCommand } from "../../../organization/structure/commands/plan";\n', '')
source = source.replace('import { replayEditorTransition } from "../../document/editor-history-transition";\n', '')
source = source.replace('import { applyCanvasGestureUpdates } from "../document/canvas-document-mutations";\n', '')
source = source.replace('        let after: SpaceEditorDocument;\n', '')
source = source.replace('          after = { ...before, ...planStructureCommand(before, command).after };\n        } else after = applyCanvasGestureUpdates(before, updates);\n        await options.document.mutate((current) => replayEditorTransition(current, before, after));', '''          await options.document.executeNative({ kind: "structure", command, selection: selected }, before);
        } else {
          await options.document.executeNative({ kind: "transform-frames", updates: updates.map((update) => ({
            nodeId: update.nodeId, frame: update.frame,
            ...(update.sizeAuthority ? { sizeAuthority: update.sizeAuthority } : {}),
            ...(update.appearance?.typographyScale !== undefined ? { typographyScale: update.appearance.typographyScale } : {}),
          })) }, before);
        }''')
start = source.index('    const layouts = new Map(projectEditorLayouts(before)', source.index('  async function runFrameMutation('))
end = source.index('\n  function cancelActiveGesture()', start)
source = source[:start] + '''    await tracked(async () => {
      await options.document.executeNative({ kind: "arrange-frames", updates }, before);
    });
  }
''' + source[end:]
if '.mutate(' in source or 'planStructureCommand' in source:
    raise RuntimeError("Gesture migration left a document-mutation path")
write(path, source)
write("src/features/spaces/canvas/konva/selection/canvas-selection-deletion.ts", r'''
import type { KonvaEditorPortOptions } from "../canvas-editor-port-options";

export function createCanvasSelectionDeletion(options: KonvaEditorPortOptions) {
  return async (ids: readonly string[], relationIds: readonly string[], cut = false) => {
    if ((!ids.length && !relationIds.length) || (cut && !ids.length)) return;
    const before = options.model.document;
    await options.onHistoryTransaction(async () => {
      if (cut) await options.onCopyNodes({ document: before, itemIds: ids });
      await options.model.executeNative({ kind: "delete-selection", ids, relationIds }, before);
    });
    options.setSelection([]);
  };
}
''')
# Text layout measurements share the existing canonical revision implementation.
path = "src/features/spaces/knowledge/model/revision.ts"
source = (root / path).read_text()
source += '''\n/** Captured render data precondition for asynchronous layout measurements. */
export function renderRevision(data: SpaceNodePayload): string {
  let hash = 0xcbf29ce484222325n;
  for (const byte of encoder.encode(stableJson(data))) {
    hash ^= BigInt(byte);
    hash = BigInt.asUintN(64, hash * 0x100000001b3n);
  }
  return `render1:${hash.toString(16).padStart(16, "0")}`;
}
'''
write(path, source)
path = "src/features/spaces/canvas/konva/node-content-height-batch.ts"
source = (root / path).read_text()
source = 'import { renderRevision } from "../../knowledge/model/revision";\nimport type { EditorTextMeasurement } from "../contract/editor-command";\n' + source
source = source.replace('const fitted = new Map<string, number>();', 'const fitted: EditorTextMeasurement[] = [];')
source = source.replace('''      fitted.set(
        id,
        Math.max(spaceNodeMinimumContentHeight(node.data, measurement.typographyScale), measurement.height),
      );''', '''      fitted.push({ itemId: id, width: measurement.width, typographyScale: measurement.typographyScale,
        height: Math.max(spaceNodeMinimumContentHeight(node.data, measurement.typographyScale), measurement.height),
        contentRevision: renderRevision(node.data) });''')
write(path, source)
print("Applied native editor command bindings and receipt-backed history")
