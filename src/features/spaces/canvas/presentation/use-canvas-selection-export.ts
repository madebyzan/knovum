import { computed, type Ref } from "vue";

import { SECTION_EXPORT_FORMATS, nodeExportFormats } from "../../assets/export/formats";
import { exportSpaceNode } from "../../assets/export/node";
import { exportSpaceSection } from "../../assets/export/section";
import type { ExportTarget } from "../../assets/export/target";
import { useExportRun } from "../../assets/export/use-export-run";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import type { CanvasSelectionProductState } from "./selection/selection-product-state";

/**
 * Exporting is two steps: the bar offers the formats the selection admits, then
 * writes the one that was chosen into a destination the person picks.
 */
export function useCanvasSelectionExport(
  state: Readonly<Ref<CanvasSelectionProductState>>,
  selectedNodes: Readonly<Ref<SpaceCanvasNode[]>>,
) {
  const store = useSpaceStore();
  const runExport = useExportRun();

  const target = computed<ExportTarget | undefined>(() => {
    const sectionId = state.value.section?.id;
    if (sectionId)
      return {
        key: `section:${sectionId}`,
        formats: () => Promise.resolve(SECTION_EXPORT_FORMATS),
        save: () => exportSpaceSection(store.editSnapshot, store.nodes, sectionId),
      };
    const spaceId = store.activeSpaceId;
    const node = selectedNodes.value.length === 1 ? selectedNodes.value[0] : undefined;
    if (!spaceId || !node) return undefined;
    return {
      key: `node:${spaceId}:${node.id}`,
      formats: () => nodeExportFormats(spaceId, node),
      save: (format, onStart) => exportSpaceNode(spaceId, node, format, { onStart }),
    };
  });

  async function run(format: string) {
    if (target.value) await runExport(target.value, format);
  }

  return { target, run };
}
