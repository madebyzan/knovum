import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { importSpaceFromPicker, type SpaceImportPlacement } from "./import-flow";

/** Importing a file the person picks, with the progress and failure every entry point shows. */
export function usePickerImport() {
  const { t } = useI18n();
  return async function importFromPicker(placement?: SpaceImportPlacement): Promise<SpaceCanvasNode[]> {
    try {
      return await importSpaceFromPicker(
        {
          title: t("spaces.importExport.progress.resource"),
          preparing: t("spaces.importExport.progress.preparing"),
        },
        placement,
      );
    } catch (error) {
      toast.error(t("spaces.importExport.import.failed"), { description: getErrorMessage(error) });
      return [];
    }
  };
}
