<script setup lang="ts">
import { FileUp } from "@lucide/vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { onScopeDispose, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import type { VideoDubbingOutputConfig } from "../../contracts/generated/VideoDubbingOutputConfig";
import { parseDubbingSubtitles, type DubbingSubtitle } from "../dubbing-subtitles";
import { readDubbingSubtitles } from "../video-delivery-ipc";
import type { VideoLineDraft } from "../video-dubbing-draft";
import SpeechMediaPlayer from "./SpeechMediaPlayer.vue";
import VideoSpeakerBindings from "./VideoSpeakerBindings.vue";

const config = defineModel<VideoDubbingOutputConfig>({ required: true });
defineProps<{
  lines: readonly VideoLineDraft[];
  voiceOptions: readonly { value: string; label: string }[];
  disabled: boolean;
}>();
const emit = defineEmits<{
  bind: [speakerKey: string, revisionId: string];
  applyAll: [revisionId: string];
  imported: [cues: DubbingSubtitle[]];
}>();
const { t } = useI18n();
const player = useTemplateRef("player");
const error = shallowRef<string>();
const importing = shallowRef(false);
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
  <section class="space-y-2.5">
    <SpeechMediaPlayer
      ref="player"
      kind="video"
      :preview="false"
      :source="convertFileSrc(config.video.assetPath)"
    />
    <div class="flex flex-wrap items-center gap-1.5">
      <UiButton size="sm" variant="secondary" :disabled="disabled || importing" @click="importSubtitles">
        <FileUp class="size-3.5" />
        {{ t("spaces.speech.videoDubbing.importSubtitles") }}
      </UiButton>
    </div>
    <p v-if="error" class="text-caption text-fg-danger">{{ error }}</p>
    <VideoSpeakerBindings
      :lines="lines"
      :voice-options="voiceOptions"
      :disabled="disabled"
      @bind="(speakerKey, revisionId) => emit('bind', speakerKey, revisionId)"
      @apply-all="(revisionId) => emit('applyAll', revisionId)"
    />
  </section>
</template>
