import { computed, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import { useSpaceStore } from "../session/space-store";
import type { SavedCapturedWorkflow } from "./captured-workflow-ipc";

export function useWorkflowChoices(items: Readonly<Ref<SavedCapturedWorkflow[]>>) {
  const store = useSpaceStore();
  const { t } = useI18n();
  return computed(() => {
    const selectedIds = new Set(store.workflowCaptureSelection);
    const available = new Map<string, number>();
    for (const node of store.nodes) {
      if (!selectedIds.has(node.id)) continue;
      available.set(node.data.kind, (available.get(node.data.kind) ?? 0) + 1);
    }
    return items.value.map((item) => {
      const required = new Map<string, number>();
      for (const slot of item.candidate.inputSlots) {
        required.set(slot.kind, (required.get(slot.kind) ?? 0) + slot.minItems);
      }
      const compatible = [...required].every(([kind, count]) => (available.get(kind) ?? 0) >= count);
      return {
        value: item.workflowId,
        label: item.name,
        disabled: !compatible,
        description: compatible
          ? t("spaces.creation.workflow.summary", {
              steps: item.candidate.steps.length,
              outputs: item.candidate.outputSlots.length,
            })
          : t("spaces.creation.workflow.inputs.missing"),
      };
    });
  });
}
