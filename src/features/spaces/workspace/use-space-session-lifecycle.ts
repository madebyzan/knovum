import { error as logError } from "@tauri-apps/plugin-log";
import { onBeforeUnmount } from "vue";
import { onBeforeRouteLeave } from "vue-router";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import { registerWindowCloseCommit } from "../../../foundation/lifecycle/window-close-coordinator";
import { isApplicationLeaveCancelled } from "../../../shared/contracts/application-leave";

export function useSpaceSessionLifecycle(commit: () => Promise<void>) {
  onBeforeRouteLeave(async () => {
    try {
      await commit();
      return true;
    } catch (error) {
      if (isApplicationLeaveCancelled(error)) return false;
      const message = getErrorMessage(error);
      void logError(`Space commit prevented navigation: ${message}`);
      toast.error(message);
      return false;
    }
  });

  const detachCloseCommit = registerWindowCloseCommit(commit);
  onBeforeUnmount(detachCloseCommit);
}
