import { queryClient } from "../query/query-client";

export const LOCAL_RESOURCES_QUERY_KEY = ["runtime", "local-resources"] as const;

export function invalidateLocalResources(): Promise<void> {
  return queryClient.invalidateQueries({ queryKey: LOCAL_RESOURCES_QUERY_KEY });
}
