import type { Ref } from "vue";

import type { CanvasSurfaceProps } from "../contracts/canvas-surface";
import type { CanvasDocumentModel } from "./document/canvas-document-model";
import { useExternalContentDrop } from "./interaction/use-external-content-drop";
import { useStageViewport } from "./viewport/use-stage-viewport";

export function useRuntimeViewport(
  props: CanvasSurfaceProps,
  host: Readonly<Ref<HTMLElement | null>>,
  model: CanvasDocumentModel,
) {
  const viewport = useStageViewport({
    host,
    initialViewport: model.document.viewport,
    update: (next) => model.setViewport(next),
  });
  useExternalContentDrop({
    host,
    viewport: () => viewport.viewport.value,
    insertText: props.events.onInsertText,
    insertUrl: props.events.onInsertUrl,
    reportError: props.events.onError,
  });
  return viewport;
}
