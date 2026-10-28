import { queryOptions, useQuery } from "@tanstack/vue-query";
import { computed } from "vue";

import { detectExternalAgentInstallations, loadExternalAgentDescriptors } from "./external-agents";

export const agentDescriptorsQueryOptions = queryOptions({
  queryKey: ["integrations", "agent-descriptors"] as const,
  queryFn: loadExternalAgentDescriptors,
  staleTime: 30_000,
});

export const agentInstallationsQueryOptions = queryOptions({
  queryKey: ["integrations", "agent-installations"] as const,
  queryFn: detectExternalAgentInstallations,
  staleTime: 30_000,
});

export function useExecutionCatalogQuery(options: { detectInstallations?: boolean } = {}) {
  const descriptors = useQuery(agentDescriptorsQueryOptions);
  const installations = useQuery({
    ...agentInstallationsQueryOptions,
    enabled: options.detectInstallations === true,
  });

  return {
    data: computed(() =>
      descriptors.data.value
        ? { agents: descriptors.data.value, installations: installations.data.value ?? [] }
        : undefined,
    ),
    descriptorError: descriptors.error,
    descriptorPending: descriptors.isPending,
    descriptorFetching: descriptors.isFetching,
    installationError: installations.error,
    installationPending: installations.isPending,
    installationFetching: installations.isFetching,
    installationInspected: computed(
      () => installations.data.value !== undefined || installations.error.value !== null,
    ),
    error: descriptors.error,
    isError: descriptors.isError,
    isPending: descriptors.isPending,
    isFetching: computed(() => descriptors.isFetching.value || installations.isFetching.value),
    refetchDescriptors: descriptors.refetch,
    refreshInstallations: installations.refetch,
    async refetch() {
      const descriptorRefresh = await descriptors.refetch();
      return {
        data: descriptorRefresh.data
          ? { agents: descriptorRefresh.data, installations: installations.data.value ?? [] }
          : undefined,
        error: descriptorRefresh.error,
      };
    },
  };
}
