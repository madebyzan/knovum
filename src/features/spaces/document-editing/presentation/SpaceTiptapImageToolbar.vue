<script setup lang="ts">
import { Maximize2, Trash2 } from "@lucide/vue";
import { useI18n } from "vue-i18n";

import { IMAGE_WIDTH_PRESETS } from "./use-space-tiptap-image-resize";

withDefaults(
  defineProps<{
    visible: boolean;
    isEditable: boolean;
    activePreset: number;
    allowPreview?: boolean;
  }>(),
  { allowPreview: true },
);

const emit = defineEmits<{
  preview: [];
  selectPreset: [fraction: number];
  remove: [];
}>();

const { t } = useI18n();
</script>

<template>
  <div
    contenteditable="false"
    class="pointer-events-auto absolute top-3 right-3 flex h-7 items-center gap-0.5 rounded-md bg-neutral-950/72 px-1 text-white/85 opacity-0 shadow-lg ring-1 ring-white/12 backdrop-blur-md transition-opacity duration-150 ease-standard group-hover:opacity-100"
    :class="visible && 'opacity-100'"
  >
    <button
      v-if="allowPreview"
      type="button"
      data-testid="image-preview"
      :aria-label="t('spaces.richtext.image.preview')"
      :title="t('spaces.richtext.image.preview')"
      class="grid size-5.5 place-items-center rounded-sm text-white/80 transition-colors hover:bg-white/14 hover:text-white"
      @mousedown.prevent.stop
      @click.stop="emit('preview')"
    >
      <Maximize2 class="size-3.5" :stroke-width="1.65" />
    </button>
    <template v-if="isEditable">
      <span v-if="allowPreview" class="mx-0.5 h-3 w-px bg-white/15" />
      <button
        v-for="[label, fraction] in IMAGE_WIDTH_PRESETS"
        :key="label"
        type="button"
        class="h-5 rounded-sm px-1.5 text-[11px] font-medium tabular-nums transition-colors"
        :class="
          activePreset === fraction
            ? 'bg-white/20 text-white'
            : 'text-white/70 hover:bg-white/12 hover:text-white'
        "
        @mousedown.prevent.stop
        @click.stop="emit('selectPreset', fraction)"
      >
        {{ label }}
      </button>
      <span class="mx-0.5 h-3 w-px bg-white/15" />
      <button
        type="button"
        data-testid="image-remove"
        :aria-label="t('common.delete')"
        :title="t('common.delete')"
        class="grid size-5.5 place-items-center rounded-sm text-white/75 transition-colors hover:bg-red-500/20 hover:text-red-300"
        @mousedown.prevent.stop
        @click.stop="emit('remove')"
      >
        <Trash2 class="size-3.5" :stroke-width="1.6" />
      </button>
    </template>
  </div>
</template>
