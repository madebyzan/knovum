import { computed, type Ref } from "vue";

import { selectionExportTarget } from "../../assets/export/batch";
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
    const spaceId = store.activeSpaceId;
    const sectionId = state.value.section?.id;
    const content = [...state.value.content];
    const batch = spaceId && content.length ? selectionExportTarget(spaceId, content) : undefined;
    if (sectionId) return sectionTarget(sectionId, batch);
    if (content.length > 1 || selectedNodes.value.length !== 1) return batch;
    const node = selectedNodes.value.length === 1 ? selectedNodes.value[0] : undefined;
    if (!spaceId || !node) return undefined;
    return {
      key: `node:${spaceId}:${node.id}`,
      formats: () => nodeExportFormats(spaceId, node),
      save: (format, onStart) => exportSpaceNode(spaceId, node, format, { onStart }),
    };
  });

  function sectionTarget(sectionId: string, batch: ExportTarget | undefined): ExportTarget {
    return {
      key: `section:${sectionId}:${batch?.key ?? "empty"}`,
      formats: async () => {
        const members = await batch?.formats();
        return {
          formats: [
            ...SECTION_EXPORT_FORMATS.formats,
            ...(members?.formats.filter((format) => format !== "json") ?? []),
          ],
          unavailable: null,
        };
      },
      save: (format, onStart) =>
        format === "json"
          ? exportSpaceSection(store.editSnapshot, store.nodes, sectionId)
          : batch!.save(format, onStart),
    };
  }

  async function run(format: string) {
    if (target.value) await runExport(target.value, format);
  }

  return { target, run };
}
