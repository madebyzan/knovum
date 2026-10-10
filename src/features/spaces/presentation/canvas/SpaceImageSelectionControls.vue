<script setup lang="ts">
import { Undo2, MousePointer2, SquareDashed, Paintbrush, X } from "@lucide/vue";
import { nextTick, onMounted, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiTooltip from "../../../../shared/ui/UiTooltip.vue";
import {
  useImageSelectionSessionStore,
  type ImageSelectionTool,
} from "../../knowledge-derivations/image-selection-session-store";
import SpaceImageExecutionSelect from "./SpaceImageExecutionSelect.vue";
import SpaceImageRangeSlider from "./SpaceImageRangeSlider.vue";

const { t } = useI18n();
const selection = useImageSelectionSessionStore();
const surface = useTemplateRef<HTMLElement>("surface");
const tools = [
  { id: "include", labelKey: "spaces.media.imageEditor.selection.include", icon: MousePointer2 },
  { id: "exclude", labelKey: "spaces.media.imageEditor.selection.exclude", icon: MousePointer2 },
  { id: "box", labelKey: "spaces.media.imageEditor.selection.box", icon: SquareDashed },
  { id: "brush", labelKey: "spaces.media.imageEditor.selection.brush", icon: Paintbrush },
] as const;
function submit(effect: "extract" | "repair") {
  void selection.submit(effect).catch((cause: unknown) => toast.error(getErrorMessage(cause)));
}
onMounted(async () => {
  await nextTick();
  surface.value?.querySelector<HTMLElement>("button:not(:disabled)")?.focus({ preventScroll: true });
});
</script>
<template>
  <section
    ref="surface"
    class="w-64 space-y-3 rounded-panel border border-stroke-strong bg-surface-overlay p-3 shadow-popover"
    :aria-label="t('spaces.media.imageEditor.selection.title')"
  >
    <header class="flex items-center justify-between">
      <h3 class="text-title-sm font-semibold">{{ t("spaces.media.imageEditor.selection.title") }}</h3>
      <div class="flex gap-1">
        <UiButton
          variant="ghost"
          size="sm"
          :disabled="!selection.canUndo"
          :aria-label="t('spaces.media.imageEditor.selection.undo')"
          @click="selection.undo"
          ><Undo2 class="size-3.5"
        /></UiButton>
        <UiButton
          variant="ghost"
          size="sm"
          :disabled="selection.submitting"
          :aria-label="t('common.cancel')"
          @click="selection.cancel"
          ><X class="size-3.5"
        /></UiButton>
      </div>
    </header>
    <SpaceImageExecutionSelect
      :options="selection.targets"
      :execution="selection.active?.execution"
      :disabled="selection.submitting"
      @select="selection.setExecution"
    />
    <div
      class="grid grid-cols-4 gap-0.5 rounded-panel bg-surface-control p-0.5"
      role="group"
      :aria-label="t('spaces.media.imageEditor.selection.tools')"
    >
      <UiTooltip v-for="item in tools" :key="item.id" :text="t(item.labelKey)">
        <UiButton
          variant="ghost"
          size="sm"
          :aria-pressed="selection.tool === item.id"
          :aria-label="t(item.labelKey)"
          :disabled="selection.submitting"
          :class="
            selection.tool === item.id
              ? 'border border-stroke-default bg-surface-raised text-fg-primary shadow-control'
              : ''
          "
          @click="selection.tool = item.id as ImageSelectionTool"
          ><component :is="item.icon" class="size-4" :class="item.id === 'exclude' && 'text-fg-danger'"
        /></UiButton>
      </UiTooltip>
    </div>
    <SpaceImageRangeSlider
      v-if="selection.tool === 'brush'"
      :label="t('spaces.media.imageEditor.removeObject.brushSize')"
      :value="selection.brushDiameter * 1000"
      :min="5"
      :max="150"
      @update="selection.brushDiameter = $event / 1000"
    />
    <p class="text-caption text-fg-secondary" role="status">
      {{
        selection.error ??
        (selection.busy
          ? t("spaces.media.imageEditor.selection.processing")
          : t("spaces.media.imageEditor.selection.help"))
      }}
    </p>
    <footer class="flex gap-2 border-t border-stroke-divider pt-2">
      <UiButton size="sm" class="flex-1" :disabled="!selection.canSubmit" @click="submit('repair')">{{
        t("spaces.media.imageEditor.selection.repair")
      }}</UiButton>
      <UiButton
        size="sm"
        variant="primary"
        class="flex-1 bg-surface-inverse text-fg-inverse hover:bg-surface-inverse"
        :disabled="!selection.canSubmit"
        @click="submit('extract')"
        >{{ t("spaces.media.imageEditor.selection.extract") }}</UiButton
      >
    </footer>
  </section>
</template>
