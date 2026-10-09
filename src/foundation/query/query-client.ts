import { QueryClient } from "@tanstack/vue-query";

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: { retry: 1, refetchOnWindowFocus: true, staleTime: 30_000, networkMode: "always" },
    mutations: { retry: 0, networkMode: "always" },
  },
});
