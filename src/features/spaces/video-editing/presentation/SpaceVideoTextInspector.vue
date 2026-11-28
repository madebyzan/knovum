<script setup lang="ts">
import { Layers, Merge, Scissors } from "@lucide/vue";
import { sortBy } from "es-toolkit";
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiNumberField from "../../../../shared/ui/UiNumberField.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import {
  applyTextStyleToAll,
  canSplitTextAt,
  mergeTextOverlayWithNext,
  splitTextOverlayAt,
} from "../edits/captions";
import { updateTextOverlay, updateTextStyle } from "../edits/text";
import type { VideoTextOverlay, VideoTextStyle } from "../project";
import { projectDurationMs } from "../timing";
import SpaceVideoCaptionStylePresets from "./SpaceVideoCaptionStylePresets.vue";
import SpaceVideoColorField from "./SpaceVideoColorField.vue";
import SpaceVideoFontControls from "./SpaceVideoFontControls.vue";
import SpaceVideoTimeField from "./SpaceVideoTimeField.vue";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ editor: VideoEditor; item: VideoTextOverlay }>();
const { t } = useI18n();
const { project, player, commit } = props.editor;
const ALIGNMENTS = ["left", "center", "right"] as const;
const positions = computed(() => [
  { key: "posTop", label: t("spaces.media.videoEditor.captions.posTop"), y: 0.14 },
  { key: "posCenter", label: t("spaces.media.videoEditor.captions.posCenter"), y: 0.5 },
  { key: "posBottom", label: t("spaces.media.videoEditor.captions.posBottom"), y: 0.88 },
]);
const draft = shallowRef(props.item.text);
const durationMs = computed(() => (project.value ? projectDurationMs(project.value) : 0));
const captionCount = computed(() => project.value?.textTrack.items.length ?? 0);
const canSplit = computed(() => canSplitTextAt(props.item, player.positionMs()));
const hasNextCaption = computed(() => {
  const ordered = sortBy(project.value?.textTrack.items ?? [], [(c) => c.startMs, (c) => c.endMs]);
  const index = ordered.findIndex((c) => c.id === props.item.id);
  return index >= 0 && index < ordered.length - 1;
});
watch(
  () => props.item.text,
  (value) => (draft.value = value),
);

const patch = (change: Partial<VideoTextOverlay>, mergeKey?: string) =>
  project.value && commit(updateTextOverlay(project.value, props.item.id, change), mergeKey);
const patchStyle = (change: Partial<VideoTextStyle>, mergeKey?: string) =>
  project.value && commit(updateTextStyle(project.value, props.item.id, change), mergeKey);
const patchSize = (value: number | undefined) =>
  patchStyle({ size: Math.round(value ?? props.item.style.size) }, `size:${props.item.id}`);

function applyToAll() {
  if (!project.value) return;
  commit(applyTextStyleToAll(project.value, props.item.id));
  toast.success(t("spaces.media.videoEditor.captions.applyStyleDone", { count: captionCount.value }));
}
function splitHere() {
  if (project.value) commit(splitTextOverlayAt(project.value, props.item.id, player.positionMs()));
}
function mergeNext() {
  if (project.value) commit(mergeTextOverlayWithNext(project.value, props.item.id));
}
</script>

<template>
  <div class="space-y-4 p-3">
    <section>
      <label class="mb-1.5 block text-caption font-medium" for="video-edit-text">
        {{ t("spaces.media.videoEditor.text.content") }}
      </label>
      <UiTextarea
        id="video-edit-text"
        v-model="draft"
        class="min-h-16 resize-none"
        :rows="2"
        @update:model-value="patch({ text: draft }, `text:${item.id}`)"
        @blur="draft = item.text"
      />
      <div class="mt-1.5 flex items-center gap-1">
        <UiButton
          size="sm"
          variant="secondary"
          class="flex-1 justify-center"
          :disabled="!canSplit"
          @click="splitHere"
        >
          <Scissors class="size-3" />{{ t("spaces.media.videoEditor.captions.splitCaption") }}
        </UiButton>
        <UiButton
          size="sm"
          variant="secondary"
          class="flex-1 justify-center"
          :disabled="!hasNextCaption"
          @click="mergeNext"
        >
          <Merge class="size-3" />{{ t("spaces.media.videoEditor.captions.mergeNext") }}
        </UiButton>
      </div>
    </section>
    <SpaceVideoCaptionStylePresets :item-id="item.id" :style="item.style" @change="patchStyle" />
    <SpaceVideoFontControls :style="item.style" @change="patchStyle" />
    <section class="grid grid-cols-2 gap-2">
      <label class="text-caption text-fg-secondary">
        {{ t("spaces.media.videoEditor.font.size") }}
        <UiNumberField
          class="mt-1"
          size="sm"
          :min="12"
          :max="200"
          :step="1"
          :model-value="item.style.size"
          @update:model-value="patchSize($event)"
        />
      </label>
      <label class="text-caption text-fg-secondary">
        {{ t("spaces.media.videoEditor.text.alignment") }}
        <UiSelect
          class="mt-1"
          size="sm"
          :model-value="item.style.align"
          :label="t('spaces.media.videoEditor.text.alignment')"
          :options="
            ALIGNMENTS.map((value) => ({
              value,
              label: t(`spaces.media.videoEditor.text.alignments.${value}`),
            }))
          "
          @update:model-value="patchStyle({ align: $event })"
        />
      </label>
    </section>
    <SpaceVideoColorField
      :key="`color-${item.id}`"
      :label="t('spaces.media.videoEditor.font.color')"
      :color="item.style.color"
      @change="patchStyle({ color: $event }, `color:${item.id}`)"
    />
    <section>
      <h3 class="mb-1.5 text-caption text-fg-secondary">
        {{ t("spaces.media.videoEditor.captions.position") }}
      </h3>
      <div class="grid grid-cols-3 gap-1">
        <UiButton
          v-for="pos in positions"
          :key="pos.key"
          size="sm"
          :variant="Math.abs(item.y - pos.y) < 0.04 ? 'primary' : 'secondary'"
          class="justify-center"
          @click="patch({ x: 0.5, y: pos.y })"
        >
          {{ pos.label }}
        </UiButton>
      </div>
    </section>
    <UiButton
      v-if="captionCount > 1"
      size="sm"
      variant="secondary"
      class="w-full justify-center"
      @click="applyToAll"
    >
      <Layers class="size-3.5" />{{ t("spaces.media.videoEditor.captions.applyStyleAll") }}
    </UiButton>
    <section class="grid grid-cols-2 gap-2 border-t border-stroke-divider pt-3">
      <SpaceVideoTimeField
        :label="t('spaces.media.videoEditor.clip.startsAt')"
        :value="item.startMs"
        :max="Math.max(0, durationMs - 100)"
        @change="patch({ startMs: $event, endMs: $event + (item.endMs - item.startMs) })"
      />
      <SpaceVideoTimeField
        :label="t('spaces.media.videoEditor.clip.endsAt')"
        :value="item.endMs"
        :min="item.startMs + 100"
        :max="durationMs"
        @change="patch({ endMs: $event })"
      />
    </section>
    <p class="text-caption-sm text-fg-tertiary">{{ t("spaces.media.videoEditor.text.dragHint") }}</p>
  </div>
</template>
