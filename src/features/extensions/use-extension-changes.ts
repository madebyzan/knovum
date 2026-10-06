import { useMutation, useQueryClient } from "@tanstack/vue-query";

import { extensionQueryKey } from "../../foundation/extensions/extension-query";

export function useExtensionChanges() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (operation: () => Promise<unknown>) => operation(),
    onSuccess: () => client.invalidateQueries({ queryKey: extensionQueryKey }),
  });
}
