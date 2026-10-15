import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { decodeAppError } from "../../../foundation/errors/app-error";
import { getErrorMessage } from "../../../foundation/errors/error-format";
import { formatBytes } from "../../../foundation/format/bytes";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { SpacePackageProgress } from "../contracts/generated/SpacePackageProgress";
import type { Space } from "../model/space";
import { useSpaceStore } from "../session/space-store";
import {
  cancelSpacePackageTransfer,
  chooseSpacePackageExportDestination,
  chooseSpacePackageImportSource,
  exportSpacePackage,
  importSpacePackage,
  type SpacePackageTransfer,
} from "./transfer";

export function useSpacePackageTransfer() {
  const store = useSpaceStore();
  const tasks = useTaskStore();
  const { t } = useI18n();

  async function run(
    title: string,
    operation: (operationId: string, progress: (value: SpacePackageProgress) => void) => SpacePackageTransfer,
    success: string,
  ) {
    const operationId = crypto.randomUUID();
    const taskId = operationId;
    const transfer = operation(operationId, ({ percent, processed, total }) => {
      tasks.progress(taskId, percent, `${formatBytes(processed)} / ${formatBytes(total)}`);
    });
    tasks.start(
      title,
      undefined,
      async () => {
        await transfer.admitted;
        await cancelSpacePackageTransfer(operationId);
        await transfer.completion.catch(() => undefined);
      },
      taskId,
    );
    try {
      await transfer.completion;
      tasks.succeed(taskId);
      toast.success(success);
    } catch (error) {
      if (decodeAppError(error)?.code === "cancelled") {
        tasks.cancelled(taskId);
        return;
      }
      tasks.fail(taskId, getErrorMessage(error));
      toast.error(getErrorMessage(error));
    }
  }

  async function exportPackage(space: Pick<Space, "id" | "title"> & { kind?: Space["kind"] }) {
    const target = {
      id: space.id,
      title: space.kind === "inbox" ? t("spaces.space.inbox") : space.title,
    };
    const destination = await chooseSpacePackageExportDestination(target.title);
    if (!destination) return;
    await store.commitActiveSession();
    await run(
      t("spaces.importExport.space.export"),
      (operationId, progress) => exportSpacePackage({ space: target }, destination, operationId, progress),
      t("spaces.importExport.space.exported"),
    );
  }

  async function importPackage() {
    const source = await chooseSpacePackageImportSource();
    if (!source) return;
    await run(
      t("spaces.importExport.space.import"),
      (operationId, progress) =>
        importSpacePackage(
          {
            spaces: store.spaces,
            openSpace: store.openSpace,
          },
          source,
          operationId,
          progress,
        ),
      t("spaces.importExport.space.imported"),
    );
  }

  return { exportPackage, importPackage };
}
