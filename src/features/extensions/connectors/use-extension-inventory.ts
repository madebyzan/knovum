import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";

import { connectionQuery, connectorQuery, pluginQuery } from "../../../foundation/extensions/extension-query";

/** The installed connectors, their connections and owning plugins, observed as one inventory. */
export function useExtensionInventory() {
  const connectors = useQuery(connectorQuery);
  const connections = useQuery(connectionQuery);
  const plugins = useQuery(pluginQuery);
  const pluginById = computed(() => new Map((plugins.data.value ?? []).map((plugin) => [plugin.id, plugin])));
  const error = computed(() => connectors.error.value || connections.error.value || plugins.error.value);
  const isLoading = computed(
    () => connectors.isPending.value || connections.isPending.value || plugins.isPending.value,
  );
  return { connectors, connections, pluginById, error, isLoading };
}
