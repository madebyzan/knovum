import { computed, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import type { SavedCapturedWorkflow } from "../contracts/generated/SavedCapturedWorkflow";
import { useSpaceStore } from "../session/space-store";

export function useWorkflowChoices(items: Readonly<Ref<SavedCapturedWorkflow[]>>) {
  const store = useSpaceStore();
  const { t } = useI18n();
  return computed(() => {
    const selectedIds = new Set(store.workflowCaptureSelection);
    const availableCountByKind = new Map<string, number>();
    for (const node of store.nodes) {
      if (!selectedIds.has(node.id)) continue;
      availableCountByKind.set(node.data.kind, (availableCountByKind.get(node.data.kind) ?? 0) + 1);
    }
    return items.value.map((item) => {
      const requiredCountByKind = new Map<string, number>();
      for (const slot of item.candidate.inputSlots) {
        requiredCountByKind.set(slot.kind, (requiredCountByKind.get(slot.kind) ?? 0) + slot.minItems);
      }
      const compatible = [...requiredCountByKind].every(
        ([kind, count]) => (availableCountByKind.get(kind) ?? 0) >= count,
      );
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
