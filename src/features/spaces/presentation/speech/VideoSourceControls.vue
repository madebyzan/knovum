<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { computed, onScopeDispose, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import type { VideoDubbingOutputConfig } from "../../contracts/generated/VideoDubbingOutputConfig";
import { parseDubbingSubtitles, type DubbingSubtitle } from "../../speech/dubbing-subtitles";
import { readDubbingSubtitles } from "../../speech/video-delivery-ipc";
import type { VideoLineDraft } from "../../speech/video-dubbing-draft";
import SpeechMediaPlayer from "./SpeechMediaPlayer.vue";

const config = defineModel<VideoDubbingOutputConfig>({ required: true });
const props = defineProps<{
  lines: readonly VideoLineDraft[];
  voiceOptions: readonly { value: string; label: string }[];
  disabled: boolean;
}>();
const emit = defineEmits<{
  bind: [speakerKey: string, revisionId: string];
  imported: [cues: DubbingSubtitle[]];
}>();
const { t } = useI18n();
const player = useTemplateRef("player");
const error = shallowRef<string>();
const importing = shallowRef(false);
const speakers = computed(() => {
  const voiceRevisionIdsBySpeakerKey = new Map<string, Set<string>>();
  for (const line of props.lines) {
    const key = line.speakerKey.trim();
    if (!key) continue;
    const revisions = voiceRevisionIdsBySpeakerKey.get(key) ?? new Set<string>();
    revisions.add(line.voiceRevisionId);
    voiceRevisionIdsBySpeakerKey.set(key, revisions);
  }
  return [...voiceRevisionIdsBySpeakerKey].map(([key, revisions]) => ({
    key,
    revisionId: revisions.size === 1 ? [...revisions][0]! : "",
  }));
});
let epoch = 0;
watch(
  () => config.value.video.sourceRevision,
  () => {
    epoch += 1;
  },
);
onScopeDispose(() => {
  epoch += 1;
});
async function importSubtitles() {
  if (importing.value) return;
  importing.value = true;
  const version = epoch;
  try {
    const path = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "Subtitles", extensions: ["srt", "vtt"] }],
    });
    if (typeof path !== "string" || version !== epoch) return;
    const text = await readDubbingSubtitles(path);
    if (version !== epoch) return;
    const cues = parseDubbingSubtitles(text, t("spaces.speech.project.defaultRole"));
    if (cues.some((cue) => cue.endMs > config.value.video.durationMs))
      throw new Error(t("spaces.speech.videoDubbing.subtitleBeyondSource"));
    emit("imported", cues);
    error.value = undefined;
  } catch (cause) {
    if (version === epoch) error.value = getErrorMessage(cause);
  } finally {
    importing.value = false;
  }
}
defineExpose({ audition: (startMs: number, endMs: number) => player.value?.seekTo(startMs, endMs) });
</script>

<template>
  <section class="space-y-2">
    <SpeechMediaPlayer
      ref="player"
      kind="video"
      :preview="false"
      :source="convertFileSrc(config.video.assetPath)"
    />
    <UiButton size="sm" :disabled="disabled || importing" @click="importSubtitles">{{
      t("spaces.speech.videoDubbing.importSubtitles")
    }}</UiButton>
    <p v-if="error" class="text-caption text-fg-danger">{{ error }}</p>
    <div v-for="speaker in speakers" :key="speaker.key" class="flex items-center gap-2">
      <span class="max-w-24 shrink-0 truncate text-caption text-fg-secondary" :title="speaker.key">{{
        speaker.key
      }}</span>
      <UiSelect
        class="min-w-0 flex-1"
        :model-value="speaker.revisionId"
        size="sm"
        :options="voiceOptions"
        :label="t('spaces.speech.voice.label')"
        :placeholder="t('spaces.speech.videoDubbing.mixedVoiceBindings')"
        :disabled
        @update:model-value="emit('bind', speaker.key, $event)"
      />
    </div>
  </section>
</template>
