"""Reviewed renderer and geometry cutover. Temporary PR-14 execution recipe."""
from pathlib import Path
ROOT=Path('.')
files={}
def read(name):
    if name in files: return files[name]
    return (ROOT/name).read_text()
def write(name,text): files[name]=text.strip()+'\n'
def replace(name,old,new):
    text=read(name)
    if old not in text: raise RuntimeError(f'Expected source not found in {name}: {old[:90]}')
    write(name,text.replace(old,new))

write('src/features/spaces/canvas/document/editor-frame-edit.ts', '''
import type { CanvasNodeFrameUpdate, SpaceEditorDocument } from "../contract/canvas-document";
import { parseSpaceEditorDocument } from "../contract/canvas-document";
import { indexStructureSnapshot } from "../../organization/structure/structure-context";
import { projectStructure, structureBoundsInParent } from "../../organization/structure/structure-projection";
import { assertStructureEditable, structureOperationRoots } from "../../organization/structure/structure-selection";
import { planStructureCommand } from "../../organization/structure/commands/plan";
import { composeStructureTransforms, invertStructureTransform, normalizeStructureRotation, transformStructurePoint, IDENTITY_STRUCTURE_TRANSFORM } from "./structure-transform";

/** Transform gestures are converted once into local records, never into inferred membership. */
export function applyEditorFrames(document: SpaceEditorDocument, updates: readonly CanvasNodeFrameUpdate[]): SpaceEditorDocument {
  const context = indexStructureSnapshot(document);
  const projection = projectStructure(context);
  const byId = new Map(updates.map((update) => [update.nodeId, update]));
  const roots = structureOperationRoots(context.tree, [...byId.keys()]);
  assertStructureEditable(context.tree, roots.map((item) => item.id));
  let after = document;
  for (const item of roots) {
    const update = byId.get(item.id)!;
    const frame = update.frame;
    if (item.kind === "section") {
      if (frame.rotation !== 0) throw new Error("Sections cannot rotate");
      after = { ...after, ...planStructureCommand(after, { kind: "resize-section", id: item.id, bounds: { x: frame.x, y: frame.y, width: frame.width, height: frame.height } }).after };
    } else if (item.kind === "group") {
      const bounds = structureBoundsInParent(context, projection, [item.id], item.id)!;
      const scale = frame.width / bounds.width;
      if (Math.abs(scale - frame.height / bounds.height) > 1e-6 * Math.max(1, scale)) throw new Error("Groups support uniform scaling only");
      const transform = { x: 0, y: 0, rotation: normalizeStructureRotation(frame.rotation), scale };
      const offset = transformStructurePoint(transform, bounds);
      after = { ...after, ...planStructureCommand(after, { kind: "transform-group", id: item.id, worldTransform: { ...transform, x: frame.x - offset.x, y: frame.y - offset.y } }).after };
    } else {
      const world = projection.transforms.get(item.id)!;
      const parent = item.parentId === null ? IDENTITY_STRUCTURE_TRANSFORM : projection.transforms.get(item.parentId)!;
      const transform = composeStructureTransforms(invertStructureTransform(parent), { ...world, x: frame.x, y: frame.y, rotation: normalizeStructureRotation(frame.rotation) });
      after = { ...after, layout: { version: 1, items: after.layout.items.map((layout) => layout.kind === "content" && layout.itemId === item.id
        ? { ...layout, transform, size: { width: frame.width / world.scale, height: frame.height / world.scale }, sizeAuthority: update.sizeAuthority ?? layout.sizeAuthority } : layout) } };
    }
  }
  return parseSpaceEditorDocument(after);
}
''')
write('src/features/spaces/canvas/document/editor-history-transition.ts', '''
import { isEqual } from "es-toolkit";
import { applyStructureChanges, diffStructureSnapshots } from "../../organization/structure/commands/changes";
import { parseSpaceEditorDocument, type SpaceEditorDocument } from "../contract/canvas-document";

/** Replays exact committed records, preserving the live camera and disjoint later edits. */
export function replayEditorTransition(current: SpaceEditorDocument, before: SpaceEditorDocument, after: SpaceEditorDocument) {
  const snapshot = applyStructureChanges(current, diffStructureSnapshots(before, after));
  const previous = new Map(before.relationStyles?.map((style) => [style.relationId, style]));
  const target = new Map(after.relationStyles?.map((style) => [style.relationId, style]));
  const styles = new Map(current.relationStyles?.map((style) => [style.relationId, style]));
  for (const id of new Set([...previous.keys(), ...target.keys()])) {
    if (isEqual(previous.get(id), target.get(id))) continue;
    if (!isEqual(styles.get(id), previous.get(id))) throw new Error("Relation appearance changed after this edit");
    const next = target.get(id);
    if (next) styles.set(id, structuredClone(next)); else styles.delete(id);
  }
  return parseSpaceEditorDocument({ ...current, ...snapshot, relationStyles: [...styles.values()] });
}
''')
write('src/features/spaces/canvas/document/editor-appearance.ts', '''
import { indexStructureSnapshot } from "../../organization/structure/structure-context";
import { assertStructureEditable } from "../../organization/structure/structure-selection";
import { parseSpaceEditorDocument, type CanvasNodeAppearance, type CanvasRelationColor, type SpaceEditorDocument } from "../contract/canvas-document";

export function setEditorAppearance(document: SpaceEditorDocument, id: string, patch: Partial<CanvasNodeAppearance>) {
  assertStructureEditable(indexStructureSnapshot(document).tree, [id]);
  const items = document.layout.items.map((layout) => {
    if (layout.itemId !== id) return layout;
    if (layout.kind === "group") throw new Error("Groups have no independent content appearance");
    const appearance = { ...layout.appearance, ...patch };
    if (patch.fill) delete appearance.customFill;
    if (patch.customFill) delete appearance.fill;
    for (const key of Object.keys(appearance) as (keyof CanvasNodeAppearance)[]) if (appearance[key] === undefined) delete appearance[key];
    return { ...layout, appearance };
  });
  return parseSpaceEditorDocument({ ...document, layout: { version: 1, items } });
}
export function setEditorRelationColor(document: SpaceEditorDocument, relationId: string, color: CanvasRelationColor): SpaceEditorDocument {
  const retained = document.relationStyles?.filter((style) => style.relationId !== relationId) ?? [];
  return { ...document, relationStyles: color === "neutral" ? retained : [...retained, { relationId, color }] };
}
''')
write('src/features/spaces/organization/structure/commands/layer-order.ts', '''
import type { CanvasLayerMove } from "../../../canvas/contract/canvas-editor-port";
import type { SpaceEditorDocument } from "../../../canvas/contract/canvas-document";
import { indexStructure } from "../../../model/structure/structure-index";
import { assertStructureEditable, structureOperationRoots } from "../structure-selection";

export function reorderStructureItems(document: SpaceEditorDocument, ids: readonly string[], mode: CanvasLayerMove): SpaceEditorDocument {
  const tree = indexStructure(document.structure);
  const roots = structureOperationRoots(tree, ids);
  assertStructureEditable(tree, roots.map((item) => item.id));
  const selected = new Set(roots.map((item) => item.id));
  const orders = new Map<string, number>();
  for (const parentId of new Set(roots.map((item) => item.parentId))) {
    const siblings = (tree.children.get(parentId) ?? []).map((item) => item.id);
    reorder(siblings, selected, mode).forEach((id, order) => orders.set(id, order));
  }
  return { ...document, structure: { version: 1, items: document.structure.items.map((item) => orders.has(item.id) ? { ...item, order: orders.get(item.id)! } : item) } };
}
function reorder(ids: string[], selected: ReadonlySet<string>, mode: CanvasLayerMove) {
  const chosen = ids.filter((id) => selected.has(id));
  const others = ids.filter((id) => !selected.has(id));
  if (mode === "front") return [...others, ...chosen];
  if (mode === "back") return [...chosen, ...others];
  const direction = mode === "forward" ? -1 : 1;
  for (let index = direction < 0 ? ids.length - 2 : 1; index >= 0 && index < ids.length; index += direction) {
    const other = index - direction;
    if (selected.has(ids[index]!) && !selected.has(ids[other]!)) [ids[index], ids[other]] = [ids[other]!, ids[index]!];
  }
  return ids;
}
''')
write('src/features/spaces/canvas/konva/document/canvas-document-mutations.ts', '''
import type { CanvasNodeFrame, CanvasNodeFrameUpdate, SpaceEditorDocument } from "../../contract/canvas-document";
import { applyEditorFrames } from "../../document/editor-frame-edit";
import { projectEditorLayouts } from "../../document/editor-layout-projection";
import type { CanvasGestureUpdate } from "../interaction/canvas-gesture-update";
export function applyCanvasFrames(document: SpaceEditorDocument, updates: readonly CanvasNodeFrameUpdate[]) { return applyEditorFrames(document, updates); }
export function applyCanvasGestureUpdates(document: SpaceEditorDocument, updates: readonly CanvasGestureUpdate[]) {
  const result = applyEditorFrames(document, updates);
  const byId = new Map(updates.map((update) => [update.nodeId, update]));
  return { ...result, layout: { version: 1 as const, items: result.layout.items.map((layout) => {
    const update = byId.get(layout.itemId);
    if (!update || !("appearance" in update) || layout.kind === "group") return layout;
    const { appearance: _appearance, ...rest } = layout;
    return update.appearance ? { ...rest, appearance: update.appearance } : rest;
  }) } };
}
export function captureCanvasGestureLayouts(document: SpaceEditorDocument, updates: readonly CanvasGestureUpdate[]) {
  const selected = new Set(updates.map((update) => update.nodeId));
  return projectEditorLayouts(document).filter((layout) => selected.has(layout.nodeId));
}
export function sameCanvasFrame(left: CanvasNodeFrame, right: CanvasNodeFrame) { return left.x === right.x && left.y === right.y && left.width === right.width && left.height === right.height && left.rotation === right.rotation; }
export function sameCanvasViewport(left: SpaceEditorDocument["viewport"], right: SpaceEditorDocument["viewport"]) { return left.x === right.x && left.y === right.y && left.zoom === right.zoom; }
''')
write('src/features/spaces/canvas/konva/document/canvas-document-model.ts', '''
import { isEqual } from "es-toolkit";
import { shallowRef, type ShallowRef } from "vue";
import type { CanvasNodeAppearance, CanvasRelationColor, SpaceEditorDocument } from "../../contract/canvas-document";
import { parseSpaceEditorDocument } from "../../contract/canvas-document";
import type { CanvasGestureCommitResult, CanvasHistoryEntry, CanvasLayerMove, CanvasViewport } from "../../contract/canvas-editor";
import type { StructureCommand } from "../../../organization/structure/commands/contract";
import { planStructureCommand } from "../../../organization/structure/commands/plan";
import { reorderStructureItems } from "../../../organization/structure/commands/layer-order";
import { setEditorAppearance, setEditorRelationColor } from "../../document/editor-appearance";
import { replayEditorTransition } from "../../document/editor-history-transition";
import { projectEditorLayouts } from "../../document/editor-layout-projection";
import { sameCanvasViewport } from "./canvas-document-mutations";

interface CanvasDocumentModelEvents {
  commitDocument(document: SpaceEditorDocument, before: SpaceEditorDocument): Promise<CanvasGestureCommitResult>;
  viewportChanged(viewport: CanvasViewport): void;
  historyEntry(entry: CanvasHistoryEntry): void;
  reportError(error: unknown): void;
}
/** Committed application state only. Gesture previews and the camera are independent. */
export class CanvasDocumentModel {
  private readonly current: ShallowRef<SpaceEditorDocument>;
  private readonly sceneCurrent: ShallowRef<SpaceEditorDocument>;
  private readonly observers = new Set<(document: SpaceEditorDocument) => void>();
  private readonly pending = new Set<Promise<unknown>>();
  private tail = Promise.resolve();
  constructor(document: SpaceEditorDocument, private readonly events: CanvasDocumentModelEvents) {
    this.current = shallowRef(document); this.sceneCurrent = shallowRef(document);
  }
  get document() { return this.current.value; }
  get documentRef(): Readonly<ShallowRef<SpaceEditorDocument>> { return this.current; }
  get sceneDocumentRef(): Readonly<ShallowRef<SpaceEditorDocument>> { return this.sceneCurrent; }
  replace(document: SpaceEditorDocument) { this.assign(document); }
  applyLocalDocument(document: SpaceEditorDocument) { this.assign(document); }
  layout(id: string) { return projectEditorLayouts(this.document).find((layout) => layout.nodeId === id); }
  async flush() { await Promise.all([...this.pending]); }
  mutate(calculate: (document: SpaceEditorDocument) => SpaceEditorDocument, record = true): Promise<void> {
    const operation = this.tail.then(async () => {
      const before = this.document;
      const after = parseSpaceEditorDocument(calculate(before));
      if (isEqual(before.structure, after.structure) && isEqual(before.layout, after.layout) && isEqual(before.relationStyles, after.relationStyles)) return;
      const result = await this.events.commitDocument(after, before);
      this.assign({ ...result.document, viewport: this.document.viewport });
      if (record) this.events.historyEntry({
        undo: () => this.mutate((current) => replayEditorTransition(current, after, before), false),
        redo: () => this.mutate((current) => replayEditorTransition(current, before, after), false),
      });
    });
    this.pending.add(operation);
    this.tail = operation.then(() => undefined, () => undefined);
    void operation.then(() => this.pending.delete(operation), () => this.pending.delete(operation));
    return operation;
  }
  async execute(command: StructureCommand, selection: readonly string[] = []) {
    let selected = selection;
    await this.mutate((document) => {
      const plan = planStructureCommand(document, command, selection);
      if (plan.effects.removedContentNodeIds.length || plan.effects.contentClones.length) throw new Error("Content lifecycle changes require a Space content transaction");
      selected = plan.selection;
      return { ...document, ...plan.after };
    });
    return selected;
  }
  moveNodes(ids: ReadonlySet<string>, mode: CanvasLayerMove) { return this.mutate((document) => reorderStructureItems(document, [...ids], mode)); }
  setRelationColor(id: string, color: CanvasRelationColor) { return this.mutate((document) => setEditorRelationColor(document, id, color)); }
  setNodeAppearance(id: string, patch: Partial<CanvasNodeAppearance>) { return this.mutate((document) => setEditorAppearance(document, id, patch)); }
  fitNodeHeights(heights: ReadonlyMap<string, number>) {
    return this.mutate((document) => ({ ...document, layout: { version: 1, items: document.layout.items.map((layout) => {
      const height = heights.get(layout.itemId);
      return layout.kind === "content" && height !== undefined ? { ...layout, size: { ...layout.size, height: Math.ceil(height) } } : layout;
    }) } }), false).catch(this.events.reportError);
  }
  setViewport(viewport: CanvasViewport) {
    if (sameCanvasViewport(this.document.viewport, viewport)) return;
    this.assign({ ...this.document, viewport: { ...viewport } }); this.events.viewportChanged(viewport);
  }
  observe(observer: (document: SpaceEditorDocument) => void) { this.observers.add(observer); observer(this.document); return () => { this.observers.delete(observer); }; }
  private assign(document: SpaceEditorDocument) {
    const previous = this.document; this.current.value = document;
    if (previous.structure !== document.structure || previous.layout !== document.layout || previous.relationStyles !== document.relationStyles) this.sceneCurrent.value = document;
    for (const observer of this.observers) observer(document);
  }
}
''')
write('src/features/spaces/canvas/host/canvas-document-rebase.ts', '''
import { isEqual } from "es-toolkit";
import { parseSpaceEditorDocument, type SpaceEditorDocument } from "../contract/canvas-document";
/** Merge disjoint edits; shared-record conflicts fail instead of overwriting topology. */
export function rebaseCanvasDocument(base: SpaceEditorDocument, committed: SpaceEditorDocument, pending: SpaceEditorDocument): SpaceEditorDocument {
  return parseSpaceEditorDocument({ ...committed,
    viewport: isEqual(base.viewport, pending.viewport) ? committed.viewport : pending.viewport,
    structure: { version: 1, items: merge(base.structure.items, committed.structure.items, pending.structure.items, (item) => item.id) },
    layout: { version: 1, items: merge(base.layout.items, committed.layout.items, pending.layout.items, (item) => item.itemId) },
    relationStyles: merge(base.relationStyles ?? [], committed.relationStyles ?? [], pending.relationStyles ?? [], (style) => style.relationId),
  });
}
function merge<T>(before: readonly T[], committed: readonly T[], pending: readonly T[], identity: (item: T) => string) {
  const base = new Map(before.map((item) => [identity(item), item]));
  const remote = new Map(committed.map((item) => [identity(item), item]));
  const local = new Map(pending.map((item) => [identity(item), item]));
  return [...new Set([...remote.keys(), ...local.keys()])].flatMap((id) => {
    const changed = !isEqual(base.get(id), local.get(id));
    if (changed && !isEqual(base.get(id), remote.get(id)) && !isEqual(local.get(id), remote.get(id))) throw new Error(`Concurrent editor change at ${id}`);
    const item = changed ? local.get(id) : remote.get(id);
    return item === undefined ? [] : [item];
  });
}
''')
write('src/features/spaces/canvas/persistence/canvas-persistence-history.ts', '''
import type { SpaceEditorDocument } from "../contract/canvas-document";
import type { CanvasHistoryEntry, CanvasPersistenceHistory, CanvasPersistenceMutationResult } from "../contract/canvas-editor";
import { replayEditorTransition } from "../document/editor-history-transition";
interface CanvasPersistenceHistoryOptions {
  before: SpaceEditorDocument; after: SpaceEditorDocument; persistence: CanvasPersistenceHistory;
  current(): SpaceEditorDocument;
  commit(document: SpaceEditorDocument, transition: CanvasPersistenceHistory["undo"], baseDocument: SpaceEditorDocument): Promise<CanvasPersistenceMutationResult & { document: SpaceEditorDocument }>;
  adopt(document: SpaceEditorDocument): void;
}
export function createCanvasPersistenceHistory(options: CanvasPersistenceHistoryOptions): CanvasHistoryEntry {
  async function replay(before: SpaceEditorDocument, after: SpaceEditorDocument, transition: CanvasPersistenceHistory["undo"]) {
    const base = options.current();
    const result = await options.commit(replayEditorTransition(base, before, after), transition, base);
    options.adopt(result.document);
  }
  return { undo: () => replay(options.after, options.before, options.persistence.undo), redo: () => replay(options.before, options.after, options.persistence.redo) };
}
''')
write('src/features/spaces/canvas/document/canvas-content.ts', '''
import type { SpaceEditorDocument } from "../contract/canvas-document";
import { planStructureCommand } from "../../organization/structure/commands/plan";
export function removeCanvasContent(document: SpaceEditorDocument, ids: ReadonlySet<string>) {
  return ids.size ? { ...document, ...planStructureCommand(document, { kind: "delete-subtrees", ids: [...ids] }).after } : document;
}
export function removeCanvasSelection(document: SpaceEditorDocument, ids: ReadonlySet<string>, relationIds: ReadonlySet<string>) {
  const after = removeCanvasContent(document, ids);
  return { ...after, relationStyles: after.relationStyles?.filter((style) => !relationIds.has(style.relationId)) ?? [] };
}
''')
# Canonical snapshots are exposed read-only to Outline, Ask and inspectors through the existing session.
name='src/features/spaces/canvas/editor-session.ts'
text=read(name)
if 'const document = shallowRef<SpaceEditorDocument>();' not in text:
    text='import type { SpaceEditorDocument } from "./contract/canvas-document";\n'+text
    text=text.replace('const viewport =','const document = shallowRef<SpaceEditorDocument>();\nconst viewport =',1)
    text=text.replace('    activeEditor,','    activeEditor,\n    document: computed(() => document.value),',1)
    text=text.replace('  viewport.value = editor.getViewport();','  const detachDocument = editor.observeDocument((next) => { document.value = next; });\n  viewport.value = editor.getViewport();',1)
    text=text.replace('  return () => {\n    if (suspendedEditor','  return () => {\n    detachDocument();\n    if (suspendedEditor',1)
    text=text.replace('    viewport.value = undefined;','    viewport.value = undefined;\n    document.value = undefined;',1)
    write(name,text)
# Transport distinguishes the originating edit snapshot from its proposed after-state.
name='src/features/spaces/canvas/contract/canvas-editor.ts'; text=read(name)
text=text.replace('import type { SectionAssignment } from "./section-membership";\n','')
text=text.replace('  assignments: readonly SectionAssignment[];\n','')
text=text.replace('export interface CanvasGestureCommitRequest {','export interface CanvasGestureCommitRequest {\n  baseDocument: SpaceEditorDocument;')
text=text.replace('export interface CanvasDeleteSelectionRequest {','export interface CanvasDeleteSelectionRequest {\n  baseDocument: SpaceEditorDocument;')
text=text.replace('  onCreateRelation(', '  onPrepareStructureEdit(): Promise<void>;\n  onStructureToolFinished(): void;\n  onCreateRelation(',1)
text=text.replace('    transition: CanvasPersistenceHistory["undo"],\n','    transition: CanvasPersistenceHistory["undo"],\n    baseDocument: SpaceEditorDocument,\n')
write(name,text)
write('src/features/spaces/session/actions/canvas-gesture-actions.ts', '''
import type { CanvasGestureCommitRequest, CanvasPersistenceMutationRequest } from "../../canvas/contract/canvas-editor";
import { commitCanvasGesture } from "../../canvas/persistence/canvas-gesture-repository";
import type { SpaceState } from "../session-state";
export function createCanvasGestureActions(state: SpaceState) {
  async function commitGesture(request: CanvasGestureCommitRequest) {
    const spaceId = state.activeSpaceId.value;
    if (!spaceId) throw new Error("An active Space is required for editing");
    const persist = (input: CanvasPersistenceMutationRequest) => commitCanvasGesture(input, spaceId);
    const result = await persist(request);
    return { ...result, document: request.document, history: { undo: persist, redo: persist } };
  }
  return { commitGesture };
}
''')
write('src/features/spaces/canvas/persistence/canvas-gesture-repository.ts', '''
import { invoke } from "@tauri-apps/api/core";
import type { CanvasPersistenceMutationRequest } from "../contract/canvas-editor";
export function commitCanvasGesture(mutation: CanvasPersistenceMutationRequest, spaceId: string) {
  return invoke<{ revision: number }>("plugin:spaces|commit_canvas_gesture", { request: { spaceId, baseRevision: mutation.baseRevision, document: mutation.document } });
}
''')
# Runtime wiring is deliberate: no autosave for geometry, a single host transaction for edits.
name='src/features/spaces/canvas/konva/use-konva-canvas-runtime.ts';text=read(name)
text='import { projectEditorLayouts } from "../document/editor-layout-projection";\n'+text
text=text.replace('    documentChanged,\n    historyEntry:', '    commitDocument: (document, before) => props.events.onCommitGesture({ document, baseDocument: before }),\n    reportError: props.events.onError,\n    historyEntry:')
text=text.replace('const layouts = computed(() => sceneDocument.value.nodes);', 'const layouts = computed(() => projectEditorLayouts(sceneDocument.value));')
write(name,text)
for name,text in files.items():
    path=ROOT/name;path.parent.mkdir(parents=True,exist_ok=True)
    if not path.exists() or path.read_text()!=text:path.write_text(text)
print('Applied structure-owned geometry and committed-state model.')
