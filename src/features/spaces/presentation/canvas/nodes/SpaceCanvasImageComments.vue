<script setup lang="ts">
import { Trash2 } from "@lucide/vue";

import UiPopover from "../../../../../shared/ui/UiPopover.vue";
import { useImageCommentSession } from "../../../comments/image-comment-session";
import { useImageComments, type ImageCommentCanvas } from "../../../comments/use-image-comments";
import SpaceImageCommentModeBar from "./SpaceImageCommentModeBar.vue";
import SpaceImageCommentShape from "./SpaceImageCommentShape.vue";

const props = defineProps<ImageCommentCanvas>();
const commentSession = useImageCommentSession();
const {
  t,
  adding,
  hintLabel,
  mode,
  setMode,
  preview,
  startGesture,
  moveGesture,
  finishGesture,
  cancelGesture,
  placeCenter,
  draft,
  text,
  selectedId,
  busy,
  comments,
  areaStyle,
  positionStyle,
  send,
  addToConversation,
  remove,
  discuss,
  close,
  cancelDraft,
} = useImageComments(props);
</script>

<template>
  <div class="pointer-events-none absolute" :style="areaStyle">
    <button
      v-if="adding"
      type="button"
      class="nodrag pointer-events-auto absolute inset-0 size-full cursor-crosshair touch-none"
      :aria-label="hintLabel"
      :tabindex="mode === 'point' ? 0 : -1"
      @pointerdown="startGesture"
      @pointermove="moveGesture"
      @pointerup="finishGesture"
      @pointercancel="cancelGesture"
      @click.stop
      @keydown.enter.prevent="placeCenter"
      @keydown.space.prevent="placeCenter"
    />
    <SpaceImageCommentShape v-for="comment in comments" :key="`${comment.id}-shape`" :mark="comment" />
    <SpaceImageCommentShape v-if="preview" :mark="preview" />
    <SpaceImageCommentShape v-else-if="draft" :mark="draft" />
    <UiPopover
      v-for="(comment, index) in comments"
      :key="comment.id"
      :open="selectedId === comment.id"
      side="right"
      align="start"
      class="w-72 p-3"
      @update:open="close($event, comment.id)"
    >
      <template #trigger>
        <button
          type="button"
          class="nodrag pointer-events-auto absolute z-10 flex size-9 items-center justify-center rounded-full border-2 border-stroke-on-image bg-(--color-content-purple) text-label font-semibold text-fg-on-image shadow-popover outline-none focus-visible:ring-2 focus-visible:ring-stroke-focus"
          :class="
            comment.shape?.kind === 'region'
              ? 'translate-x-1/2 -translate-y-1/2'
              : '-translate-x-1/2 -translate-y-full'
          "
          :style="positionStyle(comment)"
          :aria-label="`${t('spaces.media.imageEditor.comments.title')} ${index + 1}`"
          @click.stop
        >
          {{ index + 1 }}
        </button>
      </template>
      <div class="flex items-start justify-between gap-2">
        <span
          class="min-w-0 truncate rounded-md border border-stroke-default px-2 py-1 text-label font-medium text-fg-primary"
          >{{ data.title }}</span
        >
        <button
          type="button"
          class="rounded-md p-1 text-fg-tertiary hover:bg-surface-control-hover hover:text-fg-primary"
          :aria-label="t('spaces.media.imageEditor.comments.delete')"
          :disabled="busy"
          @click="remove(comment)"
        >
          <Trash2 class="size-3.5" />
        </button>
      </div>
      <p class="mt-2 text-label whitespace-pre-wrap text-fg-secondary">{{ comment.text }}</p>
      <div class="mt-2 flex justify-end">
        <button
          type="button"
          class="rounded-md bg-surface-control-hover px-3 py-1 text-label text-fg-primary"
          @click="discuss(comment)"
        >
          {{ t("spaces.media.imageEditor.comments.addToConversation") }}
        </button>
      </div>
    </UiPopover>
    <UiPopover
      v-if="draft"
      :open="selectedId === 'draft'"
      side="right"
      align="start"
      class="w-72 p-3"
      @update:open="close($event, 'draft')"
    >
      <template #trigger>
        <button
          type="button"
          class="nodrag pointer-events-auto absolute z-20 flex size-9 items-center justify-center rounded-full border-2 border-stroke-on-image bg-(--color-content-purple) text-label font-semibold text-fg-on-image shadow-popover"
          :class="
            draft.shape?.kind === 'region'
              ? 'translate-x-1/2 -translate-y-1/2'
              : '-translate-x-1/2 -translate-y-full'
          "
          :style="positionStyle(draft)"
          :aria-label="t('spaces.media.imageEditor.comments.title')"
          @click.stop
        >
          {{ comments.length + 1 }}
        </button>
      </template>
      <div class="flex items-center justify-between gap-2">
        <span
          class="min-w-0 truncate rounded-md border border-stroke-default px-2 py-1 text-label font-medium text-fg-primary"
          >{{ data.title }}</span
        ><button
          type="button"
          class="rounded-md p-1 text-fg-tertiary hover:bg-surface-control-hover"
          :aria-label="t('common.cancel')"
          @click="cancelDraft"
        >
          <Trash2 class="size-3.5" />
        </button>
      </div>
      <textarea
        v-model="text"
        rows="2"
        maxlength="2000"
        class="mt-2 block w-full resize-none bg-transparent text-label text-fg-primary outline-none placeholder:text-fg-tertiary"
        :placeholder="t('spaces.media.imageEditor.comments.prompt')"
        :aria-label="t('spaces.media.imageEditor.comments.prompt')"
      />
      <div class="mt-2 flex justify-end gap-1">
        <button
          type="button"
          class="rounded-md border border-stroke-default px-3 py-1 text-label text-fg-primary disabled:opacity-40"
          :disabled="!text.trim() || busy"
          @click="send"
        >
          {{ t("spaces.media.imageEditor.comments.send") }}</button
        ><button
          type="button"
          class="rounded-md bg-surface-control-hover px-3 py-1 text-label text-fg-primary disabled:opacity-40"
          :disabled="!text.trim() || busy"
          @click="addToConversation"
        >
          {{ t("spaces.media.imageEditor.comments.addToConversation") }}
        </button>
      </div>
    </UiPopover>
    <SpaceImageCommentModeBar
      v-if="adding"
      :mode="mode"
      @change="setMode"
      @cancel="commentSession.cancel()"
    />
  </div>
</template>
