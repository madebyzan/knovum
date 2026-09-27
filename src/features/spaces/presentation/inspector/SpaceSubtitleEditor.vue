<script setup lang="ts">
import { computed, reactive, watch } from "vue";
import { useI18n } from "vue-i18n";
import { toast } from "vue-sonner";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiNumberField from "../../../../shared/ui/UiNumberField.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import { useNodeOperationGate } from "../../canvas/node-operation-gate";
import { renderSubtitledVideo, subtitleVideoInputs } from "../../knowledge-derivations/subtitled-video";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceSubtitlePayload } from "../../model/nodes/node-payload";
import { useSpaceStore } from "../../session/space-store";
import SpaceSubtitleCuePicker from "../media/SpaceSubtitleCuePicker.vue";
import { useNodeDraft } from "../use-node-draft";

const props = defineProps<{ node: SpaceCanvasNode }>();
const store = useSpaceStore();
const { t } = useI18n();
const nodeOperations = useNodeOperationGate();
const nodeDraft = useNodeDraft(
  computed(() => props.node.id),
  "subtitle",
);
const draft = reactive({ index: 0, startMs: 0, endMs: 0, text: "" });
const data = computed(() => (props.node.data.kind === "subtitle" ? props.node.data : undefined));
const cue = computed(() => data.value?.cues[draft.index]);
const burnInInputs = computed(() => {
  const source = store.nodes.find((node) => node.id === data.value?.sourceVideoNodeId);
  return source ? subtitleVideoInputs([source, props.node]) : undefined;
});
const dirty = computed(() => {
  const current = cue.value;
  return current
    ? draft.startMs !== current.startMs || draft.endMs !== current.endMs || draft.text.trim() !== current.text
    : false;
});
const validDraft = computed(() => {
  const cues = data.value?.cues;
  const previous = cues?.[draft.index - 1];
  const next = cues?.[draft.index + 1];
  return hasValidCueContent() && fitsCueTimeline(previous?.endMs, next?.startMs);
});

function hasValidCueContent() {
  return (
    Number.isInteger(draft.startMs) &&
    Number.isInteger(draft.endMs) &&
    draft.endMs > draft.startMs &&
    draft.endMs <= (data.value?.durationMs ?? 0) &&
    Boolean(draft.text.trim())
  );
}
function fitsCueTimeline(previousEnd?: number, nextStart?: number) {
  return draft.startMs >= (previousEnd ?? 0) && draft.endMs <= (nextStart ?? Number.MAX_SAFE_INTEGER);
}

watch(cue, loadCue, { immediate: true });
function loadCue() {
  const value = cue.value;
  if (!value) return;
  draft.startMs = value.startMs;
  draft.endMs = value.endMs;
  draft.text = value.text;
}
function select(index: number) {
  if (dirty.value) return;
  nodeDraft.end();
  draft.index = index;
}
function cancelSubtitleEdit() {
  loadCue();
  nodeDraft.end();
}
async function saveSubtitleCue() {
  if (!data.value || !validDraft.value || !nodeDraft.canContinue.value) return;
  const cues = data.value.cues.map((value, index) =>
    index === draft.index ? { startMs: draft.startMs, endMs: draft.endMs, text: draft.text.trim() } : value,
  );
  try {
    await persist({ ...data.value, cues });
    nodeDraft.end();
  } catch (error) {
    toast.error(`${t("spaces.creation.video.subtitles.saveFailed")}: ${String(error)}`);
  }
}
function persist(subtitle: SpaceSubtitlePayload) {
  return store.saveNodeContent(props.node.id, props.node.data, subtitle);
}
function generateVideo() {
  if (nodeOperations.allows("derive") && burnInInputs.value)
    void renderSubtitledVideo(store, burnInInputs.value);
}
</script>

<template>
  <div v-if="data" class="space-y-3">
    <SpaceSubtitleCuePicker :cues="data.cues" :active-index="draft.index" @select="select" />
    <div
      v-if="cue"
      class="space-y-2 rounded-panel border border-divider p-2.5"
      @focusin="nodeDraft.begin()"
      @focusout="!dirty && nodeDraft.end()"
    >
      <div class="grid grid-cols-2 gap-2">
        <label class="block text-micro text-foreground-tertiary">
          <span class="mb-1 block">{{ t("spaces.creation.video.cues.start") }}</span>
          <UiNumberField
            v-model="draft.startMs"
            :disabled="!nodeDraft.canContinue.value"
            :min="0"
            size="sm"
          />
        </label>
        <label class="block text-micro text-foreground-tertiary">
          <span class="mb-1 block">{{ t("spaces.creation.video.cues.end") }}</span>
          <UiNumberField v-model="draft.endMs" :disabled="!nodeDraft.canContinue.value" :min="1" size="sm" />
        </label>
      </div>
      <UiTextarea v-model="draft.text" :disabled="!nodeDraft.canContinue.value" :rows="3" />
      <UiButton v-if="dirty" size="sm" variant="ghost" @click="cancelSubtitleEdit">{{
        t("common.cancel")
      }}</UiButton>
      <UiButton size="sm" :disabled="!dirty || !validDraft" @click="saveSubtitleCue">{{
        t("common.save")
      }}</UiButton>
    </div>
    <UiButton
      v-if="burnInInputs"
      class="w-full"
      variant="primary"
      :disabled="dirty || !nodeOperations.allows('derive')"
      @click="generateVideo"
    >
      {{ t("spaces.creation.video.subtitles.render") }}
    </UiButton>
  </div>
</template>
