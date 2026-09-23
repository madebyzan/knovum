import { shallowRef, type Ref } from "vue";

import {
  copyWorkflowTemplate,
  deleteWorkflowTemplate,
  listWorkflowTemplates,
  saveWorkflowTemplate,
  type WorkflowTemplate,
} from "./personal-template-client";
import type { WorkflowDraft } from "./workflow-client";

type Perform = (action: () => Promise<void>) => Promise<void>;

export function useWorkflowTemplates(
  spaceId: Ref<string | undefined>,
  draft: Ref<WorkflowDraft | undefined>,
  perform: Perform,
  selectDraft: (draft: WorkflowDraft) => void,
) {
  const templates = shallowRef<WorkflowTemplate[]>([]);
  const selectedTemplateId = shallowRef("");

  async function refresh() {
    templates.value = await listWorkflowTemplates();
  }

  async function save() {
    if (!draft.value) return;
    await perform(async () => {
      await saveWorkflowTemplate(draft.value!);
      await refresh();
    });
  }

  async function use() {
    if (!spaceId.value || !selectedTemplateId.value) return;
    await perform(async () => {
      selectDraft(await copyWorkflowTemplate(selectedTemplateId.value, spaceId.value!));
    });
  }

  async function remove() {
    if (!selectedTemplateId.value) return;
    await perform(async () => {
      await deleteWorkflowTemplate(selectedTemplateId.value);
      selectedTemplateId.value = "";
      await refresh();
    });
  }

  return { templates, selectedTemplateId, refresh, save, use, remove };
}
