import { computed, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { SECTION_EXPORT_FORMATS, nodeExportFormats, type ExportFormats } from "../../assets/export/formats";
import { cancelResourceExport } from "../../assets/export/ipc";
import { exportSpaceNode } from "../../assets/export/node";
import { exportSpaceSection } from "../../assets/export/section";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import type { CanvasSelectionProductState } from "./selection/selection-product-state";

/** What the bar can save, and the formats it admits. */
export interface CanvasSelectionExport {
  /** Identifies the listing, so formats are never reused across selections. */
  key: string;
  formats: () => Promise<ExportFormats>;
  /** Writes the file; `onStart` opens the work that can still be cancelled. */
  save: (format: string, onStart: (jobId: string) => void) => Promise<boolean>;
}

/**
 * Exporting is two steps: the bar offers the formats the selection admits, then
 * writes the one that was chosen into a destination the person picks.
 */
export function useCanvasSelectionExport(
  state: Readonly<Ref<CanvasSelectionProductState>>,
  selectedNodes: Readonly<Ref<SpaceCanvasNode[]>>,
) {
  const store = useSpaceStore();
  const { t } = useI18n();

  const target = computed<CanvasSelectionExport | undefined>(() => {
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
    const selected = target.value;
    if (!selected) return;
    let notice: string | number | undefined;
    let saved: boolean;
    try {
      // Encoding a recording or rendering a document takes as long as it takes,
      // so the person who started it can stop it.
      saved = await selected.save(format, (jobId) => {
        notice = toast.loading(t("spaces.importExport.export.running"), {
          duration: Infinity,
          action: { label: t("common.cancel"), onClick: () => void cancelResourceExport(jobId) },
        });
      });
    } finally {
      if (notice !== undefined) toast.dismiss(notice);
    }
    if (saved) toast.success(t("spaces.importExport.export.completed"));
  }

  return { target, run };
}
