<script setup lang="ts">
import { Check, Headphones, Music2, X } from "@lucide/vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { ToolbarButton, ToolbarRoot, ToolbarSeparator } from "reka-ui";
import { shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiActionHelp from "../../../../shared/ui/UiActionHelp.vue";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiIconButton from "../../../../shared/ui/UiIconButton.vue";
import UiPopover from "../../../../shared/ui/UiPopover.vue";
import type { useSourceSeparationCandidate } from "../../speech/use-source-separation-candidate";
import { CANVAS_ACTION_BAR_CLASSES, CANVAS_ACTION_BUTTON_CLASSES } from "../canvas/canvas-action-classes";
import SeparatedTrackActions from "./SeparatedTrackActions.vue";
import SpeechMediaPlayer from "./SpeechMediaPlayer.vue";

const props = defineProps<{
  compact?: boolean;
  separation: ReturnType<typeof useSourceSeparationCandidate>;
}>();
const emit = defineEmits<{ close: [] }>();
const { t } = useI18n();
const listened = shallowRef(new Set<string>());
const players = useTemplateRef<InstanceType<typeof SpeechMediaPlayer>[]>("players");
const playingStem = shallowRef<"vocals" | "background">();
const stems = ["vocals", "background"] as const;
watch(
  () => props.separation.candidate.value?.separationId,
  () => {
    listened.value = new Set();
  },
);
function selectStem(stem: (typeof stems)[number], open: boolean) {
  players.value?.forEach((player) => player.pause());
  playingStem.value = open ? stem : undefined;
}
async function close() {
  const pending = props.separation.candidate.value?.reviewStatus === "pending";
  if (pending) {
    await props.separation.review(false);
    if (props.separation.candidate.value?.reviewStatus !== "rejected") return;
  }
  emit("close");
}
</script>

<template>
  <ToolbarRoot
    v-if="separation.candidate.value"
    :class="CANVAS_ACTION_BAR_CLASSES"
    :aria-label="t('spaces.speech.separation.title')"
    :loop="false"
    @keydown.esc.stop="close"
  >
    <Music2 class="ml-2 size-4 text-fg-secondary" :stroke-width="1.6" />
    <span class="px-1.5 text-label font-medium" :class="compact && 'sr-only'">{{
      t("spaces.speech.separation.tracksReady")
    }}</span>
    <ToolbarSeparator class="mx-0.5 h-4 w-px bg-stroke-divider" />
    <UiPopover
      v-for="stem in stems"
      :key="stem"
      :open="playingStem === stem"
      align="start"
      class="w-72 space-y-3 p-3"
      @update:open="selectStem(stem, $event)"
    >
      <template #trigger>
        <ToolbarButton as-child
          ><button type="button" :class="CANVAS_ACTION_BUTTON_CLASSES">
            <Check v-if="listened.has(stem)" class="size-4" :stroke-width="1.6" /><Headphones
              v-else
              class="size-4"
              :stroke-width="1.6"
            />{{ t(`spaces.speech.separation.stems.${stem}`) }}
          </button></ToolbarButton
        >
      </template>
      <SpeechMediaPlayer
        ref="players"
        kind="audio"
        :source="convertFileSrc(separation.candidate.value[stem].assetPath)"
        @play="listened = new Set([...listened, stem])"
      />
      <p
        v-if="separation.candidate.value.reviewStatus === 'pending'"
        class="text-caption-sm leading-relaxed text-fg-secondary"
      >
        {{ t("spaces.speech.separation.reviewWarning") }}
      </p>
      <SeparatedTrackActions
        v-else
        :candidate="separation.candidate.value"
        :stem="stem"
        :disabled="separation.busy.value"
      />
    </UiPopover>
    <ToolbarSeparator class="mx-0.5 h-4 w-px bg-stroke-divider" />
    <UiButton
      v-if="separation.candidate.value.reviewStatus === 'pending'"
      variant="primary"
      size="sm"
      class="mx-1 bg-surface-inverse text-fg-inverse hover:bg-surface-inverse hover:opacity-90"
      :disabled="separation.busy.value || listened.size !== 2"
      @click="separation.review(true)"
      ><Check class="size-3.5" />{{ t("spaces.speech.separation.accept") }}</UiButton
    >
    <UiActionHelp
      v-if="
        separation.error.value ||
        (separation.candidate.value.reviewStatus === 'pending' && listened.size !== 2)
      "
      :label="separation.error.value || t('spaces.speech.separation.reviewWarning')"
    />
    <ToolbarButton as-child
      ><UiIconButton
        :label="
          t(
            separation.candidate.value.reviewStatus === 'pending'
              ? 'spaces.speech.separation.reject'
              : 'common.close',
          )
        "
        :disabled="separation.busy.value"
        size="sm"
        @click="close"
        ><X class="size-3.5" /></UiIconButton
    ></ToolbarButton>
  </ToolbarRoot>
</template>
