<script setup lang="ts">
import { Check, Play, Square } from "@lucide/vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import { computed, onBeforeUnmount, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import type { PreparedVoiceReference } from "../../contracts/generated/PreparedVoiceReference";
import {
  useVoiceReferenceEditor,
  type VoiceReferenceEditorProps,
} from "../../speech/use-voice-reference-editor";
import SpeechMediaPlayer from "./SpeechMediaPlayer.vue";
import VoiceReferenceRange from "./VoiceReferenceRange.vue";

const props = defineProps<VoiceReferenceEditorProps>();
const model = defineModel<PreparedVoiceReference | undefined>();
const emit = defineEmits<{ busy: [value: boolean]; dirty: [value: boolean] }>();
const { t } = useI18n();
const player = useTemplateRef("player");
const {
  path,
  start,
  end,
  durationMs,
  sourceVersion,
  jobId,
  error,
  selectionValid,
  canPrepare,
  cancel,
  choose,
  prepare,
  selectionLoaded,
} = useVoiceReferenceEditor(props, model);
const dirty = computed(() => !props.sourceNode && Boolean(path.value));
watch(dirty, (value) => emit("dirty", value));
onBeforeUnmount(() => emit("busy", false));
const editingDisabled = computed(() => props.disabled || Boolean(jobId.value));
function setRange(from: number, to: number) {
  start.value = String(from);
  end.value = String(to);
}

async function prepareReference() {
  emit("busy", true);
  try {
    await prepare();
  } finally {
    emit("busy", false);
  }
}
</script>

<template>
  <section class="space-y-2">
    <UiButton
      v-if="!sourceNode"
      data-testid="reference-choose"
      size="sm"
      :disabled="disabled || Boolean(jobId)"
      @click="choose"
      >{{ t("spaces.speech.reference.choose") }}</UiButton
    >
    <template v-if="path">
      <SpeechMediaPlayer
        :key="sourceVersion"
        ref="player"
        :kind="sourceNode?.data.kind === 'video' ? 'video' : 'audio'"
        :source="convertFileSrc(path)"
        :preview="false"
        @loaded="selectionLoaded"
      />
      <p
        v-if="durationMs !== undefined && durationMs < 5000"
        class="text-caption text-fg-secondary"
        role="status"
      >
        {{ t("spaces.speech.reference.tooShort") }}
      </p>
      <VoiceReferenceRange
        :start="Number(start)"
        :end="Number(end)"
        :duration-ms="durationMs ?? 0"
        :disabled="editingDisabled || durationMs === undefined"
        @change="setRange"
      />
      <p class="text-caption-sm" :class="selectionValid ? 'text-fg-secondary' : 'text-fg-danger'">
        {{
          t("spaces.speech.reference.selection", {
            seconds: Math.max(0, Number(end) - Number(start)).toFixed(1),
          })
        }}
      </p>
      <div class="grid grid-cols-2 gap-3">
        <label class="space-y-1 text-caption text-fg-secondary">
          <span>{{ t("spaces.speech.reference.start") }}</span>
          <UiInput
            v-model="start"
            data-testid="reference-start"
            size="sm"
            inputmode="decimal"
            :disabled="editingDisabled"
          />
        </label>
        <label class="space-y-1 text-caption text-fg-secondary">
          <span>{{ t("spaces.speech.reference.end") }}</span>
          <UiInput
            v-model="end"
            data-testid="reference-end"
            size="sm"
            inputmode="decimal"
            :disabled="editingDisabled"
          />
        </label>
      </div>
      <div class="flex flex-wrap items-center gap-2">
        <UiButton
          size="sm"
          :disabled="!selectionValid || editingDisabled"
          @click="player?.seekTo(Number(start) * 1000, Number(end) * 1000)"
        >
          <Play class="size-3.5" />{{ t("spaces.speech.reference.play") }}
        </UiButton>
        <UiButton v-if="jobId" data-testid="reference-cancel" size="sm" @click="cancel">
          <Square class="size-3.5" />{{ t("common.cancel") }}
        </UiButton>
        <UiButton
          v-else
          data-testid="reference-prepare"
          size="sm"
          :disabled="!canPrepare"
          @click="prepareReference"
        >
          <Check v-if="model" class="size-3.5 text-fg-accent" />{{
            t(model ? "spaces.speech.reference.prepared" : "spaces.speech.reference.prepare")
          }}
        </UiButton>
      </div>
    </template>
    <p v-if="error" class="text-caption text-fg-danger" role="alert">{{ error }}</p>
  </section>
</template>
