"""Reviewed hierarchical canvas integration; only the declared PR branch runs this recipe."""
from pathlib import Path
import re
ROOT=Path('.')
files={}
def read(path):
    return files[path] if path in files else (ROOT/path).read_text()
def write(path,text): files[path]=text.strip()+'\n'
def replace(path,old,new):
    text=read(path)
    if old not in text: raise RuntimeError(f'Missing expected source in {path}: {old[:80]}')
    write(path,text.replace(old,new))

# Container display handles have no knowledge identity or content lifecycle.
write('src/features/spaces/canvas/konva/scene/canvas-scene.ts', '''
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../../model/relations/relation";
import type { CanvasNodeLayoutV1, CanvasRelationColor } from "../../contract/canvas-document";
import type { CanvasDisplayNode } from "../../contract/canvas-display-node";
import type { CanvasNodeInteraction } from "./canvas-node-interaction";
export interface CanvasSceneNode { node: CanvasDisplayNode; layout: CanvasNodeLayoutV1; interaction: CanvasNodeInteraction; paintOrder: number }
export interface CanvasSceneContentNode extends CanvasSceneNode { node: SpaceCanvasNode }
export interface CanvasSceneSection extends CanvasSceneNode { members: readonly CanvasSceneContentNode[] }
export interface CanvasSceneRelation {
  id: string; key: string; kind: "relation"; label: string; color: CanvasRelationColor; relation: SpaceRelation;
  sourceNodeId: string; targetNodeId: string; parallelOffset: number;
  points: number[]; midpoint: { x: number; y: number }; bounds: { x: number; y: number; width: number; height: number };
}
export interface CanvasScene {
  nodes: readonly CanvasSceneNode[]; nodesById: ReadonlyMap<string, CanvasSceneNode>;
  contentNodes: readonly CanvasSceneContentNode[]; sections: readonly CanvasSceneSection[];
  groups: readonly CanvasSceneNode[]; relations: readonly CanvasSceneRelation[];
}
''')
write('src/features/spaces/organization/structure/structure-visibility.ts', '''
import { requireStructureItem, structureAncestors } from "../../model/structure/structure-index";
import type { StructureContext } from "./structure-context";
export function visibleStructureRepresentative(context: StructureContext, id: string): string {
  requireStructureItem(context.tree, id);
  const section = structureAncestors(context.tree, id).find((item) => item.kind === "section");
  if (!section) return id;
  const layout = context.layouts.get(section.id)!;
  return layout.kind === "section" && layout.collapsed ? section.id : id;
}
''')
write('src/features/spaces/canvas/konva/scene/scene-projection.ts', '''
import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../../model/relations/relation";
import { structureAncestors } from "../../../model/structure/structure-index";
import type { CanvasDisplayNode } from "../../contract/canvas-display-node";
import type { SpaceEditorDocument } from "../../contract/canvas-document";
import { projectEditorLayouts } from "../../document/editor-layout-projection";
import { indexStructureSnapshot } from "../../../organization/structure/structure-context";
import { structureContentNodeIds, structureSelectionTarget } from "../../../organization/structure/structure-selection";
import { visibleStructureRepresentative } from "../../../organization/structure/structure-visibility";
import { canvasNodeInteraction } from "./canvas-node-interaction";
import type { CanvasScene, CanvasSceneNode, CanvasSceneContentNode, CanvasSceneRelation } from "./canvas-scene";
import { parallelRelationOffsets } from "./parallel-relation-offsets";
import { relationGeometry } from "./relation-path";
export type { CanvasScene, CanvasSceneNode, CanvasSceneContentNode, CanvasSceneRelation, CanvasSceneSection } from "./canvas-scene";

export function projectCanvasScene(document: SpaceEditorDocument, nodes: readonly SpaceCanvasNode[], relations: readonly SpaceRelation[], scopeGroupId: string | null = null): CanvasScene {
  const context = indexStructureSnapshot(document);
  const content = new Map(nodes.map((node) => [node.id, node]));
  const layouts = projectEditorLayouts(document);
  const layoutsById = new Map(layouts.map((layout) => [layout.nodeId, layout]));
  const projected = layouts.flatMap<CanvasSceneNode>((layout) => {
    const item = context.tree.items.get(layout.nodeId)!;
    if (visibleStructureRepresentative(context, item.id) !== item.id) return [];
    let node: CanvasDisplayNode;
    if (item.kind === "content") {
      const source = content.get(item.nodeId);
      if (!source || source.hidden) return [];
      node = source;
    } else {
      const local = context.layouts.get(item.id)!;
      const contentCount = structureContentNodeIds(context.tree, [item.id]).length;
      node = item.kind === "group" ? { id: item.id, data: { kind: "group", title: item.title, collapsed: false, contentCount } }
        : { id: item.id, data: { kind: "section", title: item.title, collapsed: local.kind === "section" && local.collapsed, contentCount } };
    }
    const targetId = structureSelectionTarget(context.tree, item.id, scopeGroupId);
    const target = targetId ? context.tree.items.get(targetId) : undefined;
    const locked = !target || target.locked || structureAncestors(context.tree, target.id).some((parent) => parent.locked);
    const interaction = canvasNodeInteraction(node, locked);
    if (targetId !== item.id) interaction.connectable = false;
    return [{ node, layout, paintOrder: layout.zIndex, interaction }];
  }).toSorted((left, right) => left.paintOrder - right.paintOrder);
  const ordered = [...projected.filter((item) => item.node.data.kind === "section"), ...projected.filter((item) => item.node.data.kind !== "section")];
  const byId = new Map(ordered.map((item) => [item.node.id, item]));
  const endpoints = new Map<string, CanvasSceneNode>();
  for (const item of document.structure.items) {
    if (item.kind !== "content") continue;
    const representative = byId.get(visibleStructureRepresentative(context, item.id));
    if (representative && !content.get(item.nodeId)?.hidden) endpoints.set(item.nodeId, representative);
  }
  const offsets = parallelRelationOffsets(relations, endpoints);
  const styles = new Map(document.relationStyles?.map((style) => [style.relationId, style.color]));
  return { nodes: ordered, nodesById: byId,
    contentNodes: ordered.filter((item): item is CanvasSceneContentNode => item.node.data.kind !== "group" && item.node.data.kind !== "section"),
    groups: ordered.filter((item) => item.node.data.kind === "group"),
    sections: ordered.filter((item) => item.node.data.kind === "section").map((section) => ({ ...section,
      members: structureContentNodeIds(context.tree, [section.node.id]).flatMap((id) => {
        const node = content.get(id); const layout = layoutsById.get(id);
        return node && layout ? [{ node, layout, interaction: canvasNodeInteraction(node), paintOrder: layout.zIndex }] : [];
      }),
    })),
    relations: relations.flatMap((relation) => projectRelation(relation, endpoints, styles.get(relation.id) ?? "neutral", offsets.get(relation.id) ?? 0)),
  };
}
function projectRelation(relation: SpaceRelation, nodes: ReadonlyMap<string, CanvasSceneNode>, color: CanvasSceneRelation["color"], parallelOffset: number): CanvasSceneRelation[] {
  const source = nodes.get(relation.sourceId), target = nodes.get(relation.targetId);
  if (!source || !target || source.node.id === target.node.id || relation.status === "invalidated") return [];
  const geometry = relationGeometry(source.layout.frame, target.layout.frame, parallelOffset);
  return [{ ...geometry, id: relation.id, key: `relation:${relation.id}`, kind: "relation", label: relation.kind, relation, color, sourceNodeId: source.node.id, targetNodeId: target.node.id, parallelOffset }];
}
''')
write('src/features/spaces/canvas/konva/scene/canvas-node-interaction.ts', '''
import type { CanvasDisplayNode } from "../../contract/canvas-display-node";
import { spaceNodeResizePolicy, type SpaceNodeResizeMode } from "../../node-size";
export interface CanvasNodeInteraction {
  activationMode: "activate-to-interact" | "primary" | "select-to-interact";
  connectable: boolean; movable: boolean;
  transform: { resizeMode: SpaceNodeResizeMode; maxHeight: number; maxWidth: number; minHeight: number; minWidth: number; contentAspect?: { ratio: number; fixedVerticalChrome: number } };
}
const SELECT = new Set(["audio", "credential", "video"]);
const ACTIVATE = new Set(["code", "note", "text", "website"]);
export function canvasNodeInteraction(node: CanvasDisplayNode, locked = false): CanvasNodeInteraction {
  const data = node.data;
  if (data.kind === "group" || data.kind === "section") return {
    activationMode: "primary", connectable: false, movable: !locked,
    transform: { resizeMode: locked || data.kind === "section" && data.collapsed ? "fixed" : data.kind === "group" ? "proportional" : "free",
      minWidth: data.kind === "section" ? 240 : 1, minHeight: data.kind === "section" ? 160 : 1, maxWidth: Infinity, maxHeight: Infinity },
  };
  const resize = spaceNodeResizePolicy(data);
  const activationMode = SELECT.has(data.kind) || data.kind === "file" && Boolean(data.entries?.length) ? "select-to-interact" : ACTIVATE.has(data.kind) ? "activate-to-interact" : "primary";
  return { activationMode, connectable: !["credential", "portal"].includes(data.kind), movable: !locked,
    transform: { ...resize, minHeight: data.kind === "text" ? 1 : resize.minHeight, resizeMode: locked ? "fixed" : resize.resizeMode } };
}
''')
write('src/features/spaces/canvas/konva/selection/use-structure-scope.ts', '''
import { shallowRef, watch } from "vue";
import type { SpaceEditorDocument } from "../../contract/canvas-document";
import { indexStructure, requireStructureItem, structureAncestors } from "../../../model/structure/structure-index";
import { structureSelectionTarget } from "../../../organization/structure/structure-selection";
export function useStructureScope(document: () => SpaceEditorDocument, select: (ids: readonly string[]) => void) {
  const groupId = shallowRef<string | null>(null);
  const renameId = shallowRef<string>();
  function enter(id: string) {
    const tree = indexStructure(document().structure);
    const item = requireStructureItem(tree, id);
    if (item.kind !== "group" || item.locked || structureAncestors(tree, id).some((parent) => parent.locked)) return;
    groupId.value = id; select([]);
  }
  function exit() {
    const id = groupId.value; if (!id) return;
    groupId.value = structureAncestors(indexStructure(document().structure), id).find((parent) => parent.kind === "group")?.id ?? null;
    select([id]);
  }
  function setForSelection(ids: readonly string[]) {
    const tree = indexStructure(document().structure);
    const paths = ids.filter((id) => tree.items.has(id)).map((id) => structureAncestors(tree, id).filter((item) => item.kind === "group"));
    groupId.value = paths[0]?.find((item) => paths.every((path) => path.some((parent) => parent.id === item.id)))?.id ?? null;
  }
  function resolveHit(id: string) {
    const tree = indexStructure(document().structure);
    const target = structureSelectionTarget(tree, id, groupId.value);
    if (target !== undefined) return target;
    groupId.value = null;
    return structureSelectionTarget(tree, id, null)!;
  }
  watch(document, (current) => {
    if (groupId.value && !current.structure.items.some((item) => item.id === groupId.value && item.kind === "group")) { groupId.value = null; select([]); }
    if (renameId.value && !current.structure.items.some((item) => item.id === renameId.value)) renameId.value = undefined;
  });
  return { groupId, renameId, enter, exit, setForSelection, resolveHit };
}
''')
write('src/features/spaces/canvas/konva/selection/use-canvas-selection.ts', '''
import type Konva from "konva";
import { computed, shallowRef, watch, type Ref } from "vue";
import type { CanvasNodeLayoutV1, SpaceEditorDocument } from "../../contract/canvas-document";
import type { CanvasRect, CanvasSelectionSnapshot } from "../../contract/canvas-editor-port";
import { createCanvasSelectionSnapshot } from "../../document/canvas-selection-snapshot";
import { transformedStructureBounds } from "../../document/structure-transform";
import { indexStructure } from "../../../model/structure/structure-index";
import { structureOperationRoots, structureSelectionTarget } from "../../../organization/structure/structure-selection";
interface SelectionOptions {
  document(): SpaceEditorDocument; scopeGroupId(): string | null;
  layouts: Readonly<Ref<readonly CanvasNodeLayoutV1[]>>;
  relationBounds: Readonly<Ref<ReadonlyMap<string, CanvasRect>>>;
  changed(selection: CanvasSelectionSnapshot): void;
}
export function useCanvasSelection(options: SelectionOptions) {
  const nodeIds = shallowRef(new Set<string>()), relationIds = shallowRef(new Set<string>());
  const marquee = shallowRef<{ start: { x: number; y: number }; end: { x: number; y: number } }>();
  const marqueeRect = computed(() => marquee.value ? { x: Math.min(marquee.value.start.x, marquee.value.end.x), y: Math.min(marquee.value.start.y, marquee.value.end.y), width: Math.abs(marquee.value.end.x - marquee.value.start.x), height: Math.abs(marquee.value.end.y - marquee.value.start.y) } : undefined);
  function selectNode(id: string, event: PointerEvent) {
    const additive = event.ctrlKey || event.metaKey || event.shiftKey;
    if (!additive && nodeIds.value.has(id)) return;
    const selected = new Set(additive ? nodeIds.value : []);
    if (additive && selected.has(id)) selected.delete(id); else selected.add(id);
    setNodeIds([...selected]);
  }
  function selectRelation(id: string, event: PointerEvent) {
    const additive = event.ctrlKey || event.metaKey || event.shiftKey;
    const selected = new Set(additive ? relationIds.value : []);
    if (additive && selected.has(id)) selected.delete(id); else selected.add(id);
    nodeIds.value = new Set(); relationIds.value = selected; notify();
  }
  function beginMarquee(event: Konva.KonvaEventObject<PointerEvent>) {
    if (event.evt.button !== 0 || event.target !== event.target.getStage()) return false;
    const point = event.target.getStage()?.getRelativePointerPosition(); if (!point) return false;
    marquee.value = { start: point, end: point };
    if (!(event.evt.ctrlKey || event.evt.metaKey || event.evt.shiftKey)) clear();
    return true;
  }
  function moveMarquee(event: Konva.KonvaEventObject<PointerEvent>) { const point = event.target.getStage()?.getRelativePointerPosition(); if (marquee.value && point) marquee.value = { ...marquee.value, end: point }; }
  function endMarquee() {
    const bounds = marqueeRect.value; marquee.value = undefined;
    if (!bounds || bounds.width < 1 || bounds.height < 1) return;
    const tree = indexStructure(options.document().structure);
    const selected = new Set(nodeIds.value);
    for (const layout of options.layouts.value) {
      const item = tree.items.get(layout.nodeId)!;
      if (item.kind === "group") continue;
      const frame = transformedStructureBounds({ ...layout.frame, scale: 1 }, layout.frame);
      const inside = frame.x >= bounds.x && frame.y >= bounds.y && frame.x + frame.width <= bounds.x + bounds.width && frame.y + frame.height <= bounds.y + bounds.height;
      const overlap = frame.x <= bounds.x + bounds.width && frame.y <= bounds.y + bounds.height && frame.x + frame.width >= bounds.x && frame.y + frame.height >= bounds.y;
      if (!(item.kind === "section" ? inside : overlap)) continue;
      const target = structureSelectionTarget(tree, item.id, options.scopeGroupId()); if (target) selected.add(target);
    }
    setNodeIds([...selected]);
  }
  function setNodeIds(ids: readonly string[]) {
    const tree = indexStructure(options.document().structure);
    nodeIds.value = new Set(structureOperationRoots(tree, ids.filter((id) => tree.items.has(id))).map((item) => item.id)); relationIds.value = new Set(); notify();
  }
  function clear() { nodeIds.value = new Set(); relationIds.value = new Set(); notify(); }
  function notify() { options.changed({ ...createCanvasSelectionSnapshot(options.layouts.value, nodeIds.value, relationIds.value, options.relationBounds.value), scopeGroupId: options.scopeGroupId() }); }
  watch(options.layouts, (layouts) => { const visible = new Set(layouts.map((layout) => layout.nodeId)); nodeIds.value = new Set([...nodeIds.value].filter((id) => visible.has(id))); notify(); });
  watch(options.relationBounds, (bounds) => { relationIds.value = new Set([...relationIds.value].filter((id) => bounds.has(id))); notify(); });
  return { nodeIds, relationIds, marqueeRect, selectNode, selectRelation, setNodeIds, clear, beginMarquee, moveMarquee, endMarquee, cancelMarquee: () => { marquee.value = undefined; } };
}
''')
# Selection snapshots now expose the current editing scope, rather than guessing it from payloads.
name='src/features/spaces/canvas/contract/canvas-editor-port.ts';text=read(name)
text='import type { StructureCommand } from "../../organization/structure/commands/contract";\n'+text
text=text.replace('export interface CanvasSelectionSnapshot {','export interface CanvasSelectionSnapshot {\n  scopeGroupId?: string | null;')
text=text.replace('  cancelInteraction(): void;', '  cancelInteraction(): void;\n  executeStructureCommand(command: StructureCommand): Promise<void>;\n  enterGroup(id: string): void;\n  exitGroup(): void;\n  requestContainerRename(id: string): void;\n  deleteItems(ids: readonly string[], relationIds?: readonly string[]): Promise<void>;')
text=text.replace('nudgeSelection(offset: CanvasPoint): void','nudgeSelection(offset: CanvasPoint): Promise<void>')
text=text.replace('moveSelectionLayer(mode: CanvasLayerMove): void','moveSelectionLayer(mode: CanvasLayerMove): Promise<void>')
text=text.replace('setRelationColor(relationId: string, color: CanvasRelationColor): void','setRelationColor(relationId: string, color: CanvasRelationColor): Promise<void>')
text=text.replace('setNodeAppearance(nodeId: string, patch: Partial<CanvasNodeAppearance>): void','setNodeAppearance(nodeId: string, patch: Partial<CanvasNodeAppearance>): Promise<void>')
write(name,text)
# Node dragging resolves the whole selected subtree once per gesture.
write('src/features/spaces/canvas/konva/interaction/canvas-node-drag-state.ts', '''
import type Konva from "konva";
import type { CanvasNodeFrameUpdate, CanvasNodeLayoutV1, SpaceEditorDocument } from "../../contract/canvas-document";
import { indexStructure, structureSubtree, structureAncestors } from "../../../model/structure/structure-index";
import { assertStructureEditable, structureOperationRoots } from "../../../organization/structure/structure-selection";
import { captureCanvasNodeRefs } from "./canvas-node-access";
import { createCanvasTranslationSnapIndex, type CanvasTranslationGestureState } from "./canvas-translation-snapping";
export function captureCanvasNodeDragState(layouts: readonly CanvasNodeLayoutV1[], document: SpaceEditorDocument, selectedIds: ReadonlySet<string>, primaryId: string, stage: Konva.Stage | undefined) {
  const tree = indexStructure(document.structure);
  const roots = structureOperationRoots(tree, [...selectedIds, primaryId]);
  assertStructureEditable(tree, roots.map((item) => item.id));
  const affected = new Set(structureSubtree(tree, roots.map((item) => item.id)).map((item) => item.id));
  const ancestors = new Set(roots.flatMap((item) => structureAncestors(tree, item.id).filter((parent) => parent.kind === "group").map((parent) => parent.id)));
  const frames = new Map(layouts.filter((layout) => affected.has(layout.nodeId)).map((layout) => [layout.nodeId, { ...layout.frame }]));
  const origin = frames.get(primaryId); if (!origin) return undefined;
  const state: CanvasTranslationGestureState = { primaryId, origin, frames, snapIndex: createCanvasTranslationSnapIndex(layouts.filter((layout) => !affected.has(layout.nodeId) && !ancestors.has(layout.nodeId)).map((layout) => layout.frame)) };
  return { state, nodeRefs: captureCanvasNodeRefs(stage, frames.keys()) };
}
export function restoreCanvasNodeFrames(updates: readonly CanvasNodeFrameUpdate[], nodes: ReadonlyMap<string, Konva.Node>) {
  for (const { nodeId, frame } of updates) { const node = nodes.get(nodeId); node?.position({ x: frame.x, y: frame.y }); node?.rotation(frame.rotation); node?.scale({ x: 1, y: 1 }); }
}
''')
name='src/features/spaces/canvas/konva/interaction/node-gesture-controller.ts';text=read(name)
text=text.replace('import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";', 'import type { CanvasDisplayNode } from "../../contract/canvas-display-node";\nimport type { SpaceEditorDocument } from "../../contract/canvas-document";').replace('SpaceCanvasNode','CanvasDisplayNode')
text=text.replace('  layouts():', '  document(): SpaceEditorDocument;\n  layouts():',1)
text=text.replace('      this.options.nodes(),\n      this.options.selectedNodeIds(),\n      primaryId,','      this.options.document(),\n      this.options.selectedNodeIds(),\n      primaryId,')
write(name,text)
name='src/features/spaces/canvas/konva/interaction/canvas-transform-projection.ts';text=read(name)
text=text.replace('import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";', 'import type { CanvasDisplayNode } from "../../contract/canvas-display-node";').replace('SpaceCanvasNode','CanvasDisplayNode')
text=text.replace('    if (content.data.locked) return [];\n','')
write(name,text)
# Pointer-down collapses/selects the resolved Group, not the physically hit child.
name='src/features/spaces/canvas/konva/interaction/use-pointer-interactions.ts';text=read(name)
text=text.replace('  cancelActiveGesture(): boolean;','  cancelActiveGesture(): boolean;\n  resolveHit(id: string): string;')
text=text.replace('    const additive = event.evt.metaKey', '    nodeId = options.resolveHit(nodeId);\n    const additive = event.evt.metaKey')
write(name,text)
# Scope-aware activation cannot enter Group and edit content in the same double-click.
name='src/features/spaces/canvas/konva/interaction/use-node-content-activation.ts';text=read(name)
text=text.replace('    if (!item) return;', '    if (!item || !item.interaction.movable) return;')
text=text.replace('if (selected?.interaction.activationMode === "select-to-interact")','if (selected?.interaction.movable && selected.interaction.activationMode === "select-to-interact")')
write(name,text)
# DOM carries the very same world transform, while text measures and reflows in local coordinates.
name='src/features/spaces/canvas/konva/presentation/CanvasDomNodeOverlay.vue';text=read(name)
text=text.replace('CanvasSceneNode','CanvasSceneContentNode')
text=text.replace('const effectiveFrame =', 'const contentScale = computed(() => effectiveLayout.value.contentScale ?? 1);\nconst contentLayout = computed(() => ({ ...effectiveLayout.value, frame: { ...effectiveLayout.value.frame, width: effectiveLayout.value.frame.width / contentScale.value, height: effectiveLayout.value.frame.height / contentScale.value } }));\nconst effectiveFrame =')
text=text.replace('`${effectiveFrame.value.width}px`','`${effectiveFrame.value.width / contentScale.value}px`').replace('`${effectiveFrame.value.height}px`','`${effectiveFrame.value.height / contentScale.value}px`')
text=text.replace('`rotate(${effectiveFrame.value.rotation}deg)`','`rotate(${effectiveFrame.value.rotation}deg) scale(${contentScale.value})`').replace(':layout="effectiveLayout"', ':layout="contentLayout"')
write(name,text)
name='src/features/spaces/canvas/konva/presentation/CanvasDomOverlay.vue';write(name,read(name).replace('props.scene.nodes.filter','props.scene.contentNodes.filter'))
name='src/features/spaces/canvas/konva/interaction/canvas-gesture-update.ts';write(name,read(name).replace('export interface CanvasGestureUpdate extends CanvasNodeFrameUpdate {','export interface CanvasGestureUpdate extends CanvasNodeFrameUpdate {\n  contentScale?: number;'))
name='src/features/spaces/canvas/konva/scene/scene-preview.ts';write(name,read(name).replace('    frame: update.frame,','    frame: update.frame,\n    contentScale: update.contentScale ?? layout.contentScale,'))
# Ready registration returns the actual editor for container chrome instead of exporting a second authority.
for name in ('src/features/spaces/canvas/konva/canvas-editor-port-options.ts','src/features/spaces/canvas/konva/use-editor-registration.ts'):
    text=read(name)
    text='import type { StructureCommand } from "../../organization/structure/commands/contract";\n'+text
    text=text.replace('  cancelInteraction(): void;', '  cancelInteraction(): void;\n  executeStructureCommand(command: StructureCommand): Promise<void>;\n  enterGroup(id: string): void;\n  exitGroup(): void;\n  scopeGroupId(): string | null;\n  setEditingScope(ids: readonly string[]): void;\n  requestContainerRename(id: string): void;')
    text=text.replace('      cancelInteraction: options.cancelInteraction,','      cancelInteraction: options.cancelInteraction,\n      executeStructureCommand: options.executeStructureCommand,\n      enterGroup: options.enterGroup,\n      exitGroup: options.exitGroup,\n      scopeGroupId: options.scopeGroupId,\n      setEditingScope: options.setEditingScope,\n      requestContainerRename: options.requestContainerRename,')
    if name.endswith('use-editor-registration.ts'):
        text=text.replace('nextTick, onMounted, watch,','nextTick, onMounted, shallowRef, watch,')
        text=text.replace('  let editor:', '  const editorRef = shallowRef<ReturnType<typeof createKonvaEditorPort>>();\n  let editor:')
        text=text.replace('    options.events.onReady(editor);','    editorRef.value = editor;\n    options.events.onReady(editor);')
        last=text.rfind('\n}')
        text=text[:last]+'\n  return editorRef;'+text[last:]
    write(name,text)
# Group rotation is distinct from ordinary card resizing.
name='src/features/spaces/canvas/konva/selection/selection-transform-policy.ts';text=read(name)
text=text.replace('export interface CanvasSelectionTransformPolicy {','export interface CanvasSelectionTransformPolicy {\n  rotateEnabled?: boolean;')
text=text.replace('    nodeIds: [node.node.id],','    nodeIds: [node.node.id],\n    rotateEnabled: node.node.data.kind === "group",')
for field in ('minWidth','minHeight','maxWidth','maxHeight'):
    text=text.replace(f'{field}: transform.{field},',f'{field}: transform.{field} * (node.layout.contentScale ?? 1),')
write(name,text)
name='src/features/spaces/canvas/konva/presentation/CanvasSelectionTransformer.vue';text=read(name).replace('rotateEnabled: false,','rotateEnabled: policy.rotateEnabled === true,').replace('function isResizeAnchorVisible(anchor: Konva.Rect)', 'function isResizeAnchorVisible(anchor: Konva.Node)')
text=text.replace('  return enabled &&','  if (anchor.hasName("rotater")) return props.policy.rotateEnabled === true;\n  return enabled &&')
write(name,text)
# Runtime consumes one canonical scene and publishes scope through the established selection channel.
name='src/features/spaces/canvas/konva/use-konva-canvas-runtime.ts';text=read(name)
text='import { useStructureScope } from "./selection/use-structure-scope";\n'+text
text=text.replace('  const sceneDocument = model.sceneDocumentRef;', '  const scope = useStructureScope(() => model.documentRef.value, (ids) => selection.setNodeIds(ids));\n  const sceneDocument = model.sceneDocumentRef;')
text=text.replace('projectCanvasScene(sceneDocument.value, props.nodes, props.relations)', 'projectCanvasScene(sceneDocument.value, props.nodes, props.relations, scope.groupId.value)')
text=text.replace('  const selection = useCanvasSelection({','  const selection = useCanvasSelection({\n    document: () => model.document,\n    scopeGroupId: () => scope.groupId.value,')
text=text.replace('    nodes: () => props.nodes,\n    palette: () => palette.value,\n    previewFrames:', '    nodes: () => canonicalScene.value.nodes.map((item) => item.node),\n    previewFrames:')
text=re.sub(r'    sectionNodeIds: \(\) =>\n      new Set\([^\n]+\),\n', '', text)
text=text.replace('    selectedNodeIds: () => selection.nodeIds.value,','    selectedNodeIds: () => selection.nodeIds.value,\n    setSelection: selection.setNodeIds,',1)
text=text.replace('  const { activateNode, capturesCanvasInteraction, createTextNode, interactiveNodeId } =','  const { activateNode: activateContent, capturesCanvasInteraction, createTextNode, interactiveNodeId } =')
text=text.replace('  const pointer = usePointerInteractions({','''  function activateNode(id: string) {
    const target = scope.resolveHit(id);
    const item = model.document.structure.items.find((candidate) => candidate.id === target);
    if (item?.kind === "group") scope.enter(target);
    else if (item?.kind === "section") scope.renameId.value = target;
    else activateContent(target);
  }
  const pointer = usePointerInteractions({
    resolveHit: scope.resolveHit,''')
text=text.replace('    if (!gestures.cancelActiveGesture()) selection.clear();','    if (!gestures.cancelActiveGesture()) { if (scope.renameId.value) scope.renameId.value = undefined; else if (scope.groupId.value) scope.exit(); else selection.clear(); }')
text=text.replace('  useEditorRegistration({','''  const editor = useEditorRegistration({
    executeStructureCommand: gestures.runStructureCommand,
    enterGroup: scope.enter, exitGroup: scope.exit, scopeGroupId: () => scope.groupId.value,
    setEditingScope: scope.setForSelection,
    requestContainerRename: (id) => { scope.renameId.value = id; },''')
text=text.replace('nodeById: (id) => canonicalScene.value.nodesById.get(id)?.node','nodeById: (id) => props.nodes.find((node) => node.id === id)')
text=text.replace('selectableNodeIds: () => canonicalScene.value.nodes.map(({ node }) => node.id)', 'selectableNodeIds: () => model.document.structure.items.filter((item) => item.parentId === scope.groupId.value).map((item) => item.id)')
text=text.replace('  return {\n    activateNode,','  return {\n    editor, scope, model,\n    activateNode,')
write(name,text)
# Strict content no longer admits organizational payload fields or synthetic Section content.
name='src/features/spaces/model/nodes/schema/base-fields.ts';write(name,re.sub(r'^  (sectionId|locked):[^\n]*\n','',read(name),flags=re.M))
name='src/features/spaces/model/nodes/schema/node-variants.ts';text=read(name)
start=text.find('  z\n    .object({\n      ...baseSpaceNodeFields,\n      kind: z.literal("section")')
if start >= 0:
    end=text.index('\n]);',start)
    text=text[:start]+text[end:]
write(name,text)
name='src/features/spaces/model/nodes/node-payload.ts';write(name,re.sub(r'^export type SpaceSectionPayload[^\n]*\n','',read(name),flags=re.M))
name='src/features/spaces/canvas/node-size.ts';text=read(name)
text=re.sub(r'^  section:[^\n]*\n','',text,flags=re.M)
text=re.sub(r'  if \(data.kind === "section" && data.collapsed\) \{[^}]*}\n','',text)
write(name,text)
for name,text in files.items():
    path=ROOT/name;path.parent.mkdir(parents=True,exist_ok=True)
    if not path.exists() or path.read_text()!=text:path.write_text(text)
print('Integrated structural scene, scope, and local-to-world content rendering.')
