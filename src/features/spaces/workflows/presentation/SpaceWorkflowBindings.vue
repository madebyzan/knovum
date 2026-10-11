<script setup lang="ts">
import { useI18n } from "vue-i18n";

import type { SelectOption } from "../../../../shared/ui/UiSelect.vue";
import type { CaptureCandidate } from "../../contracts/generated/CaptureCandidate";
import SpaceWorkflowBinding from "./SpaceWorkflowBinding.vue";

type BindingSlot = CaptureCandidate["inputSlots"][number] & {
  label: string;
  options: SelectOption[];
};
defineProps<{ slots: BindingSlot[]; bindings: Record<string, string[]> }>();
const emit = defineEmits<{ change: [slotId: string, nodeIds: string[]] }>();
const { t } = useI18n();
</script>

<template>
  <section class="space-y-3">
    <h3 class="text-caption font-semibold text-fg-secondary">
      {{ t("spaces.creation.workflow.inputs.bind") }}
    </h3>
    <SpaceWorkflowBinding
      v-for="slot in slots"
      :key="slot.id"
      :slot="slot"
      :label="slot.label"
      :options="slot.options"
      :model-value="bindings[slot.id] ?? []"
      @update:model-value="emit('change', slot.id, $event)"
    />
    <p v-if="slots.some((slot) => !slot.options.length)" class="text-caption text-fg-warning">
      {{ t("spaces.creation.workflow.inputs.missing") }}
    </p>
  </section>
</template>
