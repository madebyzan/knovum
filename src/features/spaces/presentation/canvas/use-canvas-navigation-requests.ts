import { storeToRefs } from "pinia";
import { watch } from "vue";

import { getCanvasEditorSession } from "../../canvas/editor-session";
import { useSpaceStore } from "../../session/space-store";

export function useCanvasNavigationRequests() {
  const store = useSpaceStore();
  const editorSession = getCanvasEditorSession();
  const { revealRequest, selectionClearNonce, viewportRequest, zoomRequest } = storeToRefs(store);
  function consumeRevealRequest() {
    const request = revealRequest.value;
    if (!request || !editorSession.activeEditor.value) return;
    editorSession.activeEditor.value.reveal(request.ids, request.mode);
    revealRequest.value = undefined;
  }
  function consumeViewportRequest() {
    if (!viewportRequest.value || !editorSession.activeEditor.value) return;
    editorSession.activeEditor.value.navigateViewport(viewportRequest.value.viewport);
    viewportRequest.value = undefined;
  }
  function consumeZoomRequest() {
    const request = zoomRequest.value;
    const editor = editorSession.activeEditor.value;
    if (!request || !editor) return;
    if (request.mode === "in") editor.zoomIn();
    else if (request.mode === "out") editor.zoomOut();
    else if (request.mode === "selection") editor.zoomToSelection();
    else editor.zoomToFit();
    zoomRequest.value = undefined;
  }
  watch(revealRequest, consumeRevealRequest);
  watch(viewportRequest, consumeViewportRequest);
  watch(zoomRequest, consumeZoomRequest);
  watch(selectionClearNonce, () => editorSession.activeEditor.value?.clearSelection());

  return {
    consumePending() {
      consumeRevealRequest();
      consumeViewportRequest();
      consumeZoomRequest();
    },
  };
}
