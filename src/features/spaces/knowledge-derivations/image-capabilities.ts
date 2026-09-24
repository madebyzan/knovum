import { useQuery } from "@tanstack/vue-query";
import type { Ref } from "vue";

import { loadImageLayerCapability } from "./image-layers";
import { loadImageOperationCapabilities } from "./image-operation";

export function useImageLayerCapability(open: Readonly<Ref<boolean>>) {
  return useQuery({
    queryKey: ["spaces", "image-layer-capability"],
    queryFn: loadImageLayerCapability,
    enabled: open,
    staleTime: 0,
    retry: false,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
  });
}

export function useImageOperationCapabilities(open: Readonly<Ref<boolean>>) {
  return useQuery({
    queryKey: ["spaces", "image-operation-capabilities"],
    queryFn: loadImageOperationCapabilities,
    enabled: open,
    staleTime: 0,
    retry: false,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
  });
}
