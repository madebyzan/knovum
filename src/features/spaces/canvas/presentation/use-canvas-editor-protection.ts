import { computed, type Ref } from "vue";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { useSpaceStore } from "../../session/space-store";
import { editorsWithinStructure } from "../../workspace/editor-resource-protection";
import type { CanvasSelectionSnapshot } from "../contracts/canvas-editor-port";

export function useCanvasEditorProtection(
  selection: Readonly<Ref<CanvasSelectionSnapshot>>,
  roots: () => readonly string[],
) {
  const store = useSpaceStore();
  const tabs = computed(() =>
    editorsWithinStructure(store.editorTabs, store.activeSpaceId, store.editSnapshot, roots()),
  );
  return {
    tabs,
    canRemove: computed(() => selection.value.relationIds.length > 0 || tabs.value.length === 0),
    canTransfer: computed(() => tabs.value.length === 0),
    goToEditor: () => {
      const tab = tabs.value[0];
      if (tab) void store.activateTab(tab.id).catch((error: unknown) => toast.error(getErrorMessage(error)));
    },
  };
}
