<script setup lang="ts">
import { convertFileSrc } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { computed, onScopeDispose, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../../foundation/errors/error-format";
import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import type { PreparedVoiceReference } from "../../contracts/generated/PreparedVoiceReference";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import { cancelSpeechGeneration } from "../../speech/ipc";
import { authorizeVoiceReferenceSource, prepareVoiceReference } from "../../speech/voice-reference-ipc";
import SpeechMediaPlayer from "./SpeechMediaPlayer.vue";

const props = defineProps<{
  language: string;
  disabled: boolean;
  sourceNode?: SpaceCanvasNode | undefined;
}>();
const model = defineModel<PreparedVoiceReference | undefined>();
const emit = defineEmits<{ busy: [value: boolean] }>();
const { t } = useI18n();
const store = useSpaceStore();
const path = shallowRef(props.sourceNode?.data.kind === "audio" ? props.sourceNode.data.assetPath : "");
const player = useTemplateRef("player");
const start = shallowRef("0");
const end = shallowRef("10");
const jobId = shallowRef<string>();
const error = shallowRef<string>();
const selectionValid = computed(
  () =>
    Number.isFinite(Number(start.value)) &&
    Number(end.value) - Number(start.value) >= 5 &&
    Number(end.value) - Number(start.value) <= 30 &&
    Number(start.value) >= 0,
);
const canPrepare = computed(() =>
  Boolean(path.value && selectionValid.value && !jobId.value && store.activeSpaceId && !props.disabled),
);
let epoch = 0;
let disposed = false;
async function cancel() {
  if (jobId.value) await cancelSpeechGeneration(jobId.value);
}
function invalidate() {
  epoch += 1;
  model.value = undefined;
  void cancel().catch((cause) => {
    error.value = getErrorMessage(cause);
  });
}
watch([start, end, () => props.language, () => store.activeSpaceId], invalidate);
watch(
  () => props.sourceNode,
  (node) => {
    if (node?.data.kind === "audio") {
      path.value = node.data.assetPath;
      invalidate();
    }
  },
);
watch(jobId, (job) => emit("busy", Boolean(job)));
onScopeDispose(() => {
  disposed = true;
  invalidate();
});
async function choose() {
  const version = epoch;
  try {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [
        { name: t("spaces.speech.referenceAudio"), extensions: ["wav", "flac", "mp3", "m4a", "ogg"] },
      ],
    });
    if (typeof selected !== "string" || disposed || version !== epoch) return;
    const authorizedPath = await authorizeVoiceReferenceSource(selected);
    if (disposed || version !== epoch) return;
    invalidate();
    path.value = authorizedPath;
    error.value = undefined;
  } catch (cause) {
    error.value = getErrorMessage(cause);
  }
}
async function prepare() {
  const spaceId = store.activeSpaceId;
  if (!canPrepare.value || !spaceId) return;
  const version = ++epoch;
  const job = crypto.randomUUID();
  jobId.value = job;
  model.value = undefined;
  try {
    const prepared = await prepareVoiceReference({
      jobId: job,
      spaceId,
      sourcePath: path.value,
      language: props.language,
      startMs: Math.round(Number(start.value) * 1000),
      endMs: Math.round(Number(end.value) * 1000),
    });
    if (props.sourceNode && prepared.sourceRevision !== props.sourceNode.data.sourceRevision)
      throw new Error(t("spaces.speech.referenceNodeChanged"));
    if (version === epoch) {
      model.value = prepared;
      error.value = undefined;
    }
  } catch (cause) {
    if (version === epoch) error.value = getErrorMessage(cause);
  } finally {
    if (jobId.value === job) jobId.value = undefined;
  }
}
function selectionLoaded(durationMs: number) {
  end.value = String(Math.min(durationMs / 1000, 30));
}
</script>

<template>
  <section class="space-y-2">
    <UiButton v-if="!sourceNode" size="sm" :disabled="disabled || Boolean(jobId)" @click="choose">{{
      t("spaces.speech.chooseReference")
    }}</UiButton>
    <template v-if="path">
      <SpeechMediaPlayer ref="player" kind="audio" :source="convertFileSrc(path)" @loaded="selectionLoaded" />
      <UiButton
        size="sm"
        :disabled="!selectionValid"
        @click="player?.seekTo(Number(start) * 1000, Number(end) * 1000)"
        >{{ t("spaces.speech.playSelection") }}</UiButton
      >
      <div class="grid grid-cols-2 gap-2">
        <label class="text-caption text-fg-secondary"
          >{{ t("spaces.speech.referenceStart")
          }}<UiInput
            v-model="start"
            size="sm"
            type="number"
            min="0"
            step="0.1"
            :disabled="disabled || Boolean(jobId)"
        /></label>
        <label class="text-caption text-fg-secondary"
          >{{ t("spaces.speech.referenceEnd")
          }}<UiInput
            v-model="end"
            size="sm"
            type="number"
            min="0"
            step="0.1"
            :disabled="disabled || Boolean(jobId)"
        /></label>
      </div>
      <UiButton size="sm" :disabled="!canPrepare" @click="prepare">{{
        t("spaces.speech.prepareReference")
      }}</UiButton>
      <UiButton v-if="jobId" size="sm" @click="cancel">{{ t("spaces.speech.cancel") }}</UiButton>
    </template>
    <p v-if="error" class="text-caption text-fg-danger">{{ error }}</p>
  </section>
</template>
