<script setup lang="ts">
import { RotateCcw, X } from "@lucide/vue";
import { usePreferredReducedMotion } from "@vueuse/core";
import { shallowRef, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import type { ImageAdjustments } from "../image-edit";
import { IMAGE_ADJUSTMENT_GROUPS, type ImageAdjustmentGroupId } from "./image-adjustment-groups";
import SpaceImageToneSlider from "./SpaceImageToneSlider.vue";

const props = defineProps<{ adjustments: ImageAdjustments }>();
const emit = defineEmits<{
  update: [key: keyof ImageAdjustments, value: number];
  reset: [];
  cancel: [];
  apply: [];
}>();
const { t } = useI18n();
const list = useTemplateRef<{ viewportElement: HTMLElement }>("list");
const active = shallowRef<ImageAdjustmentGroupId>("light");
const reducedMotion = usePreferredReducedMotion();
const positiveControls = new Set<keyof ImageAdjustments>([
  "sharpness",
  "clarity",
  "grain",
  "softLight",
  "glow",
]);

function navigate(groupId: ImageAdjustmentGroupId) {
  const viewport = list.value?.viewportElement;
  const section = viewport?.querySelector<HTMLElement>(`[data-adjust-group="${groupId}"]`);
  if (!section || !viewport) return;
  active.value = groupId;
  viewport.scrollTo({
    top: section.offsetTop,
    behavior: reducedMotion.value === "reduce" ? "auto" : "smooth",
  });
}

function syncActive() {
  const container = list.value?.viewportElement;
  if (!container) return;
  for (const group of IMAGE_ADJUSTMENT_GROUPS.toReversed()) {
    const section = container.querySelector<HTMLElement>(`[data-adjust-group="${group.id}"]`);
    if (section && section.offsetTop <= container.scrollTop + 80) {
      active.value = group.id;
      return;
    }
  }
  active.value = "light";
}
</script>

<template>
  <section
    class="flex h-104 w-60 flex-col text-fg-primary"
    :aria-label="t('spaces.media.imageEditor.adjust.title')"
  >
    <header class="flex items-center justify-between px-1 pb-2">
      <h3 class="text-title-sm font-semibold">{{ t("spaces.media.imageEditor.adjust.title") }}</h3>
      <div class="flex items-center gap-1">
        <button
          type="button"
          class="flex size-(--knovum-control-height-md) items-center justify-center rounded-md text-fg-secondary hover:bg-surface-control-hover focus-visible:ring-2 focus-visible:ring-stroke-focus"
          :aria-label="t('common.reset')"
          @click="emit('reset')"
        >
          <RotateCcw class="size-3.5" />
        </button>
        <button
          type="button"
          class="flex size-(--knovum-control-height-md) items-center justify-center rounded-md text-fg-secondary hover:bg-surface-control-hover focus-visible:ring-2 focus-visible:ring-stroke-focus"
          :aria-label="t('common.cancel')"
          @click="emit('cancel')"
        >
          <X class="size-3.5" />
        </button>
      </div>
    </header>
    <nav
      class="mb-2 flex rounded-md bg-surface-control p-0.5"
      :aria-label="t('spaces.media.imageEditor.adjust.sections')"
    >
      <button
        v-for="group in IMAGE_ADJUSTMENT_GROUPS"
        :key="group.id"
        type="button"
        class="flex h-(--knovum-control-height-md) min-w-0 flex-1 items-center justify-center rounded-md text-fg-secondary transition-[background-color,color,box-shadow] duration-150 ease-standard hover:text-fg-primary focus-visible:ring-2 focus-visible:ring-stroke-focus motion-reduce:transition-none"
        :class="active === group.id ? 'bg-surface-raised text-fg-primary shadow-control' : ''"
        :aria-label="t(`spaces.media.imageEditor.adjustGroups.${group.id}`)"
        :aria-current="active === group.id ? 'location' : undefined"
        @click="navigate(group.id)"
      >
        <component :is="group.icon" class="size-3.5" :stroke-width="1.6" />
      </button>
    </nav>
    <UiScrollArea ref="list" class="min-h-0 flex-1" @scroll="syncActive">
      <div class="pr-3 pl-1">
        <section
          v-for="group in IMAGE_ADJUSTMENT_GROUPS"
          :key="group.id"
          :data-adjust-group="group.id"
          class="pt-2 pb-3 last:min-h-80"
        >
          <h4 class="mb-2 text-label font-medium text-fg-secondary">
            {{ t(`spaces.media.imageEditor.adjustGroups.${group.id}`) }}
          </h4>
          <div class="space-y-2">
            <SpaceImageToneSlider
              v-for="key in group.keys"
              :key="key"
              :label="t(`spaces.media.imageEditor.adjustments.${key}`)"
              :value="props.adjustments[key]"
              :min="positiveControls.has(key) ? 0 : -100"
              :color-track="key === 'warmth' || key === 'tint' ? key : undefined"
              @update="emit('update', key, $event)"
            />
          </div>
        </section>
      </div>
    </UiScrollArea>
    <footer class="mt-3 flex gap-2 border-t border-stroke-divider pt-3">
      <UiButton
        variant="secondary"
        size="sm"
        class="min-w-0 flex-1 border border-stroke-default bg-surface-raised"
        @click="emit('cancel')"
        >{{ t("common.cancel") }}</UiButton
      >
      <UiButton variant="primary" size="sm" class="min-w-0 flex-1" @click="emit('apply')">{{
        t("common.apply")
      }}</UiButton>
    </footer>
  </section>
</template>
