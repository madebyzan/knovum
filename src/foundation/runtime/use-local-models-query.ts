import { queryOptions, useQuery } from "@tanstack/vue-query";
import type { MaybeRefOrGetter } from "vue";
import { computed, toValue } from "vue";

import { getLocalModelCatalog } from "./local-resources";
import { LOCAL_RESOURCES_QUERY_KEY } from "./local-resources-query";

export const localModelsQueryOptions = queryOptions({
  queryKey: [...LOCAL_RESOURCES_QUERY_KEY, "models"] as const,
  queryFn: getLocalModelCatalog,
});

export function useLocalModelsQuery(enabled: MaybeRefOrGetter<boolean> = true) {
  return useQuery(computed(() => ({ ...localModelsQueryOptions, enabled: toValue(enabled) })));
}
