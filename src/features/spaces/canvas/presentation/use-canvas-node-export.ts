import type { Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { exportSpaceNode, exportSpaceSection } from "../../assets/node-export";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import type { CanvasSelectionProductState } from "./selection/selection-product-state";

export function useCanvasNodeExport(
  state: Readonly<Ref<CanvasSelectionProductState>>,
  selectedNodes: Readonly<Ref<SpaceCanvasNode[]>>,
) {
  const store = useSpaceStore();
  const { t } = useI18n();

  return async () => {
    const sectionId = state.value.section?.id;
    const node = selectedNodes.value.length === 1 ? selectedNodes.value[0] : undefined;
    let saved: boolean;
    if (sectionId) saved = await exportSpaceSection(store.editSnapshot, store.nodes, sectionId);
    else if (node && store.activeSpaceId) saved = await exportSpaceNode(store.activeSpaceId, node);
    else return;
    if (saved) toast.success(t("spaces.importExport.export.completed"));
  };
}
