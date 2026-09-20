import { watch } from "vue";

import type { SpaceEditorDocument } from "../../contract/canvas-document";
import type { CanvasDocumentModel } from "./canvas-document-model";
import { sameCanvasViewport } from "./canvas-document-mutations";

export function useCanvasDocumentSync(
  source: () => SpaceEditorDocument,
  model: CanvasDocumentModel,
  setViewport: (viewport: SpaceEditorDocument["viewport"], notify: boolean) => void,
  getViewport: () => SpaceEditorDocument["viewport"],
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
