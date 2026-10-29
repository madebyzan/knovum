import { listen } from "@tauri-apps/api/event";
import { onScopeDispose } from "vue";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import { queryClient } from "../../../../foundation/query/query-client";

/**
 * Reads a query again whenever the backend announces that its data changed. The announcement only
 * names what changed; `keyOf` answers which query that is, or nothing when it concerns another one.
 */
export function useInvalidatingEvent<T>(event: string, keyOf: (payload: T) => readonly unknown[] | null) {
  let disposed = false;
  const registration = listen<T>(event, ({ payload }) => {
    const queryKey = keyOf(payload);
    if (queryKey) void queryClient.invalidateQueries({ queryKey });
  });
  void registration.then((unlisten) => (disposed ? unlisten() : undefined)).catch(reportFailure);
  onScopeDispose(() => {
    disposed = true;
    void registration.then((unlisten) => unlisten()).catch(reportFailure);
  });
}

// Without the subscription the view would silently stop following the backend.
function reportFailure(error: unknown) {
  toast.error(getErrorMessage(error));
}
