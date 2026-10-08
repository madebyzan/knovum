import { useQuery } from "@tanstack/vue-query";

import { queryClient } from "../query/query-client";
import { getLocalResourcesStatus, getLocalModelCatalog } from "./local-resources";

const LOCAL_RESOURCES_QUERY_KEY = ["runtime", "local-resources"] as const;

export function useLocalResourcesQuery() {
  return useQuery({ queryKey: LOCAL_RESOURCES_QUERY_KEY, queryFn: getLocalResourcesStatus });
}

export function invalidateLocalResources(): Promise<void> {
  return queryClient.invalidateQueries({ queryKey: LOCAL_RESOURCES_QUERY_KEY });
}

export function useLocalModelsQuery() {
  return useQuery({ queryKey: [...LOCAL_RESOURCES_QUERY_KEY, "models"], queryFn: getLocalModelCatalog });
}
