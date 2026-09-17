import { watch } from "vue";

import type { CanvasDocumentV1 } from "../../contract/canvas-document";
import type { CanvasDocumentModel } from "./canvas-document-model";
import { sameCanvasViewport } from "./canvas-document-mutations";

export function useCanvasDocumentSync(
  source: () => CanvasDocumentV1,
  model: CanvasDocumentModel,
  setViewport: (viewport: CanvasDocumentV1["viewport"], notify: boolean) => void,
  getViewport: () => CanvasDocumentV1["viewport"],
) {
  watch(source, (next, previous) => {
    if (next === model.document) return;
    if (!sameCanvasViewport(next.viewport, previous.viewport)) {
      setViewport(next.viewport, false);
      model.replace(next);
      return;
    }
    const viewport = getViewport();
    if (sameCanvasViewport(next.viewport, viewport)) model.replace(next);
    else model.applyLocalDocument({ ...next, viewport: { ...viewport } });
  });
}
