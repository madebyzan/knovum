<script setup lang="ts">
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiNumberField from "../../../../shared/ui/UiNumberField.vue";
import type { VideoTextStyle } from "../project";
import { SUBTITLE_STYLE_TEMPLATES, type SubtitleStyleTemplate } from "../subtitle-presets";
import SpaceVideoColorField from "./SpaceVideoColorField.vue";

const props = defineProps<{ itemId: string; style: VideoTextStyle }>();
const emit = defineEmits<{ change: [patch: Partial<VideoTextStyle>, mergeKey?: string] }>();
const { t } = useI18n();
const DECORATIONS = ["plain", "outline", "shadow", "box"] as const;

function isActiveTemplate(tpl: SubtitleStyleTemplate) {
  return (
    props.style.decoration === tpl.patch.decoration &&
    props.style.color.toUpperCase() === tpl.patch.color?.toUpperCase()
  );
}

function patchStrength(value: number | undefined) {
  emit("change", { decorationStrength: Math.round(value ?? props.style.decorationStrength) });
}
</script>

<template>
  <div class="space-y-3">
    <section>
      <h3 class="mb-1.5 text-caption text-fg-secondary">
        {{ t("spaces.media.videoEditor.captions.templateTitle") }}
      </h3>
      <div class="grid grid-cols-3 gap-1.5">
        <button
          v-for="tpl in SUBTITLE_STYLE_TEMPLATES"
          :key="tpl.id"
          type="button"
          class="flex h-9 items-center justify-center rounded-md px-1.5 text-caption-sm transition-all focus-visible:outline-2 focus-visible:outline-stroke-focus"
          :class="[
            tpl.previewClass,
            isActiveTemplate(tpl) ? 'ring-2 ring-stroke-accent' : 'opacity-90 hover:opacity-100',
          ]"
          @click="emit('change', tpl.patch)"
        >
          <span class="truncate">{{ t(`spaces.media.videoEditor.captions.templates.${tpl.id}`) }}</span>
        </button>
      </div>
    </section>
    <section>
      <h3 class="mb-1.5 text-caption text-fg-secondary">
        {{ t("spaces.media.videoEditor.text.presets.title") }}
      </h3>
      <div class="grid grid-cols-4 gap-1.5">
        <UiButton
          v-for="decoration in DECORATIONS"
          :key="decoration"
          size="sm"
          :variant="style.decoration === decoration ? 'primary' : 'secondary'"
          class="justify-center"
          @click="emit('change', { decoration })"
        >
          {{ t(`spaces.media.videoEditor.text.presets.${decoration}`) }}
        </UiButton>
      </div>
    </section>
    <section v-if="style.decoration !== 'plain'" class="space-y-2">
      <label class="text-caption text-fg-secondary">
        {{ t("spaces.media.videoEditor.text.strength") }}
        <UiNumberField
          class="mt-1"
          size="sm"
          :min="0"
          :max="20"
          :step="1"
          :model-value="style.decorationStrength"
          @update:model-value="patchStrength"
        />
      </label>
      <SpaceVideoColorField
        :key="`decoration-${itemId}`"
        :label="t('spaces.media.videoEditor.text.decorationColor')"
        :color="style.decorationColor"
        @change="emit('change', { decorationColor: $event }, `decoration:${itemId}`)"
      />
    </section>
  </div>
</template>
