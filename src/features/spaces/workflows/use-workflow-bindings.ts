import { computed, ref, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import { useSpaceStore } from "../session/space-store";
import type { SavedCapturedWorkflow } from "./captured-workflow-ipc";

export function useWorkflowBindings(workflow: Readonly<Ref<SavedCapturedWorkflow | undefined>>) {
  const store = useSpaceStore();
  const { t } = useI18n();
  const bindings = ref<Record<string, string[]>>({});
  const selectedNodes = computed(() => {
    const selectedIds = new Set(store.workflowCaptureSelection);
    return store.nodes.filter(({ id }) => selectedIds.has(id));
  });
  const slots = computed(
    () =>
      workflow.value?.candidate.inputSlots.map((slot, index) => ({
        ...slot,
        label: t("spaces.creation.workflow.inputs.slot", { index: index + 1 }),
        options: selectedNodes.value
          .filter(({ data }) => data.kind === slot.kind)
          .map(({ id, data }) => ({ value: id, label: data.title })),
      })) ?? [],
  );
  const ready = computed(() => {
    if (!slots.value.length) return false;
    const all = Object.values(bindings.value).flat();
    return (
      slots.value.every((slot) => {
        const ids = bindings.value[slot.id] ?? [];
        return (
          ids.length >= slot.minItems &&
          ids.length <= slot.maxItems &&
          ids.every((id) => slot.options.some((option) => option.value === id))
        );
      }) && new Set(all).size === all.length
    );
  });

  function reset() {
    const next: Record<string, string[]> = {};
    for (const slot of slots.value) {
      if (
        !slot.ordered &&
        slot.options.length === 1 &&
        slots.value.filter((other) => other.kind === slot.kind).length === 1
      ) {
        next[slot.id] = [slot.options[0]!.value];
      }
    }
    bindings.value = next;
  }

  function set(slotId: string, nodeIds: string[]) {
    bindings.value = { ...bindings.value, [slotId]: nodeIds };
  }

  return { slots, ready, bindings, reset, set };
}
