<script setup lang="ts">
import { open } from "@tauri-apps/plugin-dialog";
import { computed, shallowRef } from "vue";
import { useI18n } from "vue-i18n";

import { getErrorMessage } from "../../../foundation/errors/error-format";
import UiButton from "../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../shared/ui/UiCheckbox.vue";
import UiInput from "../../../shared/ui/UiInput.vue";
import UiSelect from "../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../shared/ui/UiTextarea.vue";
import type { VoiceProfileSummary } from "./contracts";
import { useVoiceCandidates } from "./use-voice-candidates";

const emit = defineEmits<{ created: [voice: VoiceProfileSummary] }>();
const { t } = useI18n();
const mode = shallowRef("reference");
const name = shallowRef("");
const language = shallowRef("zh-CN");
const description = shallowRef("");
const referencePath = shallowRef("");
const transcript = shallowRef("");
const transcriptConfirmed = shallowRef(false);
const authorized = shallowRef(false);
const text = shallowRef(t("spaces.speech.candidateSample"));
const dialogError = shallowRef<string>();
const { candidate, activeRunId, busy, error, draft, save, cancel, discard, audition } = useVoiceCandidates(
  (voice) => emit("created", voice),
);
const modes = computed(() => [
  { value: "reference", label: t("spaces.speech.cloneVoice") },
  { value: "designed", label: t("spaces.speech.designVoice") },
]);
const languages = [
  { value: "zh-CN", label: "中文" },
  { value: "en", label: "English" },
];
const ready = computed(
  () =>
    authorized.value &&
    text.value.trim() &&
    (mode.value === "designed"
      ? description.value.trim()
      : referencePath.value && (!transcript.value.trim() || transcriptConfirmed.value)),
);

async function chooseReference() {
  try {
    const path = await open({
      multiple: false,
      directory: false,
      filters: [
        { name: t("spaces.speech.referenceAudio"), extensions: ["wav", "flac", "mp3", "m4a", "ogg"] },
      ],
    });
    if (typeof path === "string") {
      referencePath.value = path;
      transcriptConfirmed.value = false;
    }
    dialogError.value = undefined;
  } catch (cause) {
    dialogError.value = getErrorMessage(cause);
  }
}
function generate() {
  if (!ready.value || busy.value) return;
  void draft({
    language: language.value,
    text: text.value.trim(),
    authorized: authorized.value,
    source:
      mode.value === "designed"
        ? { kind: "designed", description: description.value.trim() }
        : {
            kind: "reference",
            sourcePath: referencePath.value,
            referenceText: transcript.value.trim() || null,
            referenceTextConfirmed: transcriptConfirmed.value,
          },
  });
}
</script>

<template>
  <section class="space-y-2 border-t border-stroke-divider pt-2">
    <div class="flex gap-2">
      <UiSelect
        v-model="mode"
        class="min-w-0 flex-1"
        size="sm"
        :label="t('spaces.speech.voiceMode')"
        :options="modes"
        :disabled="busy"
      />
      <UiSelect
        v-model="language"
        size="sm"
        :label="t('spaces.speech.language')"
        :options="languages"
        :disabled="busy"
      />
    </div>
    <template v-if="mode === 'reference'">
      <UiButton size="sm" :disabled="busy" @click="chooseReference">{{
        t("spaces.speech.chooseReference")
      }}</UiButton>
      <p v-if="referencePath" class="truncate text-caption text-fg-secondary" :title="referencePath">
        {{ referencePath }}
      </p>
      <p class="text-caption-sm text-fg-tertiary">{{ t("spaces.speech.referenceRequirements") }}</p>
      <UiTextarea
        v-model="transcript"
        :rows="2"
        :disabled="busy"
        :placeholder="t('spaces.speech.referenceTranscript')"
        @update:model-value="transcriptConfirmed = false"
      />
      <UiCheckbox
        v-if="transcript.trim()"
        v-model="transcriptConfirmed"
        :disabled="busy"
        :label="t('spaces.speech.transcriptConfirmed')"
      />
    </template>
    <UiTextarea
      v-else
      v-model="description"
      :rows="2"
      :disabled="busy"
      :placeholder="t('spaces.speech.designDescription')"
    />
    <UiTextarea v-model="text" :rows="3" :disabled="busy" :placeholder="t('spaces.speech.candidateText')" />
    <UiCheckbox v-model="authorized" :disabled="busy" :label="t('spaces.speech.voiceAuthorization')" />
    <div class="flex gap-2">
      <UiButton size="sm" :disabled="busy || !ready || Boolean(candidate)" @click="generate">{{
        t("spaces.speech.generateCandidate")
      }}</UiButton>
      <UiButton v-if="activeRunId" size="sm" variant="dangerSubtle" @click="cancel">{{
        t("spaces.speech.cancel")
      }}</UiButton>
    </div>
    <p v-if="error || dialogError" class="text-caption text-fg-danger">{{ error || dialogError }}</p>
    <div v-if="candidate" class="space-y-2">
      <audio class="h-8 w-full" controls :src="candidate.source" @ended="audition" />
      <p v-if="candidate.auditionedAt === null" class="text-caption text-fg-tertiary">
        {{ t("spaces.speech.candidateAuditionRequired") }}
      </p>
      <UiInput v-model="name" size="sm" :disabled="busy" :placeholder="t('spaces.speech.voiceName')" />
      <div class="flex gap-2">
        <UiButton
          size="sm"
          variant="primary"
          :disabled="busy || !name.trim() || !authorized || candidate.auditionedAt === null"
          @click="save(name.trim(), authorized)"
          >{{ t("spaces.speech.saveCandidate") }}</UiButton
        >
        <UiButton size="sm" :disabled="busy" @click="discard">{{
          t("spaces.speech.discardCandidate")
        }}</UiButton>
      </div>
    </div>
  </section>
</template>
