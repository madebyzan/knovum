<script setup lang="ts">
import { Trash2 } from "@lucide/vue";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import { videoTransitionKinds } from "../../model/nodes/schema/video-edit-media";
import { applyTransition, maxTransitionMs, removeTransition, resizeTransition } from "../edits/transition";
import type { VideoTransition, VideoTransitionKind } from "../project";
import SpaceVideoTimeField from "./SpaceVideoTimeField.vue";
import type { VideoEditor } from "./use-video-editor";

const props = defineProps<{ editor: VideoEditor; transition: VideoTransition }>();
const { t } = useI18n();
const { project, commit } = props.editor;
const maximum = computed(() =>
  project.value ? maxTransitionMs(project.value, props.transition.fromClipId) : 0,
);
const kinds = computed(() =>
  videoTransitionKinds.map((kind) => ({
    value: kind,
    label: t(`spaces.media.videoEditor.transitions.kinds.${kind}`),
  })),
);

function setKind(kind: VideoTransitionKind) {
  if (!project.value) return;
  commit(applyTransition(project.value, props.transition.fromClipId, kind, props.transition.durationMs));
}
</script>

<template>
  <div class="space-y-4 p-3">
    <label class="block space-y-1 text-caption text-fg-secondary">
      {{ t("spaces.media.videoEditor.transitions.choose") }}
      <UiSelect
        size="sm"
        :model-value="transition.kind"
        :options="kinds"
        :label="t('spaces.media.videoEditor.transitions.choose')"
        @update:model-value="setKind"
      />
    </label>
    <SpaceVideoTimeField
      :label="t('spaces.media.videoEditor.transitions.duration')"
      :value="transition.durationMs"
      :min="100"
      :max="maximum"
      @change="commit(resizeTransition(project!, transition.id, $event))"
    />
    <p class="text-caption-sm text-fg-tertiary">
      {{ t("spaces.media.videoEditor.transitions.hint") }}
    </p>
    <UiButton
      size="sm"
      class="w-full justify-center"
      @click="commit(removeTransition(project!, transition.id))"
    >
      <Trash2 class="size-3.5" />{{ t("spaces.media.videoEditor.transitions.remove") }}
    </UiButton>
  </div>
</template>
