import { useMutation, useQueryClient } from "@tanstack/vue-query";

import { EXTENSION_QUERY_KEY } from "../../foundation/extensions/extension-query";
import type { ExtensionChange } from "../../shared/contracts/generated/ExtensionChange";

export function useExtensionChanges() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: (operation: () => Promise<ExtensionChange<unknown> | void>) => operation(),
    onSettled: () => client.invalidateQueries({ queryKey: EXTENSION_QUERY_KEY }),
  });
}
