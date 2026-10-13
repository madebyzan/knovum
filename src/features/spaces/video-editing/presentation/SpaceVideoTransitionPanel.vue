<script setup lang="ts">
import {
  ArrowDownToLine,
  ArrowLeftToLine,
  ArrowRightToLine,
  ArrowUpToLine,
  Blend,
  Contrast,
  MoveDown,
  MoveLeft,
  MoveRight,
  MoveUp,
  Sun,
  Trash2,
} from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiScrollArea from "../../../../shared/ui/UiScrollArea.vue";
import { applyTransition, maxTransitionMs, removeTransition } from "../edits/transition";
import type { VideoTransitionKind } from "../project";
import { clipEndMs, clipIndexOf, transitionFrom } from "../timing";
import SpaceVideoTimeField from "./SpaceVideoTimeField.vue";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ editor: VideoEditor }>();
const { t } = useI18n();
const { project, selection, engine, commit } = props.editor;

const KINDS: { kind: VideoTransitionKind; icon: unknown }[] = [
  { kind: "dissolve", icon: Blend },
  { kind: "fadeBlack", icon: Contrast },
  { kind: "fadeWhite", icon: Sun },
  { kind: "wipeLeft", icon: ArrowLeftToLine },
  { kind: "wipeRight", icon: ArrowRightToLine },
  { kind: "wipeUp", icon: ArrowUpToLine },
  { kind: "wipeDown", icon: ArrowDownToLine },
  { kind: "slideLeft", icon: MoveLeft },
  { kind: "slideRight", icon: MoveRight },
  { kind: "slideUp", icon: MoveUp },
  { kind: "slideDown", icon: MoveDown },
];

/** The cut a transition would apply to: the selected clip, otherwise the cut nearest the playhead. */
const target = computed(() => {
  const current = project.value;
  if (!current) return undefined;
  const clips = current.videoTrack.clips;
  const selected = selection.clip.value;
  const eligible =
    selected && clipIndexOf(current, selected.id) < clips.length - 1
      ? selected
      : clips
          .slice(0, -1)
          .toSorted(
            (left, right) =>
              Math.abs(clipEndMs(left) - engine.positionMs()) -
              Math.abs(clipEndMs(right) - engine.positionMs()),
          )[0];
  return eligible && maxTransitionMs(current, eligible.id) >= 100 ? eligible : undefined;
});
const current = computed(() =>
  project.value && target.value ? transitionFrom(project.value, target.value.id) : undefined,
);
const maximum = computed(() =>
  project.value && target.value ? maxTransitionMs(project.value, target.value.id) : 0,
);

function apply(kind: VideoTransitionKind) {
  if (project.value && target.value) commit(applyTransition(project.value, target.value.id, kind));
}
function clear() {
  if (project.value && current.value) commit(removeTransition(project.value, current.value.id));
}
function resize(durationMs: number) {
  if (project.value && target.value && current.value)
    commit(applyTransition(project.value, target.value.id, current.value.kind, durationMs));
}
</script>

<template>
  <div class="flex min-h-0 flex-1 flex-col">
    <p class="shrink-0 px-3 py-2 text-caption text-fg-secondary">
      {{
        target
          ? t("spaces.media.videoEditor.transitions.target", {
              time: Math.round(clipEndMs(target) / 100) / 10,
            })
          : t("spaces.media.videoEditor.transitions.unavailable")
      }}
    </p>
    <UiScrollArea class="min-h-0 flex-1">
      <div class="grid grid-cols-2 gap-1.5 p-2">
        <button
          v-for="entry in KINDS"
          :key="entry.kind"
          type="button"
          class="flex h-16 flex-col items-center justify-center gap-1 rounded-lg border text-caption-sm transition-colors focus-visible:outline-2 focus-visible:outline-stroke-focus disabled:opacity-40"
          :class="
            current?.kind === entry.kind
              ? 'border-stroke-accent bg-surface-selected text-fg-accent'
              : 'border-stroke-default bg-surface-muted text-fg-secondary hover:border-stroke-accent hover:text-fg-primary'
          "
          :aria-pressed="current?.kind === entry.kind"
          :disabled="!target"
          @click="apply(entry.kind)"
        >
          <component :is="entry.icon" class="size-4" />
          {{ t(`spaces.media.videoEditor.transitions.kinds.${entry.kind}`) }}
        </button>
      </div>
    </UiScrollArea>
    <div v-if="current" class="shrink-0 space-y-2 border-t border-stroke-divider p-2">
      <SpaceVideoTimeField
        :label="t('spaces.media.videoEditor.transitions.duration')"
        :value="current.durationMs"
        :min="100"
        :max="maximum"
        @change="resize"
      />
      <UiButton size="sm" class="w-full justify-center" @click="clear">
        <Trash2 class="size-3.5" />{{ t("spaces.media.videoEditor.transitions.remove") }}
      </UiButton>
    </div>
  </div>
</template>
