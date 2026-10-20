import { useMutation } from "@tanstack/vue-query";
import type { ShallowRef } from "vue";

import { checkMcpConnection } from "../../../foundation/extensions/connection-operations";
import type { McpConnection } from "../../../shared/contracts/generated/McpConnection";
import type { McpConnectionCatalog } from "../../../shared/contracts/generated/McpConnectionCatalog";

/** A late probe cannot replace the catalog of a different connection revision. */
export function useMcpCatalogCheck(
  current: () => McpConnection,
  catalog: ShallowRef<McpConnectionCatalog | undefined>,
) {
  return useMutation({
    mutationFn: (connection: McpConnection) => checkMcpConnection(connection.id),
    onSuccess: (discovered, connection) => {
      if (connection.id !== current().id || connection.revision !== current().revision) return;
      catalog.value = discovered;
    },
  });
}
