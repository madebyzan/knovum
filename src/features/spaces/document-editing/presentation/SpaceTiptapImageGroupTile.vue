<script setup lang="ts">
import { PenLine, X } from "@lucide/vue";
import { nextTick, useTemplateRef } from "vue";
import { useI18n } from "vue-i18n";

import {
  parseImageCardCaption,
  type ImageGroupLayout,
  type LightboxImageItem,
} from "../../../../shared/ui/image-group-layout";

defineProps<{
  item: LightboxImageItem;
  index: number;
  layout: ImageGroupLayout;
  isEditable: boolean;
  editing: boolean;
  dragSource: boolean;
  dragTarget: boolean;
}>();

const emit = defineEmits<{
  preview: [];
  remove: [];
  startEdit: [];
  stopEdit: [];
  updateAlt: [value: string];
  dragStart: [event: PointerEvent];
}>();

const { t } = useI18n();
const inputRef = useTemplateRef<HTMLInputElement>("inputRef");

function beginEdit() {
  emit("startEdit");
  void nextTick(() => inputRef.value?.focus());
}
</script>

<template>
  <figure
    :data-image-tile-index="index"
    class="ui-image-group-item group/item"
    :class="[
      dragSource && 'opacity-45',
      dragTarget && 'ring-2 ring-fg-accent/80 ring-offset-1 ring-offset-surface-canvas',
    ]"
  >
    <div
      class="ui-image-group-media"
      :class="isEditable ? 'cursor-grab active:cursor-grabbing' : 'cursor-zoom-in'"
      @pointerdown="(event) => emit('dragStart', event)"
      @click.stop="emit('preview')"
    >
      <img :src="item.src" :alt="item.caption" draggable="false" />
      <button
        v-if="isEditable"
        type="button"
        data-image-group-interactive
        :aria-label="t('common.delete')"
        :title="t('common.delete')"
        class="absolute top-1.5 right-1.5 grid size-5.5 place-items-center rounded-full bg-neutral-950/55 text-white/85 opacity-0 backdrop-blur-xs transition-[opacity,background-color] duration-150 group-hover/item:opacity-100 hover:bg-red-500/85 hover:text-white"
        @pointerdown.stop
        @mousedown.prevent.stop
        @click.stop="emit('remove')"
      >
        <X class="size-3" :stroke-width="2" />
      </button>
      <button
        v-if="isEditable && layout !== 'cards' && !item.caption.trim() && !editing"
        type="button"
        data-image-group-interactive
        :aria-label="t('spaces.richtext.image.caption')"
        :title="t('spaces.richtext.image.caption')"
        class="absolute right-1.5 bottom-1.5 grid size-5.5 place-items-center rounded-full bg-neutral-950/55 text-white/85 opacity-0 backdrop-blur-xs transition-[opacity,background-color] duration-150 group-hover/item:opacity-100 hover:bg-neutral-950/80 hover:text-white"
        @pointerdown.stop
        @mousedown.prevent.stop
        @click.stop="beginEdit"
      >
        <PenLine class="size-2.5" :stroke-width="2" />
      </button>
    </div>

    <figcaption
      v-if="layout === 'cards' || item.caption.trim() || editing"
      contenteditable="false"
      data-image-group-interactive
      :class="layout === 'cards' ? 'ui-image-group-card-body' : 'ui-image-group-caption'"
    >
      <div
        v-if="layout === 'cards' && !editing && item.caption.trim()"
        class="cursor-text"
        @click.stop="isEditable && beginEdit()"
      >
        <div class="ui-image-group-card-title">{{ parseImageCardCaption(item.caption).title }}</div>
        <div v-if="parseImageCardCaption(item.caption).description" class="ui-image-group-card-desc">
          {{ parseImageCardCaption(item.caption).description }}
        </div>
      </div>
      <input
        v-else
        ref="inputRef"
        type="text"
        :value="item.caption"
        :disabled="!isEditable"
        :placeholder="
          layout === 'cards'
            ? t('spaces.richtext.image.group.cardPlaceholder')
            : t('spaces.richtext.image.captionPlaceholder')
        "
        class="w-full cursor-text rounded bg-transparent px-1 py-0.5 text-[12.5px] text-fg-secondary caret-fg-accent outline-none select-text placeholder:text-fg-tertiary/60 focus:bg-surface-muted/70 focus:text-fg-primary"
        :class="layout === 'cards' ? 'text-left' : 'text-center'"
        @mousedown.stop
        @click.stop
        @focus="emit('startEdit')"
        @blur="emit('stopEdit')"
        @input="(event) => emit('updateAlt', (event.target as HTMLInputElement).value)"
        @keydown.stop
        @keydown.enter.prevent="($event.target as HTMLInputElement).blur()"
      />
    </figcaption>
  </figure>
</template>
