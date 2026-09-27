import { queryOptions, useQuery } from "@tanstack/vue-query";
import type { Ref } from "vue";

import { getImageLayerAvailability } from "./image-layers";
import { getImageOperationCapabilities } from "./image-operation";

function capabilityQuery<T>(key: string, queryFn: () => Promise<T>, open: Readonly<Ref<boolean>>) {
  return queryOptions({
    queryKey: ["spaces", key] as const,
    queryFn,
    enabled: open,
    staleTime: 0,
    retry: false,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
  });
}

export function useImageLayerAvailability(open: Readonly<Ref<boolean>>) {
  return useQuery(capabilityQuery("image-layer-availability", getImageLayerAvailability, open));
}

export function useImageOperationCapabilities(open: Readonly<Ref<boolean>>) {
  return useQuery(capabilityQuery("image-operation-capabilities", getImageOperationCapabilities, open));
}
