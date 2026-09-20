import { shallowRef, type ShallowRef } from "vue";

import {
  sameCanvasNodeAppearance,
  type SpaceEditorDocument,
  type CanvasNodeAppearance,
  type CanvasRelationColor,
} from "../../contract/canvas-document";
import type { CanvasHistoryEntry, CanvasLayerMove, CanvasViewport } from "../../contract/canvas-editor";
import { applyCanvasLayerOrder, sameCanvasViewport } from "./canvas-document-mutations";
import { reorderCanvasNodes } from "./canvas-layer-order";

interface CanvasDocumentModelEvents {
  documentChanged(document: SpaceEditorDocument): void;
  historyEntry(entry: CanvasHistoryEntry): void;
  viewportChanged(viewport: CanvasViewport): void;
}

export class CanvasDocumentModel {
  private readonly current: ShallowRef<SpaceEditorDocument>;
  private readonly sceneCurrent: ShallowRef<SpaceEditorDocument>;
  private readonly observers = new Set<(document: SpaceEditorDocument) => void>();
  private layoutIndices = new Map<string, number>();

  constructor(
    document: SpaceEditorDocument,
    private readonly events: CanvasDocumentModelEvents,
  ) {
    this.current = shallowRef(document);
    this.sceneCurrent = shallowRef(document);
    this.indexLayouts(document);
  }

  get document() {
    return this.current.value;
  }

  get documentRef(): Readonly<ShallowRef<SpaceEditorDocument>> {
    return this.current;
  }

  get sceneDocumentRef(): Readonly<ShallowRef<SpaceEditorDocument>> {
    return this.sceneCurrent;
  }

  replace(document: SpaceEditorDocument) {
    this.assign(document);
  }

  applyLocalDocument(document: SpaceEditorDocument) {
    this.update(document);
  }

  moveNodes(nodeIds: ReadonlySet<string>, mode: CanvasLayerMove) {
    const nodes = reorderCanvasNodes(this.document.nodes, nodeIds, mode);
    if (nodes.every((node, index) => node === this.document.nodes[index])) return;
    const before = new Map(this.document.nodes.map(({ nodeId, zIndex }) => [nodeId, zIndex]));
    const after = new Map(nodes.map(({ nodeId, zIndex }) => [nodeId, zIndex]));
    this.update(applyCanvasLayerOrder(this.document, after));
    this.events.historyEntry({
      undo: async () => this.update(applyCanvasLayerOrder(this.document, before)),
      redo: async () => this.update(applyCanvasLayerOrder(this.document, after)),
    });
  }

  setRelationColor(relationId: string, color: CanvasRelationColor) {
    const before = relationColor(this.document, relationId);
    if (before === color) return;
    this.applyRelationColor(relationId, color);
    this.events.historyEntry({
      undo: async () => this.applyRelationColor(relationId, before),
      redo: async () => this.applyRelationColor(relationId, color),
    });
  }

  setNodeAppearance(nodeId: string, patch: Partial<CanvasNodeAppearance>) {
    const layout = this.document.nodes.find((node) => node.nodeId === nodeId);
    const appearance = layout ? mergeAppearance(layout.appearance, patch) : undefined;
    if (!layout || sameCanvasNodeAppearance(layout.appearance, appearance)) return;
    const before = layout.appearance;
    this.applyNodeAppearance(nodeId, appearance);
    this.events.historyEntry({
      undo: async () => this.applyNodeAppearance(nodeId, before),
      redo: async () => this.applyNodeAppearance(nodeId, appearance),
    });
  }

  fitNodeHeights(heights: ReadonlyMap<string, number>) {
    let changed = false;
    const nodes = [...this.document.nodes];
    for (const [nodeId, height] of heights) {
      const index = this.layoutIndices.get(nodeId);
      const node = index === undefined ? undefined : nodes[index];
      const fittedHeight = Math.ceil(height);
      if (!node || fittedHeight === node.frame.height) continue;
      changed = true;
      nodes[index] = {
        ...node,
        frame: { ...node.frame, height: fittedHeight },
        sizeAuthority: "frame" as const,
      };
    }
    if (!changed) return;
    this.update({ ...this.document, nodes }, false);
  }

  setViewport(viewport: CanvasViewport) {
    if (sameCanvasViewport(this.document.viewport, viewport)) return;
    this.assign({ ...this.document, viewport: { ...viewport } }, false);
    this.events.documentChanged(this.document);
    this.events.viewportChanged(viewport);
  }

  private update(document: SpaceEditorDocument, reindex = true) {
    this.assign(document, reindex);
    this.events.documentChanged(document);
  }

  observe(observer: (document: SpaceEditorDocument) => void) {
    this.observers.add(observer);
    observer(this.document);
    return () => this.observers.delete(observer);
  }

  private applyRelationColor(relationId: string, color: CanvasRelationColor) {
    const retained = this.document.relationStyles?.filter((style) => style.relationId !== relationId) ?? [];
    const relationStyles = color === "neutral" ? retained : [...retained, { relationId, color }];
    this.update(withRelationStyles(this.document, relationStyles), false);
  }

  private applyNodeAppearance(nodeId: string, appearance: CanvasNodeAppearance | undefined) {
    this.update(
      {
        ...this.document,
        nodes: this.document.nodes.map((node) => (node.nodeId === nodeId ? { ...node, appearance } : node)),
      },
      false,
    );
  }

  private assign(document: SpaceEditorDocument, reindex = true) {
    const sceneChanged =
      this.current.value.nodes !== document.nodes ||
      this.current.value.relationStyles !== document.relationStyles;
    this.current.value = document;
    if (sceneChanged) this.sceneCurrent.value = document;
    if (reindex) this.indexLayouts(document);
    for (const observer of this.observers) observer(document);
  }

  layout(nodeId: string) {
    const index = this.layoutIndices.get(nodeId);
    return index === undefined ? undefined : this.document.nodes[index];
  }

  private indexLayouts(document: SpaceEditorDocument) {
    this.layoutIndices = new Map(document.nodes.map(({ nodeId }, index) => [nodeId, index]));
  }
}

function mergeAppearance(
  current: CanvasNodeAppearance | undefined,
  patch: Partial<CanvasNodeAppearance>,
): CanvasNodeAppearance {
  const appearance = { ...current, ...patch };
  if (appearance.customFill === undefined) delete appearance.customFill;
  return appearance;
}

function relationColor(document: SpaceEditorDocument, relationId: string): CanvasRelationColor {
  return document.relationStyles?.find((style) => style.relationId === relationId)?.color ?? "neutral";
}

function withRelationStyles(
  document: SpaceEditorDocument,
  relationStyles: NonNullable<SpaceEditorDocument["relationStyles"]>,
): SpaceEditorDocument {
  const { relationStyles: _current, ...content } = document;
  return relationStyles.length ? { ...content, relationStyles } : content;
}
