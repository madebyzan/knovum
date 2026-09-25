import { ref, shallowRef } from "vue";

import {
  listCapturedWorkflows,
  listCapturedWorkflowRuns,
  type CapturedWorkflowRun,
  type SavedCapturedWorkflow,
} from "./captured-workflow-ipc";

export function useCapturedWorkflowLibrary() {
  const items = shallowRef<SavedCapturedWorkflow[]>([]);
  const history = shallowRef<CapturedWorkflowRun[]>([]);
  const loading = ref(true);

  async function load(spaceId: string) {
    loading.value = true;
    try {
      const [workflows, runs] = await Promise.all([
        listCapturedWorkflows(),
        listCapturedWorkflowRuns(spaceId),
      ]);
      items.value = workflows.items;
      history.value = runs;
    } finally {
      loading.value = false;
    }
  }

  function renamed(value: SavedCapturedWorkflow) {
    items.value = items.value.map((item) => (item.workflowId === value.workflowId ? value : item));
  }

  function deleted(workflowId: string) {
    items.value = items.value.filter((item) => item.workflowId !== workflowId);
  }

  function addRun(run: CapturedWorkflowRun) {
    history.value = [run, ...history.value].slice(0, 30);
  }

  return { items, history, loading, load, renamed, deleted, addRun };
}
