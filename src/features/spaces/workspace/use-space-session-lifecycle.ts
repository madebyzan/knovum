import { error as logError } from "@tauri-apps/plugin-log";
import { onBeforeUnmount } from "vue";
import { onBeforeRouteLeave } from "vue-router";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { registerWindowCloseCommit } from "../../../foundation/lifecycle/window-close-coordinator";

export function useSpaceSessionLifecycle(commit: () => Promise<void>) {
  onBeforeRouteLeave(async () => {
    try {
      await commit();
      return true;
    } catch (error) {
      const message = getErrorMessage(error);
      void logError(`Space commit prevented navigation: ${message}`);
      toast.error(message);
      return false;
    }
  });

  const detachCloseCommit = registerWindowCloseCommit(commit);
  onBeforeUnmount(detachCloseCommit);
}
