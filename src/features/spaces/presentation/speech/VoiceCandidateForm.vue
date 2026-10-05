<script setup lang="ts">
import { computed, shallowRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiButton from "../../../../shared/ui/UiButton.vue";
import UiCheckbox from "../../../../shared/ui/UiCheckbox.vue";
import UiInput from "../../../../shared/ui/UiInput.vue";
import UiSelect from "../../../../shared/ui/UiSelect.vue";
import UiTextarea from "../../../../shared/ui/UiTextarea.vue";
import type { PreparedVoiceReference } from "../../contracts/generated/PreparedVoiceReference";
import type { VoiceLibraryRevision } from "../../contracts/generated/VoiceLibraryRevision";
import { useVoiceCandidates } from "../../speech/use-voice-candidates";
import VoiceCandidateCard from "./VoiceCandidateCard.vue";
import VoiceReferenceEditor from "./VoiceReferenceEditor.vue";

const emit = defineEmits<{ created: [voice: VoiceLibraryRevision] }>();
const { t } = useI18n();
const mode = shallowRef("reference");
const seed = shallowRef("0");
function newSeed() {
  seed.value = String(crypto.getRandomValues(new Uint32Array(1))[0]);
}
const language = shallowRef("zh-CN");
const description = shallowRef("");
const reference = shallowRef<PreparedVoiceReference>();
const preparingReference = shallowRef(false);
const transcript = shallowRef("");
const transcriptConfirmed = shallowRef(false);
const authorized = shallowRef(false);
const text = shallowRef(t("spaces.speech.candidateSample"));
watch(reference, (prepared) => {
  transcript.value = prepared?.transcript ?? "";
  transcriptConfirmed.value = false;
});
const { candidates, activeRunId, busy, error, draft, save, cancel, discard, audition } = useVoiceCandidates(
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
    /^\d+$/u.test(seed.value) &&
    Number(seed.value) <= 4_294_967_295 &&
    authorized.value &&
    text.value.trim() &&
    (mode.value === "designed"
      ? description.value.trim()
      : reference.value &&
        !preparingReference.value &&
        (!transcript.value.trim() || transcriptConfirmed.value)),
);

function generate() {
  if (!ready.value || busy.value) return;
  const prepared = reference.value;
  if (mode.value === "reference" && !prepared) return;
  const source =
    mode.value === "designed"
      ? { kind: "designed" as const, description: description.value.trim() }
      : prepared
        ? {
            kind: "reference" as const,
            sourcePath: prepared.sourcePath,
            sourceRevision: prepared.sourceRevision,
            startMs: prepared.startMs,
            endMs: prepared.endMs,
            referenceText: transcript.value.trim() || null,
            referenceTextConfirmed: transcriptConfirmed.value,
          }
        : undefined;
  if (!source) return;
  void draft({
    language: language.value,
    text: text.value.trim(),
    authorized: authorized.value,
    seed: Number(seed.value),
    source,
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
      <VoiceReferenceEditor
        v-model="reference"
        :language="language"
        :disabled="busy"
        @busy="preparingReference = $event"
      />
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
      <UiInput
        v-model="seed"
        size="sm"
        type="number"
        min="0"
        max="4294967295"
        :disabled="busy"
        :aria-label="t('spaces.speech.candidateSeed')"
      />
      <UiButton size="sm" :disabled="busy" @click="newSeed">{{ t("spaces.speech.newSeed") }}</UiButton>
    </div>
    <div class="flex gap-2">
      <UiButton size="sm" :disabled="busy || !ready || candidates.length >= 16" @click="generate">{{
        t("spaces.speech.generateCandidate")
      }}</UiButton>
      <UiButton v-if="activeRunId" size="sm" variant="dangerSubtle" @click="cancel">{{
        t("spaces.speech.cancel")
      }}</UiButton>
    </div>
    <p v-if="error" class="text-caption text-fg-danger">{{ error }}</p>
    <VoiceCandidateCard
      v-for="candidate in candidates"
      :key="candidate.candidateId"
      :candidate="candidate"
      :disabled="busy"
      :authorized="authorized"
      @audition="audition(candidate.candidateId)"
      @save="save(candidate.candidateId, $event, authorized)"
      @discard="discard(candidate.candidateId)"
    />
  </section>
</template>
