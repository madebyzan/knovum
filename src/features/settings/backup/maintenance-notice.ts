import { error as logError } from "@tauri-apps/plugin-log";
import type { ComposerTranslation } from "vue-i18n";
import { toast } from "vue-sonner";
import { z } from "zod";

import { formatErrorDiagnostic, getErrorMessage } from "../../../foundation/errors/error-format";
import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import { maintenanceOperations, maintenanceStatuses } from "../contracts/generated/contract-values";
import type { MaintenanceReceipt } from "../contracts/generated/MaintenanceReceipt";

const receiptSchema: z.ZodType<MaintenanceReceipt> = z
  .object({
    operation: z.enum(maintenanceOperations),
    status: z.enum(maintenanceStatuses),
    message: z.string(),
    credentialsReconfigurationRequired: z.boolean(),
    cleanupWarning: z.string().nullable(),
  })
  .strict();

export async function showBackupMaintenanceNotice(t: ComposerTranslation): Promise<void> {
  try {
    const raw = await invokeCommand<unknown>("plugin:backup|get_maintenance_receipt");
    if (raw === null) return;
    const receipt = receiptSchema.parse(raw);
    const title = t(`settings.data.receipt.${receipt.operation}.${receipt.status}`);
    const description =
      receipt.status === "failed"
        ? receipt.message
        : [
            t("settings.data.receipt.exclusions"),
            ...(receipt.credentialsReconfigurationRequired ? [t("settings.data.receipt.credentials")] : []),
            ...(receipt.cleanupWarning ? [receipt.cleanupWarning] : []),
          ].join("\n");
    const options = {
      description,
      duration: Infinity,
      action: {
        label: t("common.close"),
        onClick: () => void acknowledgeReceipt(t),
      },
    };
    if (receipt.cleanupWarning) toast.warning(title, options);
    else if (receipt.status === "success") toast.success(title, options);
    else toast.error(title, options);
  } catch (error) {
    showReadFailure(t, error);
  }
}

async function acknowledgeReceipt(t: ComposerTranslation) {
  try {
    await invokeCommand("plugin:backup|acknowledge_maintenance_receipt");
  } catch (error) {
    toast.error(t("settings.data.receipt.acknowledgeFailed"), {
      id: "backup-receipt-acknowledge-failed",
      description: getErrorMessage(error),
      duration: Infinity,
      action: { label: t("common.retry"), onClick: () => void acknowledgeReceipt(t) },
    });
    void logError(`Failed to acknowledge backup maintenance receipt: ${formatErrorDiagnostic(error)}`);
  }
}

function showReadFailure(t: ComposerTranslation, error: unknown) {
  toast.error(t("settings.data.receipt.readFailed"), {
    id: "backup-receipt-read-failed",
    description: getErrorMessage(error),
    duration: Infinity,
    action: { label: t("common.retry"), onClick: () => void showBackupMaintenanceNotice(t) },
  });
  void logError(`Failed to read backup maintenance receipt: ${formatErrorDiagnostic(error)}`);
}
