import { storeToRefs } from "pinia";
import { watch } from "vue";

import { useSpaceStore } from "../../session/space-store";
import { getCanvasEditorSession } from "../editor-session";

export function useCanvasNavigationRequests() {
  const store = useSpaceStore();
  const editorSession = getCanvasEditorSession();
  const { revealRequest, selectionClearNonce, viewportRequest, zoomRequest } = storeToRefs(store);
  function consumeRevealRequest() {
    const request = revealRequest.value;
    if (!store.workbench.canvas || !request || !editorSession.activeEditor.value) return;
    editorSession.activeEditor.value.reveal(request.ids, request.mode);
    revealRequest.value = undefined;
  }
  function consumeViewportRequest() {
    if (!store.workbench.canvas || !viewportRequest.value || !editorSession.activeEditor.value) return;
    editorSession.activeEditor.value.navigateViewport(viewportRequest.value.viewport);
    viewportRequest.value = undefined;
  }
  function consumeZoomRequest() {
    const request = zoomRequest.value;
    const editor = editorSession.activeEditor.value;
    if (!store.workbench.canvas || !request || !editor) return;
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

  function consumePending() {
    consumeRevealRequest();
    consumeViewportRequest();
    consumeZoomRequest();
  }
  watch([() => store.workbench.canvas, editorSession.activeEditor], consumePending);
  return { consumePending };
}
