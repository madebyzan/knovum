import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { cancelResourceExport } from "./ipc";
import type { ExportTarget } from "./target";

/**
 * Saves a target in the chosen format and tells the person how it went. Encoding a recording or
 * rendering a document takes as long as it takes, so work that reports a job can be stopped.
 */
export function useExportRun() {
  const { t } = useI18n();
  return async function run(target: ExportTarget, format: string) {
    let notice: string | number | undefined;
    let saved: boolean;
    try {
      saved = await target.save(format, (jobId) => {
        notice = toast.loading(t("spaces.importExport.export.running"), {
          duration: Infinity,
          action: { label: t("common.cancel"), onClick: () => void cancelResourceExport(jobId) },
        });
      });
    } finally {
      if (notice !== undefined) toast.dismiss(notice);
    }
    if (saved) toast.success(t("spaces.importExport.export.completed"));
  };
}
