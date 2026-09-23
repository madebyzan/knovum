<script setup lang="ts">
import { RotateCcw, X } from "@lucide/vue";
import { shallowRef, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import type { ImageAdjustments } from "../../image-editing/image-edit";
import { imageAdjustmentGroups, type ImageAdjustmentGroupId } from "./image-adjustment-groups";
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
const positiveControls = new Set<keyof ImageAdjustments>(["sharpness", "clarity", "grain", "softLight", "glow"]);

function navigate(id: ImageAdjustmentGroupId) {
  const viewport = list.value?.viewportElement;
  const section = viewport?.querySelector<HTMLElement>(`[data-adjust-group="${id}"]`);
  if (!section || !viewport) return;
  active.value = id;
  viewport.scrollTo({
    top: section.offsetTop,
    behavior: window.matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth",
  });
}

function syncActive() {
  const container = list.value?.viewportElement;
  if (!container) return;
  for (const group of [...imageAdjustmentGroups].reverse()) {
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
  <section class="w-60 text-foreground" :aria-label="t('spaces.media.adjustImage')">
    <header class="flex items-center justify-between px-1 pb-2">
      <h3 class="text-body-lg font-semibold">{{ t("spaces.media.adjustImage") }}</h3>
      <div class="flex items-center gap-1">
        <button type="button" class="flex size-(--knovum-control-height-md) items-center justify-center rounded-control text-foreground-secondary hover:bg-control-hover focus-visible:ring-2 focus-visible:ring-focus-ring" :aria-label="t('common.reset')" @click="emit('reset')">
          <RotateCcw class="size-(--knovum-icon-size-sm)" />
        </button>
        <button type="button" class="flex size-(--knovum-control-height-md) items-center justify-center rounded-control text-foreground-secondary hover:bg-control-hover focus-visible:ring-2 focus-visible:ring-focus-ring" :aria-label="t('common.cancel')" @click="emit('cancel')">
          <X class="size-(--knovum-icon-size-sm)" />
        </button>
      </div>
    </header>
    <nav class="mb-2 flex rounded-control bg-control p-0.5" :aria-label="t('spaces.media.adjustSections')">
      <button
        v-for="group in imageAdjustmentGroups"
        :key="group.id"
        type="button"
        class="flex h-(--knovum-control-height-md) min-w-0 flex-1 items-center justify-center rounded-control text-foreground-secondary transition-[background-color,color,box-shadow] duration-150 ease-standard hover:text-foreground focus-visible:ring-2 focus-visible:ring-focus-ring motion-reduce:transition-none"
        :class="active === group.id ? 'bg-raised text-foreground shadow-control' : ''"
        :aria-label="t(`spaces.media.adjustGroups.${group.id}`)"
        :aria-current="active === group.id ? 'location' : undefined"
        @click="navigate(group.id)"
      >
        <component :is="group.icon" class="size-(--knovum-icon-size-sm)" :stroke-width="1.6" />
      </button>
    </nav>
    <UiScrollArea ref="list" class="h-80" @scroll="syncActive">
      <div class="pl-1 pr-3">
        <section v-for="group in imageAdjustmentGroups" :key="group.id" :data-adjust-group="group.id" class="pb-3 pt-2 last:min-h-80">
          <h4 class="mb-2 text-label font-medium text-foreground-secondary">{{ t(`spaces.media.adjustGroups.${group.id}`) }}</h4>
          <div class="space-y-2">
            <SpaceImageToneSlider
              v-for="key in group.keys"
              :key="key"
              :label="t(`spaces.media.adjustments.${key}`)"
              :value="props.adjustments[key]"
              :min="positiveControls.has(key) ? 0 : -100"
              :color-track="key === 'warmth' || key === 'tint' ? key : undefined"
              @update="emit('update', key, $event)"
            />
          </div>
        </section>
      </div>
    </UiScrollArea>
    <footer class="mt-3 flex gap-2 border-t border-divider pt-3">
      <UiButton variant="secondary" size="sm" class="min-w-0 flex-1 border border-border bg-raised" @click="emit('cancel')">{{ t("common.cancel") }}</UiButton>
      <UiButton variant="primary" size="sm" class="min-w-0 flex-1 bg-foreground text-canvas hover:bg-foreground hover:opacity-90" @click="emit('apply')">{{ t("common.apply") }}</UiButton>
    </footer>
  </section>
</template>
