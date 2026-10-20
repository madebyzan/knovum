import { useQuery } from "@tanstack/vue-query";

import { getLocalResourcesStatus } from "./local-resources";
import { LOCAL_RESOURCES_QUERY_KEY } from "./local-resources-query";

export function useLocalResourcesQuery() {
  return useQuery({ queryKey: LOCAL_RESOURCES_QUERY_KEY, queryFn: getLocalResourcesStatus });
}
