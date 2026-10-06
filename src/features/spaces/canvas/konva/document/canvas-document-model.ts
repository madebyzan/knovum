import { shallowRef, type ShallowRef } from "vue";

import type { CanvasViewport } from "../../contracts/canvas-editor-port";
import { type CanvasDocumentV1 } from "../../schema/document";
import { sameCanvasViewport } from "./canvas-document-mutations";

interface CanvasDocumentModelEvents {
  documentChanged(document: CanvasDocumentV1): void;
  viewportChanged(viewport: CanvasViewport): void;
}

export class CanvasDocumentModel {
  private readonly current: ShallowRef<CanvasDocumentV1>;
  private readonly sceneCurrent: ShallowRef<CanvasDocumentV1>;
  private readonly observers = new Set<(document: CanvasDocumentV1) => void>();

  constructor(
    document: CanvasDocumentV1,
    private readonly events: CanvasDocumentModelEvents,
  ) {
    this.current = shallowRef(document);
    this.sceneCurrent = shallowRef(document);
  }

  get document() {
    return this.current.value;
  }

  get documentState(): Readonly<ShallowRef<CanvasDocumentV1>> {
    return this.current;
  }

  get sceneDocumentState(): Readonly<ShallowRef<CanvasDocumentV1>> {
    return this.sceneCurrent;
  }

  replace(document: CanvasDocumentV1) {
    this.assign(document);
  }

  applyLocalDocument(document: CanvasDocumentV1) {
    this.update(document);
  }

  setViewport(viewport: CanvasViewport) {
    if (sameCanvasViewport(this.document.viewport, viewport)) return;
    this.assign({ ...this.document, viewport: { ...viewport } });
    this.events.documentChanged(this.document);
    this.events.viewportChanged(viewport);
  }

  private update(document: CanvasDocumentV1) {
    this.assign(document);
    this.events.documentChanged(document);
  }

  observe(observer: (document: CanvasDocumentV1) => void) {
    this.observers.add(observer);
    observer(this.document);
    return () => this.observers.delete(observer);
  }

  private assign(document: CanvasDocumentV1) {
    const sceneChanged =
      this.current.value.nodes !== document.nodes ||
      this.current.value.relationStyles !== document.relationStyles;
    this.current.value = document;
    if (sceneChanged) this.sceneCurrent.value = document;
    for (const observer of this.observers) observer(document);
  }
}
