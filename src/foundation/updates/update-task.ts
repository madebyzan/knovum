import type { UpdateSnapshot } from "../../shared/contracts/generated/UpdateSnapshot";
import { i18n } from "../i18n/i18n";
import { useTaskStore } from "../tasks/task-store";

/** The native update session owns execution; Task Center only mirrors its download. */
export function createUpdateTask(cancel: () => Promise<void>) {
  const tasks = useTaskStore();
  const t = i18n.global.t;
  let id: string | undefined;

  return (snapshot: UpdateSnapshot) => {
    if (snapshot.phase === "downloading" || snapshot.phase === "verifying") {
      id ??= tasks.start(t("settings.updates.title"), undefined, cancel);
      const verifying = snapshot.phase === "verifying";
      tasks.setCancellation(id, verifying ? undefined : cancel);
      const message = verifying
        ? t("settings.updates.download.verifying")
        : t("settings.updates.download.downloading");
      if (snapshot.total && !verifying) {
        tasks.progress(id, (snapshot.downloaded / snapshot.total) * 100, message);
      } else tasks.indeterminate(id, message);
      return;
    }
    if (!id) return;
    if (snapshot.phase === "ready") tasks.succeed(id);
    else if (snapshot.failure) tasks.fail(id, t("settings.updates.download.failed"));
    else tasks.cancelled(id);
    id = undefined;
  };
}
