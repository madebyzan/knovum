import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { formatBytes } from "../../../foundation/format/bytes";
import { useTaskStore } from "../../../foundation/tasks/task-store";
import type { Space } from "../model/space";
import { useSpaceStore } from "../session/space-store";
import { exportSpacePackage, importSpacePackage, type SpacePackageProgress } from "./transfer";

export function useSpacePackageTransfer() {
  const store = useSpaceStore();
  const tasks = useTaskStore();
  const { t } = useI18n();

  async function run(
    title: string,
    operation: (progress: (value: SpacePackageProgress) => void) => Promise<boolean>,
    success: string,
  ) {
    // The picker is part of this intent; cancellation is not a successful transfer.
    const taskId = tasks.start(title);
    try {
      const completed = await operation(({ percent, processed, total }) => {
        tasks.progress(taskId, percent, `${formatBytes(processed)} / ${formatBytes(total)}`);
      });
      if (completed) {
        tasks.succeed(taskId);
        toast.success(success);
      } else {
        tasks.dismiss(taskId);
      }
    } catch (error) {
      tasks.fail(taskId, String(error));
      toast.error(String(error));
    }
  }

  function exportPackage(space: Pick<Space, "id" | "title">) {
    const target = { id: space.id, title: space.title };
    return run(
      t("spaces.importExport.space.export"),
      (progress) =>
        exportSpacePackage(
          {
            space: target,
            commitActiveSession: store.commitActiveSession,
          },
          progress,
        ),
      t("spaces.importExport.space.exported"),
    );
  }

  function importPackage() {
    return run(
      t("spaces.importExport.space.import"),
      (progress) =>
        importSpacePackage(
          {
            spaces: store.spaces,
            openSpace: store.openSpace,
          },
          progress,
        ),
      t("spaces.importExport.space.imported"),
    );
  }

  return { exportPackage, importPackage };
}
