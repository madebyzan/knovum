<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { SpeechMediaSource } from "../../contracts/generated/SpeechMediaSource";
import type { SpeechSourceSeparationCandidate } from "../../contracts/generated/SpeechSourceSeparationCandidate";
import type { VideoDubbingOutputConfig } from "../../contracts/generated/VideoDubbingOutputConfig";
import { listAcceptedSourceSeparations } from "../../speech/source-separation-ipc";
import { useSpeechMediaSources } from "../../speech/use-speech-media-sources";

const props = defineProps<{ spaceId: string; config: VideoDubbingOutputConfig; disabled: boolean }>();
const emit = defineEmits<{
  background: [source: SpeechMediaSource | null];
  separated: [candidate: SpeechSourceSeparationCandidate];
}>();
const { t } = useI18n();
const { backgrounds } = useSpeechMediaSources();
const accepted = useQuery({
  queryKey: computed(() => ["spaces", "speech", "separations", props.spaceId, props.config.video]),
  queryFn: () => listAcceptedSourceSeparations(props.spaceId, props.config.video),
});
const value = computed(() => {
  if (props.config.backgroundSeparation)
    return `separation:${props.config.backgroundSeparation.separationId}`;
  const current = props.config.background;
  if (!current) return "mute";
  return (
    backgrounds.value.find(
      ({ source }) =>
        source.assetPath === current.assetPath && source.sourceRevision === current.sourceRevision,
    )?.id ?? "current-background"
  );
});
const options = computed(() => {
  const choices = [
    { value: "mute", label: t("spaces.speech.videoDubbing.muteOriginal") },
    ...backgrounds.value.map(({ id, name }) => ({ value: id, label: name })),
    ...(accepted.data.value ?? []).map((candidate, index) => ({
      value: `separation:${candidate.separationId}`,
      label: `${t("spaces.speech.separation.acceptedBackground")} · ${index + 1}`,
    })),
  ];
  if (!choices.some((choice) => choice.value === value.value)) {
    choices.push({
      value: value.value,
      label: t(
        props.config.backgroundSeparation
          ? "spaces.speech.separation.acceptedBackground"
          : "spaces.speech.videoDubbing.currentBackground",
      ),
    });
  }
  return choices;
});
function select(selected: string) {
  if (selected === value.value) return;
  const separation = accepted.data.value?.find(
    (candidate) => `separation:${candidate.separationId}` === selected,
  );
  if (separation) emit("separated", separation);
  else
    emit(
      "background",
      selected === "mute" ? null : backgrounds.value.find(({ id }) => id === selected)!.source,
    );
}
</script>

<template>
  <UiSelect
    :model-value="value"
    size="sm"
    :label="t('spaces.speech.videoDubbing.backgroundAudio')"
    :options
    :disabled
    @update:model-value="select"
  />
  <div v-if="accepted.error.value" role="alert" class="flex items-center gap-2 text-caption text-fg-danger">
    <span>{{ accepted.error.value.message }}</span>
    <UiButton size="sm" :disabled="accepted.isFetching.value" @click="accepted.refetch()">
      {{ t("common.retry") }}
    </UiButton>
  </div>
</template>
