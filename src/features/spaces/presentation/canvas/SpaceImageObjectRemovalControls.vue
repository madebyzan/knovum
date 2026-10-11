<script setup lang="ts">
import { Undo2 } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiActionHelp from "../../../../shared/ui/UiActionHelp.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { ImageRemovalMode } from "../../knowledge-derivations/image-removal-session-store";
import type { ImageActionHelp } from "./image-action-help";
import SpaceImageRangeSlider from "./SpaceImageRangeSlider.vue";

const props = defineProps<{
  mode: ImageRemovalMode;
  prompt: string;
  diameter: number;
  canUndo: boolean;
  canGenerate: boolean;
  submitting: boolean;
  help: ImageActionHelp | undefined;
}>();
const emit = defineEmits<{
  updateMode: [mode: ImageRemovalMode];
  updatePrompt: [prompt: string];
  updateDiameter: [diameter: number];
  undo: [];
  cancel: [];
  generate: [];
  openSettings: [];
}>();
const { t } = useI18n();
const brushSize = computed({
  get: () => Math.round(props.diameter * 1000),
  set: (value: number) => emit("updateDiameter", value / 1000),
});
</script>

<template>
  <section
    class="w-56 space-y-3 text-fg-primary"
    :aria-label="t('spaces.media.imageEditor.removeObject.tool')"
  >
    <header class="flex items-center justify-between px-1">
      <h3 class="text-title-sm font-semibold">{{ t("spaces.media.imageEditor.removeObject.tool") }}</h3>
      <UiButton
        v-if="mode === 'mask'"
        variant="ghost"
        size="sm"
        class="shadow-none"
        :disabled="!canUndo || submitting"
        :aria-label="t('spaces.media.imageEditor.removeObject.undo')"
        @click="emit('undo')"
      >
        <Undo2 class="size-3.5" />
      </UiButton>
    </header>
    <div
      class="grid grid-cols-2 rounded-panel bg-surface-control p-0.5"
      role="group"
      :aria-label="t('spaces.media.imageEditor.removeObject.mode')"
    >
      <button
        v-for="item in ['mask', 'prompt'] as const"
        :key="item"
        type="button"
        class="h-(--knovum-control-height-md) rounded-md text-label text-fg-secondary transition-colors focus-visible:ring-2 focus-visible:ring-stroke-focus"
        :class="
          mode === item && 'border border-stroke-default bg-surface-raised text-fg-primary shadow-control'
        "
        :aria-pressed="mode === item"
        :disabled="submitting"
        @click="emit('updateMode', item)"
      >
        {{ t(`spaces.media.imageEditor.removeObject.modes.${item}`) }}
      </button>
    </div>
    <SpaceImageRangeSlider
      v-if="mode === 'mask'"
      minimal
      :label="t('spaces.media.imageEditor.removeObject.brushSize')"
      :value="brushSize"
      :min="5"
      :max="150"
      :disabled="submitting"
      @update="brushSize = $event"
    />
    <label v-else class="grid text-label">
      <span class="sr-only">{{ t("spaces.media.imageEditor.removeObject.prompt.label") }}</span>
      <UiTextarea
        :model-value="prompt"
        :placeholder="t('spaces.media.imageEditor.removeObject.prompt.placeholder')"
        :disabled="submitting"
        :maxlength="1000"
        :rows="2"
        class="h-20 min-h-0 resize-none"
        @update:model-value="emit('updatePrompt', $event)"
      />
    </label>
    <footer class="flex gap-2 border-t border-stroke-divider pt-2">
      <UiButton
        variant="secondary"
        size="sm"
        class="min-w-0 flex-1 border border-stroke-default bg-surface-raised"
        :disabled="submitting"
        @click="emit('cancel')"
        >{{ t("common.cancel") }}</UiButton
      >
      <UiButton
        variant="primary"
        size="sm"
        class="min-w-0 flex-1 bg-surface-inverse text-fg-inverse hover:bg-surface-inverse hover:opacity-90"
        :disabled="!canGenerate || submitting"
        @click="emit('generate')"
        >{{ t("spaces.media.imageEditor.removeObject.generate") }}</UiButton
      >
      <UiActionHelp
        v-if="help"
        :label="help.label"
        :action-label="help.actionLabel"
        @action="emit('openSettings')"
      />
    </footer>
  </section>
</template>
