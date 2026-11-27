<script setup lang="ts">
import { X } from "@lucide/vue";
import { reactive, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import { CANVAS_ACTION_BAR_CLASSES } from "../../canvas/presentation/canvas-action-classes";
import type { ImageLayersChoiceGroup } from "../../knowledge-derivations/image-layers-models";

const props = defineProps<{ groups: readonly ImageLayersChoiceGroup[] }>();
const emit = defineEmits<{ run: [selections: Record<string, string>]; cancel: [] }>();
const { t } = useI18n();
const selections = reactive<Record<string, string>>({});

watch(
  () => props.groups,
  (groups) => {
    for (const key of Object.keys(selections)) delete selections[key];
    for (const group of groups) selections[group.key] = group.selectedId;
  },
  { immediate: true },
);

function optionsFor(group: ImageLayersChoiceGroup) {
  return group.options.map((option) => ({
    value: option.id,
    label: option.name,
    description: option.description,
  }));
}
</script>

<template>
  <div
    data-testid="layers-model-bar"
    :class="CANVAS_ACTION_BAR_CLASSES"
    class="h-auto flex-wrap gap-2 px-2.5 py-1.5"
    @keydown.esc.stop="emit('cancel')"
  >
    <label
      v-for="group in groups"
      :key="group.key"
      class="flex items-center gap-1.5 text-caption font-medium text-fg-secondary"
    >
      <span class="shrink-0">{{ t(group.labelKey) }}</span>
      <UiSelect
        :model-value="selections[group.key]!"
        :label="t(group.labelKey)"
        :options="optionsFor(group)"
        size="sm"
        @update:model-value="(value) => (selections[group.key] = value)"
      />
    </label>
    <UiButton
      data-testid="layers-model-run"
      variant="primary"
      size="sm"
      @click="emit('run', { ...selections })"
    >
      {{ t("ai.modelChoice.run") }}
    </UiButton>
    <UiIconButton :label="t('common.cancel')" size="sm" @click="emit('cancel')">
      <X class="size-3.5" />
    </UiIconButton>
  </div>
</template>
