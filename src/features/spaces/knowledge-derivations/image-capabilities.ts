import { useQuery } from "@tanstack/vue-query";

import { loadImageLayerCapability } from "./image-layers";
import { loadImageOperationCapabilities } from "./image-operation";

export function useImageLayerCapability() {
  return useQuery({
    queryKey: ["spaces", "image-layer-capability"],
    queryFn: loadImageLayerCapability,
  });
}

export function useImageOperationCapabilities() {
  return useQuery({
    queryKey: ["spaces", "image-operation-capabilities"],
    queryFn: loadImageOperationCapabilities,
  });
}
