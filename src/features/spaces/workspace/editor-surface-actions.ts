import type { SpaceSessionContext } from "./space-session-context";
import { discardWorkbenchTab } from "./workbench-close-actions";
import { openEditorTab } from "./workbench-navigation-actions";

/** Editor intents that need the editor session owners; generic tab intents stay with the strip. */
export function createEditorSurfaceActions(context: SpaceSessionContext) {
  return {
    openDocumentEditor: async (nodeId: string) => {
      await openEditorTab(context, "document", nodeId);
    },
    openVideoEditor: async (nodeId: string) => {
      await openEditorTab(context, "video", nodeId);
    },
    discardUnavailableEditorTab: (tabId: string) => discardUnavailableEditorTab(context, tabId),
  };
}

/** A failed resource is the only reason a draft may be dropped without a save decision. */
async function discardUnavailableEditorTab(context: SpaceSessionContext, tabId: string) {
  const tab = context.editorTabs.value.find(({ id }) => id === tabId);
  if (!tab || !context.editors.canDiscard(tab))
    throw new Error("Editor draft cannot be discarded without a resource error");
  await discardWorkbenchTab(context, tab);
}
