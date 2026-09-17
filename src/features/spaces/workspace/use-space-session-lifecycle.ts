import { onBeforeUnmount } from "vue";
import { onBeforeRouteLeave } from "vue-router";

import { registerWindowCloseCommit } from "../../../foundation/lifecycle/window-close-coordinator";

export function useSpaceSessionLifecycle(commit: () => Promise<void>) {
  onBeforeRouteLeave(async () => {
    try {
      await commit();
      return true;
    } catch {
      return false;
    }
  });

  const detachCloseCommit = registerWindowCloseCommit(commit);
  onBeforeUnmount(detachCloseCommit);
}
