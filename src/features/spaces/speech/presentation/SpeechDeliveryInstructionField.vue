<script setup lang="ts">
import { SlidersHorizontal } from "@lucide/vue";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiTextarea from "../../../../shared/ui/UiTextarea.vue";

const model = defineModel<string>({ required: true });
const props = withDefaults(
  defineProps<{
    disabled?: boolean | undefined;
    collapsible?: boolean | undefined;
    surface?: "raised" | "muted" | undefined;
  }>(),
  { disabled: false, collapsible: false, surface: "raised" },
);
const { t } = useI18n();
const open = shallowRef(Boolean(model.value.trim()));
watch(model, (value) => {
  if (value.trim()) open.value = true;
});
const expanded = computed(() => !props.collapsible || open.value || Boolean(model.value.trim()));
const length = computed(() => [...model.value].length);
</script>

<template>
  <div v-if="!expanded" class="flex items-center">
    <button
      type="button"
      class="inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-caption-sm text-fg-tertiary transition-colors hover:bg-surface-control-hover hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus disabled:pointer-events-none disabled:opacity-40"
      :disabled="disabled"
      @click="open = true"
    >
      <SlidersHorizontal class="size-3" />
      <span>{{ t("spaces.speech.project.deliveryInstruction.label") }}</span>
    </button>
  </div>
  <div v-else class="space-y-1">
    <div class="flex items-center justify-between gap-2">
      <button
        v-if="collapsible && !model.trim()"
        type="button"
        class="inline-flex items-center gap-1 text-caption-sm font-medium text-fg-secondary hover:text-fg-primary focus-visible:outline-2 focus-visible:outline-stroke-focus"
        @click="open = false"
      >
        <SlidersHorizontal class="size-3" />
        <span>{{ t("spaces.speech.project.deliveryInstruction.label") }}</span>
      </button>
      <span v-else class="text-caption-sm font-medium text-fg-secondary">{{
        t("spaces.speech.project.deliveryInstruction.label")
      }}</span>
      <span class="text-caption-sm text-fg-tertiary tabular-nums">{{ length }}/500</span>
    </div>
    <UiTextarea
      v-model="model"
      :rows="2"
      :maxlength="500"
      :disabled="disabled"
      :class="surface === 'muted' ? 'bg-surface-muted' : ''"
      :placeholder="t('spaces.speech.project.deliveryInstruction.placeholder')"
      :aria-label="t('spaces.speech.project.deliveryInstruction.label')"
    />
  </div>
</template>
