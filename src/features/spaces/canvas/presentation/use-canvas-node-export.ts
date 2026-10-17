import type { Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { cancelResourceExport } from "../../assets/export/ipc";
import { exportSpaceNode } from "../../assets/export/node";
import { exportSpaceSection } from "../../assets/export/section";
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
    let notice: string | number | undefined;
    let saved: boolean;
    try {
      if (sectionId) saved = await exportSpaceSection(store.editSnapshot, store.nodes, sectionId);
      else if (node && store.activeSpaceId)
        // Encoding a recording or rendering a document takes as long as it
        // takes, so the person who started it can stop it.
        saved = await exportSpaceNode(store.activeSpaceId, node, {
          onStart: (jobId) => {
            notice = toast.loading(t("spaces.importExport.export.running"), {
              duration: Infinity,
              action: {
                label: t("common.cancel"),
                onClick: () => void cancelResourceExport(jobId),
              },
            });
          },
        });
      else return;
    } finally {
      if (notice !== undefined) toast.dismiss(notice);
    }
    if (saved) toast.success(t("spaces.importExport.export.completed"));
  };
}
