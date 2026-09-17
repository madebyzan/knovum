import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";

import { detectAgentInstallations, loadAgentDescriptors } from "./agents";

const agentDescriptorsQueryKey = ["integrations", "agent-descriptors"] as const;
const agentInstallationsQueryKey = ["integrations", "agent-installations"] as const;

export function useExecutionCatalogQuery(options: { detectInstallations?: boolean } = {}) {
  const descriptors = useQuery({
    queryKey: agentDescriptorsQueryKey,
    queryFn: loadAgentDescriptors,
    staleTime: 30_000,
  });
  const installations = useQuery({
    queryKey: agentInstallationsQueryKey,
    queryFn: detectAgentInstallations,
    enabled: options.detectInstallations === true,
    staleTime: 30_000,
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
    async refetchDescriptors() {
      return descriptors.refetch();
    },
    async refreshInstallations() {
      return installations.refetch();
    },
    async refetch() {
      const descriptorResult = await descriptors.refetch();
      return {
        data: descriptorResult.data
          ? { agents: descriptorResult.data, installations: installations.data.value ?? [] }
          : undefined,
        error: descriptorResult.error,
      };
    },
  };
}
