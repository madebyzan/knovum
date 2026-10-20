import { useQuery } from "@tanstack/vue-query";

import { getLocalModelCatalog } from "./local-resources";
import { LOCAL_RESOURCES_QUERY_KEY } from "./local-resources-query";

export function useLocalModelsQuery() {
  return useQuery({ queryKey: [...LOCAL_RESOURCES_QUERY_KEY, "models"], queryFn: getLocalModelCatalog });
}
