import { useQuery, useQueryClient } from "@tanstack/vue-query";
import { computed, toValue, type MaybeRefOrGetter } from "vue";

import {
  listCapturedWorkflows,
  listCapturedWorkflowRuns,
  type CapturedWorkflowRun,
  type SavedCapturedWorkflow,
} from "./captured-workflow-ipc";

const LIBRARY_KEY = ["spaces", "captured-workflows"] as const;
const RUN_HISTORY_LIMIT = 30;

function runsKey(spaceId: string | null | undefined) {
  return ["spaces", spaceId ?? null, "captured-workflow-runs"] as const;
}

export function useCapturedWorkflowLibrary(spaceId: MaybeRefOrGetter<string | null | undefined>) {
  const queryClient = useQueryClient();
  const library = useQuery({
    queryKey: LIBRARY_KEY,
    queryFn: async () => (await listCapturedWorkflows()).items,
  });
  const runs = useQuery(
    computed(() => {
      const id = toValue(spaceId);
      return {
        queryKey: runsKey(id),
        queryFn: () => (id ? listCapturedWorkflowRuns(id) : []),
        enabled: Boolean(id),
      };
    }),
  );

  function editLibrary(update: (current: SavedCapturedWorkflow[]) => SavedCapturedWorkflow[]) {
    queryClient.setQueryData<SavedCapturedWorkflow[]>(LIBRARY_KEY, (current) => update(current ?? []));
  }

  function addRun(run: CapturedWorkflowRun) {
    queryClient.setQueryData<CapturedWorkflowRun[]>(runsKey(toValue(spaceId)), (current) =>
      [run, ...(current ?? [])].slice(0, RUN_HISTORY_LIMIT),
    );
  }

  return {
    items: computed(() => library.data.value ?? []),
    history: computed(() => runs.data.value ?? []),
    loading: computed(() => library.isLoading.value || runs.isLoading.value),
    error: computed(() => {
      if (!toValue(spaceId)) return "Space is unavailable";
      const cause = library.error.value ?? runs.error.value;
      return cause ? String(cause) : "";
    }),
    renamed: (value: SavedCapturedWorkflow) =>
      editLibrary((current) => current.map((item) => (item.workflowId === value.workflowId ? value : item))),
    deleted: (workflowId: string) =>
      editLibrary((current) => current.filter((item) => item.workflowId !== workflowId)),
    addRun,
  };
}
