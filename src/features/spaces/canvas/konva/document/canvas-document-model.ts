import { isEqual } from "es-toolkit";
import { shallowRef, type ShallowRef } from "vue";

import type { StructureCommand } from "../../../organization/structure/commands/contract";
import { reorderStructureItems } from "../../../organization/structure/commands/layer-order";
import { planStructureCommand } from "../../../organization/structure/commands/plan";
import type {
  CanvasNodeAppearance,
  CanvasRelationColor,
  SpaceEditorDocument,
} from "../../contract/canvas-document";
import { parseSpaceEditorDocument } from "../../contract/canvas-document";
import type {
  CanvasGestureCommitResult,
  CanvasHistoryEntry,
  CanvasLayerMove,
  CanvasViewport,
} from "../../contract/canvas-editor";
import { setEditorAppearance, setEditorRelationColor } from "../../document/editor-appearance";
import { replayEditorTransition } from "../../document/editor-history-transition";
import { projectEditorLayouts } from "../../document/editor-layout-projection";
import { sameCanvasViewport } from "./canvas-document-mutations";

interface CanvasDocumentModelEvents {
  commitDocument(
    document: SpaceEditorDocument,
    before: SpaceEditorDocument,
  ): Promise<CanvasGestureCommitResult>;
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
  constructor(
    document: SpaceEditorDocument,
    private readonly events: CanvasDocumentModelEvents,
  ) {
    this.current = shallowRef(document);
    this.sceneCurrent = shallowRef(document);
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
    this.assign(document);
  }
  layout(id: string) {
    return projectEditorLayouts(this.document).find((layout) => layout.nodeId === id);
  }
  async flush() {
    await Promise.all([...this.pending]);
  }
  mutate(calculate: (document: SpaceEditorDocument) => SpaceEditorDocument, record = true): Promise<void> {
    const operation = this.tail.then(async () => {
      const before = this.document;
      const after = parseSpaceEditorDocument(calculate(before));
      if (
        isEqual(before.structure, after.structure) &&
        isEqual(before.layout, after.layout) &&
        isEqual(before.relationStyles, after.relationStyles)
      )
        return;
      const result = await this.events.commitDocument(after, before);
      this.assign({ ...result.document, viewport: this.document.viewport });
      if (record)
        this.events.historyEntry({
          undo: () => this.mutate((current) => replayEditorTransition(current, after, before), false),
          redo: () => this.mutate((current) => replayEditorTransition(current, before, after), false),
        });
    });
    this.pending.add(operation);
    this.tail = operation.then(
      () => undefined,
      () => undefined,
    );
    void operation.then(
      () => this.pending.delete(operation),
      () => this.pending.delete(operation),
    );
    return operation;
  }
  async execute(command: StructureCommand, selection: readonly string[] = []) {
    let selected = selection;
    await this.mutate((document) => {
      const plan = planStructureCommand(document, command, selection);
      if (plan.effects.removedContentNodeIds.length || plan.effects.contentClones.length)
        throw new Error("Content lifecycle changes require a Space content transaction");
      selected = plan.selection;
      return { ...document, ...plan.after };
    });
    return selected;
  }
  moveNodes(ids: ReadonlySet<string>, mode: CanvasLayerMove) {
    return this.mutate((document) => reorderStructureItems(document, [...ids], mode));
  }
  setRelationColor(id: string, color: CanvasRelationColor) {
    return this.mutate((document) => setEditorRelationColor(document, id, color));
  }
  setNodeAppearance(id: string, patch: Partial<CanvasNodeAppearance>) {
    return this.mutate((document) => setEditorAppearance(document, id, patch));
  }
  fitNodeHeights(heights: ReadonlyMap<string, number>) {
    return this.mutate(
      (document) => ({
        ...document,
        layout: {
          version: 1,
          items: document.layout.items.map((layout) => {
            const height = heights.get(layout.itemId);
            return layout.kind === "content" && height !== undefined
              ? { ...layout, size: { ...layout.size, height: Math.ceil(height) } }
              : layout;
          }),
        },
      }),
      false,
    ).catch(this.events.reportError);
  }
  setViewport(viewport: CanvasViewport) {
    if (sameCanvasViewport(this.document.viewport, viewport)) return;
    this.assign({ ...this.document, viewport: { ...viewport } });
    this.events.viewportChanged(viewport);
  }
  observe(observer: (document: SpaceEditorDocument) => void) {
    this.observers.add(observer);
    observer(this.document);
    return () => {
      this.observers.delete(observer);
    };
  }
  private assign(document: SpaceEditorDocument) {
    const previous = this.document;
    this.current.value = document;
    if (
      previous.structure !== document.structure ||
      previous.layout !== document.layout ||
      previous.relationStyles !== document.relationStyles
    )
      this.sceneCurrent.value = document;
    for (const observer of this.observers) observer(document);
  }
}
