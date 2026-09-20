import { shallowRef, type ShallowRef } from "vue";

import type { StructureCommand } from "../../../organization/structure/commands/contract";
import type {
  CanvasNodeAppearance,
  CanvasRelationColor,
  SpaceEditorDocument,
} from "../../contract/canvas-document";
import type { CanvasHistoryEntry, CanvasLayerMove, CanvasViewport } from "../../contract/canvas-editor";
import type {
  ExecuteSpaceEditDraft,
  ReplaySpaceEditDraft,
  SpaceEditorCommand,
  EditorTextMeasurement,
} from "../../contract/editor-command";
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

  executeNative(command: SpaceEditorCommand, baseDocument?: SpaceEditorDocument, record = true) {
    return this.enqueue(async () => {
      const result = await this.events.executeCommand({
        command,
        mutationId: crypto.randomUUID(),
        baseDocument: baseDocument ?? this.document,
      });
      this.adopt(result);
      const receiptId = result.historyReceiptId;
      if (record && receiptId)
        this.events.historyEntry({
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
    await this.executeNative({
      kind: "set-relation-color",
      relationId,
      color: color === "neutral" ? null : color,
    });
  }
  async setNodeAppearance(itemId: string, patch: Partial<CanvasNodeAppearance>) {
    const changes = Object.fromEntries(Object.entries(patch).map(([field, value]) => [field, value ?? null]));
    await this.executeNative({ kind: "set-appearance", itemId, changes });
  }
  fitNodeHeights(measurements: readonly EditorTextMeasurement[]) {
    if (!measurements.length) return Promise.resolve();
    return this.executeNative({ kind: "measure-text", measurements }, undefined, false)
      .then(() => undefined)
      .catch(this.events.reportError);
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
    this.tail = pending.then(
      () => undefined,
      () => undefined,
    );
    void pending.then(
      () => this.pending.delete(pending),
      () => this.pending.delete(pending),
    );
    return pending;
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
