import { useDebounceFn } from "@vueuse/core";
import { computed, onMounted, ref, shallowRef, watch, type Ref } from "vue";

import { useWorkflowExecution } from "./use-workflow-execution";
import { useWorkflowHistory } from "./use-workflow-history";
import { useWorkflowTemplates } from "./use-workflow-templates";
import {
  listWorkflowDrafts,
  listWorkflowVersions,
  extractWorkflowDraftFromHistory,
  deleteWorkflowDraft,
  newWorkflowDraft,
  saveWorkflowDraft,
  type WorkflowDraft,
  type WorkflowNodeKind,
  type WorkflowVersion,
} from "./workflow-client";
import {
  addWorkflowNode,
  connectWorkflowNodes,
  copyWorkflowVersion,
  duplicateWorkflowDraft,
  removeWorkflowNode,
} from "./workflow-draft-operations";

export function useWorkflowEditor(spaceId: Ref<string | undefined>) {
  const drafts = shallowRef<WorkflowDraft[]>([]);
  const versions = shallowRef<WorkflowVersion[]>([]);
  const draft = ref<WorkflowDraft>();
  const busy = shallowRef(false);
  const error = shallowRef<string>();
  const execution = useWorkflowExecution(draft, perform);
  const history = useWorkflowHistory(draft);
  const templateLibrary = useWorkflowTemplates(spaceId, draft, perform, selectDraft);
  const selectedId = computed({
    get: () => draft.value?.id ?? "",
    set: (id) => {
      const selected = drafts.value.find((item) => item.id === id);
      if (selected) selectDraft(selected);
    },
  });

  async function refresh() {
    if (!spaceId.value) return;
    [drafts.value, versions.value] = await Promise.all([
      listWorkflowDrafts(spaceId.value),
      listWorkflowVersions(spaceId.value),
      templateLibrary.refresh(),
    ]);
    if (!draft.value || draft.value.spaceId !== spaceId.value) {
      const first = drafts.value[0];
      if (first) selectDraft(first);
      else draft.value = undefined;
    }
  }

  function create() {
    if (spaceId.value) selectDraft(newWorkflowDraft(spaceId.value));
  }

  function duplicate() {
    if (!draft.value || !spaceId.value) return;
    selectDraft(duplicateWorkflowDraft(draft.value));
  }

  async function extractHistory() {
    if (!spaceId.value) return;
    await perform(async () => {
      const extracted = await extractWorkflowDraftFromHistory(spaceId.value!, "Workflow from history");
      selectDraft(extracted);
    });
  }

  function copyVersion(version: WorkflowVersion) {
    if (!spaceId.value) return;
    selectDraft(copyWorkflowVersion(spaceId.value, version));
  }

  async function removeDraft() {
    if (!draft.value || draft.value.revision === 0) {
      draft.value = undefined;
      return;
    }
    await perform(async () => {
      await deleteWorkflowDraft(draft.value!.id);
      draft.value = undefined;
      await refresh();
    });
  }

  function addNode(kind: WorkflowNodeKind) {
    if (!draft.value) return;
    addWorkflowNode(draft.value, kind);
  }

  async function save() {
    if (!draft.value) return;
    await perform(async () => {
      draft.value = await saveWorkflowDraft(draft.value!);
      history.markSaved();
      await refresh();
    });
  }

  async function perform(action: () => Promise<void>) {
    busy.value = true;
    error.value = undefined;
    try {
      await action();
    } catch (reason) {
      error.value = String(reason);
    } finally {
      busy.value = false;
    }
  }

  watch(spaceId, refresh);
  onMounted(refresh);
  const autosave = useDebounceFn(async () => {
    if (!draft.value || busy.value) return;
    if (!history.needsSave(draft.value.definition)) return;
    await save();
  }, 800);
  watch(
    () => draft.value?.definition,
    (definition) => {
      if (!definition || !history.track(definition)) return;
      void autosave();
    },
    { deep: true },
  );
  function removeNode(nodeId: string) {
    if (!draft.value || ["input", "output"].includes(nodeId)) return;
    removeWorkflowNode(draft.value, nodeId);
  }

  function connectNodes(sourceNodeId: string, targetNodeId: string) {
    if (draft.value) connectWorkflowNodes(draft.value, sourceNodeId, targetNodeId);
  }

  function selectDraft(value: WorkflowDraft) {
    draft.value = value;
    history.reset(value);
    execution.reset(value);
  }

  return {
    drafts,
    versions,
    templates: templateLibrary.templates,
    selectedTemplateId: templateLibrary.selectedTemplateId,
    draft,
    selectedId,
    busy,
    error,
    undoStack: history.undoStack,
    redoStack: history.redoStack,
    create,
    duplicate,
    extractHistory,
    saveTemplate: templateLibrary.save,
    useTemplate: templateLibrary.use,
    removeTemplate: templateLibrary.remove,
    copyVersion,
    removeDraft,
    addNode,
    removeNode,
    connectNodes,
    save,
    undo: () => history.restore("undo"),
    redo: () => history.restore("redo"),
    ...execution,
  };
}
