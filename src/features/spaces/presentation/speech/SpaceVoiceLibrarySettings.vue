<script setup lang="ts">
import { computed, shallowRef, useTemplateRef, watch } from "vue";
import { useI18n } from "vue-i18n";

import UiSelect from "../../../../shared/ui/UiSelect.vue";
import { useSpaceStore } from "../../session/space-store";
import { useSpeechVoices } from "../../speech/use-speech-voices";
import VoiceCandidateForm from "./VoiceCandidateForm.vue";
import VoiceManagement from "./VoiceManagement.vue";
import VoiceProfileForm from "./VoiceProfileForm.vue";

const { t } = useI18n();
const store = useSpaceStore();
const management = useTemplateRef("management");
const error = shallowRef<string>();
const selectedSpaceId = shallowRef(store.activeSpaceId);
const spaceOptions = computed(() =>
  store.spaces
    .filter((item) => item.archivedAt == null)
    .map((item) => ({ value: item.id, label: item.title })),
);
watch(
  spaceOptions,
  (options) => {
    if (options.some(({ value }) => value === selectedSpaceId.value)) return;
    selectedSpaceId.value =
      options.find(({ value }) => value === store.activeSpaceId)?.value ?? options[0]?.value;
  },
  { immediate: true },
);
const space = computed(() => store.spaces.find(({ id }) => id === selectedSpaceId.value));
let epoch = 0;
watch(
  () => selectedSpaceId.value,
  () => {
    epoch += 1;
    error.value = undefined;
  },
);
const library = useSpeechVoices({
  activeSpaceId: () => selectedSpaceId.value,
  contextEpoch: () => epoch,
  isCurrent: (spaceId, version) => selectedSpaceId.value === spaceId && epoch === version,
  setError: (message) => {
    error.value = message;
  },
});
function refresh() {
  void management.value?.reload();
}
watch(library.voices, refresh);
</script>

<template>
  <section data-testid="voice-library-settings" class="space-y-5">
    <p class="text-body text-fg-secondary">{{ t("spaces.speech.library.description") }}</p>
    <UiSelect
      v-if="spaceOptions.length"
      :model-value="selectedSpaceId ?? ''"
      :options="spaceOptions"
      :label="t('spaces.speech.library.spaceLabel')"
      @update:model-value="selectedSpaceId = $event"
    />
    <template v-if="space">
      <p class="text-caption text-fg-secondary">
        {{ t("spaces.speech.library.space", { name: space.title }) }}
      </p>
      <p v-if="error" role="alert" class="text-caption text-fg-danger">{{ error }}</p>
      <section :key="space.id" class="space-y-5">
        <section class="space-y-2">
          <h2 class="text-label font-medium text-fg-primary">{{ t("spaces.speech.library.presets") }}</h2>
          <VoiceProfileForm
            :space-id="space.id"
            :disabled="library.creating.value"
            @create="library.addPreset"
          />
        </section>
        <section class="space-y-2">
          <h2 class="text-label font-medium text-fg-primary">{{ t("spaces.speech.library.add") }}</h2>
          <VoiceCandidateForm :space-id="space.id" @created="refresh" />
        </section>
        <VoiceManagement ref="management" :space-id="space.id" />
      </section>
    </template>
    <p v-else class="text-caption text-fg-secondary">{{ t("spaces.speech.library.requiresSpace") }}</p>
  </section>
</template>
