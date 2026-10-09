import { onBeforeUnmount, shallowRef, watch, type Ref } from "vue";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import type { CanvasExchangePort } from "../../canvas-exchange/canvas-exchange-port";
import { registerCanvasExchangePort } from "../../canvas-exchange/canvas-exchange-session";
import { useCanvasToolStore } from "../../canvas/canvas-tool-store";
import type { CanvasEditorPort } from "../../canvas/contracts/canvas-editor-port";
import { registerCanvasEditor } from "../../canvas/editor-session";
import { useSpaceStore } from "../../session/space-store";
import type { CanvasContentLayoutCoordinator } from "./canvas-content-layout-coordinator";
import type { useCanvasNavigationRequests } from "./use-canvas-navigation-requests";

interface SpaceCanvasLifecycleOptions {
  contentLayout: CanvasContentLayoutCoordinator;
  navigation: ReturnType<typeof useCanvasNavigationRequests>;
  spaceId: Readonly<Ref<string>>;
}

export function useSpaceCanvasLifecycle(options: SpaceCanvasLifecycleOptions) {
  const store = useSpaceStore();
  const tools = useCanvasToolStore();
  const error = shallowRef<string>();
  let detachEditor: (() => void) | undefined;
  let detachOverviewObserver: (() => void) | undefined;
  let detachExchange: (() => void) | undefined;

  function reportError(cause: unknown) {
    error.value = getErrorMessage(cause);
  }

  function attachEditor(editor: CanvasEditorPort) {
    detachOverviewObserver?.();
    detachEditor?.();
    detachEditor = registerCanvasEditor(editor);
    let previous = editor.getDocument();
    detachOverviewObserver = editor.observeDocument((document) => {
      if (document.nodes !== previous.nodes || document.relationStyles !== previous.relationStyles)
        store.markCanvasOverviewDirty();
      previous = document;
    });
    options.contentLayout.sync();
    error.value = undefined;
    options.navigation.consumePending();
  }

  function attachExchange(exchange: CanvasExchangePort) {
    detachExchange?.();
    detachExchange = registerCanvasExchangePort(exchange);
  }

  function detach() {
    detachOverviewObserver?.();
    detachOverviewObserver = undefined;
    detachEditor?.();
    detachEditor = undefined;
    detachExchange?.();
    detachExchange = undefined;
  }

  watch(options.spaceId, () => {
    tools.resetTool();
    detach();
    error.value = undefined;
  });
  onBeforeUnmount(() => {
    tools.resetTool();
    detach();
  });
  return { attachEditor, attachExchange, error, reportError };
}
