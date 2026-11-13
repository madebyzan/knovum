<script setup lang="ts">
import { Layers3 } from "@lucide/vue";
import { reactive, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { ImageActionAvailability } from "../../canvas/presentation/images/image-action-availability";
import { DERIVATION_LABEL_KEY } from "../../canvas/presentation/images/image-derivation-actions";
import SpaceCanvasImageActionTrigger from "../../canvas/presentation/images/SpaceCanvasImageActionTrigger.vue";
import type { ImageLayersChoiceGroup } from "../../knowledge-derivations/image-layers-models";

const props = defineProps<{
  availability: ImageActionAvailability;
  reason: string;
  compact?: boolean;
  menuItem?: boolean;
  open?: boolean;
  groups?: ImageLayersChoiceGroup[] | undefined;
}>();
const emit = defineEmits<{
  invoke: [];
  run: [selections: Record<string, string>];
  "update:open": [open: boolean];
}>();
const { t } = useI18n();
const selections = reactive<Record<string, string>>({});

watch(
  () => props.groups,
  (groups) => {
    for (const key of Object.keys(selections)) delete selections[key];
    for (const group of groups ?? []) selections[group.key] = group.selectedId;
  },
  { immediate: true },
);

function optionsFor(group: ImageLayersChoiceGroup) {
  return group.options.map((option) => ({
    value: option.id,
    label: option.name,
    description: option.detail,
  }));
}
</script>

<template>
  <SpaceCanvasImageActionTrigger
    v-if="menuItem"
    :availability="availability"
    :reason="reason"
    :icon="Layers3"
    :label-key="DERIVATION_LABEL_KEY['split-image-layers']"
    :compact="compact"
    menu-item
    @begin="emit('invoke')"
  />
  <UiPopover
    v-else
    :open="Boolean(open && groups?.length)"
    side="bottom"
    align="start"
    class="w-56 space-y-2.5 p-2.5"
    @update:open="emit('update:open', $event)"
  >
    <template #trigger>
      <SpaceCanvasImageActionTrigger
        :availability="availability"
        :reason="reason"
        :icon="Layers3"
        :label-key="DERIVATION_LABEL_KEY['split-image-layers']"
        :compact="compact"
        @begin="emit('invoke')"
      />
    </template>
    <div v-if="groups?.length" data-testid="layers-model-bar" class="space-y-2">
      <label
        v-for="group in groups"
        :key="group.key"
        class="block space-y-1 text-caption font-medium text-fg-secondary"
      >
        {{ t(group.labelKey) }}
        <UiSelect
          :model-value="selections[group.key]!"
          :label="t(group.labelKey)"
          :options="optionsFor(group)"
          size="sm"
          @update:model-value="(value) => (selections[group.key] = value)"
        />
      </label>
      <div class="flex justify-end pt-0.5">
        <UiButton
          data-testid="layers-model-run"
          variant="primary"
          size="sm"
          @click="emit('run', { ...selections })"
        >
          {{ t("ai.modelChoice.run") }}
        </UiButton>
      </div>
    </div>
  </UiPopover>
</template>
